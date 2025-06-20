use cordyceps::{
    list::{self as cordy_list, List},
    Linked,
};
use std::{
    ffi::c_void,
    pin::Pin,
    ptr::{self, NonNull},
    sync::Arc,
    thread,
};
// // -*- mode: c++; c-basic-offset: 2; indent-tabs-mode: nil -*-
// // Copyright 2019 The Mesh Authors. All rights reserved.
// // Use of this source code is governed by the Apache License,
// // Version 2.0, that can be found in the LICENSE file.

// #pragma once
// #ifndef MESH_INTERNAL_H
// #define MESH_INTERNAL_H

// #ifdef __linux__
// #include <sys/syscall.h>
// #include <unistd.h>
// #endif

// #include <atomic>
// #include <unordered_set>

// #include <signal.h>
// #include <stdint.h>

// #include "common.h"
// #include "rng/mwc.h"

// // never allocate executable heap
// #define HL_MMAP_PROTECTION_MASK (PROT_READ | PROT_WRITE)
// #define MALLOC_TRACE 0

// #include "heaplayers.h"

// #include "partitioned_heap.h"

// #ifdef __linux__
// inline pid_t gettid(void) {
//   return syscall(__NR_gettid);
// }
// #endif
// #ifdef __APPLE__
// inline pid_t gettid(void) {
//   uint64_t tid;
//   pthread_threadid_np(NULL, &tid);
//   return static_cast<uint32_t>(tid);
// }
// #endif

// namespace mesh {

use std::os::fd::{AsRawFd, OwnedFd, RawFd};

use crate::{mini_heap::MiniHeap, PAGE_SIZE};

const SPAN_CLASS_COUNT: u32 = 256;

// namespace internal {
// enum PageType {
pub enum PageType {
    //   Clean = 0,
    Clean = 0,
    //   Dirty = 1,
    Dirty = 1,
    //   Meshed = 2,
    Meshed = 2,
    //   Unknown = 3,
    Unknown = 3,
    // };
}
// }  // namespace internal

// class MiniHeapID {
#[derive(Debug, PartialEq, Eq)]
pub struct MiniHeapId(RawFd);

// public:
impl MiniHeapId {
    //   MiniHeapID() noexcept : _id{0} {
    //   }

    //   explicit constexpr MiniHeapID(uint32_t id) : _id{id} {
    //   }
    pub const fn new(id: RawFd) -> Self {
        Self(id)
    }

    //   MiniHeapID(const MiniHeapID &rhs) = default;

    //   constexpr MiniHeapID(MiniHeapID &&rhs) = default;

    //   MiniHeapID &operator=(const MiniHeapID &rhs) = default;

    //   bool operator!=(const MiniHeapID &rhs) const {
    //     return _id != rhs._id;
    //   }

    //   bool operator==(const MiniHeapID &rhs) const {
    //     return _id == rhs._id;
    //   }

    //   bool hasValue() const {
    pub const fn has_value(&self) -> bool {
        //     return _id != 0;
        self.0 != 0
        //   }
    }

    //   uint32_t value() const {
    pub const fn value(&self) -> RawFd {
        //     return _id;
        self.0
        //   }
    }

    // private:
    //   uint32_t _id;
    // };
}

impl AsRawFd for MiniHeapId {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

// namespace list {
// static constexpr MiniHeapID Head{UINT32_MAX};
pub mod list {
    use crate::internal::MiniHeapId;

