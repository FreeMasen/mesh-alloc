use std::{sync::{atomic::{AtomicUsize, Ordering}, Mutex}, time::{Duration, Instant}};

use crate::{common::{power_of_two::byte_size_for_class, DEFAULT_MESH_PERIOD, MESH_PERIOD_MS, NUM_BINS}, internal::{list::HEAD, MiniHeapListEntry}, mini_heap::MiniHeap};
// // -*- mode: c++; c-basic-offset: 2; indent-tabs-mode: nil -*-
// // Copyright 2019 The Mesh Authors. All rights reserved.
// // Use of this source code is governed by the Apache License,
// // Version 2.0, that can be found in the LICENSE file.

// #pragma once
// #ifndef MESH_GLOBAL_HEAP_H
// #define MESH_GLOBAL_HEAP_H

// #include <algorithm>
// #include <array>
// #include <mutex>

// #include "internal.h"
// #include "meshable_arena.h"
// #include "mini_heap.h"

// #include "heaplayers.h"

// using namespace HL;

// namespace mesh {

// static constexpr std::pair<MiniHeapListEntry, size_t> Head{MiniHeapListEntry{list::Head, list::Head}, 0};

// class EpochLock {
#[derive(Default)]
struct EpochLock(AtomicUsize);
// private:
//   DISALLOW_COPY_AND_ASSIGN(EpochLock);

// public:
//   EpochLock() {
//   }
impl EpochLock {
//   inline size_t ATTRIBUTE_ALWAYS_INLINE current() const noexcept {
    pub fn current(&self) -> usize {
        //     return _epoch.load(std::memory_order::memory_order_seq_cst);
        self.0.load(Ordering::SeqCst)
        //   }
    }

//   inline size_t ATTRIBUTE_ALWAYS_INLINE isSame(size_t startEpoch) const noexcept {
    pub fn is_same(&self, start_epoch: usize) -> bool {
//     return current() == startEpoch;
        self.current() == start_epoch
//   }
    }

//   inline void ATTRIBUTE_ALWAYS_INLINE lock() noexcept {
    pub fn lock(&mut self) {
//     // make sure that the previous epoch was even
//     const auto old = _epoch.fetch_add(1, std::memory_order::memory_order_seq_cst);
        let old = self.0.fetch_add(1, Ordering::SeqCst);
//     hard_assert(old % 2 == 0);
        assert!(old % 2 == 0)
//   }
    }

//   inline void ATTRIBUTE_ALWAYS_INLINE unlock() noexcept {
    pub fn unlock(&mut self) {
// #ifndef NDEBUG
//     // make sure that the previous epoch was odd
//     const auto old = _epoch.fetch_add(1, std::memory_order::memory_order_seq_cst);
        let old = self.0.fetch_add(1, Ordering::SeqCst);
//     d_assert(old % 2 == 1);
        debug_assert!(old % 2 == 1);
// #else
//     _epoch.fetch_add(1, std::memory_order::memory_order_seq_cst);
// #endif
//   }
    }

// private:
//   atomic_size_t _epoch{0};
// };
}
// class GlobalHeapStats {
#[derive(Debug, Default)]
pub struct GlobalHeapStats {
// public:
//   atomic_size_t meshCount;
    pub mesh_count: AtomicUsize,
//   size_t mhFreeCount;
    pub mh_free_count: usize,
//   size_t mhAllocCount;
    pub mh_alloc_count: usize,
//   size_t mhHighWaterMark;
    pub mh_high_water_mark: usize,
// };
}

