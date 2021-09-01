use std::{fmt, hash::Hash, mem, ops, slice};

/// A key into a slotmap.
/// # Safety
///
/// implementation must guarentee that the key returns the index it was created with.
pub unsafe trait SlotKey:
    Copy + Clone + Default + PartialEq + Eq + Hash + fmt::Debug
{
    type Version: Default + Eq + PartialEq + Clone + Copy + fmt::Debug;

    fn new(v: usize) -> Self;

    #[inline(always)]
    fn new_version(idx: usize, _version: Self::Version) -> Self {
        Self::new(idx)
    }

    fn index(&self) -> usize;
    fn max() -> usize;

    #[inline(always)]
    fn version(&self) -> Self::Version {
        Default::default()
    }

    #[inline(always)]
    fn next_version(self) -> Self {
        self
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct VersionKey {
    idx: u32,
    version: u32,
}

unsafe impl SlotKey for VersionKey {
    type Version = u32;

    #[inline(always)]
    fn new(idx: usize) -> Self {
        assert!(idx <= u32::MAX as usize);
        VersionKey {
            idx: idx as u32,
            version: 0,
        }
    }

    fn new_version(idx: usize, version: Self::Version) -> Self {
        VersionKey {
            idx: idx as u32,
            version,
        }
    }

    #[inline(always)]
    fn index(&self) -> usize {
        self.idx as usize
    }

    #[inline(always)]
    fn max() -> usize {
        u32::MAX as usize
    }

    #[inline(always)]
    fn version(&self) -> u32 {
        self.version
    }

    fn next_version(mut self) -> Self {
        self.version = self.version.wrapping_add(1);
        self
    }
}

unsafe impl SlotKey for usize {
    type Version = ();

    #[inline(always)]
    fn new(x: usize) -> Self {
        x
    }

    #[inline(always)]
    fn index(&self) -> usize {
        *self
    }

    #[inline(always)]
    fn max() -> Self {
        usize::MAX
    }
}

unsafe impl SlotKey for u32 {
    type Version = ();

    #[inline(always)]
    fn new(x: usize) -> Self {
        assert!(x <= u32::MAX as usize);
        x as u32
    }

    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    fn max() -> usize {
        u32::MAX as usize
    }
}

unsafe impl SlotKey for u16 {
    type Version = ();

    #[inline(always)]
    fn new(x: usize) -> Self {
        assert!(x <= u16::MAX as usize);
        x as u16
    }

    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    fn max() -> usize {
        u16::MAX as usize
    }
}

unsafe impl SlotKey for u8 {
    type Version = ();

    #[inline(always)]
    fn new(x: usize) -> Self {
        assert!(x <= u8::MAX as usize);
        x as u8
    }

    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    fn max() -> usize {
        u8::MAX as usize
    }
}

#[derive(Debug)]
enum SlotMapValue<T, Idx: SlotKey> {
    Free(Option<Idx>),
    Value { value: T, version: Idx::Version },
}

#[derive(Debug)]
/// A datastructure mainting an intrusive free list which allows for 0(1) insertion and removal without changing indecies of
/// values.
pub struct SlotMap<T, Idx: SlotKey = usize> {
    values: Vec<SlotMapValue<T, Idx>>,
    free: Option<Idx>,
}

impl<T, Idx: SlotKey> SlotMap<T, Idx> {
    /// Create a new list.
    pub fn new() -> Self {
        SlotMap {
            values: Vec::new(),
            free: None,
        }
    }

    /// Create a list with a given capacity.
    pub fn with_capacity(capacity: usize) -> SlotMap<T> {
        SlotMap {
            values: Vec::with_capacity(capacity),
            free: None,
        }
    }

    pub fn get(&self, idx: Idx) -> Option<&T> {
        if let SlotMapValue::Value { ref value, version } = self.values.get(idx.index())? {
            if idx.version() == *version {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, idx: Idx) -> Option<&mut T> {
        if let SlotMapValue::Value {
            ref mut value,
            version,
        } = self.values.get_mut(idx.index())?
        {
            if idx.version() == *version {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Insert a value into the list.
    /// Returns the index at which the value was inserted.
    pub fn insert(&mut self, v: T) -> Idx {
        if let Some(x) = self.free {
            match self.values[x.index()] {
                SlotMapValue::Free(free) => {
                    self.free = free;
                    self.values[x.index()] = SlotMapValue::Value {
                        value: v,
                        version: x.next_version().version(),
                    };
                    x.next_version()
                }
                _ => panic!("invalid free list!"),
            }
        } else {
            if self.values.len() >= <Idx as SlotKey>::max().index() {
                panic!("to many values for the given index")
            }
            let idx = Idx::new(self.values.len());
            self.values.push(SlotMapValue::Value {
                value: v,
                version: idx.version(),
            });
            idx
        }
    }

    /// Remove a value at the given index.
    /// Returns `Some` if there is a value at the given index else returns none.
    pub fn remove(&mut self, idx: Idx) -> Option<T> {
        if idx.index() >= self.values.len() {
            return None;
        }
        match mem::replace(&mut self.values[idx.index()], SlotMapValue::Free(self.free)) {
            SlotMapValue::Value { value, .. } => {
                self.free = Some(idx);
                Some(value)
            }
            SlotMapValue::Free(x) => {
                self.values[idx.index()] = SlotMapValue::Free(x);
                None
            }
        }
    }

    /// Returns wether a value is present at the given index.
    pub fn is_present(&self, idx: Idx) -> bool {
        idx.index() <= self.values.len()
            && match self.values[idx.index()] {
                SlotMapValue::Free(_) => false,
                SlotMapValue::Value { version, .. } => version == idx.version(),
            }
    }

    /// Returns the amount of entries used, both free and used.
    pub fn entries(&self) -> usize {
        self.values.len()
    }

    /// Returns the amount of values that can be inserted before a new allocations.
    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    /// Returns an iterator over all present values.
    pub fn iter<'a>(&'a self) -> Iter<'a, T, Idx> {
        Iter {
            v: self.values.iter(),
        }
    }

    /// Returns an iterator over all present values.
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T, Idx> {
        IterMut {
            v: self.values.iter_mut(),
        }
    }
}

pub struct Iter<'a, T, Idx: SlotKey> {
    v: slice::Iter<'a, SlotMapValue<T, Idx>>,
}

impl<'a, T, Idx: SlotKey> Iterator for Iter<'a, T, Idx> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.v.next() {
                None => return None,
                Some(SlotMapValue::Value { ref value, .. }) => return Some(value),
                Some(SlotMapValue::Free(_)) => {}
            }
        }
    }
}

pub struct IterMut<'a, T, Idx: SlotKey> {
    v: slice::IterMut<'a, SlotMapValue<T, Idx>>,
}

impl<'a, T, Idx: SlotKey> Iterator for IterMut<'a, T, Idx> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        loop {
            match self.v.next() {
                None => return None,
                Some(SlotMapValue::Value { ref mut value, .. }) => return Some(value),
                Some(SlotMapValue::Free(_)) => {}
            }
        }
    }
}

impl<T, Idx: SlotKey> ops::Index<Idx> for SlotMap<T, Idx> {
    type Output = T;

    #[inline(always)]
    fn index(&self, idx: Idx) -> &T {
        match self.values[idx.index()] {
            SlotMapValue::Value { ref value, version } => {
                if version != idx.version() {
                    panic!("invalid version of key")
                }
                value
            }
            SlotMapValue::Free(_) => panic!("no value at given index"),
        }
    }
}

impl<T, Idx: SlotKey> ops::IndexMut<Idx> for SlotMap<T, Idx> {
    #[inline(always)]
    fn index_mut(&mut self, idx: Idx) -> &mut T {
        match self.values[idx.index()] {
            SlotMapValue::Value {
                ref mut value,
                version,
            } => {
                if version != idx.version() {
                    panic!("invalid version of key")
                }
                value
            }
            SlotMapValue::Free(_) => panic!("no value at given index"),
        }
    }
}
