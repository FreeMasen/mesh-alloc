// // -*- C++ -*-

// /*

//   Heap Layers: An Extensible Memory Allocation Infrastructure
  
//   Copyright (C) 2000-2020 by Emery Berger
//   http://www.emeryberger.com
//   emery@cs.umass.edu
  
//   Heap Layers is distributed under the terms of the Apache 2.0 license.

//   You may obtain a copy of the License at
//   http://www.apache.org/licenses/LICENSE-2.0

// */

// #ifndef HL_BUMPALLOC_H
// #define HL_BUMPALLOC_H

// #include <cstddef>

// #include "utility/gcd.h"

// #if defined(__clang__)
// #pragma clang diagnostic push
// #pragma clang diagnostic ignored "-Wunused-variable"
// #endif

// /**
//  * @class BumpAlloc
//  * @brief Obtains memory in chunks and bumps a pointer through the chunks.
//  * @author Emery Berger <http://www.cs.umass.edu/~emery>
//  */

// namespace HL {

use core::{ptr::NonNull, sync::atomic::Ordering};

use crate::stl::{allocator::RawAllocator, AllocError};


impl<const CHUNK_SIZE: usize, SuperHeap, const ALLIGNMENT: usize> 
crate::stl::FlexableAllocator for BumpAlloc<CHUNK_SIZE, SuperHeap, ALLIGNMENT> 
where SuperHeap: RawAllocator
{}
//   template <size_t ChunkSize,
// 	    class SuperHeap,
// 	    size_t Alignment_ = 1UL>
//   class BumpAlloc : public SuperHeap {
//   public:
pub(crate) struct BumpAlloc<const CHUNK_SIZE: usize, SuperHeap, const ALLIGNMENT: usize = 1> {
    _bump: core::sync::atomic::AtomicPtr<u8>,
    _remaining: core::sync::atomic::AtomicUsize,
    alloc: SuperHeap,
}

unsafe impl<const CHUNK_SIZE: usize, SuperHeap, const ALLIGNMENT: usize> crate::stl::Allocator for BumpAlloc<CHUNK_SIZE, SuperHeap, ALLIGNMENT> 
where SuperHeap: RawAllocator
{
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let raw_ptr = self.malloc(layout.size());
        let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        self.free();
    }
}

impl<const CHUNK_SIZE: usize, SuperHeap, const ALLIGNMENT: usize> BumpAlloc<CHUNK_SIZE, SuperHeap, ALLIGNMENT> 
where SuperHeap: crate::stl::allocator::RawAllocator {
//     enum { Alignment = Alignment_ };
    pub fn new(alloc: SuperHeap) -> Self {
        Self {
            _bump: core::sync::atomic::AtomicPtr::new(core::ptr::dangling_mut()),
            _remaining: core::sync::atomic::AtomicUsize::new(0),
            alloc,
        }
    }
//     BumpAlloc()
//       : _bump (nullptr),
// 	_remaining (0)
//     {
//       static_assert((int) gcd<ChunkSize, Alignment>::VALUE == Alignment,
// 		    "Alignment must be satisfiable.");
//       static_assert((int) gcd<SuperHeap::Alignment, Alignment>::VALUE == Alignment,
// 		    "Alignment must be compatible with the SuperHeap's alignment.");
//       static_assert((Alignment & (Alignment-1)) == 0,
// 		    "Alignment must be a power of two.");
//     }

    
//     inline void * malloc (size_t sz) {
    pub fn malloc(&self, sz: usize) -> *mut u8 {
//       // Round up the size if necessary.
//       size_t newSize = (sz + Alignment - 1UL) & ~(Alignment - 1UL);
        let new_size = (sz + ALLIGNMENT - 1) & !(ALLIGNMENT - 1);
//       // If there's not enough space left to fulfill this request, get
//       // another chunk.
//       if (_remaining < newSize) {
        if self._remaining.load(Ordering::Relaxed) < new_size {
//       	refill(newSize);
            self.refill(new_size);
//       }
        }
//       // Bump that pointer.
//       char * old = _bump;
//       _bump += newSize;
        let old = self._bump.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            Some(unsafe {v.add(sz)})
        });
            // .map(|nn| unsafe {nn.add(new_size)});
//       _remaining -= newSize;
        self._remaining.fetch_sub(new_size, Ordering::Relaxed);
        let old = unsafe {
            old.unwrap_unchecked()
        };
//       assert ((size_t) old % Alignment == 0);
        assert!(old as usize % ALLIGNMENT == 0);
//       return old;
        old
//     }
    }

//     /// Free is disabled (we only bump, never reclaim).
//     inline bool free (void *) { return false; }
    pub fn free(&self) -> bool {
        false
    }

//   private:

//     /// The bump pointer.
//     char * _bump;

//     /// How much space remains in the current chunk.
//     size_t _remaining;

//     // Get another chunk.
//     void refill (size_t sz) {
    fn refill(&self, sz: usize) {
//       if (sz < ChunkSize) {
//       	sz = ChunkSize;
//       }
        let sz = sz.min(CHUNK_SIZE);
//       _bump = (char *) SuperHeap::malloc (sz);
//       assert ((size_t) _bump % Alignment == 0);
        self._bump.store(self.alloc.malloc(sz), Ordering::Relaxed);
        //       _remaining = sz;
        self._remaining.store(sz, Ordering::Relaxed);
//     }
    }

//   };

//   // We are going to rename this layer to BumpHeap and deprecate BumpAlloc eventually.
//   template <size_t Size, class X>
//   using BumpHeap = BumpAlloc<Size, X>;
  
// }
}
// #if defined(__clang__)
// #pragma clang diagnostic pop
// #endif

// #endif
unsafe impl<const CHUNK_SIZE: usize, SuperHeap, const ALLIGNMENT: usize> crate::stl::allocator::RawAllocator for BumpAlloc<CHUNK_SIZE, SuperHeap, ALLIGNMENT> 
where SuperHeap: crate::stl::allocator::RawAllocator
{
    const ALLIGNMENT: usize = ALLIGNMENT;

    fn malloc(&self, n: usize) -> *mut u8 {
        Self::malloc(&self, n)
    }

    fn free(&self, _ptr: *mut u8, _sz: usize) {
        Self::free(&self);
    }

    fn memalign(&self, alignment: usize, sz: usize) -> *mut u8 {
        todo!()
    }

    fn get_size(&self, ptr: *mut u8) -> usize {
        todo!()
    }
}
