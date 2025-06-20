// // -*- mode: c++; c-basic-offset: 2; indent-tabs-mode: nil -*-
// // Copyright 2019 The Heap-Layers and Mesh Authors. All rights reserved.
// // Use of this source code is governed by the Apache License,
// // Version 2.0, that can be found in the LICENSE file.

// #pragma once
// #ifndef MESH_ONE_WAY_MMAP_HEAP_H
// #define MESH_ONE_WAY_MMAP_HEAP_H

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
// #endif

// #include "common.h"

// #ifndef HL_MMAP_PROTECTION_MASK
// #error "define HL_MMAP_PROTECTION_MASK before including mmapheap.h"
// #endif

// #if !defined(MAP_ANONYMOUS) && defined(MAP_ANON)
// #define MAP_ANONYMOUS MAP_ANON
// #endif

// namespace mesh {

use std::{
    ffi::c_void,
    os::fd::{BorrowedFd, RawFd},
    ptr::null_mut,
};

use crate::PAGE_SIZE;

// // OneWayMmapHeap allocates address space through calls to mmap and
// // will never unmap address space.
// class OneWayMmapHeap {
pub struct OneWayMmapHeap;
// private:
//   DISALLOW_COPY_AND_ASSIGN(OneWayMmapHeap);

// public:
//   enum { Alignment = MmapWrapper::Alignment };

//   OneWayMmapHeap() {
//   }
impl OneWayMmapHeap {
    //   inline void *map(size_t sz, int flags, int fd = -1) {
    pub fn map(sz: usize, flags: rustix::mm::MapFlags) -> *mut c_void {
        //     if (sz == 0)
        if sz == 0 {
            //       return nullptr;
            return null_mut();
        }
        //     // Round up to the size of a page.
        //     sz = (sz + kPageSize - 1) & (size_t) ~(kPageSize - 1);
        let sz = (sz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        //     void *ptr = mmap(nullptr, sz, HL_MMAP_PROTECTION_MASK, flags, fd, 0);
        //     if (ptr == MAP_FAILED)
        //       abort();
        let ptr = unsafe {
            rustix::mm::mmap_anonymous(null_mut(), sz, rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE, flags).unwrap()
        };

        //     d_assert(reinterpret_cast<size_t>(ptr) % Alignment == 0);

        //     return ptr;
        ptr
        //   }
    }
    //   inline void *malloc(size_t sz) {
    pub fn malloc(sz: usize) -> *mut c_void {
        //     return map(sz, MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE, -1);
        Self::map(
            sz,
            rustix::mm::MapFlags::PRIVATE | rustix::mm::MapFlags::NORESERVE,
        )
        //   }
    }

    //   inline size_t getSize(void *ATTRIBUTE_UNUSED ptr) const {
    pub fn get_size(_: *mut c_void) -> usize {
        //     return 0;
        0
        //   }
    }

    //   inline void free(void *ATTRIBUTE_UNUSED ptr) {
    pub fn free(_: *mut c_void) {}
    //   }
    // };
}

// }  // namespace mesh
// #endif  // MESH_ONE_WAY_MMAP_HEAP_H
