#![allow(dead_code)]

use prelude::*;
use std::cell::UnsafeCell;
use std::ptr;

/// Result of AVec::get(). While this reference is valid, the associated AVec will panic on
/// concurrent writes.
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
    fn deref(self: &Self) -> &[T] {
        let data = self.owner.read();
        &data[0..self.size]
    }
}

impl<'a, T> Drop for AVecReadGuard<'a, T> {
    fn drop(self: &mut Self) {
        self.owner.end_read();
    }
}

/// Result of AVec::map(). While this reference is valid, the associated AVec will panic on
/// concurrent reads.
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

    /// Returns the actual range on the associated AVec mapped by this guard.
    pub fn mapped_range(self: &Self) -> (usize, usize) {
        (self.start, self.start + self.size)
    }

    /// Returns the length of the mapped range.
    pub fn len(self: &Self) -> usize {
        self.size
    }

    /// Sets an element within the mapped ranged of 0 <= index < size().
    pub fn set(self: &Self, index: usize, value: T) {
        if index < self.size {
            self.owner.write(self.start + index, value);
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
/// !todo panics on concurrent reads+writes
pub struct AVec<T> {
    /// A vector acting as buffer. Resizes are mutex-guarded.
    data: UnsafeCell<Vec<T>>,
    /// Next insert position.
    insert: AtomicUsize,
    /// Number of concurrent readers.
    readers: AtomicUsize,
    /// Number of concurrent writers.
    writers: AtomicUsize,
    /// Number of initialized buffer elements. On Drop, the buffer will set_len to this and let the underlying vector perform the drops.
    initialized: UnsafeCell<usize>,
    /// Locked while the underlying vector is being resized. Contains dummy data.
    grow: Mutex<usize>,
}

unsafe impl<T> Sync for AVec<T> { }
unsafe impl<T> Send for AVec<T> { }

impl<'a, T> Drop for AVec<T> {
    fn drop(&mut self) {
        self.shrink_to_initialized();
    }
}

impl<T> AVec<T> {

    /// Creates a new instance with given inital capacity
    pub fn new(initial_capacity: usize) -> AVec<T> {
        let instance = AVec::<T> {
            data        : UnsafeCell::new(Vec::new()),
            insert      : AtomicUsize::new(0),
            readers     : AtomicUsize::new(0),
            writers     : AtomicUsize::new(0),
            initialized : UnsafeCell::new(0),
            grow        : Mutex::new(initial_capacity),
        };
        instance.resize(initial_capacity);
        instance
    }

    /// Adds an element to the vector and returns the insert position.
    pub fn push(self: &Self, value: T) -> usize {
        self.begin_write();
        let insert_pos = self.insert.fetch_add(1, Ordering::Relaxed);
        while insert_pos >= self.capacity() {
            self.grow(insert_pos + 1);
        }
        self.write(insert_pos, value);
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
    pub fn len(self: &Self) -> usize {
        self.insert.load(Ordering::Relaxed)
    }

    /// Grow vector by given number of elements.
    pub fn grow(self: &Self, required_capacity: usize) {
        let mut guard = self.grow.lock().unwrap();
        let capacity = self.capacity();
        if required_capacity > capacity {
            let new_capacity = if required_capacity > capacity * 2 {
                // need more than 2x the current capacity: resize exactly
                self.resize(required_capacity)
            } else {
                // need less than 2x the current capacity: grow by current capacity (=double it)
                self.resize(capacity * 2)
            };
            *guard.deref_mut() = new_capacity; // dummy
        }
    }

    /// Clears the vector.
    pub fn clear(self: &Self) {
        // since we decrease the insert position new inserts might run into still-in-progress
        // inserts from before the clear if we didn't prevent that.
        if self.readers.fetch_add(1, Ordering::SeqCst) > 0 {
            panic!("Attempt read+write with concurrent readers");
        }
        if self.writers.fetch_add(1, Ordering::SeqCst) > 0 {
            panic!("Attempt read+write with concurrent writers");
        }
        let current_max = self.insert.swap(0, Ordering::SeqCst);
        self.update_initialized(current_max);
        self.readers.fetch_sub(1, Ordering::SeqCst);
        self.writers.fetch_sub(1, Ordering::SeqCst);
    }

    /// Maps the entire vector for read access.
    pub fn get<'a>(&'a self) -> AVecReadGuard<'a, T> {
        self.begin_read();
        let size = self.insert.load(Ordering::SeqCst);
        AVecReadGuard::new(&self, size)
    }

    /// Returns current capacity. The vector will reallocate once this is exceeded.
    pub fn capacity(self: &Self) -> usize {
        self.read().capacity()
    }

    /// Shrinks the underlying vector to contain only initialized elements
    fn shrink_to_initialized(self: &Self) {
        let mut guard = self.grow.lock().unwrap();
        let num_initialized = self.initialized();
        self.resize(num_initialized);
        *guard.deref_mut() = num_initialized;
    }

    /// Resize to given size. Returns new capacity, which may be larger than the given size.
    fn resize(self: &Self, new_size: usize) -> usize {
        unsafe {
            let mut data = &mut *self.data.get();
            let capacity = data.capacity();
            if new_size > capacity {
                data.reserve(new_size - capacity);
                data.set_len(new_size);
            } else if new_size < capacity {
                if new_size < self.initialized() {
                    panic!("Attempted to reduce size below initialized number of elements.")
                }
                data.set_len(new_size);
                data.shrink_to_fit();
            }
            data.capacity()
        }
    }

    /// Updates number of initialized buffer elements if given value is larger than the current one.
    fn update_initialized(self: &Self, num_initialized: usize) {
        unsafe {
            let current = *self.initialized.get();
            if num_initialized > current {
                *self.initialized.get() = num_initialized;
            }
        }
    }

    /// Returns number of initialized buffer elements.
    fn initialized(self: &Self) -> usize {
        // initialized is only updated on clear, so the true result is the max. of initialized and current-length
        unsafe { cmp::max(*self.initialized.get(), self.len()) }
    }

    /// Write element to the underlying vector, dropping the previous occupant, if any.
    fn write(self: &Self, insert_pos: usize, value: T) {
        unsafe {
            let mut data = &mut *self.data.get();
            if insert_pos < *self.initialized.get() {
                ptr::drop_in_place(&mut data[insert_pos]);
            }
            ptr::write(&mut data[insert_pos], value);
        }
    }

    /// Returns underlying buffer for reading
    fn read(self: &Self) -> &Vec<T> {
        unsafe { &*self.data.get() }
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