    pub static HEAD: MiniHeapId = MiniHeapId::new(u32::MAX as _);
    // // TODO: add a type to represent this
    // static constexpr uint8_t Full = 0;
    pub const FULL: u8 = 0;
    // static constexpr uint8_t Partial = 1;
    pub const PARTIAL: u8 = 1;
    // static constexpr uint8_t Empty = 2;
    pub const EMPTY: u8 = 2;
    // static constexpr uint8_t Attached = 3;
    pub const ATTACHED: u8 = 3;
    // static constexpr uint8_t Max = 4;
    pub const MAX: u8 = 4;
    // }  // namespace list
}
// class MiniHeap;
// MiniHeap *GetMiniHeap(const MiniHeapID id);
// MiniHeapID GetMiniHeapID(const MiniHeap *mh);

// typedef std::array<MiniHeap *, kMaxSplitListSize> SplitArray;
// typedef std::array<std::pair<MiniHeap *, MiniHeap *>, kMaxMergeSets> MergeSetArray;

pub struct MiniHeapList(List<MiniHeapListEntry>);

pub struct MiniHeapListEntry {
    id: MiniHeapId,
    heap: MiniHeap,
    links: cordyceps::list::Links<Self>,
}

unsafe impl cordyceps::Linked<cordyceps::list::Links<Self>> for MiniHeapListEntry {
    type Handle = Pin<Box<Self>>;

    fn into_ptr(r: Self::Handle) -> std::ptr::NonNull<Self> {
        unsafe { NonNull::from(Box::leak(Pin::into_inner_unchecked(r))) }
    }

    unsafe fn from_ptr(ptr: std::ptr::NonNull<Self>) -> Self::Handle {
        // Safety: if this function is only called by the linked list
        // implementation (and it is not intended for external use), we can
        // expect that the `NonNull` was constructed from a reference which
        // was pinned.
        //
        // If other callers besides `List`'s internals were to call this on
        // some random `NonNull<Entry>`, this would not be the case, and
        // this could be constructing an erroneous `Pin` from a referent
        // that may not be pinned!
        Pin::new_unchecked(Box::from_raw(ptr.as_ptr()))
    }

    unsafe fn links(ptr: std::ptr::NonNull<Self>) -> NonNull<cordy_list::Links<Self>> {
        // Using `ptr::addr_of_mut!` permits us to avoid creating a temporary
        // reference without using layout-dependent casts.
        let links = ptr::addr_of_mut!((*ptr.as_ptr()).links);

        // `NonNull::new_unchecked` is safe to use here, because the pointer that
        // we offset was not null, implying that the pointer produced by offsetting
        // it will also not be null.
        NonNull::new_unchecked(links)
    }
}
// template <typename Object, typename ID>
// class ListEntry {
// public:
//   typedef ListEntry<Object, ID> Entry;

//   ListEntry() noexcept : _prev{}, _next{} {
//   }

//   explicit constexpr ListEntry(ID prev, ID next) : _prev{prev}, _next{next} {
//   }

//   ListEntry(const ListEntry &rhs) = default;
//   ListEntry &operator=(const ListEntry &) = default;
impl MiniHeapList {
    //   inline bool empty() const {
    pub fn empty(&self) -> bool {
        //     return !_prev.hasValue() || !_next.hasValue();
        self.0.is_empty()
        //   }
    }

    //   inline ID next() const {
    //     return _next;
    //   }

    //   inline ID prev() const {
    //     return _prev;
    //   }

    //   inline void setNext(ID next) {
    //     // mesh::debug("%p.setNext: %u\n", this, next);
    //     _next = next;
    //   }

    //   inline void setPrev(ID prev) {
    //     _prev = prev;
    //   }

    //   // add calls remove for you
    //   void add(Entry *listHead, uint8_t listId, ID selfId, Object *newEntry) {
    pub fn add(&mut self, list_id: u8, self_id: MiniHeapId, new_entry: MiniHeap) {
        //     const uint8_t oldId = newEntry->freelistId();
        //     d_assert(oldId != listId);
        //     d_assert(!newEntry->isLargeAlloc());

        //     Entry *newEntryFreelist = newEntry->getFreelist();
        //     if (likely(newEntryFreelist->next().hasValue())) {
        //       // we will be part of a list every time except the first time add is called after alloc
        //       newEntryFreelist->remove(listHead);
        //     }

        //     newEntry->setFreelistId(listId);

        //     const ID newEntryId = GetMiniHeapID(newEntry);
        //     ID lastId = prev();
        //     Entry *prevList = nullptr;
        //     if (lastId == list::Head) {
        //       prevList = this;
        //     } else {
        //       Object *last = GetMiniHeap(lastId);
        //       prevList = last->getFreelist();
        //     }
        //     prevList->setNext(newEntryId);
        //     *newEntry->getFreelist() = ListEntry{lastId, selfId};
        //     this->setPrev(newEntryId);
        //   }
    }

