use ahash::AHasher;
use hashbrown::HashMap;

use crate::building_blocks::free_list_heap::{FreeListHeap, LockedFreeListHeap};
use crate::special::bump_alloc::BumpAlloc;
use crate::stl::{allocator::RawAllocator, AllocError};
use crate::wrappers::mmap::MmapWrapper as Wrapper;

use core::{alloc::Layout, ptr::NonNull};
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
// #ifndef HL_MMAPHEAP_H
// #define HL_MMAPHEAP_H

// #if defined(_WIN32)
// #include <windows.h>
// #else
// // UNIX
// #include <sys/types.h>
// #include <sys/stat.h>
// #include <fcntl.h>
// #include <unistd.h>
// #include <sys/mman.h>
// #include <map>
// #endif

// #include <new>
// #include <unordered_map>

// #include "heaps/buildingblock/freelistheap.h"
// #include "heaps/special/zoneheap.h"
// #include "heaps/special/bumpalloc.h"
// #include "heaps/threads/lockedheap.h"
// #include "locks/posixlock.h"
// #include "threads/cpuinfo.h"
// #include "wrappers/mmapwrapper.h"
// #include "wrappers/stlallocator.h"

// #ifndef HL_MMAP_PROTECTION_MASK
// #if HL_EXECUTABLE_HEAP
// #define HL_MMAP_PROTECTION_MASK (PROT_READ | PROT_WRITE | PROT_EXEC)
// #else
// #define HL_MMAP_PROTECTION_MASK (PROT_READ | PROT_WRITE)
// #endif
// #endif
use crate::wrappers::mmap::HL_MMAP_PROTECTION_MASK;

// #if !defined(MAP_ANONYMOUS) && defined(MAP_ANON)
// #define MAP_ANONYMOUS MAP_ANON
// #endif

// /**
//  * @class MmapHeap
//  * @brief A "source heap" that manages memory via calls to the VM interface.
//  * @author Emery Berger
//  */
// namespace HL {

//   /**
//    * @class SizedMmapHeap
//    * @brief A heap around mmap, but only a sized-free is supported for Unix-like systems.
//    */
//   class SizedMmapHeap {
pub struct SizedMmapHeap;
//   public:
impl SizedMmapHeap {
    pub const ZERO_MEMORY: usize = 1;
    pub const ALIGNMENT: usize = Wrapper::ALIGNMENT;
    //     /// All memory from here is zeroed.
    //     enum { ZeroMemory = 1 };

    //     enum { Alignment = MmapWrapper::Alignment };
}

#[cfg(windows)]
impl SizedMmapHeap {
    // #if defined(_WIN32)

    //     static inline void * malloc (size_t sz) {
    pub fn malloc(sz: usize) -> *mut u8 {
        // #if HL_EXECUTABLE_HEAP
        //       char * ptr = (char *) VirtualAlloc (NULL, sz, MEM_RESERVE | MEM_COMMIT | MEM_TOP_DOWN, PAGE_EXECUTE_READWRITE);
        // #else
        //       char * ptr = (char *) VirtualAlloc (NULL, sz, MEM_RESERVE | MEM_COMMIT | MEM_TOP_DOWN, PAGE_READWRITE);
        // #endif
        //       return (void *) ptr;
        //     }
        todo!()
    }

    //     static inline void free (void * ptr, size_t) {
    pub fn free(ptr: *mut u8, _: usize) {
        //       // No need to keep track of sizes in Windows.
        //       VirtualFree (ptr, 0, MEM_RELEASE);
        todo!()
        //     }
    }

    //     static inline void free (void * ptr) {
    //       // No need to keep track of sizes in Windows.
    //       VirtualFree (ptr, 0, MEM_RELEASE);
    //     }
    pub fn get_size(ptr: *mut u8) -> usize {
        //     inline static size_t getSize (void * ptr) {
        //       MEMORY_BASIC_INFORMATION mbi;
        //       VirtualQuery (ptr, &mbi, sizeof(mbi));
        //       return (size_t) mbi.RegionSize;
        todo!()
        //     }
    }
}

