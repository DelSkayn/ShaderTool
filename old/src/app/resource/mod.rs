use anyhow::Result;
use glium::Display;
use std::{
    cmp::PartialEq,
    fs::File,
    marker::PhantomData,
    mem,
    sync::{mpsc::Sender, Arc},
};
mod resources;
pub use resources::Resources;

#[repr(C)]
struct RawTrait {
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

#[derive(Clone)]
struct ResourceIdData {
    id: AnyResourceId,
    sender: Sender<AnyResourceId>,
}

impl Drop for ResourceIdData {
    fn drop(&mut self) {
        self.sender.send(self.id).unwrap();
    }
}

pub struct ResourceId<T: Resource> {
    id: Arc<ResourceIdData>,
    __marker: PhantomData<T>,
}

impl<T: Resource> ResourceId<T> {
    pub fn id(&self) -> u32 {
        self.id.id.idx
    }

    pub fn generation(&self) -> u32 {
        self.id.id.generation
    }

    pub fn into_any(&self) -> AnyResourceId {
        self.id.id
    }
}

impl<T: Resource> PartialEq for ResourceId<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.id, &other.id)
    }
}

impl<T: Resource> Clone for ResourceId<T> {
    fn clone(&self) -> Self {
        ResourceId {
            id: self.id.clone(),
            __marker: PhantomData,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct AnyResourceId {
    idx: u32,
    generation: u32,
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
