#![allow(dead_code)]

use prelude::*;
use std::cell::UnsafeCell;
use std::ptr;

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
    pub fn mapped_range(self: &Self) -> (usize, usize) {
        (self.start, self.start + self.size)
    }
    pub fn set(self: &Self, index: usize, value: T) {
        if index < self.size {
            // insert element, dropping any previous occupants, if any
            unsafe {
                let insert_pos = self.start + index;
                let mut data = &mut *self.owner.data.get();
                if insert_pos < *self.owner.max_init.get() {
                    ptr::drop_in_place(&mut data[insert_pos]);
                }
                ptr::write(&mut data[insert_pos], value);
            }

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

/// Vector supporting multiple non-overlapping writers or multiple readers. Blocks during resizes only.
pub struct AVec<T> {
    /// A vector acting as buffer. Resizes are mutec-guarded.
    data    : UnsafeCell<Vec<T>>,
    /// Next insert position.
    insert  : AtomicUsize,
    /// Number of concurrent readers.
    readers : AtomicUsize,
    /// Number of concurrent writers.
    writers : AtomicUsize,
    /// Number of initialized buffer elements. On Drop, the buffer will set_len to this and let the underlying vector perform the drops.
    max_init: UnsafeCell<usize>,
    /// Current capacity of the underlying vector
    capacity: UnsafeCell<usize>,
    /// Locked while the underlying vector is being resized.
    grow    : Mutex<usize>,
}

unsafe impl<T> Sync for AVec<T> { }
unsafe impl<T> Send for AVec<T> { }

impl<'a, T> Drop for AVec<T> {
    fn drop(&mut self) {
        // set_len buffer to number of actually initialized data and let it handle the drops
        unsafe {
            let mut data = &mut *self.data.get();
            data.set_len(*self.max_init.get());
        }
    }
}

impl<T> AVec<T> {

    /// Creates a new instance with given inital capacity
    pub fn new(initial_capacity: usize) -> AVec<T> {
        let mut data = Vec::with_capacity(initial_capacity);
        unsafe {
            data.set_len(initial_capacity);
        }
        AVec::<T> {
            data        : UnsafeCell::new(data),
            insert      : AtomicUsize::new(0),
            readers     : AtomicUsize::new(0),
            writers     : AtomicUsize::new(0),
            max_init    : UnsafeCell::new(0),
            capacity    : UnsafeCell::new(initial_capacity),
            grow        : Mutex::new(initial_capacity),
        }
    }

    /// Adds an element to the vector and returns the insert position.
    pub fn push(&self, value: T) -> usize {
        self.begin_write();
        let insert_pos = self.insert.fetch_add(1, Ordering::Relaxed);
        while insert_pos >= self.capacity() {
            self.grow(insert_pos + 1);
        }
        // insert element, dropping any previous occupants, if any
        unsafe {
            let mut data = &mut *self.data.get();
            if insert_pos < *self.max_init.get() {
                ptr::drop_in_place(&mut data[insert_pos]);
            }
            ptr::write(&mut data[insert_pos], value);
        }
        self.end_write();
        insert_pos
    }

    /// Maps a slice of the vector for write access. Faster than individual pushes for size > 2.
    pub fn map<'a>(&'a self, size: usize) -> AVecMapGuard<'a, T>  {
        self.begin_write();
        let insert_pos = self.insert.fetch_add(size, Ordering::Relaxed);
        let required_capacity = insert_pos + size;
        while required_capacity > self.capacity() {
            self.grow(required_capacity);
        }
        AVecMapGuard::new(&self, insert_pos, size)
    }

    /// Returns the current length of the vector.
    pub fn len(&self) -> usize {
        self.insert.load(Ordering::Relaxed)
    }

    /// Returns maximum capacity.
    pub fn capacity(self: &Self) -> usize {
        unsafe { *self.capacity.get() }
    }

    /// Grow vector by given number of elements.
    pub fn grow(self: &Self, required_capacity: usize) {
        let mut guard = self.grow.lock().unwrap();
        let capacity = self.capacity();
        if required_capacity > capacity {
            let new_capacity = if required_capacity > capacity * 2 {
                // need more than 2x the current capacity: resize exactly
                unsafe { self.internal_grow(required_capacity - capacity) }
            } else {
                // need less than 2x the current capacity: grow by current capacity (=double it)
                unsafe { self.internal_grow(capacity) }
            };
            *guard.deref_mut() = new_capacity;
        }
    }

    /// Clears the vector.
    pub fn clear(&self) {
        // since we decrease the insert position new inserts might run into still-in-progress
        // inserts from before the clear if we didn't prevent that.
        self.begin_rw();
        let current_max = self.insert.swap(0, Ordering::SeqCst);
        unsafe {
            if current_max > *self.max_init.get() {
                *self.max_init.get() = current_max;
            }
        }
        self.end_rw();
    }

    /// Maps the entire vector for read access.
    pub fn get<'a>(&'a self) -> AVecReadGuard<'a, T> {
        self.begin_read();
        let size = self.insert.load(Ordering::SeqCst);
        AVecReadGuard::new(&self, size)
    }

    /// grow by additional elements and return total available number of elements
    unsafe fn internal_grow(self: &Self, additional: usize) -> usize {
        let mut capacity = self.capacity();
        let data = self.data.get();
        capacity += additional;
        (*data).reserve(additional);
        (*data).set_len(capacity);
        *self.capacity.get() = capacity;
        capacity
    }

    /// Begin a read/write block and check for concurrent readers/writers
    fn begin_rw(self: &Self) {
        if self.readers.fetch_add(1, Ordering::SeqCst) > 0 {
            panic!("Attempt read+write with concurrent readers");
        }
        if self.writers.fetch_add(1, Ordering::SeqCst) > 0 {
            panic!("Attempt read+write with concurrent writers");
        }
    }

    /// Ends a read/write block
    fn end_rw(self: &Self) {
        self.readers.fetch_sub(1, Ordering::SeqCst);
        self.writers.fetch_sub(1, Ordering::SeqCst);
    }

    /// Begin a read and check for concurrent writers
    fn begin_read(self: &Self) {
        self.readers.fetch_add(1, Ordering::SeqCst);
        if self.writers.load(Ordering::SeqCst) > 0 {
            panic!("Attempt to read during concurrent write");
        }
    }

    /// End a read (really?...)
    fn end_read(self: &Self) {
        self.readers.fetch_sub(1, Ordering::SeqCst);
    }

    /// Begin a write and check for concurrent readers
    fn begin_write(self: &Self) {
        self.writers.fetch_add(1, Ordering::SeqCst);
        if self.readers.load(Ordering::SeqCst) > 0 {
            panic!("Attempt to write during concurrent reads");
        }
    }

    /// End a write
    fn end_write(self: &Self) {
        self.writers.fetch_sub(1, Ordering::SeqCst);
    }
}