// #else
#[cfg(not(windows))]
impl SizedMmapHeap {
    //     static inline void * malloc (size_t sz) {
    pub fn malloc(sz: usize) -> *mut u8 {
        use rustix::mm::{self, MapFlags};
        //       // Round up to the size of a page.
        //       sz = (sz + CPUInfo::PageSize - 1) & (size_t) ~(CPUInfo::PageSize - 1);
        let sz = (sz + Wrapper::SIZE - 1) & !(Wrapper::SIZE - 1);
        //       void * addr = 0;
        let addr: *mut u8 = core::ptr::null_mut();
        //       int flags = 0;
        let mut flags = MapFlags::empty();
        //       static int fd = -1;
        // #if defined(MAP_ALIGN) && defined(MAP_ANON)
        //       addr = (void *)Alignment;
        //       flags |= MAP_PRIVATE | MAP_ALIGN | MAP_ANON;
        // #elif !defined(MAP_ANONYMOUS)
        //       if (fd == -1) {
        // 	fd = ::open ("/dev/zero", O_RDWR);
        //       }
        //       flags |= MAP_PRIVATE;
        // #else
        //       flags |= MAP_PRIVATE | MAP_ANONYMOUS;
        flags |= MapFlags::PRIVATE;
        // #if HL_EXECUTABLE_HEAP
        // #if defined(MAP_JIT)
        //       flags |= MAP_JIT;
        // #endif
        // #endif
        // #endif
        let ptr = unsafe {
            mm::mmap_anonymous(addr.cast(), sz, HL_MMAP_PROTECTION_MASK, flags)
                .unwrap_or(core::ptr::null_mut())
        };
        //       auto ptr = mmap (addr, sz, HL_MMAP_PROTECTION_MASK, flags, fd, 0);
        //       if (ptr == MAP_FAILED) {
        // 	ptr = nullptr;
        //       }
        //       return ptr;
        ptr.cast()
        //     }
    }

    //     static void free (void * ptr, size_t sz)
    pub fn free(ptr: *mut u8, sz: usize) {
        //     {
        //       if ((long) sz < 0) {
        if sz < 0 {
            // 	abort();
            // TODO.rm: can we abort w/o std? Do we need to avoid std?
            //       }
        }
        //       munmap (reinterpret_cast<char *>(ptr), sz);
        unsafe {
            let _ = rustix::mm::munmap(ptr.cast(), sz);
        }
        //     }
    }

    // #endif

    //   };
}

unsafe impl crate::stl::Allocator for SizedMmapHeap {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let raw_ptr = self.malloc(layout.size());
        let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.free(ptr.as_ptr(), layout.size());
    }
}

unsafe impl crate::stl::allocator::RawAllocator for SizedMmapHeap {
    const ALLIGNMENT: usize = SizedMmapHeap::ALIGNMENT;

    fn malloc(&self, n: usize) -> *mut u8 {
        SizedMmapHeap::malloc(n)
    }

    fn free(&self, ptr: *mut u8, sz: usize) {
        SizedMmapHeap::free(ptr, sz)
    }

    fn memalign(&self, _alignment: usize, _sz: usize) -> *mut u8 {
        // TODO.rm: find the posix_memalign function...
        core::ptr::null_mut()
    }

    fn get_size(&self, _ptr: *mut u8) -> usize {
        0
    }
}

//   class MmapHeap : public SizedMmapHeap {
pub struct MmapHeap {
    my_map: std::sync::Mutex<MapType>,
}

// #if !defined(_WIN32)

//   private:

