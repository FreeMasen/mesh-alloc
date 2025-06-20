// // -*- mode: c++; c-basic-offset: 2; indent-tabs-mode: nil -*-
// // Copyright 2019 The Heap-Layers and Mesh Authors. All rights reserved.
// // Use of this source code is governed by the Apache License,
// // Version 2.0, that can be found in the LICENSE file.

// #pragma once
// #ifndef MESH_MMAP_HEAP_H
// #define MESH_MMAP_HEAP_H

// #if defined(_WIN32)
// #error "TODO"
// #include <windows.h>
// #else
// // UNIX
// #include <fcntl.h>
// #include <stdlib.h>
// #include <sys/mman.h>
// #include <sys/stat.h>
// #include <sys/types.h>
// #include <unistd.h>
// #include <map>
// #endif

// #include "internal.h"
// #include "one_way_mmap_heap.h"

// namespace mesh {

use std::{collections::HashMap, ffi::c_void, process::abort};

use crate::one_way_mmap_heap::OneWayMmapHeap;

// // MmapHeap extends OneWayMmapHeap to track allocated address space
// // and will free memory with calls to munmap.
#[derive(Default)]
pub struct MmapHeap {
    vma_map: HashMap<*mut c_void, usize>,
}
// class MmapHeap : public OneWayMmapHeap {
// private:
//   DISALLOW_COPY_AND_ASSIGN(MmapHeap);
//   typedef OneWayMmapHeap SuperHeap;

// public:
//   enum { Alignment = MmapWrapper::Alignment };

//   MmapHeap() : SuperHeap() {
//   }
impl MmapHeap {
    //   inline void *malloc(size_t sz) {
    pub fn malloc(&mut self, sz: usize) -> *mut c_void {
        //     auto ptr = map(sz, MAP_PRIVATE | MAP_ANONYMOUS, -1);
        let ptr = OneWayMmapHeap::map(sz, rustix::mm::MapFlags::PRIVATE);
        //     d_assert(_vmaMap.find(ptr) == _vmaMap.end());
        debug_assert!(!self.vma_map.contains_key(&ptr));
        //     _vmaMap[ptr] = sz;
        self.vma_map.insert(ptr, sz);
        //     d_assert(_vmaMap.find(ptr) != _vmaMap.end());
        //     d_assert(_vmaMap[ptr] == sz);
        debug_assert_eq!(self.vma_map.get(&ptr), Some(&sz));
        //     return ptr;
        ptr
        //   }
    }

    //   inline size_t getSize(void *ptr) const {
    pub fn get_size(&mut self, ptr: *mut c_void) -> usize {
        //     auto entry = _vmaMap.find(ptr);
        let Some(&entry) = self.vma_map.get(&ptr) else {
            //     if (unlikely(entry == _vmaMap.end())) {

            //       debug("mmap: invalid getSize: %p", ptr);
            //       abort();
            //       return 0;
            //     }
            abort()
        };
        //     return entry->second;
        entry
        //   }
    }

    //   inline bool inBounds(void *ptr) const {
    pub fn in_bounds(&self, ptr: *mut c_void) -> bool {
        //     auto entry = _vmaMap.find(ptr);
        //     // FIXME: this isn't right -- we want inclusion not exact match
        //     return true;
        self.vma_map.get(&ptr).is_some()
        //   }
    }

    //   inline void free(void *ptr) {
    pub fn free(&mut self, ptr: *mut c_void) {
        let Some(sz) = self.vma_map.remove(&ptr) else {
            //     auto entry = _vmaMap.find(ptr);
            //     if (unlikely(entry == _vmaMap.end())) {
            //       debug("mmap: invalid free, possibly from memalign: %p", ptr);
            //       // abort();
            //       return;
            //     }
            abort()
        };

        //     auto sz = entry->second;

        unsafe {
            //     munmap(ptr, sz);
            rustix::mm::munmap(ptr, sz).unwrap();
            //     // madvise(ptr, sz, MADV_DONTNEED);
            rustix::mm::madvise(ptr, sz, rustix::mm::Advice::DontNeed).unwrap();
            //     // mprotect(ptr, sz, PROT_NONE);
            rustix::mm::mprotect(ptr, sz, rustix::mm::MprotectFlags::empty()).unwrap();
        }

        //     _vmaMap.erase(entry);
        //     d_assert(_vmaMap.find(ptr) == _vmaMap.end());
        debug_assert!(!self.vma_map.contains_key(&ptr));
        //   }
    }

    //   // return the sum of the sizes of all large allocations
    //   size_t arenaSize() const {
    pub fn arena_size(&self) -> usize {
        //     size_t sz = 0;
        //     for (auto it = _vmaMap.begin(); it != _vmaMap.end(); it++) {
        //       sz += it->second;
        //     }
        //     return sz;
        self.vma_map.values().sum()
        //   }
    }

    // protected:
    //   internal::unordered_map<void *, size_t> _vmaMap{};
    // };
}
// }  // namespace mesh

// #endif  // MESH_MESH_MMAP_H
