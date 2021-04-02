use super::*;
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fs::File,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Weak,
    },
};
use vulkano::device::Device;

pub struct Filled {
    generation: u32,
    parent: Option<AnyResourceId>,
    name: PathBuf,
    file: Option<Box<dyn DynResource>>,
    key: Weak<ResourceIdData>,
}

pub struct Empty {
    generation: u32,
    next: Option<u32>,
}

enum ResourceEntry {
    Empty(Empty),
    File(Filled),
}

impl ResourceEntry {
    pub fn as_filled(&self) -> Option<&Filled> {
        match *self {
            ResourceEntry::File(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn as_filled_mut(&mut self) -> Option<&mut Filled> {
        match *self {
            ResourceEntry::File(ref mut x) => Some(x),
            _ => None,
        }
    }

    pub fn as_empty(&self) -> Option<&Empty> {
        match *self {
            ResourceEntry::Empty(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn as_empty_mut(&mut self) -> Option<&mut Empty> {
        match *self {
            ResourceEntry::Empty(ref mut x) => Some(x),
            _ => None,
        }
    }
}

pub struct Resources {
    names: HashMap<PathBuf, AnyResourceId>,
    res: Vec<ResourceEntry>,
    first_empty: Option<u32>,
    parent_stack: Vec<AnyResourceId>,
    clean_reciever: Receiver<AnyResourceId>,
    clean_sender: Sender<AnyResourceId>,
}

impl Resources {
    pub fn new() -> Self {
        let (send, recv) = mpsc::channel();
        Resources {
            names: HashMap::new(),
            res: Vec::new(),
            first_empty: None,
            parent_stack: Vec::new(),
            clean_reciever: recv,
            clean_sender: send,
        }
    }

    pub fn get<T: Resource>(&self, id: &ResourceId<T>) -> Option<&T> {
        self.res.get(id.id() as usize).and_then(|x| {
            let filled = x.as_filled()?;
            if filled.generation != id.generation() {
                return None;
            }
            return Some(unsafe { downcast(filled.file.as_deref().unwrap()) });
        })
    }

    pub fn get_mut<T: Resource>(&mut self, id: &ResourceId<T>) -> Option<&mut T> {
        self.res.get_mut(id.id() as usize).and_then(|x| {
            let filled = x.as_filled_mut()?;
            if filled.generation != id.generation() {
                return None;
            }
            return Some(unsafe { downcast_mut(filled.file.as_deref_mut().unwrap()) });
        })
    }

    fn clean(&mut self) {
        while let Ok(id) = self.clean_reciever.try_recv() {
            if let Some(x) = self
                .res
                .get_mut(id.idx as usize)
                .and_then(ResourceEntry::as_filled_mut)
            {
                if x.generation != id.generation {
                    return;
                }
                dbg!(&x.name);
                self.names.remove(&x.name);

                self.res[id.idx as usize] = ResourceEntry::Empty(Empty {
                    next: self.first_empty,
                    generation: id.generation,
                });
                self.first_empty = Some(id.idx)
            }
        }
    }

    pub fn insert<T: Resource, P: Into<PathBuf>>(
        &mut self,
        path: P,
        device: &Device,
    ) -> Result<ResourceId<T>> {
        self.insert_res(path.into(), display)
    }

    fn insert_res<T: Resource>(
        &mut self,
        base_name: PathBuf,
        device: &Device,
    ) -> Result<ResourceId<T>> {
        self.clean();
        trace!("loading {}", base_name.display());
        let name = base_name
            .canonicalize()
            .with_context(|| format!("Failed to open file for {}", base_name.display()))?;

        // Handle pressent value
        if let Some(x) = self.names.get(&name) {
            let id = self.res[x.idx as usize]
                .as_filled()
                .unwrap()
                .key
                .upgrade()
                .unwrap();
            return Ok(ResourceId {
                id,
                __marker: PhantomData,
            });
        }

        // Generate idx and generation
        let (idx, generation) = if let Some(x) = self.first_empty {
            let empty = self.res[x as usize].as_empty().unwrap();
            self.first_empty = empty.next;
            (x, empty.generation.wrapping_add(1))
        } else {
            assert!(self.res.len() < u32::MAX as usize);
            let idx = self.res.len();
            self.res.push(ResourceEntry::Empty(Empty {
                generation: 0,
                next: None,
            }));
            (idx as u32, 0)
        };

        let file = File::open(&name)
            .with_context(|| format!("Failed to open file for {}", name.display()))?;
        self.parent_stack.push(AnyResourceId { idx, generation });
        let res = T::load(file, display, self)
            .with_context(|| format!("Loading resource {}", base_name.display()))?;
        self.parent_stack.pop();

        let any_id = AnyResourceId { idx, generation };

        let key = Arc::new(ResourceIdData {
            id: any_id,
            sender: self.clean_sender.clone(),
        });

        self.res[idx as usize] = ResourceEntry::File(Filled {
            file: Some(Box::new(res)),
            parent: self.parent_stack.last().copied(),
            generation,
            name: name.clone(),
            key: Arc::downgrade(&key),
        });
        self.names.insert(name, any_id);

        Ok(ResourceId {
            id: key,
            __marker: PhantomData,
        })
    }

    pub fn reload<P: AsRef<Path>>(&mut self, path: P, device: &Device) -> Result<bool> {
        let orig_path = path.as_ref();
        trace!("reloading: {}", orig_path.display());
        let path = match path.as_ref().canonicalize() {
            Ok(x) => x,
            Err(_) => return Ok(false),
        };
        if let Some(x) = self.names.get(&path).copied() {
            trace!("reloading {}", orig_path.display());
            let mut f = self.res[x.idx as usize]
                .as_filled_mut()
                .unwrap()
                .file
                .take()
                .unwrap();
            let file = File::open(path)?;
            let error = f.reload(file, display, self);
            let entry = self.res[x.idx as usize].as_filled_mut().unwrap();
            entry.file = Some(f);
            error?;
            if let Some(parent) = entry.parent {
                self.reload_dependency(parent, x, display)?;
            }
            self.clean();
            return Ok(true);
        }
        self.clean();
        Ok(false)
    }

    fn reload_dependency(
        &mut self,
        id: AnyResourceId,
        reloaded: AnyResourceId,
        device: &Device,
    ) -> Result<()> {
        let entry = self.res[id.idx as usize].as_filled_mut().unwrap();
        let mut f = entry.file.take().unwrap();
        let reloaded = f.reload_dependency(reloaded, display, &*self);
        let entry = self.res[id.idx as usize].as_filled_mut().unwrap();
        entry.file = Some(f);
        let reloaded = reloaded?;
        if reloaded {
            if let Some(x) = entry.parent {
                self.reload_dependency(x, id, display)?
            }
        }
        Ok(())
    }
}
