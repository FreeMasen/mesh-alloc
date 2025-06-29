// -*- C++ -*-

// /*

//   Heap Layers: An Extensible Memory Allocation Infrastructure

//   Copyright (C) 2000-2020 by Emery Berger
//   http://www.emeryberger.com
//   emery@cs.umass.edu

//   Heap Layers is distributed under the terms of the Apache 2.0 license.

//   You may obtain a copy of the License at
//   http://www.apache.org/licenses/LICENSE-2.0

// */
// #ifndef HL_MMAPWRAPPER_H
// #define HL_MMAPWRAPPER_H

// #include "utility/arch.h"

// #if defined(_WIN32)
#[cfg(windows)]
// #include <windows.h>
use windows::Win32::System::Memory::{
    MEM_RESET, PAGE_NOACCESS, PAGE_PROTECTION_FLAGS, VirtualAlloc, VirtualProtect,
};
// #else
// // UNIX
// #include <sys/types.h>
// #include <sys/stat.h>
// #include <fcntl.h>
// #include <unistd.h>
// #include <sys/mman.h>
// #if defined(__linux__)
// #include <sys/prctl.h>
// #if !defined(PR_SET_VMA)
// #define PR_SET_VMA 0x53564d41
// #define PR_SET_VMA_ANON_NAME 0
// #endif
// #endif
// #include <map>
// #endif

// #if defined(__SVR4)
// // Not sure why this is needed for Solaris, but there it is.
// extern "C" int madvise (caddr_t, size_t, int);
use rustix::mm::ProtFlags;
pub use rustix::mm::madvise;
// #endif

// #if !defined(HL_MMAP_PROTECTION_MASK)
// #if HL_EXECUTABLE_HEAP
// #define HL_MMAP_PROTECTION_MASK (PROT_READ | PROT_WRITE | PROT_EXEC)
#[cfg(feature = "hl-executable-heap")]
pub const HL_MMAP_PROTECTION_MASK: ProtFlags = ProtFlags::READ
    .union(ProtFlags::WRITE)
    .union(ProtFlags::EXEC);
// #else
// #if !defined(PROT_MAX)
// #define PROT_MAX(p) 0
// #endif
// #define HL_MMAP_PROTECTION_MASK (PROT_READ | PROT_WRITE | PROT_MAX(PROT_READ | PROT_WRITE))
#[cfg(not(feature = "hl-executable-heap"))]
pub const HL_MMAP_PROTECTION_MASK: ProtFlags = ProtFlags::READ.union(ProtFlags::WRITE);

// #endif
// #endif

// #if !defined(MAP_ANONYMOUS) && defined(MAP_ANON)
// #define MAP_ANONYMOUS MAP_ANON
// #endif

// namespace HL {

//   class MmapWrapper {
pub struct MmapWrapper;

//   public:
impl MmapWrapper {
    // #if defined(_WIN32)

    //     // Microsoft Windows has 4K pages aligned to a 64K boundary.
    //     enum { Size = 4 * 1024UL };
    //     enum { Alignment = 64 * 1024UL };
    #[cfg(windows)]
    pub const SIZE: usize = 4 * 1024;
    #[cfg(windows)]
    pub const ALIGNMENT: usize = 64 * 1024;
    // #elif defined(__SVR4)
    //     // Solaris aligns 8K pages to a 64K boundary.
    //     enum { Size = 8 * 1024UL };
    //     enum { Alignment = 64 * 1024UL };

