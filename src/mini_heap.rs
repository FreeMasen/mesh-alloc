use std::{
    os::fd::RawFd,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
    u8,
};

use crate::{
    bitmap::{Bitmap, Bitmapper, RelaxedFixedBitmap},
    internal::{self, MiniHeapId, MiniHeapList},
};

// // -*- mode: c++; c-basic-offset: 2; indent-tabs-mode: nil -*-
// // Copyright 2019 The Mesh Authors. All rights reserved.
// // Use of this source code is governed by the Apache License,
// // Version 2.0, that can be found in the LICENSE file.

// #pragma once
// #ifndef MESH_MINI_HEAP_H
// #define MESH_MINI_HEAP_H

// #include <pthread.h>

// #include <atomic>
// #include <random>

// #include "bitmap.h"
// #include "fixed_array.h"
// #include "internal.h"

// #include "rng/mwc.h"

// #include "heaplayers.h"

// namespace mesh {

// class MiniHeap;

// class Flags {
pub struct Flags(AtomicU32);

impl Flags {
    // private:
    //   DISALLOW_COPY_AND_ASSIGN(Flags);

    //   static inline constexpr uint32_t ATTRIBUTE_ALWAYS_INLINE getSingleBitMask(uint32_t pos) {
    //     return 1UL << pos;
    //   }
    const fn get_single_bit_mask(pos: u32) -> u32 {
        1 << pos
    }
    //   static constexpr uint32_t SizeClassShift = 0;
    const SIZE_CLASS_SHIFT: u32 = 0;
    //   static constexpr uint32_t FreelistIdShift = 6;
    const FREE_LIST_ID_SHIFT: u32 = 6;
    //   static constexpr uint32_t ShuffleVectorOffsetShift = 8;
    const SHUFFLE_VECTOR_OFFSET_SHIFT: u32 = 8;
    //   static constexpr uint32_t MaxCountShift = 16;
    const MAX_COUNT_SHIFT: u32 = 16;
    //   static constexpr uint32_t MeshedOffset = 30;
    const MESHED_OFFSET: u32 = 30;

