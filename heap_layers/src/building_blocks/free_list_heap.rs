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
// #ifndef HL_FREELISTHEAP_H
// #define HL_FREELISTHEAP_H

use std::{ptr::NonNull, sync::Mutex};

// /**
//  * @class FreelistHeap
//  * @brief Manage freed memory on a linked list.
//  * @warning This is for one "size class" only.
//  *
//  * Note that the linked list is threaded through the freed objects,
//  * meaning that such objects must be at least the size of a pointer.
//  */
// #include <assert.h>
// #include "utility/freesllist.h"
pub use crate::utilities::free_sl_list::{self, FreeSLList, Node};

// namespace HL {

//   template <class SuperHeap>
//   class FreelistHeap : public SuperHeap {
//   public:
pub(crate) struct FreeListHeap<A: crate::stl::FlexableAllocator> {
    _free_list: FreeSLList<NonNull<u8>, A>,
}

impl<A: crate::stl::FlexableAllocator> FreeListHeap<A> {
    pub fn new(alloc: A) -> Self {
        Self {
            _free_list: FreeSLList::new(alloc)
        }
    }
}

impl<A: crate::stl::FlexableAllocator> FreeListHeap<A> {
//     inline void * malloc (size_t sz) {
    pub fn malloc(&mut self, sz: usize) -> *mut u8 {
//       // Check the free list first.
//       void * ptr = _freelist.get();
        if let Some(free) = self._free_list.get() {
            return free_sl_list::Node::into_element(free).as_ptr();
        }
//       // If it's empty, get more memory;
//       // otherwise, advance the free list pointer.
//       if (!ptr) {
//         ptr = SuperHeap::malloc (sz);
//       }
        self._free_list.alloc.malloc(sz)
//       return ptr;
//     }
    }

//     inline void free (void * ptr) {
    pub fn free(&mut self, ptr: *mut u8) {
        let Some(nn) = NonNull::new(ptr) else {
            return;
        };
//       if (!ptr) {
//         return;
//       }
//       _freelist.insert (ptr);
        self._free_list.insert(nn);
//     }
    }

//     inline void clear (void) {
    pub fn clear(&mut self) {
//       void * ptr;
//       while ((ptr = _freelist.get())) {
//         SuperHeap::free (ptr);
//       }
        loop {
            let ptr = {
                let Some(ptr) = self._free_list.get() else {
                    break;
                };
                Node::into_element(ptr)
            };
            self._free_list.alloc.free(ptr.as_ptr(), 0);
        }
    }
//     }
}
//   private:

//     FreeSLList _freelist;

//   };

// }

// #endif
#[repr(transparent)]
pub struct LockedFreeListHeap<A: crate::stl::FlexableAllocator>(std::sync::Mutex<FreeListHeap<A>>);

impl<A: crate::stl::FlexableAllocator> LockedFreeListHeap<A> {
    pub fn new(alloc: A) -> Self {
        Self(Mutex::new(FreeListHeap::new(alloc)))
    }
}

unsafe impl<A: crate::stl::FlexableAllocator> crate::stl::Allocator for LockedFreeListHeap<A> {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<NonNull<[u8]>, allocator_api2::alloc::AllocError> {
        let ptr = self.0.lock().unwrap_or_else(|e| e.into_inner()).malloc(layout.size());
        let ptr = NonNull::new(ptr).ok_or(crate::stl::AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: std::alloc::Layout) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).free(ptr.as_ptr());
    }
}

unsafe impl<A: crate::stl::FlexableAllocator> crate::stl::allocator::RawAllocator for LockedFreeListHeap<A> {
    const ALLIGNMENT: usize = A::ALLIGNMENT;

    fn malloc(&self, n: usize) -> *mut u8 {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).malloc(n)
    }

    fn free(&self, ptr: *mut u8, _sz: usize) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).free(ptr);
    }

    fn memalign(&self, _alignment: usize, _sz: usize) -> *mut u8 {
        todo!()
    }

    fn get_size(&self, _ptr: *mut u8) -> usize {
        todo!()
    }
}
