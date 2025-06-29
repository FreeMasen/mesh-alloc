// // -*- mode: c++; c-basic-offset: 2; indent-tabs-mode: nil -*-
// // Copyright 2019 The Mesh Authors. All rights reserved.
// // Use of this source code is governed by the Apache License,
// // Version 2.0, that can be found in the LICENSE file.

// #pragma once
// #ifndef MESH_CHEAP_HEAP_H
// #define MESH_CHEAP_HEAP_H

// #include "internal.h"
// #include "one_way_mmap_heap.h"

// namespace mesh {

use std::{
    ffi::{c_char, c_void},
    mem::transmute,
    num::NonZeroUsize,
    ptr::slice_from_raw_parts_mut,
    thread::current,
};

use crate::{mmap_heap::MmapHeap, one_way_mmap_heap::OneWayMmapHeap, slab::Slab};

// // Fast allocation for a single size-class
// template <size_t allocSize, size_t maxCount>
// class CheapHeap : public OneWayMmapHeap {
pub struct CheapHeap<const ALLOC_SIZE: usize, const MAX_COUNT: usize> {
    // protected:
    //   char *_arena{nullptr};
    _arena: *mut u8,
    //   void **_freelist{nullptr};
    _free_list: Slab<*mut u8>,
    //   size_t _arenaOff{1};
    _arena_offset: usize,
    //   ssize_t _freelistOff{-1};
    _free_list_off: Option<usize>,
    // };
}

impl<const ALLOC_SIZE: usize, const MAX_COUNT: usize> CheapHeap<ALLOC_SIZE, MAX_COUNT> {
    fn allocate_one_way(size: usize) -> *mut u8 {
        OneWayMmapHeap::malloc(size).cast()
    }
    // private:
    //   DISALLOW_COPY_AND_ASSIGN(CheapHeap);
    //   typedef OneWayMmapHeap SuperHeap;

    //   static_assert(maxCount <= (1 << 30), "expected maxCount <= 2^30");
    //   static_assert(allocSize % 2 == 0, "expected allocSize to be even");

    // public:
    //   // cacheline-sized alignment
    //   enum { Alignment = 64 };
    pub fn new() -> Self {
        //   CheapHeap() : SuperHeap() {
        //     // TODO: check allocSize + maxCount doesn't overflow?
        //     _arena = reinterpret_cast<char *>(SuperHeap::malloc(allocSize * maxCount));
        let _arena: *mut u8 = OneWayMmapHeap::malloc(ALLOC_SIZE * MAX_COUNT).cast();
        //     _freelist = reinterpret_cast<void **>(SuperHeap::malloc(maxCount * sizeof(void *)));
        let _free_list = Slab::try_allocate(MAX_COUNT, Self::allocate_one_way).unwrap();
        //     hard_assert(_arena != nullptr);
        //     hard_assert(_freelist != nullptr);
        //     d_assert(reinterpret_cast<uintptr_t>(_arena) % Alignment == 0);
        //     d_assert(reinterpret_cast<uintptr_t>(_freelist) % Alignment == 0);
        //   }

        Self {
            _arena,
            _free_list,
            _arena_offset: 0,
            _free_list_off: None,
        }
    }

    //   inline void *alloc() {
    pub fn alloc(&mut self) -> *mut u8 {
        //     if (likely(_freelistOff >= 0)) {
        if let Some(offset) = self._free_list_off.take() {
            let ptr = *self._free_list.get(offset).unwrap();
            self._free_list_off = offset.checked_sub(1);
            return ptr;
        }

        //     const auto off = _arenaOff++;
        let Some(off) = self._arena_offset.checked_add(1) else {
            panic!("arena overflow")
        };
        if off > MAX_COUNT {
            panic!("arena exhausted")
        }
        self._arena_offset = off;

        //     const auto ptr = ptrFromOffset(off);
        let ptr = unsafe { self.ptr_from_offset(off) };
        //     hard_assert(ptr < arenaEnd());
        //     return ptr;
        ptr
        //   }
    }

    //   constexpr size_t getSize(void *ATTRIBUTE_UNUSED ptr) const {
    pub fn get_size(&self, _: *mut c_void) -> usize {
        //     return allocSize;
        ALLOC_SIZE
        //   }
    }