    //   inline void ATTRIBUTE_ALWAYS_INLINE setMasked(uint32_t mask, uint32_t newVal) {
    fn set_masked(&mut self, mask: u32, new_val: u32) {
        //     uint32_t oldFlags = _flags.load(std::memory_order_relaxed);
        let old = self.0.load(Ordering::Relaxed);
        //     while (!atomic_compare_exchange_weak_explicit(&_flags,
        //                                                   &oldFlags,                   // old val
        //                                                   (oldFlags & mask) | newVal,  // new val
        //                                                   std::memory_order_release,   // success mem model
        //                                                   std::memory_order_relaxed)) {
        //     }
        //   }
        while self
            .0
            .compare_exchange_weak(
                old,
                (old & mask) | new_val,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_err()
        {}
    }
    // public:
    //   explicit Flags(uint32_t maxCount, uint32_t sizeClass, uint32_t svOffset, uint32_t freelistId) noexcept
    pub const fn new(max_count: u32, size_class: u32, sv_offset: u32, free_list_id: u32) -> Self {
        //       : _flags{(maxCount << MaxCountShift) + (sizeClass << SizeClassShift) + (svOffset << ShuffleVectorOffsetShift) +
        //                (freelistId << FreelistIdShift)} {
        let _flags = ((max_count << Self::MAX_COUNT_SHIFT)
            + (size_class << Self::SIZE_CLASS_SHIFT)
            + (sv_offset << Self::SHUFFLE_VECTOR_OFFSET_SHIFT)
            + (free_list_id << Self::FREE_LIST_ID_SHIFT));
        //     d_assert((freelistId & 0x3) == freelistId);
        debug_assert!(free_list_id & 0x3 == free_list_id);
        //     d_assert((sizeClass & ((1 << FreelistIdShift) - 1)) == sizeClass);
        debug_assert!(size_class & ((1 << Self::FREE_LIST_ID_SHIFT) - 1) == size_class);
        //     d_assert(svOffset < 255);
        debug_assert!(sv_offset < 255);
        //     d_assert_msg(sizeClass < 255, "sizeClass: %u", sizeClass);
        debug_assert!(size_class < 255);
        //     d_assert(maxCount <= 256);
        debug_assert!(max_count < 256);
        //     d_assert(this->maxCount() == maxCount);
        debug_assert!(max_count == Self::MAX_COUNT_SHIFT);
        Self(AtomicU32::new(_flags))
    }

    //   inline uint32_t freelistId() const {
    pub fn free_list_id(&self) -> u32 {
        //     return (_flags.load(std::memory_order_seq_cst) >> FreelistIdShift) & 0x3;
        self.0.load(Ordering::SeqCst) >> Self::FREE_LIST_ID_SHIFT & 0x3
        //   }
    }

    //   inline void setFreelistId(uint32_t freelistId) {
    pub fn set_free_list_id(&mut self, id: u32) {
        //     static_assert(list::Max <= (1 << FreelistIdShift), "expected max < 4");
        debug_assert!(internal::list::MAX <= (1 << Self::FREE_LIST_ID_SHIFT));
        //     d_assert(freelistId < list::Max);
        debug_assert!(id < internal::list::MAX as u32);
        //     uint32_t mask = ~(static_cast<uint32_t>(0x3) << FreelistIdShift);
        let mask = !(0x3 << Self::FREE_LIST_ID_SHIFT);
        //     uint32_t newVal = (static_cast<uint32_t>(freelistId) << FreelistIdShift);
        let new_val = (id << Self::FREE_LIST_ID_SHIFT);
        //     setMasked(mask, newVal);
        self.set_masked(mask, new_val);
        //   }
    }

    //   inline uint32_t maxCount() const {
    pub fn max_count(&self) -> u32 {
        //     // XXX: does this assume little endian?
        //     return (_flags.load(std::memory_order_seq_cst) >> MaxCountShift) & 0x1ff;
        (self.0.load(Ordering::SeqCst) << Self::MAX_COUNT_SHIFT) & 0x1ff
        //   }
    }

    //   inline uint32_t sizeClass() const {
    pub fn size_class(&self) -> u32 {
        //     return (_flags.load(std::memory_order_seq_cst) >> SizeClassShift) & 0x3f;
        (self.0.load(Ordering::SeqCst) >> Self::SIZE_CLASS_SHIFT) & 0x3f
        //   }
    }

    //   inline uint8_t svOffset() const {
    pub fn sv_offset(&self) -> u8 {
        //     return (_flags.load(std::memory_order_seq_cst) >> ShuffleVectorOffsetShift) & 0xff;
        u8::try_from((self.0.load(Ordering::SeqCst) >> Self::SHUFFLE_VECTOR_OFFSET_SHIFT) & 0xff)
            .unwrap_or(u8::MAX)
        //   }
    }

    //   inline void setSvOffset(uint8_t off) {
    pub fn set_sv_offet(&mut self, off: u8) {
        //     d_assert(off < 255);
        debug_assert!(off < 255);
        //     uint32_t mask = ~(static_cast<uint32_t>(0xff) << ShuffleVectorOffsetShift);
        let mask = !(0xff << Self::SHUFFLE_VECTOR_OFFSET_SHIFT);
        //     uint32_t newVal = (static_cast<uint32_t>(off) << ShuffleVectorOffsetShift);
        let new_val = ((off as u32) << Self::SHUFFLE_VECTOR_OFFSET_SHIFT);
        //     setMasked(mask, newVal);
        self.set_masked(mask, new_val);
        //   }
    }

    //   inline void setMeshed() {
    pub fn set_meshed(&mut self) {
        //     set(MeshedOffset);
        self.set(Self::MESHED_OFFSET);
        //   }
    }

    //   inline void unsetMeshed() {
    pub fn unset_meshed(&mut self) {
        //     unset(MeshedOffset);
        self.unset(Self::MESHED_OFFSET);
        //   }
    }

    //   inline bool ATTRIBUTE_ALWAYS_INLINE isMeshed() const {
    pub fn is_meshed(&self) -> bool {
        //     return is(MeshedOffset);
        self.is(Self::MESHED_OFFSET)
        //   }
    }

    // private:
    //   inline bool ATTRIBUTE_ALWAYS_INLINE is(size_t offset) const {
    fn is(&self, offset: u32) -> bool {
        //     const auto mask = getSingleBitMask(offset);
        let mask = Self::get_single_bit_mask(offset);
        (self.0.load(Ordering::Acquire) & mask) == mask
        //     return (_flags.load(std::memory_order_acquire) & mask) == mask;
        //   }
    }

    //   inline void set(size_t offset) {
    fn set(&mut self, offset: u32) {
        //     const uint32_t mask = getSingleBitMask(offset);
        let mask = Self::get_single_bit_mask(offset);
        //     uint32_t oldFlags = _flags.load(std::memory_order_relaxed);
        let old_flags = self.0.load(Ordering::Relaxed);
        //     while (!atomic_compare_exchange_weak_explicit(&_flags,
        //                                                   &oldFlags,                  // old val
        //                                                   oldFlags | mask,            // new val
        //                                                   std::memory_order_release,  // success mem model
        //                                                   std::memory_order_relaxed)) {
        //     }
        //   }
        while self
            .0
            .compare_exchange_weak(
                old_flags,
                old_flags | mask,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_err()
        {}
    }

    //   inline void unset(size_t offset) {
    fn unset(&mut self, offset: u32) {
        //     const uint32_t mask = getSingleBitMask(offset);

        //     uint32_t oldFlags = _flags.load(std::memory_order_relaxed);
        //     while (!atomic_compare_exchange_weak_explicit(&_flags,
        //                                                   &oldFlags,                  // old val
        //                                                   oldFlags & ~mask,           // new val
        //                                                   std::memory_order_release,  // success mem model
        //                                                   std::memory_order_relaxed)) {
        //     }
        //   }
        let mask = Self::get_single_bit_mask(offset);
        let old_flags = self.0.load(Ordering::Relaxed);
        while self
            .0
            .compare_exchange_weak(
                old_flags,
                old_flags & !mask,
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_err()
        {}
    }

    //   std::atomic<uint32_t> _flags;
    // };
}

// class MiniHeap {

pub struct MiniHeap {
    //   internal::Bitmap _bitmap;           // 32 bytes 32
    _bitmap: crate::bitmap::Bitmap,
    //   const Span _span;                   // 8        40
    // _span: todo!()
    //   MiniHeapListEntry _freelist{};      // 8        48
    free_list: MiniHeapList,
    //   atomic<pid_t> _current{0};          // 4        52
    _current: AtomicUsize,
    //   Flags _flags;                       // 4        56
    _flags: Flags,
    //   const float _objectSizeReciprocal;  // 4        60
    object_size_reciprocal: f32,
    //   MiniHeapID _nextMeshed{};           // 4        64
    _next_meshed: MiniHeapList,
}
// private:
//   DISALLOW_COPY_AND_ASSIGN(MiniHeap);

impl MiniHeap {
    // public:
    //   MiniHeap(void *arenaBegin, Span span, size_t objectCount, size_t objectSize)
    //       : _bitmap(objectCount),
    //         _span(span),
    //         _flags(objectCount, objectCount > 1 ? SizeMap::SizeClass(objectSize) : 1, 0, list::Attached),
    //         _objectSizeReciprocal(1.0 / (float)objectSize) {
    //     // debug("sizeof(MiniHeap): %zu", sizeof(MiniHeap));

    //     d_assert(_bitmap.inUseCount() == 0);

    //     const auto expectedSpanSize = _span.byteLength();
    //     d_assert_msg(expectedSpanSize == spanSize(), "span size %zu == %zu (%u, %u)", expectedSpanSize, spanSize(),
    //                  maxCount(), this->objectSize());

    //     // d_assert_msg(spanSize == static_cast<size_t>(_spanSize), "%zu != %hu", spanSize, _spanSize);
    //     // d_assert_msg(objectSize == static_cast<size_t>(objectSize()), "%zu != %hu", objectSize, _objectSize);

    //     d_assert(!_nextMeshed.hasValue());

    //     // debug("new:\n");
    //     // dumpDebug();
    //   }

    //   ~MiniHeap() {
    //     // debug("destruct:\n");
    //     // dumpDebug();
    //   }

    //   inline Span span() const {
    //     return _span;
    //   }
    //   void printOccupancy() const {
    pub fn print_occupancy(&self) {
        println!(
            r#"{{"name": "{name}", "object-size": {obj_size}, "length": {length}, "mesh-count": {mesh_count}, "bitmap": "{bitmap:?}"}}"#,
            name = self._current.load(Ordering::Relaxed),
            obj_size = self.object_size(),
            length = self.max_count(),
            mesh_count = self.mesh_count(),
            bitmap = self._bitmap,
        );
        //     mesh::debug("{\"name\": \"%p\", \"object-size\": %d, \"length\": %d, \"mesh-count\": %d, \"bitmap\": \"%s\"}\n",
        //                 this, objectSize(), maxCount(), meshCount(), _bitmap.to_string(maxCount()).c_str());
        //   }
    }

    //   inline void ATTRIBUTE_ALWAYS_INLINE free(void *arenaBegin, void *ptr) {
    pub fn free(&mut self) {
        //     // the logic in globalFree is
        //     // updated to allow the 'race' between lock-free freeing and
        //     // meshing
        //     // d_assert(!isMeshed());
        //     const ssize_t off = getOff(arenaBegin, ptr);
        //     if (unlikely(off < 0)) {
        //       d_assert(false);
        //       return;
        //     }

        //     freeOff(off);
        //   }
    }

    //   inline bool clearIfNotFree(void *arenaBegin, void *ptr) {
    pub fn clear_if_not_free(&mut self) -> bool {
        //     const ssize_t off = getOff(arenaBegin, ptr);
        //     const auto notWasSet = _bitmap.unset(off);
        //     const auto wasSet = !notWasSet;
        //     return wasSet;
        todo!()
        //   }
    }

    //   inline void ATTRIBUTE_ALWAYS_INLINE freeOff(size_t off) {
    pub fn free_off(&mut self, off: usize) {
        //     d_assert_msg(_bitmap.isSet(off), "MiniHeap(%p) expected bit %zu to be set (svOff:%zu)", this, off, svOffset());
        //     _bitmap.unset(off);
        todo!()
        //   }
    }

    //   /// Copies (for meshing) the contents of src into our span.
    //   inline void consume(const void *arenaBegin, MiniHeap *src) {
    pub fn consume(&mut self) {
        //     // this would be bad
        //     d_assert(src != this);
        //     d_assert(objectSize() == src->objectSize());

        //     src->setMeshed();
        //     const auto srcSpan = src->getSpanStart(arenaBegin);
        //     const auto objectSize = this->objectSize();

        //     // this both avoids the need to call `freeOff` in the loop
        //     // below, but it ensures we will be able to check for bitmap
        //     // setting races in GlobalHeap::freeFor
        //     const auto srcBitmap = src->takeBitmap();

        //     // for each object in src, copy it to our backing span + update
        //     // our bitmap and in-use count
        //     for (auto const &off : srcBitmap) {
        //       d_assert(off < maxCount());
        //       d_assert(!_bitmap.isSet(off));

        //       void *srcObject = reinterpret_cast<void *>(srcSpan + off * objectSize);
        //       // need to ensure we update the bitmap and in-use count
        //       void *dstObject = mallocAt(arenaBegin, off);
        //       // debug("meshing: %zu (%p <- %p)\n", off, dstObject, srcObject);
        //       d_assert(dstObject != nullptr);
        //       memcpy(dstObject, srcObject, objectSize);
        //       // debug("\t'%s'\n", dstObject);
        //       // debug("\t'%s'\n", srcObject);
        //     }

        //     trackMeshedSpan(GetMiniHeapID(src));
        //   }
    }

    //   inline size_t spanSize() const {
    pub fn span_size(&self) -> usize {
        //     return _span.byteLength();
        todo!()
        //   }
    }

    //   inline uint32_t ATTRIBUTE_ALWAYS_INLINE maxCount() const {
    pub fn max_count(&self) -> u32 {
        //     return _flags.maxCount();
        self._flags.max_count()
        //   }
    }

    //   inline bool ATTRIBUTE_ALWAYS_INLINE isLargeAlloc() const {
    pub fn is_large_alloc(&self) -> bool {
        //     return maxCount() == 1;
        self.max_count() == 1
        //   }
    }

    //   inline size_t objectSize() const {
    pub fn object_size(&self) -> usize {
        //     if (likely(!isLargeAlloc())) {
        if !self.is_large_alloc() {
            //       // this doesn't handle all the corner cases of roundf(3),
            //       // but it does work for all of our small object size classes
            (1.0 / (self.object_size_reciprocal + 0.5)) as usize
        //       return static_cast<size_t>(1 / _objectSizeReciprocal + 0.5);
        //     } else {
        } else {
            //       return _span.length * kPageSize;
            todo!()
            //     }
        }
        //   }
    }

    //   inline int sizeClass() const {
    pub fn size_class(&self) -> u32 {
        //     return _flags.sizeClass();
        self._flags.size_class()
        //   }
    }

    //   inline uintptr_t getSpanStart(const void *arenaBegin) const {
    pub fn get_span_start(&self) -> usize {
        //     const auto beginval = reinterpret_cast<uintptr_t>(arenaBegin);
        //     return beginval + _span.offset * kPageSize;
        todo!()
        //   }
    }

    //   inline bool ATTRIBUTE_ALWAYS_INLINE isEmpty() const {
    pub fn is_empty(&self) -> bool {
        //     return _bitmap.inUseCount() == 0;
        self._bitmap.in_use_ct() == 0
        //   }
    }

    //   inline bool ATTRIBUTE_ALWAYS_INLINE isFull() const {
    pub fn is_full(&self) -> bool {
        //     return _bitmap.inUseCount() == maxCount();
        self.in_use_ct() == self.max_count()
        //   }
    }

    //   inline uint32_t ATTRIBUTE_ALWAYS_INLINE inUseCount() const {
    pub fn in_use_ct(&self) -> u32 {
        //     return _bitmap.inUseCount();
        self._bitmap.in_use_ct() as u32
        //   }
    }

    //   inline size_t bytesFree() const {
    pub fn bytes_free(&self) -> u32 {
        //     return (maxCount() - inUseCount()) * objectSize();
        (self.max_count() - self.in_use_ct()) * self.object_size() as u32
        //   }
    }

    //   inline void setMeshed() {
    pub fn set_meshed(&mut self) {
        //     _flags.setMeshed();
        self._flags.set_meshed();
        //   }
    }

    //   inline void setAttached(pid_t current, MiniHeapListEntry *listHead) {
    pub fn set_attached(&mut self, current: RawFd) {
        //     // mesh::debug("MiniHeap(%p:%5zu): current <- %u\n", this, objectSize(), current);
        //     _current.store(current, std::memory_order::memory_order_release);
        //     if (listHead != nullptr) {
        //       _freelist.remove(listHead);
        //     }
        //     this->setFreelistId(list::Attached);
        todo!()
        //   }
    }

    //   inline uint8_t svOffset() const {
    pub fn sv_offset(&self) -> u8 {
        //     return _flags.svOffset();
        self._flags.sv_offset()
        //   }
    }

    //   inline void setSvOffset(uint8_t off) {
    pub fn set_sv_offset(&mut self, off: u8) {
        //     // debug("MiniHeap(%p) SET svOff:%zu)", this, off);
        //     _flags.setSvOffset(off);
        self._flags.set_sv_offet(off);
        //   }
    }

    //   inline uint8_t freelistId() const {
    pub fn free_list_id(&self) -> u32 {
        //     return _flags.freelistId();
        self._flags.free_list_id()
        //   }
    }

    //   inline void setFreelistId(uint8_t id) {
    pub fn set_free_list_id(&mut self, id: u32) {
        //     _flags.setFreelistId(id);
        self._flags.set_free_list_id(id);
        //   }
    }

    //   inline pid_t current() const {
    pub fn current(&self) -> RawFd {
        //     return _current.load(std::memory_order::memory_order_acquire);
        self._current.load(Ordering::Acquire) as _
        //   }
    }

    //   inline void unsetAttached() {
    pub fn unset_attached(&mut self) {
        //     // mesh::debug("MiniHeap(%p:%5zu): current <- UNSET\n", this, objectSize());
        //     _current.store(0, std::memory_order::memory_order_release);
        self._current.store(0, Ordering::Release);
        //   }
    }

    //   inline bool isAttached() const {
    pub fn is_attached(&self) -> bool {
        //     return current() != 0;
        self.current() != 0
        //   }
    }

    //   inline bool ATTRIBUTE_ALWAYS_INLINE isMeshed() const {
    pub fn is_meshed(&self) -> bool {
        //     return _flags.isMeshed();
        self._flags.is_meshed()
        //   }
    }

    //   inline bool ATTRIBUTE_ALWAYS_INLINE hasMeshed() const {
    pub fn has_meshed(&self) -> bool {
        //     return _nextMeshed.hasValue();
        todo!()
        //   }
    }

    //   inline bool isMeshingCandidate() const {
    pub fn is_meshing_candidate(&self) -> bool {
        //     return !isAttached() && objectSize() < kPageSize;
        !self.is_attached() && self.object_size() < crate::PAGE_SIZE
        //   }
    }

    //   /// Returns the fraction full (in the range [0, 1]) that this miniheap is.
    //   inline double fullness() const {
    pub fn fullness(&self) -> f64 {
        //     return static_cast<double>(inUseCount()) / static_cast<double>(maxCount());
        self.in_use_ct() as f64 / self.max_count() as f64
        //   }
    }

    //   internal::RelaxedFixedBitmap takeBitmap() {
    pub fn take_bitmap(&mut self) -> RelaxedFixedBitmap {
        //     const auto capacity = this->maxCount();
        //     internal::RelaxedFixedBitmap zero{capacity};
        let zero = Bitmap::new_relaxed_fixed();
        //     internal::RelaxedFixedBitmap result{capacity};
        let mut result = Bitmap::new_relaxed_fixed();
        //     _bitmap.setAndExchangeAll(result.mut_bits(), zero.bits());
        self._bitmap
            .set_and_exchange_all(result.bits_mut(), zero.bits());
        //     return result;
        result
        //   }
    }

    //   const internal::Bitmap &bitmap() const {
    pub fn bitmap(&self) -> &Bitmap {
        //     return _bitmap;
        &self._bitmap
        //   }
    }

    //   internal::Bitmap &writableBitmap() {
    pub fn writable_bitmap(&mut self) -> &mut Bitmap {
        &mut self._bitmap
        //     return _bitmap;
        //   }
    }

    //   void trackMeshedSpan(MiniHeapID id) {
    pub fn track_meshed_span(id: MiniHeapId) {
        //     hard_assert(id.hasValue());
        assert!(id.has_value());
        //     if (!_nextMeshed.hasValue()) {
        //       _nextMeshed = id;
        //     } else {
        //       GetMiniHeap(_nextMeshed)->trackMeshedSpan(id);
        //     }
        todo!()
        //   }
    }

    // public:
    //   template <class Callback>
    //   inline void forEachMeshed(Callback cb) const {
    pub fn for_each_meshed(&self, cb: impl Fn(&Self) -> bool) {
        //     if (cb(this))
        if cb(self) {
            //       return;
            return;
        }

        //     if (_nextMeshed.hasValue()) {
        //       const auto mh = GetMiniHeap(_nextMeshed);
        //       mh->forEachMeshed(cb);
        //     }
        //   }
        todo!()
    }

    //   template <class Callback>
    //   inline void forEachMeshed(Callback cb) {
    //     if (cb(this))
    //       return;

    //     if (_nextMeshed.hasValue()) {
    //       auto mh = GetMiniHeap(_nextMeshed);
    //       mh->forEachMeshed(cb);
    //     }
    //   }

    //   bool isRelated(MiniHeap *other) const {
    pub fn is_related(other: &MiniHeap) -> bool {
        //     auto otherFound = false;
        //     this->forEachMeshed([&](const MiniHeap *eachMh) {
        //       const auto found = eachMh == other;
        //       otherFound = found;
        //       return found;
        //     });
        //     return otherFound;
        todo!()
        //   }
    }

    //   size_t meshCount() const {
    pub fn mesh_count(&self) -> usize {
        //     size_t count = 0;
        let mut count = 0;
        //     const MiniHeap *mh = this;
        //     while (mh != nullptr) {
        //       count++;

        //       auto next = mh->_nextMeshed;
        //       mh = next.hasValue() ? GetMiniHeap(next) : nullptr;
        //     }

        //     return count;
        todo!()
        //   }
    }

    //   MiniHeapListEntry *getFreelist() {
    pub fn get_free_list(&self) -> &MiniHeapList {
        //     return &_freelist;
        todo!()
        //   }
    }

    //   /// public for meshTest only
    //   inline void *mallocAt(const void *arenaBegin, size_t off) {
    pub fn malloc_at(&mut self) {
        //     if (!_bitmap.tryToSet(off)) {
        //       mesh::debug("%p: MA %u", this, off);
        //       dumpDebug();
        //       return nullptr;
        //     }

        //     return ptrFromOffset(arenaBegin, off);
        todo!()
        //   }
    }

    //   inline void *ptrFromOffset(const void *arenaBegin, size_t off) {
    pub fn ptr_from_offset(&self) {
        //     return reinterpret_cast<void *>(getSpanStart(arenaBegin) + off * objectSize());
        todo!()
        //   }
    }

    //   inline bool operator<(MiniHeap *&rhs) noexcept {
    //     return this->inUseCount() < rhs->inUseCount();
    //   }

    //   void dumpDebug() const {
    pub fn dump_debug(&self) {
        //     const auto heapPages = spanSize() / HL::CPUInfo::PageSize;
        let heap_pages = self.span_size() / crate::PAGE_SIZE;
        //     const size_t inUseCount = this->inUseCount();
        let in_use_count = self.in_use_ct();
        //     const size_t meshCount = this->meshCount();
        let mesh_count = self.mesh_count();
        //     mesh::debug(
        //         "MiniHeap(%p:%5zu): %3zu objects on %2zu pages (inUse: %zu, spans: %zu)\t%p-%p\tFreelist{prev:%u, next:%u}\n",
        //         this, objectSize(), maxCount(), heapPages, inUseCount, meshCount, _span.offset * kPageSize,
        //         _span.offset * kPageSize + spanSize(), _freelist.prev(), _freelist.next());
        //     mesh::debug("\t%s\n", _bitmap.to_string(maxCount()).c_str());
        todo!()
        //   }
    }

    //   // this only works for unmeshed miniheaps
    //   inline uint8_t ATTRIBUTE_ALWAYS_INLINE getUnmeshedOff(const void *arenaBegin, void *ptr) const {
    pub fn get_unmeshed_off(&self) -> u8 {
        //     const auto ptrval = reinterpret_cast<uintptr_t>(ptr);

        //     uintptr_t span = reinterpret_cast<uintptr_t>(arenaBegin) + _span.offset * kPageSize;
        //     d_assert(span != 0);

        //     const size_t off = (ptrval - span) * _objectSizeReciprocal;
        //     d_assert(off < maxCount());

        //     return off;
        todo!()
        //   }
    }

    //   inline uint8_t ATTRIBUTE_ALWAYS_INLINE getOff(const void *arenaBegin, void *ptr) const {
    pub fn get_off(&self) -> u8 {
        //     const auto span = spanStart(reinterpret_cast<uintptr_t>(arenaBegin), ptr);
        //     d_assert(span != 0);
        //     const auto ptrval = reinterpret_cast<uintptr_t>(ptr);

        //     const size_t off = (ptrval - span) * _objectSizeReciprocal;
        //     d_assert(off < maxCount());

        //     return off;
        todo!()
        //   }
    }

    // protected:
    //   inline uintptr_t ATTRIBUTE_ALWAYS_INLINE spanStart(uintptr_t arenaBegin, void *ptr) const {
    pub(crate) fn span_start(&self) -> usize {
        //     const auto ptrval = reinterpret_cast<uintptr_t>(ptr);
        //     const auto len = _span.byteLength();

        //     // manually unroll loop once to capture the common case of
        //     // un-meshed miniheaps
        //     uintptr_t spanptr = arenaBegin + _span.offset * kPageSize;
        //     if (likely(spanptr <= ptrval && ptrval < spanptr + len)) {
        //       return spanptr;
        //     }

        //     return spanStartSlowpath(arenaBegin, ptrval);
        todo!()
        //   }
    }

    //   uintptr_t ATTRIBUTE_NEVER_INLINE spanStartSlowpath(uintptr_t arenaBegin, uintptr_t ptrval) const {
    pub fn span_start_slow_path(&self) -> usize {
        //     const auto len = _span.byteLength();
        //     uintptr_t spanptr = 0;

        //     const MiniHeap *mh = this;
        //     while (true) {
        //       if (unlikely(!mh->_nextMeshed.hasValue())) {
        //         abort();
        //       }

        //       mh = GetMiniHeap(mh->_nextMeshed);

        //       const uintptr_t meshedSpanptr = arenaBegin + mh->span().offset * kPageSize;
        //       if (meshedSpanptr <= ptrval && ptrval < meshedSpanptr + len) {
        //         spanptr = meshedSpanptr;
        //         break;
        //       }
        //     };

        //     return spanptr;
        todo!()
        //   }
    }

    //   internal::Bitmap _bitmap;           // 32 bytes 32
    //   const Span _span;                   // 8        40
    //   MiniHeapListEntry _freelist{};      // 8        48
    //   atomic<pid_t> _current{0};          // 4        52
    //   Flags _flags;                       // 4        56
    //   const float _objectSizeReciprocal;  // 4        60
    //   MiniHeapID _nextMeshed{};           // 4        64
    // };

    // typedef FixedArray<MiniHeap, 63> MiniHeapArray;

    // static_assert(sizeof(pid_t) == 4, "pid_t not 32-bits!");
    // static_assert(sizeof(mesh::internal::Bitmap) == 32, "Bitmap too big!");
    // static_assert(sizeof(MiniHeap) == 64, "MiniHeap too big!");
    // static_assert(sizeof(MiniHeap) == kMiniHeapSize, "MiniHeap size mismatch");
    // static_assert(sizeof(MiniHeapArray) == 64 * sizeof(void *), "MiniHeapArray too big!");
    // }  // namespace mesh

    // #endif  // MESH_MINI_HEAP_H
}