    //   void remove(Entry *listHead) {
    //     ID prevId = _prev;
    //     ID nextId = _next;
    //     // we may have just been created + not added to any freelist yet
    //     if (!prevId.hasValue() || !nextId.hasValue()) {
    //       return;
    //     }
    //     Entry *prev = nullptr;
    //     if (prevId == list::Head) {
    //       prev = listHead;
    //     } else {
    //       Object *mh = GetMiniHeap(prevId);
    //       d_assert(mh != nullptr);
    //       prev = mh->getFreelist();
    //     }
    //     Entry *next = nullptr;
    //     if (nextId == list::Head) {
    //       next = listHead;
    //     } else {
    //       Object *mh = GetMiniHeap(nextId);
    //       d_assert(mh != nullptr);
    //       next = mh->getFreelist();
    //     }

    //     prev->setNext(nextId);
    //     next->setPrev(prevId);
    //     _prev = MiniHeapID{};
    //     _next = MiniHeapID{};
    //   }

    // private:
    //   MiniHeapID _prev{};
    //   MiniHeapID _next{};
    // };
}

// typedef ListEntry<MiniHeap, MiniHeapID> MiniHeapListEntry;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct Span {
    // typedef uint32_t Offset;
    offset: u32,
    // typedef uint32_t Length;
    length: u32,
}
// struct Span {
//   // offset and length are in pages
//   explicit Span(Offset _offset, Length _length) : offset(_offset), length(_length) {
//   }
impl Span {
    //   Span(const Span &rhs) : offset(rhs.offset), length(rhs.length) {
    //   }
    fn new(offset: u32, length: u32) -> Self {
        Self { offset, length }
    }
    //   Span &operator=(const Span &rhs) {
    //     offset = rhs.offset;
    //     length = rhs.length;
    //     return *this;
    //   }

    //   Span(Span &&rhs) : offset(rhs.offset), length(rhs.length) {
    //   }

    //   bool empty() const {
    //     return length == 0;
    //   }
    pub fn empty(&self) -> bool {
        self.length == 0
    }

    //   // reduce the size of this span to pageCount, return another span
    //   // with the rest of the pages.
    //   Span splitAfter(Length pageCount) {
    pub fn split_after(&mut self, page_count: u32) -> Span {
        //     d_assert(pageCount <= length);
        debug_assert!(page_count <= self.length);
        //     auto restPageCount = length - pageCount;
        let rest_page_count = self.length - page_count;
        //     length = pageCount;
        self.length = page_count;
        //     return Span(offset + pageCount, restPageCount);
        Span::new(self.offset + page_count, rest_page_count)
        //   }
    }

    //   uint32_t spanClass() const {
    pub fn span_class(&self) -> u32 {
        //     return std::min(length, kSpanClassCount) - 1;
        self.length.min(SPAN_CLASS_COUNT) - 1
        //   }
    }

    //   size_t byteLength() const {
    pub fn byte_length(&self) -> usize {
        //     return length * kPageSize;
        self.length as usize * PAGE_SIZE
        //   }
    }

    //   inline bool operator==(const Span &rhs) {
    //     return offset == rhs.offset && length == rhs.length;
    //   }

    //   inline bool operator!=(const Span &rhs) {
    //     return !(*this == rhs);
    //   }

    //   Offset offset;
    //   Length length;
    // };
}

// // keep in-sync with the version in plasma/mesh.h
// enum BitType {
pub enum BitType {
    //   MESH_BIT_0,
    MeshBit0,
    //   MESH_BIT_1,
    MeshBit1,
    //   MESH_BIT_2,
    MeshBit2,
    //   MESH_BIT_3,
    MeshBit3,
    //   MESH_BIT_COUNT,
    MeshBitCount,
    // };
}

