use crate::util::Downcast;
use anyhow::Result;
use std::{
    cell::{self, RefCell},
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    rc::{Rc, Weak},
};

thread_local!( static ACTIVE_ASSETS: RefCell<HashMap<PathBuf,Vec<WeakAssetRef>>> = RefCell::new(HashMap::new()));
thread_local!( static LOAD_STACK: RefCell<Vec<Vec<WeakAssetRef>>> = RefCell::new(Vec::new()) );

pub struct AssetRef<A: Asset>(Rc<RefCell<AssetData<A>>>);
pub struct DynAssetRef(Rc<dyn DynAsset>);
type WeakAssetRef = Weak<dyn DynAsset>;

pub struct AssetData<A> {
    parent: Option<WeakAssetRef>,
    asset: A,
}

pub trait Asset: Downcast {
    fn reload(&mut self, path: &Path) -> Result<()>;

    fn reload_dependency(&mut self, asset: &DynAssetRef) -> Result<bool>;
}

trait DynAsset: Downcast {
    fn reload(&self, path: &Path) -> Result<()>;

    fn reload_dependency(&self, asset: &DynAssetRef) -> Result<bool>;

    fn set_parent(&self, parent: WeakAssetRef);

    fn get_parent(&self) -> Option<WeakAssetRef>;
}

impl<A: Asset + 'static> DynAsset for RefCell<AssetData<A>> {
    fn reload(&self, path: &Path) -> Result<()> {
        self.try_borrow_mut()
            .expect("reference to asset is being held while reloading")
            .asset
            .reload(path)
    }

    fn reload_dependency(&self, asset: &DynAssetRef) -> Result<bool> {
        self.try_borrow_mut()
            .expect("reference to asset is being held while reloading")
            .asset
            .reload_dependency(asset)
    }

    fn set_parent(&self, parent: WeakAssetRef) {
        self.borrow_mut().parent = Some(parent);
    }

    fn get_parent(&self) -> Option<WeakAssetRef> {
        self.borrow().parent.clone()
    }
}

impl DynAssetRef {
    pub fn same<A: Asset + 'static>(&self, rf: &AssetRef<A>) -> bool {
        Rc::as_ptr(&self.0) as *const _ == Rc::as_ptr(&rf.0)
    }
}

impl<A: Asset> PartialEq for AssetRef<A> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

pub struct Ref<'a, A>(cell::Ref<'a, AssetData<A>>);

impl<'a, A> Deref for Ref<'a, A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.0.asset
    }
}

pub struct RefMut<'a, A>(cell::RefMut<'a, AssetData<A>>);

impl<'a, A> Deref for RefMut<'a, A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.0.asset
    }
}

impl<'a, A> DerefMut for RefMut<'a, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.asset
    }
}

impl<A: Asset + fmt::Debug> fmt::Debug for AssetRef<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut writer = f.debug_tuple("AssetRef");
        if let Ok(x) = self.0.try_borrow() {
            writer.field(&x.asset);
        } else {
            writer.field(&"BORROWED");
        }
        writer.finish()
    }
}

impl<A: Asset + 'static> AssetRef<A> {
    pub fn build<G, P: AsRef<Path>, F: FnOnce(&Path, G) -> Result<A>>(
        f: F,
        path: P,
        args: G,
    ) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let v = ACTIVE_ASSETS.with(|k| {
            k.borrow().get(&path).and_then(|e| {
                e.iter().find_map(|v| {
                    v.upgrade()
                        .and_then(|e| e.into_any_rc().downcast::<RefCell<AssetData<A>>>().ok())
                })
            })
        });
        if let Some(x) = v {
            return Ok(AssetRef(x));
        }

        LOAD_STACK.with(|x| x.borrow_mut().push(Vec::new()));

        let asset = f(&path, args)?;

        let asset = Rc::new(RefCell::new(AssetData {
            parent: None,
            asset,
        }));
        let weak = Rc::downgrade(&asset) as WeakAssetRef;

        LOAD_STACK.with(|x| {
            let mut guard = x.borrow_mut();
            let children = match guard.pop() {
                Some(x) => x,
                _ => return,
            };
            dbg!(children.len());
            for a in children.into_iter() {
                if let Some(x) = a.upgrade() {
                    x.set_parent(weak.clone())
                }
            }
            if let Some(x) = guard.last_mut() {
                x.push(weak.clone())
            }
        });

        ACTIVE_ASSETS.with(|k| {
            k.borrow_mut()
                .entry(path)
                .or_insert_with(Vec::new)
                .push(weak)
        });
        return Ok(AssetRef(asset));
    }

    pub fn borrow(&self) -> Ref<'_, A> {
        Ref(self.0.borrow())
    }

    pub fn borrow_mut(&self) -> RefMut<'_, A> {
        RefMut(self.0.borrow_mut())
    }
}

pub fn reload(path: &Path) -> Result<()> {
    let mut reload = Vec::new();
    ACTIVE_ASSETS.with(|v| {
        dbg!(v.borrow_mut().keys().collect::<Vec<_>>());
        if let Some(x) = v.borrow_mut().get_mut(path) {
            println!("found path");
            for a in x.iter() {
                if let Some(a) = a.upgrade() {
                    reload.push(a);
                }
            }
        }
    });
    for a in reload.into_iter() {
        println!("reloaded");
        a.reload(path)?;
        let mut cur = a;
        while let Some(x) = cur.get_parent().and_then(|x| x.upgrade()) {
            if !x.reload_dependency(&DynAssetRef(cur))? {
                break;
            }
            cur = x;
        }
    }
    Ok(())
}

/// Cleans hanging refrences to dropped assets.
pub fn clean() {
    ACTIVE_ASSETS.with(|v| {
        v.borrow_mut().retain(|_, v| {
            v.retain(|v| v.strong_count() != 0);
            !v.is_empty()
        });
    })
}