pub struct GlobalHeapLists {
    //   // these must only be accessed or modified with the _miniheapLock held
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _emptyFreelist{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
    
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _partialFreelist{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
    
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _fullList{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
    
}




// class GlobalHeap : public MeshableArena {
pub struct GlobalHeap {
//   const size_t _maxObjectSize;
    _max_object_size: usize,
//   atomic_size_t _meshPeriod{kDefaultMeshPeriod};
    _mesh_period: AtomicUsize,
//   std::chrono::milliseconds _meshPeriodMs{kMeshPeriodMs};
    _mesh_period_ms: Duration,
//   atomic_size_t ATTRIBUTE_ALIGNED(CACHELINE_SIZE) _lastMeshEffective{0};
    _last_mesh_effective: AtomicUsize,
//   // we want this on its own cacheline
//   EpochLock ATTRIBUTE_ALIGNED(CACHELINE_SIZE) _meshEpoch{};
    _mesh_epoch: EpochLock,
//   // always accessed with the mhRWLock exclusively locked.  cachline
//   // aligned to avoid sharing cacheline with _meshEpoch
//   size_t ATTRIBUTE_ALIGNED(CACHELINE_SIZE) _miniheapCount{0};
    _miniheap_count: usize,

//   // these must only be accessed or modified with the _miniheapLock held
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _emptyFreelist{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
    pub _empty_free_list: [(MiniHeapListEntry, usize); NUM_BINS],
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _partialFreelist{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
    pub _partial_free_list: [(MiniHeapListEntry, usize); NUM_BINS],
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _fullList{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
    pub _full_list: [(MiniHeapListEntry, usize); NUM_BINS],

//   mutable mutex _miniheapLock{};
    pub _mini_heap_lock: Mutex<()>,

//   GlobalHeapStats _stats{};
    pub _stats: GlobalHeapStats,

//   // XXX: should be atomic, but has exception spec?
//   time::time_point _lastMesh;
    pub _last_mesh: Instant,
}
// private:
//   DISALLOW_COPY_AND_ASSIGN(GlobalHeap);
//   typedef MeshableArena Super;

//   static_assert(HL::gcd<MmapHeap::Alignment, Alignment>::value == Alignment,
//                 "expected MmapHeap to have 16-byte alignment");

// public:
//   enum { Alignment = 16 };

impl GlobalHeap {
    pub fn new() -> Self {
//   GlobalHeap() : Super(), _maxObjectSize(SizeMap::ByteSizeForClass(kNumBins - 1)), _lastMesh{time::now()} {
        Self {
            _max_object_size: byte_size_for_class((NUM_BINS-1) as u32),
            _last_mesh_effective: 0.into(),
            _mesh_epoch: EpochLock::default(),
            _mesh_period: DEFAULT_MESH_PERIOD.into(),
            _mesh_period_ms: MESH_PERIOD_MS,
            _mini_heap_lock: Mutex::new(()),
            _miniheap_count: 0,
            _last_mesh: Instant::now(),
            _stats: GlobalHeapStats::default(),
            _full_list: todo!(),
            _empty_free_list: todo!(),
            _partial_free_list: todo!(),
        }
//   }
    }

//   inline void dumpStrings() const {
//     lock_guard<mutex> lock(_miniheapLock);

//     mesh::debug("TODO: reimplement printOccupancy\n");
//     // for (size_t i = 0; i < kNumBins; i++) {
//     //   _littleheaps[i].printOccupancy();
//     // }
//   }

//   inline void flushAllBins() {
    pub fn flush_all_bins(&mut self) { 
//     for (size_t sizeClass = 0; sizeClass < kNumBins; sizeClass++) {
        for size_class in 0..NUM_BINS {
//       flushBinLocked(sizeClass);
            self.flush_bin_locked(size_class);
        }
//     }
//   }
    }

//   void scavenge(bool force = false) {
    pub fn scavenge(&mut self, force: bool) {
//     lock_guard<mutex> lock(_miniheapLock);
        let _lock = self._mini_heap_lock.lock().unwrap();
//     Super::scavenge(force);
        todo!("Super::scavenge(force);")
//   }
    }

//   void dumpStats(int level, bool beDetailed) const;
    pub fn dump_stats(&self, level: u32, be_detailed: bool) {}

//   // must be called with exclusive _mhRWLock held
//   inline MiniHeap *ATTRIBUTE_ALWAYS_INLINE allocMiniheapLocked(int sizeClass, size_t pageCount, size_t objectCount,
//                                                                size_t objectSize, size_t pageAlignment = 1) {
    pub fn alloc_miniheap_locked(&mut self, size_class: u32, page_count: usize, object_count: usize, object_size: usize, page_alignment: usize) {
//     d_assert(0 < pageCount);
        debug_assert!(0 < page_count);
//     void *buf = _mhAllocator.alloc();
        
//     d_assert(buf != nullptr);

//     // allocate out of the arena
//     Span span{0, 0};
//     char *spanBegin = Super::pageAlloc(span, pageCount, pageAlignment);
//     d_assert(spanBegin != nullptr);
//     d_assert((reinterpret_cast<uintptr_t>(spanBegin) / kPageSize) % pageAlignment == 0);

//     MiniHeap *mh = new (buf) MiniHeap(arenaBegin(), span, objectCount, objectSize);

//     const auto miniheapID = MiniHeapID{_mhAllocator.offsetFor(buf)};
//     Super::trackMiniHeap(span, miniheapID);

//     // mesh::debug("%p (%u) created!\n", mh, GetMiniHeapID(mh));

//     _miniheapCount++;
//     _stats.mhAllocCount++;
//     _stats.mhHighWaterMark = max(_miniheapCount, _stats.mhHighWaterMark);

//     return mh;
//   }
    }

//   inline void *pageAlignedAlloc(size_t pageAlignment, size_t pageCount) {
//     // if given a very large allocation size (e.g. (uint64_t)-8), it is possible
//     // the pageCount calculation overflowed.  An allocation that big is impossible
//     // to satisfy anyway, so just fail early.
//     if (unlikely(pageCount == 0)) {
//       return nullptr;
//     }

//     lock_guard<mutex> lock(_miniheapLock);

//     MiniHeap *mh = allocMiniheapLocked(-1, pageCount, 1, pageCount * kPageSize, pageAlignment);

//     d_assert(mh->isLargeAlloc());
//     d_assert(mh->spanSize() == pageCount * kPageSize);
//     // d_assert(mh->objectSize() == pageCount * kPageSize);

//     void *ptr = mh->mallocAt(arenaBegin(), 0);

//     return ptr;
//   }

//   inline MiniHeapListEntry *freelistFor(uint8_t freelistId, int sizeClass) {
//     switch (freelistId) {
//     case list::Empty:
//       return &_emptyFreelist[sizeClass].first;
//     case list::Partial:
//       return &_partialFreelist[sizeClass].first;
//     case list::Full:
//       return &_fullList[sizeClass].first;
//     }
//     // remaining case is 'attached', for which there is no freelist
//     return nullptr;
//   }

//   inline bool postFreeLocked(MiniHeap *mh, int sizeClass, size_t inUse) {
//     // its possible we raced between reading isAttached + grabbing a lock.
//     // just check here to avoid having to play whack-a-mole at each call site.
//     if (mh->isAttached()) {
//       return false;
//     }
//     const auto currFreelistId = mh->freelistId();
//     auto currFreelist = freelistFor(currFreelistId, sizeClass);
//     const auto max = mh->maxCount();

//     std::pair<MiniHeapListEntry, size_t> *list;
//     uint8_t newListId;

//     if (inUse == 0) {
//       // if the miniheap is already in the right list there is nothing to do
//       if (currFreelistId == list::Empty) {
//         return false;
//       }
//       newListId = list::Empty;
//       list = &_emptyFreelist[sizeClass];
//     } else if (inUse == max) {
//       if (currFreelistId == list::Full) {
//         return false;
//       }
//       newListId = list::Full;
//       list = &_fullList[sizeClass];
//     } else {
//       if (currFreelistId == list::Partial) {
//         return false;
//       }
//       newListId = list::Partial;
//       list = &_partialFreelist[sizeClass];
//     }

//     list->first.add(currFreelist, newListId, list::Head, mh);
//     list->second++;

//     return _emptyFreelist[sizeClass].second > kBinnedTrackerMaxEmpty;
//   }

//   inline void releaseMiniheapLocked(MiniHeap *mh, int sizeClass) {
//     // ensure this flag is always set with the miniheap lock held
//     mh->unsetAttached();
//     const auto inUse = mh->inUseCount();
//     postFreeLocked(mh, sizeClass, inUse);
//   }

//   template <uint32_t Size>
//   inline void releaseMiniheaps(FixedArray<MiniHeap, Size> &miniheaps) {
//     if (miniheaps.size() == 0) {
//       return;
//     }

//     lock_guard<mutex> lock(_miniheapLock);
//     for (auto mh : miniheaps) {
//       releaseMiniheapLocked(mh, mh->sizeClass());
//     }
//     miniheaps.clear();
//   }

//   template <uint32_t Size>
//   size_t fillFromList(FixedArray<MiniHeap, Size> &miniheaps, pid_t current,
//                       std::pair<MiniHeapListEntry, size_t> &freelist, size_t bytesFree) {
//     if (freelist.first.empty()) {
//       return bytesFree;
//     }

//     auto nextId = freelist.first.next();
//     while (nextId != list::Head && bytesFree < kMiniheapRefillGoalSize && !miniheaps.full()) {
//       auto mh = GetMiniHeap(nextId);
//       d_assert(mh != nullptr);
//       nextId = mh->getFreelist()->next();

//       // TODO: we can eventually remove this
//       d_assert(!(mh->isFull() || mh->isAttached() || mh->isMeshed()));

//       // TODO: this is commented out to match a bug in the previous implementation;
//       // it turns out if you don't track bytes free and give more memory to the
//       // thread-local cache, things perform better!
//       // bytesFree += mh->bytesFree();
//       d_assert(!mh->isAttached());
//       mh->setAttached(current, freelistFor(mh->freelistId(), mh->sizeClass()));
//       d_assert(mh->isAttached() && mh->current() == current);
//       hard_assert(!miniheaps.full());
//       miniheaps.append(mh);
//       d_assert(freelist.second > 0);
//       freelist.second--;
//     }

//     return bytesFree;
//   }

//   template <uint32_t Size>
//   size_t selectForReuse(int sizeClass, FixedArray<MiniHeap, Size> &miniheaps, pid_t current) {
//     size_t bytesFree = fillFromList(miniheaps, current, _partialFreelist[sizeClass], 0);

//     if (bytesFree >= kMiniheapRefillGoalSize || miniheaps.full()) {
//       return bytesFree;
//     }

//     // we've exhausted all of our partially full MiniHeaps, but there
//     // might still be empty ones we could reuse.
//     return fillFromList(miniheaps, current, _emptyFreelist[sizeClass], bytesFree);
//   }

//   template <uint32_t Size>
//   inline void allocSmallMiniheaps(int sizeClass, uint32_t objectSize, FixedArray<MiniHeap, Size> &miniheaps,
//                                   pid_t current) {
//     lock_guard<mutex> lock(_miniheapLock);

//     d_assert(sizeClass >= 0);

//     for (MiniHeap *oldMH : miniheaps) {
//       releaseMiniheapLocked(oldMH, sizeClass);
//     }
//     miniheaps.clear();

//     d_assert(objectSize <= _maxObjectSize);

// #ifndef NDEBUG
//     const size_t classMaxSize = SizeMap::ByteSizeForClass(sizeClass);

//     d_assert_msg(objectSize == classMaxSize, "sz(%zu) shouldn't be greater than %zu (class %d)", objectSize,
//                  classMaxSize, sizeClass);
// #endif
//     d_assert(sizeClass >= 0);
//     d_assert(sizeClass < kNumBins);

//     d_assert(miniheaps.size() == 0);

//     // check our bins for a miniheap to reuse
//     auto bytesFree = selectForReuse(sizeClass, miniheaps, current);
//     if (bytesFree >= kMiniheapRefillGoalSize || miniheaps.full()) {
//       return;
//     }

//     // if we have objects bigger than the size of a page, allocate
//     // multiple pages to amortize the cost of creating a
//     // miniheap/globally locking the heap.  For example, asking for
//     // 2048 byte objects would allocate 4 4KB pages.
//     const size_t objectCount = max(kPageSize / objectSize, kMinStringLen);
//     const size_t pageCount = PageCount(objectSize * objectCount);

//     while (bytesFree < kMiniheapRefillGoalSize && !miniheaps.full()) {
//       auto mh = allocMiniheapLocked(sizeClass, pageCount, objectCount, objectSize);
//       d_assert(!mh->isAttached());
//       mh->setAttached(current, freelistFor(mh->freelistId(), sizeClass));
//       d_assert(mh->isAttached() && mh->current() == current);
//       miniheaps.append(mh);
//       bytesFree += mh->bytesFree();
//     }

//     return;
//   }

//   // large, page-multiple allocations
//   void *ATTRIBUTE_NEVER_INLINE malloc(size_t sz);

//   inline MiniHeap *ATTRIBUTE_ALWAYS_INLINE miniheapForWithEpoch(const void *ptr, size_t &currentEpoch) const {
//     currentEpoch = _meshEpoch.current();
//     return miniheapFor(ptr);
//   }

//   inline MiniHeap *ATTRIBUTE_ALWAYS_INLINE miniheapFor(const void *ptr) const {
//     auto mh = reinterpret_cast<MiniHeap *>(Super::lookupMiniheap(ptr));
//     return mh;
//   }

//   inline MiniHeap *ATTRIBUTE_ALWAYS_INLINE miniheapForID(const MiniHeapID id) const {
//     auto mh = reinterpret_cast<MiniHeap *>(_mhAllocator.ptrFromOffset(id.value()));
//     __builtin_prefetch(mh, 1, 2);
//     return mh;
//   }

//   inline MiniHeapID miniheapIDFor(const MiniHeap *mh) const {
//     return MiniHeapID{_mhAllocator.offsetFor(mh)};
//   }

//   void untrackMiniheapLocked(MiniHeap *mh) {
//     // mesh::debug("%p (%u) untracked!\n", mh, GetMiniHeapID(mh));
//     _stats.mhAllocCount -= 1;
//     mh->getFreelist()->remove(freelistFor(mh->freelistId(), mh->sizeClass()));
//   }

//   void freeFor(MiniHeap *mh, void *ptr, size_t startEpoch);

//   // called with lock held
//   void freeMiniheapAfterMeshLocked(MiniHeap *mh, bool untrack = true) {
//     // don't untrack a meshed miniheap -- it has already been untracked
//     if (untrack && !mh->isMeshed()) {
//       untrackMiniheapLocked(mh);
//     }

//     d_assert(!mh->getFreelist()->prev().hasValue());
//     d_assert(!mh->getFreelist()->next().hasValue());
//     mh->MiniHeap::~MiniHeap();
//     // memset(reinterpret_cast<char *>(mh), 0x77, sizeof(MiniHeap));
//     _mhAllocator.free(mh);
//     _miniheapCount--;
//   }

//   void freeMiniheap(MiniHeap *&mh, bool untrack = true) {
//     lock_guard<mutex> lock(_miniheapLock);
//     freeMiniheapLocked(mh, untrack);
//   }

//   void freeMiniheapLocked(MiniHeap *&mh, bool untrack) {
//     const auto spanSize = mh->spanSize();
//     MiniHeap *toFree[kMaxMeshes];
//     size_t last = 0;

//     memset(toFree, 0, sizeof(*toFree) * kMaxMeshes);

//     // avoid use after frees while freeing
//     mh->forEachMeshed([&](MiniHeap *mh) {
//       toFree[last++] = mh;
//       return false;
//     });

//     for (size_t i = 0; i < last; i++) {
//       MiniHeap *mh = toFree[i];
//       const bool isMeshed = mh->isMeshed();
//       const auto type = isMeshed ? internal::PageType::Meshed : internal::PageType::Dirty;
//       Super::free(reinterpret_cast<void *>(mh->getSpanStart(arenaBegin())), spanSize, type);
//       _stats.mhFreeCount++;
//       freeMiniheapAfterMeshLocked(mh, untrack);
//     }

//     mh = nullptr;
//   }

//   // flushBinLocked empties _emptyFreelist[sizeClass]
//   inline void flushBinLocked(size_t sizeClass) {
    pub fn flush_bin_locked(&mut self, size_class: usize) {
//     // mesh::debug("flush bin %zu\n", sizeClass);
//     d_assert(!_emptyFreelist[sizeClass].first.empty());
        // TODO: figure out how to actually use this fucking free list
//     if (_emptyFreelist[sizeClass].first.next() == list::Head) {
//       return;
//     }

//     std::pair<MiniHeapListEntry, size_t> &empty = _emptyFreelist[sizeClass];
//     MiniHeapID nextId = empty.first.next();
//     while (nextId != list::Head) {
//       auto mh = GetMiniHeap(nextId);
//       nextId = mh->getFreelist()->next();
//       freeMiniheapLocked(mh, true);
//       empty.second--;
//     }

//     d_assert(empty.first.next() == list::Head);
//     d_assert(empty.first.prev() == list::Head);
//   }
    }

//   void ATTRIBUTE_NEVER_INLINE free(void *ptr);

//   inline size_t getSize(void *ptr) const {
//     if (unlikely(ptr == nullptr))
//       return 0;

//     lock_guard<mutex> lock(_miniheapLock);
//     auto mh = miniheapFor(ptr);
//     if (likely(mh)) {
//       return mh->objectSize();
//     } else {
//       return 0;
//     }
//   }

//   int mallctl(const char *name, void *oldp, size_t *oldlenp, void *newp, size_t newlen);

//   size_t getAllocatedMiniheapCount() const {
//     lock_guard<mutex> lock(_miniheapLock);
//     return _miniheapCount;
//   }

//   void setMeshPeriodMs(std::chrono::milliseconds period) {
//     _meshPeriodMs = period;
//   }

//   void lock() {
//     _miniheapLock.lock();
//     // internal::Heap().lock();
//   }

//   void unlock() {
//     // internal::Heap().unlock();
//     _miniheapLock.unlock();
//   }

//   // PUBLIC ONLY FOR TESTING
//   // after call to meshLocked() completes src is a nullptr
//   void ATTRIBUTE_NEVER_INLINE meshLocked(MiniHeap *dst, MiniHeap *&src);

//   inline void ATTRIBUTE_ALWAYS_INLINE maybeMesh() {
//     if (!kMeshingEnabled) {
//       return;
//     }

//     if (_meshPeriod == 0) {
//       return;
//     }

//     if (_meshPeriodMs == kZeroMs) {
//       return;
//     }

//     const auto now = time::now();
//     auto duration = chrono::duration_cast<chrono::milliseconds>(now - _lastMesh);

//     if (likely(duration < _meshPeriodMs)) {
//       return;
//     }

//     lock_guard<mutex> lock(_miniheapLock);

//     {
//       // ensure if two threads tried to grab the mesh lock at the same
//       // time, the second one bows out gracefully without meshing
//       // twice in a row.
//       const auto lockedNow = time::now();
//       auto duration = chrono::duration_cast<chrono::milliseconds>(lockedNow - _lastMesh);

//       if (unlikely(duration < _meshPeriodMs)) {
//         return;
//       }
//     }

//     _lastMesh = now;

//     meshAllSizeClassesLocked();
//   }

//   inline bool okToProceed(void *ptr) const {
//     lock_guard<mutex> lock(_miniheapLock);

//     if (ptr == nullptr) {
//       return false;
//     }

//     return miniheapFor(ptr) != nullptr;
//   }

//   inline internal::vector<MiniHeap *> meshingCandidatesLocked(int sizeClass) const {
//     // FIXME: duplicated with code in halfSplit
//     internal::vector<MiniHeap *> bucket{};

//     auto nextId = _partialFreelist[sizeClass].first.next();
//     while (nextId != list::Head) {
//       auto mh = GetMiniHeap(nextId);
//       if (mh->isMeshingCandidate() && (mh->fullness() < kOccupancyCutoff)) {
//         bucket.push_back(mh);
//       }
//       nextId = mh->getFreelist()->next();
//     }

//     return bucket;
//   }

// private:
//   // check for meshes in all size classes -- must be called LOCKED
//   void meshAllSizeClassesLocked();
//   // meshSizeClassLocked returns the number of merged sets found
//   size_t meshSizeClassLocked(size_t sizeClass, MergeSetArray &mergeSets, SplitArray &left, SplitArray &right);
}
//   const size_t _maxObjectSize;
//   atomic_size_t _meshPeriod{kDefaultMeshPeriod};
//   std::chrono::milliseconds _meshPeriodMs{kMeshPeriodMs};

//   atomic_size_t ATTRIBUTE_ALIGNED(CACHELINE_SIZE) _lastMeshEffective{0};

//   // we want this on its own cacheline
//   EpochLock ATTRIBUTE_ALIGNED(CACHELINE_SIZE) _meshEpoch{};

//   // always accessed with the mhRWLock exclusively locked.  cachline
//   // aligned to avoid sharing cacheline with _meshEpoch
//   size_t ATTRIBUTE_ALIGNED(CACHELINE_SIZE) _miniheapCount{0};

//   // these must only be accessed or modified with the _miniheapLock held
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _emptyFreelist{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _partialFreelist{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};
//   std::array<std::pair<MiniHeapListEntry, size_t>, kNumBins> _fullList{
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head,
//       Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head, Head};

//   mutable mutex _miniheapLock{};

//   GlobalHeapStats _stats{};

//   // XXX: should be atomic, but has exception spec?
//   time::time_point _lastMesh;
// };

// static_assert(kNumBins == 25, "if this changes, add more 'Head's above");
// static_assert(sizeof(std::array<MiniHeapListEntry, kNumBins>) == kNumBins * 8, "list size is right");
// static_assert(sizeof(GlobalHeap) < (kNumBins * 8 * 3 + 64 * 7 + 100000), "gh small enough");
// }  // namespace mesh

// #endif  // MESH_GLOBAL_HEAP_H
