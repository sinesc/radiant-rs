use prelude::*;
use std::cell::UnsafeCell;

/// read guard
pub struct AVecReadGuard<'a, T: 'a> {
    owner   : &'a AVec<T>,
    size    : usize,
}

impl<'a, T> AVecReadGuard<'a, T> {
    fn new(owner: &'a AVec<T>, size: usize) -> AVecReadGuard<'a, T> {
        AVecReadGuard {
            owner   : &owner,
            size    : size,
        }
    }
}

impl<'a, T> Deref for AVecReadGuard<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        let data = unsafe { &*self.owner.data.get() };
        &data[0..self.size]
    }
}

impl<'a, T> Drop for AVecReadGuard<'a, T> {
    fn drop(&mut self) {
        self.owner.end_read();
    }
}

/// map guard
pub struct AVecMapGuard<'a, T: 'a> {
    owner   : &'a AVec<T>,
    start   : usize,
    size    : usize,
}

impl<'a, T> AVecMapGuard<'a, T> {
    fn new(owner: &'a AVec<T>, start: usize, size: usize) -> AVecMapGuard<'a, T> {
        AVecMapGuard {
            owner   : &owner,
            start   : start,
            size    : size,
        }
    }
    #[allow(dead_code)]
    pub fn mapped_range(self: &Self) -> (usize, usize) {
        (self.start, self.start + self.size)
    }
    #[inline]
    pub fn set(self: &Self, index: usize, value: T) {
        if index < self.size {
            let mut data = unsafe { &mut *self.owner.data.get() };
            data[self.start + index] = value;
        } else {
            panic!("index out of bounds");
        }
    }
}

impl<'a, T> Drop for AVecMapGuard<'a, T> {
    fn drop(&mut self) {
        self.owner.end_write();
    }
}

/// Vector supporting multiple non-overlapping writers or multiple readers. Never blocks,
/// panics on concurrent r/w.
pub struct AVec<T> {
    data    : UnsafeCell<Vec<T>>,
    insert  : AtomicUsize,
    readers : AtomicUsize,
    writers : AtomicUsize,
    capacity: usize,
}

unsafe impl<T> Sync for AVec<T> { }
unsafe impl<T> Send for AVec<T> { }

impl<T> AVec<T> {

    /// Creates a new instance with given maximum capacity
    pub fn new(capacity: u32) -> AVec<T> {
        let capacity = capacity as usize;
        let mut data = Vec::with_capacity(capacity);
        unsafe {
            data.set_len(capacity);
        }
        AVec::<T> {
            data    : UnsafeCell::new(data),
            insert  : AtomicUsize::new(0),
            readers : AtomicUsize::new(0),
            writers : AtomicUsize::new(0),
            capacity: capacity,
        }
    }

    /// Adds an element to the vector and returns the insert position.
    pub fn push(&self, value: T) -> usize {
        self.begin_write();
        let insert_pos = self.insert.fetch_add(1, Ordering::Relaxed);
        if insert_pos >= self.capacity {
            panic!("AVec::push: index {} out of range for AVec of capacity {}", insert_pos as u32, self.capacity);
        }
        unsafe {
            let data = self.data.get();
            (*data)[insert_pos] = value;
        }
        self.end_write();
        insert_pos
    }

    /// Maps a slice of the vector for write access. Faster than individual pushes for size > 2.
    pub fn map<'a>(&'a self, size: usize) -> AVecMapGuard<'a, T>  {
        self.begin_write();
        let insert_pos = self.insert.fetch_add(size as usize, Ordering::Relaxed);
        if insert_pos + size > self.capacity {
            panic!("AVec::map: range({},{}) out of range for AVec of capacity {}", insert_pos as u32, insert_pos as u32 + size as u32, self.capacity);
        }
        AVecMapGuard::new(&self, insert_pos, size as usize)
    }

    /// Returns the current length of the vector.
    pub fn len(&self) -> usize {
        self.insert.load(Ordering::Relaxed)
    }

    /// Returns maximum capacity.
    pub fn capacity(self: &Self) -> usize {
        self.capacity
    }

    /// Clears the vector.
    pub fn clear(&self) {
        // this is just an atomic store, but since we effectively decrese the insert position,
        // new inserts might run into still-in-progress inserts from before the clear. these checks prevent that.
        if self.readers.fetch_add(1, Ordering::SeqCst) > 0 {
            panic!("Attempt clear with concurrent readers");
        }
        if self.writers.fetch_add(1, Ordering::SeqCst) > 0 {
            panic!("Attempt clear with concurrent writers");
        }
        self.insert.store(0, Ordering::SeqCst);
        self.readers.fetch_sub(1, Ordering::SeqCst);
        self.writers.fetch_sub(1, Ordering::SeqCst);
    }

    /// Maps the entire vector for read access.
    pub fn get<'a>(&'a self) -> AVecReadGuard<'a, T> {
        self.begin_read();
        let size = self.insert.load(Ordering::SeqCst);
        AVecReadGuard::new(&self, size)
    }

    /// Begin a read and check for concurrent writers
    #[inline]
    fn begin_read(self: &Self) {
        self.readers.fetch_add(1, Ordering::SeqCst);
        if self.writers.load(Ordering::SeqCst) > 0 {
            panic!("Attempt to read during concurrent write");
        }
    }

    /// End a read (really?...)
    #[inline]
    fn end_read(self: &Self) {
        self.readers.fetch_sub(1, Ordering::SeqCst);
    }

    /// Begin a write and check for concurrent readers
    #[inline]
    fn begin_write(self: &Self) {
        self.writers.fetch_add(1, Ordering::SeqCst);
        if self.readers.load(Ordering::SeqCst) > 0 {
            panic!("Attempt to write during concurrent reads");
        }
    }

    /// End a write
    #[inline]
    fn end_write(self: &Self) {
        self.writers.fetch_sub(1, Ordering::SeqCst);
    }
}
