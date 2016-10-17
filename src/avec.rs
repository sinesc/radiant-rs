use prelude::*;
use std::cell::UnsafeCell;

/// read guard
#[allow(dead_code)]
pub struct AVecReadGuard<'a, T: 'a> {
    lock: RwLockWriteGuard<'a , AtomicUsize>,
    data: &'a mut Vec<T>,
    size: usize,
    readers: &'a AtomicUsize,
}

impl<'a, T> AVecReadGuard<'a, T> {
    unsafe fn new(lock: RwLockWriteGuard<'a, AtomicUsize>, data: &'a UnsafeCell<Vec<T>>, size: usize, readers: &'a AtomicUsize) -> AVecReadGuard<'a, T> {
        AVecReadGuard {
            lock: lock,
            data: &mut *data.get(),
            size: size,
            readers: readers,
        }
    }
}

impl<'a, T> Deref for AVecReadGuard<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data[0..self.size]
    }
}

impl<'a, T> Drop for AVecReadGuard<'a, T> {
    fn drop(&mut self) {
        self.readers.fetch_sub(1, Ordering::Relaxed);
    }
}

/// map guard
#[allow(dead_code)]
pub struct AVecMapGuard<'a, T: 'a> {
    lock: RwLockReadGuard<'a, AtomicUsize>,
    data: &'a mut Vec<T>,
    start: usize,
    size: usize,
}

impl<'a, T> AVecMapGuard<'a, T> {
    unsafe fn new(lock: RwLockReadGuard<'a, AtomicUsize>, data: &'a UnsafeCell<Vec<T>>, start: usize, size: usize) -> AVecMapGuard<'a, T> {
        AVecMapGuard {
            lock: lock,
            data: &mut *data.get(),
            start: start,
            size: size,
        }
    }
    pub fn mapped_range(self: &Self) -> (u32, u32) {
        (self.start as u32, self.start as u32 + self.size as u32)
    }
}

impl<'a, T> DerefMut for AVecMapGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.data[self.start..(self.start + self.size)]
    }
}

impl<'a, T> Deref for AVecMapGuard<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data[self.start..(self.start + self.size)]
    }
}

/// Vector supporting multiple non-blocking writers and a single blocking reader.
pub struct AVec<T> {
    data    : UnsafeCell<Vec<T>>,
    insert  : Arc<RwLock<AtomicUsize>>,
    readers : AtomicUsize,
    capacity: usize,
}

unsafe impl<T> Sync for AVec<T> { }
unsafe impl<T> Send for AVec<T> { }

impl<T> AVec<T> where T: Default {

    /// Creates a new instance with given maximum capacity
    pub fn new(capacity: u32) -> AVec<T> {
        let capacity = capacity as usize;
        let mut data = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            data.push(T::default());
        }
        AVec::<T> {
            data    : UnsafeCell::new(data),
            insert  : Arc::new(RwLock::new(ATOMIC_USIZE_INIT)),
            readers : ATOMIC_USIZE_INIT,
            capacity: capacity,
        }
    }

    /// Adds an element to the vector and returns the insert position. This function blocks get()
    /// and clear() until it returns.
    pub fn push(&self, value: T) -> u32 {
        let guard = self.insert.read().unwrap();
        let insert_pos = guard.fetch_add(1, Ordering::Relaxed);
        if insert_pos >= self.capacity {
            panic!("AVec::push: index {} out of range for AVec of capacity {}", insert_pos as u32, self.capacity);
        }
        unsafe {
            let data = self.data.get();
            (*data)[insert_pos] = value;
        }
        insert_pos as u32
    }

    /// Maps a slice of the vector for r/w access. This function blocks get()
    /// and clear() until the mapped result goes out of scope.
    pub fn map<'a>(&'a self, size: u32) -> AVecMapGuard<'a, T>  {
        let guard = self.insert.read().unwrap();
        let insert_pos = guard.fetch_add(size as usize, Ordering::Relaxed);
        if insert_pos + size as usize > self.capacity {
            panic!("AVec::map: range({},{}) out of range for AVec of capacity {}", insert_pos as u32, insert_pos as u32 + size, self.capacity);
        }
        unsafe { AVecMapGuard::new(guard, &self.data, insert_pos, size as usize) }
    }

    /// Returns the current length of the vector. This function blocks get()
    /// and clear() until it returns.
    pub fn len(&self) -> usize {
        self.insert.read().unwrap().load(Ordering::Relaxed)
    }

    /// Clear the vector. This function blocks until it returns.
    pub fn clear(&self) {
        let guard = self.insert.write().unwrap();
        guard.store(0, Ordering::Relaxed);
    }

    /// Maps the entire vector for read access. This function blocks until the mapped result
    /// goes out of scope.
    pub fn get<'a>(&'a self) -> AVecReadGuard<'a, T> {
        if self.readers.fetch_add(1, Ordering::Relaxed) > 0 {
            panic!("AVec::get: instance already exclusively borrowed");
        }
        let guard = self.insert.write().unwrap();
        let size = guard.load(Ordering::Relaxed);
        unsafe { AVecReadGuard::new(guard, &self.data, size, &self.readers) }
    }
}
