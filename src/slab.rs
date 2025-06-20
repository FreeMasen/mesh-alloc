//! An instance of a single slab of equally sized elements.
//!
//! Similar to a Vec but with a custom source for the initial
//! buffer allocation.
//!

use std::{alloc::Layout, ptr::NonNull};

pub struct Slab<T> {
    ptr: NonNull<T>,
    pub capacity: usize,
}

impl<T> Slab<T> {
    pub fn try_allocate(size: usize, allocate: fn(usize) -> *mut u8) -> Option<Self> {
        let layout = Layout::new::<T>();
        let bytes = allocate(layout.size() * size);
        let ptr = NonNull::new(bytes.cast())?;
        Some(Self {
            ptr,
            capacity: size,
        })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if self.capacity <= index {
            return None;
        }
        Some(unsafe { self.ptr.add(index).as_ref() })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if self.capacity <= index {
            return None;
        }
        Some(unsafe { self.ptr.add(index).as_mut() })
    }

    pub fn set(&mut self, index: usize, value: T) -> Option<()> {
        if self.capacity <= index {
            return None;
        }
        unsafe {
            *self.ptr.add(index).as_mut() = value;
        }
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allocator(byte_ct: usize) -> *mut u8 {
        let layout = Layout::from_size_align(byte_ct, 4).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        ptr
    }

    #[test]
    fn u8s() {
        const SLAB_LENGTH: usize = 1024;
        let mut slab: Slab<u8> = Slab::try_allocate(SLAB_LENGTH, allocator).unwrap();
        println!("allocated slab!");
        for i in 0..SLAB_LENGTH {
            let v = (i % 127) as u8;
            println!("attempting to set {i} to {v}");
            slab.set(i, v);
            println!("getting value at {i}");
            let element = slab.get(i).unwrap();
            assert_eq!(element, &v);
            println!("getting mutable value at {i}");
            let v = slab.get_mut(i).unwrap();
            let updated = v.wrapping_mul(2);
            println!("assigning mut ref at {i} to {updated}");
            *v = updated;
            println!("getting value at {i} again");
            let element = slab.get(i).unwrap();
            assert_eq!(element, &updated);
        }
    }

    #[test]
    fn ptrs() {
        const SLAB_LENGTH: usize = 1024;
        let mut slab: Slab<*mut u8> = Slab::try_allocate(SLAB_LENGTH, allocator).unwrap();
        println!("allocated slab!");
        for i in 0..SLAB_LENGTH {
            let v = i as *mut u8;
            println!("attempting to set {i} to {v:?}");
            slab.set(i, v);
            println!("getting value at {i}");
            let element = slab.get(i).unwrap();
            assert_eq!(element, &v);
            println!("getting mutable value at {i}");
            let v = slab.get_mut(i).unwrap();
            let updated = unsafe { v.add(2) };
            println!("assigning mut ref at {i} to {updated:?}");
            *v = updated;
            println!("getting value at {i} again");
            let element = slab.get(i).unwrap();
            assert_eq!(element, &updated);
        }
    }
}
