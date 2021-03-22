use anyhow::{Context, Result};
use glium::Display;
use std::{
    cmp::PartialEq,
    collections::HashMap,
    fs::File,
    marker::PhantomData,
    mem,
    path::{Path, PathBuf},
};

#[repr(C)]
pub struct RawTrait {
    pub data: *mut (),
    pub vtable: *mut (),
}

unsafe fn downcast<T: DynResource>(t: &dyn DynResource) -> &T {
    let value: RawTrait = mem::transmute(t);
    &*mem::transmute::<_, *mut T>(value.data)
}

unsafe fn downcast_mut<T: DynResource>(t: &mut dyn DynResource) -> &mut T {
    let value: RawTrait = mem::transmute(t);
    &mut *mem::transmute::<_, *mut T>(value.data)
}

#[derive(Eq, PartialEq)]
pub struct ResourceId<T: Resource> {
    idx: u32,
    generation: u32,
    __marker: PhantomData<T>,
}

impl<T: Resource> Clone for ResourceId<T> {
    fn clone(&self) -> Self {
        ResourceId {
            idx: self.idx,
            generation: self.generation,
            __marker: PhantomData,
        }
    }
}

impl<T: Resource> Copy for ResourceId<T> {}

impl<T: Resource> ResourceId<T> {
    pub fn into_any(self) -> AnyResourceId {
        AnyResourceId {
            idx: self.idx,
            generation: self.generation,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct AnyResourceId {
    idx: u32,
    generation: u32,
}

impl<T: Resource> PartialEq<AnyResourceId> for ResourceId<T> {
    fn eq(&self, other: &AnyResourceId) -> bool {
        self.idx == other.idx && self.generation == other.generation
    }
}

impl<T: Resource> PartialEq<ResourceId<T>> for AnyResourceId {
    fn eq(&self, other: &ResourceId<T>) -> bool {
        self.idx == other.idx && self.generation == other.generation
    }
}

pub trait Resource: 'static + Sized {
    fn load(file: File, display: &Display, res: &mut Resources) -> Result<Self>;

    fn reload(&mut self, file: File, display: &Display, res: &mut Resources) -> Result<()> {
        *self = Self::load(file, display, res)?;
        Ok(())
    }

    fn reload_dependency(
        &mut self,
        _dependency: AnyResourceId,
        _display: &Display,
        _res: &Resources,
    ) -> Result<bool> {
        Ok(false)
    }
}

trait DynResource {
    fn reload(&mut self, file: File, display: &Display, res: &mut Resources) -> Result<()>;

    fn reload_dependency(
        &mut self,
        dependency: AnyResourceId,
        display: &Display,
        res: &Resources,
    ) -> Result<bool>;
}

impl<T: Resource> DynResource for T {
    fn reload(&mut self, file: File, display: &Display, res: &mut Resources) -> Result<()> {
        (*self).reload(file, display, res)
    }

    fn reload_dependency(
        &mut self,
        dependency: AnyResourceId,
        display: &Display,
        res: &Resources,
    ) -> Result<bool> {
        (*self).reload_dependency(dependency, display, res)
    }
}

pub struct Filled {
    generation: u32,
    parent: Option<AnyResourceId>,
    name: PathBuf,
    file: Option<Box<dyn DynResource>>,
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
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            names: HashMap::new(),
            res: Vec::new(),
            first_empty: None,
            parent_stack: Vec::new(),
        }
    }

    pub fn get<T: Resource>(&self, id: ResourceId<T>) -> Option<&T> {
        self.res.get(id.idx as usize).and_then(|x| {
            let filled = x.as_filled()?;
            if filled.generation != id.generation {
                return None;
            }
            return Some(unsafe { downcast(filled.file.as_deref().unwrap()) });
        })
    }

    pub fn get_mut<T: Resource>(&mut self, id: ResourceId<T>) -> Option<&mut T> {
        self.res.get_mut(id.idx as usize).and_then(|x| {
            let filled = x.as_filled_mut()?;
            if filled.generation != id.generation {
                return None;
            }
            return Some(unsafe { downcast_mut(filled.file.as_deref_mut().unwrap()) });
        })
    }

    pub fn insert<T: Resource, P: Into<PathBuf>>(
        &mut self,
        path: P,
        display: &Display,
    ) -> Result<ResourceId<T>> {
        self.insert_res(path.into(), display)
    }

    fn insert_res<T: Resource>(
        &mut self,
        base_name: PathBuf,
        display: &Display,
    ) -> Result<ResourceId<T>> {
        trace!("loading {}", base_name.display());
        let name = base_name
            .canonicalize()
            .with_context(|| format!("Failed to open file for {}", base_name.display()))?;

        if let Some(x) = self.names.get(&name) {
            return Ok(ResourceId {
                generation: x.generation,
                idx: x.idx,
                __marker: PhantomData,
            });
        }

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
        let name = name.canonicalize().unwrap();
        self.parent_stack.push(AnyResourceId { idx, generation });
        let res = T::load(file, display, self)
            .with_context(|| format!("Loading resource {}", base_name.display()))?;
        self.parent_stack.pop();
        self.res[idx as usize] = ResourceEntry::File(Filled {
            file: Some(Box::new(res)),
            parent: self.parent_stack.last().copied(),
            generation,
            name: name.clone(),
        });
        self.names.insert(name, AnyResourceId { idx, generation });

        Ok(ResourceId {
            idx,
            generation,
            __marker: PhantomData,
        })
    }

    pub fn remove<T: Resource>(&mut self, id: ResourceId<T>) {
        self.remove_any(id.into_any());
    }

    pub fn remove_any(&mut self, id: AnyResourceId) {
        if let Some(x) = self
            .res
            .get_mut(id.idx as usize)
            .and_then(ResourceEntry::as_filled_mut)
        {
            if x.generation != id.generation {
                return;
            }
            self.names.remove(&x.name);

            self.res[id.idx as usize] = ResourceEntry::Empty(Empty {
                next: self.first_empty,
                generation: id.generation,
            });
            self.first_empty = Some(id.idx)
        }
    }

    pub fn reload<P: AsRef<Path>>(&mut self, path: P, display: &Display) -> Result<bool> {
        let orig_path = path.as_ref();
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
            return Ok(true);
        }
        Ok(false)
    }

    fn reload_dependency(
        &mut self,
        id: AnyResourceId,
        reloaded: AnyResourceId,
        display: &Display,
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