    //   inline void free(void *ptr) {
    pub fn free(&mut self, ptr: *mut u8) {
        println!(
            "{:?} > {ptr:?} < {:x?}",
            self._arena,
            self.offset_after_arena()
        );
        //     d_assert(ptr >= _arena);
        assert!(ptr.cast() >= self._arena);
        //     d_assert(ptr < arenaEnd());
        debug_assert!((ptr as usize) < self.offset_after_arena(), 
            "attempt to free pointer from another heap: {} >= {}", ptr as usize, self.offset_after_arena());
        // assert!(ptr.cast() < unsafe { self._arena.add(count)});
        //     _freelistOff++;
        //     _freelist[_freelistOff] = ptr;
        match self._free_list_off.take() {
            Some(current) => {
                self._free_list.set(current, ptr).unwrap();
                let next = current.checked_add(1).unwrap();
                self._free_list_off = Some(next);
            }
            None => {
                self._free_list_off = Some(0);
                self._free_list.set(0, ptr);
            }
        }

        //   }
    }

    //   inline char *arenaBegin() const {
    //     return _arena;
    //   }

    //   inline uint32_t offsetFor(const void *ptr) const {
    pub fn offset_for(&self, ptr: *mut u8) -> usize {
        //     const uintptr_t ptrval = reinterpret_cast<uintptr_t>(ptr);
        let ptr_val = ptr as usize;
        //     const uintptr_t arena = reinterpret_cast<uintptr_t>(_arena);
        let arena = self._arena as usize;
        //     d_assert(ptrval >= arena);
        debug_assert!(ptr_val >= arena, "Expected {ptr_val:?} to be >= {arena:?}");
        //     return (ptrval - arena) / allocSize;
        ptr_val - arena / ALLOC_SIZE
        //   }
    }

    //   inline char *ptrFromOffset(size_t off) const {
    pub unsafe fn ptr_from_offset(&self, off: usize) -> *mut u8 {
        //     d_assert(off < _arenaOff);
        // debug_assert!(off < self._arena_offset);
        //     return _arena + off * allocSize;
        unsafe { self._arena.add(off * ALLOC_SIZE) }
        //   }
    }

    //   inline char *arenaEnd() const {
    pub fn offset_after_arena(&self) -> usize {
        let arena = self._arena as usize;
        arena + (ALLOC_SIZE * MAX_COUNT) + 1
    }
    //     return _arena + allocSize * maxCount;
    //   }

    // class DynCheapHeap : public OneWayMmapHeap {
    // private:
    //   DISALLOW_COPY_AND_ASSIGN(DynCheapHeap);
    //   typedef OneWayMmapHeap SuperHeap;

    // public:
    //   // cacheline-sized alignment
    //   enum { Alignment = 64 };

    //   DynCheapHeap() : SuperHeap() {
    //     d_assert(_allocSize % 2 == 0);
    //   }

    //   inline void init(size_t allocSize, size_t maxCount, char *arena, void **freelist) {
    //     _arena = arena;
    //     _freelist = freelist;
    //     _allocSize = allocSize;
    //     _maxCount = maxCount;
    //     hard_assert(_arena != nullptr);
    //     hard_assert(_freelist != nullptr);
    //     hard_assert(reinterpret_cast<uintptr_t>(_arena) % Alignment == 0);
    //     hard_assert(reinterpret_cast<uintptr_t>(_freelist) % Alignment == 0);
    //   }

    //   inline void *alloc() {
    //     if (likely(_freelistOff >= 0)) {
    //       const auto ptr = _freelist[_freelistOff];
    //       _freelistOff--;
    //       return ptr;
    //     }

    //     const auto off = _arenaOff++;
    //     const auto ptr = ptrFromOffset(off);
    //     hard_assert(ptr < arenaEnd());
    //     return ptr;
    //   }

    //   size_t getSize(void *ATTRIBUTE_UNUSED ptr) const {
    //     return _allocSize;
    //   }

    //   inline void free(void *ptr) {
    //     d_assert(ptr >= _arena);
    //     d_assert(ptr < arenaEnd());

