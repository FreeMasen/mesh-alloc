// /* -*- C++ -*- */
// /*

//   Heap Layers: An Extensible Memory Allocation Infrastructure

//   Copyright (C) 2000-2020 by Emery Berger
//   http://www.emeryberger.com
//   emery@cs.umass.edu

//   Heap Layers is distributed under the terms of the Apache 2.0 license.

//   You may obtain a copy of the License at
//   http://www.apache.org/licenses/LICENSE-2.0

// */
// #ifndef HL_LOCKEDHEAP_H
// #define HL_LOCKEDHEAP_H

// #include <mutex>
// #include <cstddef>

use std::sync::{Mutex, MutexGuard};

// namespace HL {
#[repr(transparent)]
pub struct LockedHeap<Super: crate::stl::FlexableAllocator>(Mutex<Super>);
//   template <class LockType, class Super>
//   class LockedHeap : public Super {

//   public:

//     enum { Alignment = Super::Alignment };

//     inline void * malloc (size_t sz) {
//       std::lock_guard<LockType> l (thelock);
//       return Super::malloc (sz);
//     }

//     inline auto free (void * ptr) {
//       std::lock_guard<LockType> l (thelock);
//       return Super::free (ptr);
//     }

//     inline auto free (void * ptr, size_t sz) {
//       std::lock_guard<LockType> l (thelock);
//       return Super::free (ptr, sz);
//     }

//     inline void * memalign (size_t alignment, size_t sz) {
//       std::lock_guard<LockType> l (thelock);
//       return Super::memalign (alignment, sz);
//     }

//     inline size_t getSize (void * ptr) const {
//       std::lock_guard<LockType> l (thelock);
//       return Super::getSize (ptr);
//     }

//     inline size_t getSize (void * ptr) {
//       std::lock_guard<LockType> l (thelock);
//       return Super::getSize (ptr);
//     }

//     inline void lock() {
//       thelock.lock();
//     }

//     inline void unlock() {
//       thelock.unlock();
//     }

//   private:
//     //    char dummy[128]; // an effort to avoid false sharing.
//     LockType thelock;
//   };

// }

impl<A: crate::stl::FlexableAllocator> LockedHeap<A> {
    fn lock(&self) -> MutexGuard<A> {
        self.0.lock().unwrap_or_else(|e|e.into_inner())
    }
}

unsafe impl<A: crate::stl::FlexableAllocator> crate::stl::Allocator for LockedHeap<A> {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, allocator_api2::alloc::AllocError> {
        self.lock().allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        unsafe {
            self.lock().deallocate(ptr, layout);
        }
    }
}

unsafe impl<A: crate::stl::FlexableAllocator> crate::stl::allocator::RawAllocator for LockedHeap<A> {
    const ALLIGNMENT: usize = A::ALLIGNMENT;

    fn malloc(&self, n: usize) -> *mut u8 {
        self.lock().malloc(n)
    }

    fn free(&self, ptr: *mut u8, sz: usize) {
        self.lock().free(ptr, sz);
    }

    fn memalign(&self, alignment: usize, sz: usize) -> *mut u8 {
        self.lock().memalign(alignment, sz)
    }

    fn get_size(&self, ptr: *mut u8) -> usize {
        self.lock().get_size(ptr)
    }
}

impl<A: crate::stl::FlexableAllocator> crate::stl::FlexableAllocator for LockedHeap<A> {}
// #endif