// namespace internal {

// // return the kernel's perspective on our proportional set size
// size_t measurePssKiB();

// inline void *MaskToPage(const void *ptr) {
fn mask_to_page(ptr: *const c_void) -> *const c_void {
    //   const auto ptrval = reinterpret_cast<uintptr_t>(ptr);
    let ptr_val = ptr.addr();
    //   return reinterpret_cast<void *>(ptrval & (uintptr_t) ~(CPUInfo::PageSize - 1));
    ptr.map_addr(|ptr_val| ptr_val & !(PAGE_SIZE - 1))
    // }
}

// // efficiently copy data from srcFd to dstFd
// int copyFile(int dstFd, int srcFd, off_t off, size_t sz);
pub fn copy_file(dst_fd: RawFd, arc_fd: RawFd, off: usize, size: usize) -> u32 {
    todo!()
}

struct Heap<T> {
    inner: T,
}

// // for mesh-internal data structures, like heap metadata
// class Heap : public ExactlyOneHeap<LockedHeap<PosixLockType, PartitionedHeap>> {
// private:
//   typedef ExactlyOneHeap<LockedHeap<PosixLockType, PartitionedHeap>> SuperHeap;

// public:
//   Heap() : SuperHeap() {
//     static_assert(Alignment % 16 == 0, "16-byte alignment");
//   }
// };

// // make a shared pointer allocated from our internal heap that will
// // also free to the internal heap when all references have been
// // dropped.
// template <typename T, class... Args>
// inline std::shared_ptr<T> make_shared(Args &&...args) {
//   // FIXME: somehow centralize this static.
//   static STLAllocator<T, Heap> heap;
//   return std::allocate_shared<T, STLAllocator<T, Heap>, Args...>(heap, std::forward<Args>(args)...);
// }

// extern STLAllocator<char, Heap> allocator;

// template <typename K, typename V>
// using unordered_map = std::unordered_map<K, V, hash<K>, equal_to<K>, STLAllocator<pair<const K, V>, Heap>>;

// template <typename K>
// using unordered_set = std::unordered_set<K, hash<K>, equal_to<K>, STLAllocator<K, Heap>>;

// template <typename K, typename V>
// using map = std::map<K, V, std::less<K>, STLAllocator<pair<const K, V>, Heap>>;

// typedef std::basic_string<char, std::char_traits<char>, STLAllocator<char, Heap>> string;

// template <typename T>
// using vector = std::vector<T, STLAllocator<T, Heap>>;

// // https://stackoverflow.com/questions/529831/returning-the-greatest-key-strictly-less-than-the-given-key-in-a-c-map
// template <typename Map>
// typename Map::const_iterator greatest_leq(Map const &m, typename Map::key_type const &k) {
//   typename Map::const_iterator it = m.upper_bound(k);
//   if (it != m.begin()) {
//     return --it;
//   }
//   return m.end();
// }

// // https://stackoverflow.com/questions/529831/returning-the-greatest-key-strictly-less-than-the-given-key-in-a-c-map
// template <typename Map>
// typename Map::iterator greatest_leq(Map &m, typename Map::key_type const &k) {
//   typename Map::iterator it = m.upper_bound(k);
//   if (it != m.begin()) {
//     return --it;
//   }
//   return m.end();
// }

// // based on LLVM's libcxx std::shuffle
// template <class _RandomAccessIterator, class _RNG>
// inline void mwcShuffle(_RandomAccessIterator __first, _RandomAccessIterator __last, _RNG &__rng) {
//   typedef typename iterator_traits<_RandomAccessIterator>::difference_type difference_type;

//   difference_type __d = __last - __first;
//   if (__d > 1) {
//     for (--__last, --__d; __first < __last; ++__first, --__d) {
//       difference_type __i = __rng.inRange(0, __d);
//       if (__i != difference_type(0))
//         swap(*__first, *(__first + __i));
//     }
//   }
// }
// }  // namespace internal
// }  // namespace mesh

// #endif  // MESH_INTERNAL_H