//     // Note: we never reclaim memory obtained for MyHeap, even when
//     // this heap is destroyed.
//     class MyHeap : public LockedHeap<PosixLockType, FreelistHeap<BumpAlloc<65536, SizedMmapHeap>>> {};
//       //class MyHeap : public LockedHeap<PosixLockType, FreelistHeap<ZoneHeap<SizedMmapHeap, 65536>>> {};
// TODO.rm: this should be updated to use FreeListHeap
type Bump = BumpAlloc<65536, SizedMmapHeap>;
type MyHeap = LockedFreeListHeap<Bump>;
type MapType = HashMap<*mut u8, usize, ahash::RandomState, MyHeap>;
//     typedef std::unordered_map<void *,
// 			       size_t,
// 			       std::hash<void *>,
// 			       std::equal_to<void *>,
// 			       HL::STLAllocator<std::pair<void * const, size_t>, MyHeap>>
//     mapType;

//     mapType MyMap;
//     PosixLockType MyMapLock;

//   public:
unsafe impl crate::stl::allocator::RawAllocator for MmapHeap {
    const ALLIGNMENT: usize = SizedMmapHeap::ALIGNMENT;
//     enum { Alignment = SizedMmapHeap::Alignment };

//     inline void * malloc (size_t sz) {
    fn malloc(&self, sz: usize) -> *mut u8 {
//       void * ptr = SizedMmapHeap::malloc (sz);
        let ptr = SizedMmapHeap::malloc(sz);
//       MyMapLock.lock();
        self.my_map.lock().unwrap_or_else(|e| e.into_inner()).insert(ptr, sz);
//       MyMap[ptr] = sz;
//       MyMapLock.unlock();
//       assert (reinterpret_cast<size_t>(ptr) % Alignment == 0);
        assert!(ptr as usize % Self::ALLIGNMENT == 0);
//       return const_cast<void *>(ptr);
        ptr
//     }
    }

//     inline size_t getSize (void * ptr) {
    fn get_size(&self, ptr: *mut u8) -> usize {
        self.my_map.lock().unwrap_or_else(|e| e.into_inner()).get(&ptr).copied().unwrap_or_default()
//       MyMapLock.lock();
//       size_t sz = MyMap[ptr];
//       MyMapLock.unlock();
//       return sz;
//     }
    }

// #if 1
//     void free (void * ptr, size_t sz) {
//       SizedMmapHeap::free (ptr, sz);
//     }
// #endif

//     inline void free (void * ptr) {
    fn free(&self, ptr: *mut u8, _sz: usize) {
//       assert (reinterpret_cast<size_t>(ptr) % Alignment == 0);
        assert!(ptr as usize % Self::ALLIGNMENT == 0);
//       MyMapLock.lock();
//       size_t sz = MyMap[ptr];
//       SizedMmapHeap::free (ptr, sz);
//       MyMap.erase (ptr);
//       MyMapLock.unlock();
        let Some(sz) = self.my_map.lock().unwrap_or_else(|e| e.into_inner()).remove(&ptr) else {
            return;
        };
        SizedMmapHeap::free(ptr, sz);
//     }
    }

    fn memalign(&self, _alignment: usize, _sz: usize) -> *mut u8 {
        todo!()
    }
// #endif
//   };
}
// }

unsafe impl crate::stl::Allocator for MmapHeap {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.malloc(layout.size());
        let ptr = NonNull::new(ptr).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.free(ptr.as_ptr(), layout.size());
    }
}

// #endif
#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[test]
    fn can_use_raw_api() {
        let mmap_heap = make_test_map();
        for i in 0..(u16::MAX as usize) {
            let p = mmap_heap.malloc(i);
            mmap_heap.free(p, i);
        }
    }

    fn make_test_map() -> MmapHeap {
        let bump = BumpAlloc::<65536, SizedMmapHeap>::new(SizedMmapHeap);
        let fl = LockedFreeListHeap::new(bump);
        let my_map = HashMap::with_hasher_in(ahash::RandomState::new(), fl);
        MmapHeap {
            my_map: Mutex::new(my_map),
        }
    }
}