    //     _freelistOff++;
    //     _freelist[_freelistOff] = ptr;
    //   }

    //   inline char *arenaBegin() const {
    //     return _arena;
    //   }

    //   inline uint32_t offsetFor(const void *ptr) const {
    //     const uintptr_t ptrval = reinterpret_cast<uintptr_t>(ptr);
    //     const uintptr_t arena = reinterpret_cast<uintptr_t>(_arena);
    //     d_assert(ptrval >= arena);
    //     return (ptrval - arena) / _allocSize;
    //   }

    //   inline char *ptrFromOffset(size_t off) const {
    //     d_assert(off < _arenaOff);
    //     return _arena + off * _allocSize;
    //   }

    //   inline char *arenaEnd() const {
    //     return _arena + _allocSize * _maxCount;
    //   }

    // protected:
    //   char *_arena{nullptr};
    //   void **_freelist{nullptr};
    //   size_t _arenaOff{1};
    //   ssize_t _freelistOff{-1};
    //   size_t _allocSize{0};
    //   size_t _maxCount{0};
    // };
}
// }  // namespace mesh

// #endif  // MESH_CHEAP_HEAP_H

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use paste::paste;

    use super::*;

    macro_rules! generate_test_for {
        ($alloc_size:literal, $max_count:literal) => {
            paste! {
                #[test]
                fn [<generate_test_ $alloc_size _ $max_count>]() {
                    do_general_test::<$alloc_size, $max_count>();
                }
            }
        };

    }
    /*16,  16,  32,  48,  64,  80,  96,  112,  128,  160,  192,  224,   256,
    320, 384, 448, 512, 640, 768, 896, 1024, 2048, 4096, 8192, 16384, */
    generate_test_for!(16, 0x800000);
    generate_test_for!(16, 512);
    generate_test_for!(32, 512);
    generate_test_for!(48, 512);
    generate_test_for!(64, 512);
    generate_test_for!(80, 512);
    generate_test_for!(96, 512);
    generate_test_for!(112, 512);
    generate_test_for!(128, 512);
    generate_test_for!(160, 512);
    generate_test_for!(192, 512);
    generate_test_for!(224, 512);
    generate_test_for!(256, 512);
    generate_test_for!(320, 512);
    generate_test_for!(384, 512);
    generate_test_for!(448, 512);
    generate_test_for!(512, 512);
    generate_test_for!(640, 512);
    generate_test_for!(768, 512);
    generate_test_for!(896, 512);
    generate_test_for!(1024, 512);
    generate_test_for!(2048, 512);
    generate_test_for!(4096, 512);
    generate_test_for!(8192, 512);
    generate_test_for!(16384, 512);
    generate_test_for!(16384, 0x800000);
    // 768, 896, 1024, 2048, 4096, 8192, 16384,

    fn do_general_test<const ALLOC_SIZE: usize, const MAX_COUNT: usize>() {
        let mut cheap = CheapHeap::<ALLOC_SIZE, MAX_COUNT>::new();
        let mut ps = HashSet::with_capacity(MAX_COUNT);
        for i in 0..MAX_COUNT {
            assert_eq!(cheap._arena_offset, i);
            let p = cheap.alloc();
            assert!(ps.insert(p), "Allocation {i} already in use");
        }
        assert_eq!(cheap._arena_offset, MAX_COUNT);
        assert_eq!(cheap._free_list_off, None);
        for (idx, ptr) in ps.into_iter().enumerate() {
            println!("!!!: {:?}, {}", cheap._free_list_off, MAX_COUNT - idx - 1);
            cheap.free(ptr);
            assert_eq!(cheap._free_list_off, Some(idx));
        }
    }

    #[test]
    #[should_panic = "attempt to free pointer from another heap: "]
    fn free_wrong_heap() {
        let mut h1 = CheapHeap::<1, 8>::new();
        let mut h2 = CheapHeap::<1, 8>::new();
        let _p1 = h1.alloc();
        let p1 = h1.alloc();
        let _p2 = h2.alloc();
        let p2 = h2.alloc();
        h1.free(p2);
    }
}