    // #elif defined(HL_APPLE_SILICON)
    //     // macOS on Apple Silicon aligns 16K pages to a 16K boundary.
    //     enum { Size = 16 * 1024UL };
    //     enum { Alignment = 16 * 1024UL };
    #[cfg(all(target_os = "macos", target_arch = "aarch64",))]
    pub const SIZE: usize = 16 * 1024;
    #[cfg(all(target_os = "macos", target_arch = "aarch64",))]
    pub const ALIGNMENT: usize = 16 * 1024;
    // #else
    //     // Linux and most other operating systems align memory to a 4K boundary.
    //     enum { Size = 4 * 1024UL };
    //     enum { Alignment = 4 * 1024UL };
    #[cfg(all(
        not(all(target_os = "macos", target_arch = "aarch64",)),
        target_family = "unix"
    ))]
    pub const SIZE: usize = 4 * 1024;
    #[cfg(all(
        not(all(target_os = "macos", target_arch = "aarch64",)),
        target_family = "unix"
    ))]
    pub const ALIGNMENT: usize = 4 * 1024;
    // #endif

    //     // Release the given range of memory to the OS (without unmapping it).
    //     void release (void * ptr, size_t sz) {
    pub fn release(ptr: *mut u8, sz: usize) {
        //       if ((size_t) ptr % Alignment == 0) {
        if (ptr as usize) % Self::ALIGNMENT != 0 {
            return;
        }
        // 	// Extra sanity check in case the superheap's declared alignment is wrong!
        // #if defined(_WIN32)
        // 	VirtualAlloc (ptr, sz, MEM_RESET, PAGE_NOACCESS);
        #[cfg(windows)]
        VirtualAlloc(Some(ptr.cast()), sz, MEM_RESET, PAGE_NOACCESS);
        // #elif defined(__APPLE__)
        #[cfg(target_vendor = "apple")]
        unsafe {
            // 	madvise (ptr, sz, MADV_DONTNEED);
            let _ = rustix::mm::madvise(ptr.cast(), sz, rustix::mm::Advice::DontNeed);
            // 	madvise (ptr, sz, MADV_FREE);
            // TODO.rm: madv_free is not defined by rustix...
        }
        // #else
        // 	// Assume Unix platform.
        // 	madvise ((caddr_t) ptr, sz, MADV_DONTNEED);
        #[cfg(all(not(target_vendor = "apple"), not(windows),))]
        unsafe {
            let _ = rustix::mm::madvise(ptr.cast(), sz, rustix::mm::Advice::DontNeed);
        }
        // #endif
        //       }
        //     }
    }

    // #if defined(_WIN32)
    #[cfg(windows)]
    //     static void protect (void * ptr, size_t sz) {
    pub fn protect(ptr: *mut u8, sz: usize) {
        //       DWORD oldProtection;
        let mut old_protection = PAGE_PROTECTION_FLAGS(0);
        //       VirtualProtect (ptr, sz, PAGE_NOACCESS, &oldProtection);
        let _ = VirtualProtect(ptr.cast(), sz, PAGE_NOACCESS, &mut old_protection);
        //     }
    }
    #[cfg(windows)]
    //     static void unprotect (void * ptr, size_t sz) {
    pub fn unprotect(ptr: *mut u8, sz: usize) {
        //       DWORD oldProtection;
        //       VirtualProtect (ptr, sz, PAGE_READWRITE, &oldProtection);
        //     }
        todo!()
    }

    #[cfg(windows)]
    //     static void * map (size_t sz) {
    pub fn map(sz: usize) -> *mut u8 {
        //       void * ptr;
        // #if HL_EXECUTABLE_HEAP
        //       const int permflags = PAGE_EXECUTE_READWRITE;
        // #else
        //       const int permflags = PAGE_READWRITE;
        // #endif
        //       ptr = VirtualAlloc(nullptr, sz, MEM_RESERVE | MEM_COMMIT | MEM_TOP_DOWN, permflags);
        //       return  ptr;
        //     }
        todo!()
    }

    //     static void unmap (void * ptr, size_t) {
    #[cfg(windows)]
    pub fn unmap(ptr: *mut u8, _: usize) {
        //       VirtualFree (ptr, 0, MEM_RELEASE);
        //     }
        todo!()
    }

    // #else // UNIX
    #[cfg(unix)]
    //     static void protect (void * ptr, size_t sz) {
    pub fn protect(ptr: *mut u8, sz: usize) {
        use rustix::mm::MprotectFlags;
        unsafe {
            //       mprotect ((char *) ptr, sz, PROT_NONE);
            let _ = rustix::mm::mprotect(ptr.cast(), sz, MprotectFlags::empty());
        }
        //     }
    }

    #[cfg(unix)]
    //     static void unprotect (void * ptr, size_t sz) {
    pub fn unprotect(ptr: *mut u8, sz: usize) {
        use rustix::mm::MprotectFlags;
        unsafe {
            //       mprotect ((char *) ptr, sz, PROT_READ | PROT_WRITE | PROT_EXEC);
            let _ = rustix::mm::mprotect(
                ptr.cast(),
                sz,
                MprotectFlags::READ | MprotectFlags::WRITE | MprotectFlags::EXEC,
            );
            //     }
        }
    }

    #[cfg(unix)]
    //     static void * map (size_t sz) {
    pub fn map(sz: usize) -> *mut u8 {
        use rustix::mm::MapFlags;
        //       if (sz == 0) {
        if sz == 0 {
            // 	return nullptr;
            return core::ptr::null_mut();
            //       }
        }

        //       // Round up the size to a page-sized value.
        //       sz = Size * ((sz + Size - 1) / Size);
        let sz = Self::SIZE * ((sz + Self::SIZE - 1) / Self::SIZE);
        //       void * ptr;
        let mut ptr: *mut u8 = core::ptr::null_mut();
        //       int mapFlag = 0;
        let mut map_flags = MapFlags::empty();
        //       char * startAddress = 0;
        let start_address: *mut u8 = core::ptr::null_mut();

        // #if defined(MAP_ALIGN) && defined(MAP_ANON)
        //       int fd = -1;
        //       startAddress = (char *) Alignment;
        //       mapFlag |= MAP_PRIVATE | MAP_ALIGN | MAP_ANON;
        // #elif defined(MAP_ALIGNED)
        //       int fd = -1;
        //       // On allocations equal or larger than page size, we align it to the log2 boundary
        //       // in those contexts, sometimes (on NetBSD notably) large mappings tends to fail
        //       // without this flag.
        //       size_t alignment = ilog2(sz);
        //       mapFlag |= MAP_PRIVATE | MAP_ANON;
        //       if (alignment >= 12ul)
        //           mapFlag |= MAP_ALIGNED(alignment);
        // #elif !defined(MAP_ANONYMOUS)
        //       static int fd = ::open ("/dev/zero", O_RDWR);
        //       mapFlag |= MAP_PRIVATE;
        // #else
        //       int fd = -1;
        //       //      mapFlag |= MAP_ANONYMOUS | MAP_PRIVATE;
        //       mapFlag |= MAP_ANON | MAP_PRIVATE;
        map_flags |= MapFlags::PRIVATE;
        // #if HL_EXECUTABLE_HEAP
        // #if defined(MAP_JIT)
        //       mapFlag |= MAP_JIT;
        // #endif
        // #endif
        // #endif
        //       ptr = mmap(startAddress, sz, HL_MMAP_PROTECTION_MASK, mapFlag, fd, 0);
        unsafe {
            if let Ok(p) = rustix::mm::mmap_anonymous(
                start_address.cast(),
                sz,
                HL_MMAP_PROTECTION_MASK,
                map_flags,
            ) {
                ptr = p.cast();
            }
        }
        //       #ifdef HL_APPLE_SILICON
        #[cfg(all(target_vendor = "apple", target_arch = "aarch64",))]
        //         #include <mach/vm_page_size.h>
        {
            //         assert(sz % vm_page_size == 0);
            //         assert((reinterpret_cast<uintptr_t>(ptr) % vm_page_size) == 0);
            // TODO.rm: resolve the above
        }
        //       #endif

        //       if (ptr == MAP_FAILED) {
        if ptr as isize == -1 {
            // 	// tprintf::tprintf("MmapWrapper: MAP_FAILED\n");
            // 	return nullptr;
            core::ptr::null_mut()
        //       } else {
        } else {
            // #if HL_NAMED_HEAP
            //         // For every anonymous map (assuming CONFIG_ANON_VMA_NAME is on), the range appears in /proc/<pid>/maps as
            // 	// `560215e4f000-560215e81000 rw-p 00000000 00:00 0 7ffaf89c0000-7ffaf8aa1000 rw-p 00000000 00:00 0 [anon:Heap Layers]`
            // 	//
            //     // By default the region name is `Heap Layers` but can be defined in heaplayers.h via the HL_HEAP_NAME constant.
            //     // If the kernel did not optin the `CONFIG_ANON_VMA_NAME` option, or even with old kernels prior to 5.17
            //     // the following call will be a no-op
            //         (void)prctl(PR_SET_VMA, PR_SET_VMA_ANON_NAME,
            // 			reinterpret_cast<uintptr_t>(ptr),
            // 			sz,
            // 			reinterpret_cast<uintptr_t>(HL_HEAP_NAME));
            // #endif
            // TODO.rm do we need the above?
            // 	return ptr;
            ptr
            //       }
        }
        //     }
    }

    //     static void unmap (void * ptr, size_t sz) {
    pub fn unmap(ptr: *mut u8, sz: usize) {
        //       // Round up the size to a page-sized value.
        //       sz = Size * ((sz + Size - 1) / Size);
        let sz = Self::SIZE * ((sz + Self::SIZE - 1) / Self::SIZE);
        //       munmap ((caddr_t) ptr, sz);
        unsafe {
            let _ = rustix::mm::munmap(ptr.cast(), sz);
        }
        //     }
    }

    // #endif

    //   };

    // }

    // #endif
}
