//! Mesh Memory Allocator - Rust Port
#![allow(unused)]
pub mod bitmap;
pub mod cheap_heap;
pub mod heap_entry;
pub(crate) mod internal;
pub mod mini_heap;
pub mod mmap_heap;
pub mod one_way_mmap_heap;
pub mod shuffle_vec;
pub mod slab;
pub mod span;

use rand::{rngs::SmallRng, Rng, SeedableRng};
use shuffle_vec::ShuffleVector;
use std::alloc::{GlobalAlloc, Layout};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::ptr::{self, NonNull};

/// Page size constant (4KB on most architectures)
const PAGE_SIZE: usize = 4096;
const MAX_SIZE_CLASS: usize = 256;
const MIN_MESH_THRESHOLD: usize = 2;

/// Represents a span of contiguous pages
#[derive(Debug, Clone)]
struct Span {
    start: *mut u8,
    size: usize,
    mini_heap: Option<MiniHeap>,
}

/// Metadata for tracking allocations within a span
#[derive(Debug, Clone)]
struct MiniHeap {
    span: NonNull<u8>,
    size_class: usize,
    object_size: usize,
    max_objects: usize,
    allocated_objects: usize,
    bitmap: Vec<usize>, // Bitmap to track allocated slots (using CPU word size)
    shuffle_vector: ShuffleVector<usize, rand::rngs::SmallRng>,
}

impl MiniHeap {
    fn new(span: NonNull<u8>, size_class: usize, span_size: usize) -> Self {
        const WORD_SIZE_BITS: usize = std::mem::size_of::<usize>() * 8;

        let object_size = Self::size_class_to_size(size_class);
        let max_objects = span_size / object_size;
        let bitmap_size = (max_objects + WORD_SIZE_BITS - 1) / WORD_SIZE_BITS; // Round up to nearest word
        let rng = SmallRng::from_os_rng();
        let shuffle_vector = ShuffleVector::from_parts((0..max_objects).collect(), rng);
        Self {
            span,
            size_class,
            object_size,
            max_objects,
            allocated_objects: 0,
            bitmap: vec![0; bitmap_size],
            shuffle_vector,
        }
    }

    fn size_class_to_size(size_class: usize) -> usize {
        // Simple size class mapping - could be more sophisticated
        match size_class {
            0 => 8,
            1 => 16,
            2 => 32,
            3 => 64,
            4 => 128,
            5 => 256,
            _ => 256,
        }
    }

    fn allocate(&mut self) -> Option<NonNull<u8>> {
        if self.allocated_objects >= self.max_objects {
            return None;
        }

        if let Some(&index) = self.shuffle_vector.get() {
            const WORD_SIZE_BITS: usize = std::mem::size_of::<usize>() * 8;
            let word_index = index / WORD_SIZE_BITS;
            let bit_index = index % WORD_SIZE_BITS;

            if self.bitmap[word_index] & (1usize << bit_index) == 0 {
                // Mark as allocated
                self.bitmap[word_index] |= 1usize << bit_index;
                self.allocated_objects += 1;

                let offset = index * self.object_size;
                let ptr = unsafe { self.span.as_ptr().add(offset) };
                return NonNull::new(ptr);
            }
        }

        None
    }

    fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
        let span_start = self.span.as_ptr() as usize;
        let ptr_addr = ptr.as_ptr() as usize;

        if ptr_addr < span_start || ptr_addr >= span_start + (self.max_objects * self.object_size) {
            return false; // Not in this mini heap
        }

        let offset = ptr_addr - span_start;
        let index = offset / self.object_size;

        if index >= self.max_objects {
            return false;
        }

        const WORD_SIZE_BITS: usize = std::mem::size_of::<usize>() * 8;
        let word_index = index / WORD_SIZE_BITS;
        let bit_index = index % WORD_SIZE_BITS;

        if self.bitmap[word_index] & (1usize << bit_index) != 0 {
            // Mark as free
            self.bitmap[word_index] &= !(1usize << bit_index);
            self.allocated_objects -= 1;
            self.shuffle_vector.put(index);
            return true;
        }

        false
    }

    fn can_mesh_with(&self, other: &Self) -> bool {
        if self.size_class != other.size_class {
            return false;
        }

        // Check if the combined allocation patterns would fit in one span
        let total_allocated = self.allocated_objects + other.allocated_objects;
        total_allocated <= self.max_objects
    }
}

/// Thread-local heap for fast allocation
struct LocalHeap {
    mini_heaps: Vec<Option<MiniHeap>>,
    global_heap: *mut GlobalHeap,
}

impl LocalHeap {
    fn new(global_heap: *mut GlobalHeap) -> Self {
        Self {
            mini_heaps: vec![None; MAX_SIZE_CLASS],
            global_heap,
        }
    }

    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        let size_class = self.size_to_class(size);

        // Try local mini heap first
        if let Some(Some(mini_heap)) = self.mini_heaps.get_mut(size_class) {
            if let Some(ptr) = mini_heap.allocate() {
                return Some(ptr);
            }
        }
        let global = unsafe { self.global_heap.as_mut() }?;
        // Get new mini heap from global heap
        let mut new_mini_heap = global.get_mini_heap(size_class)?;
        let result = new_mini_heap.allocate();
        self.mini_heaps[size_class] = Some(new_mini_heap);
        result
    }

    fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
        // Try to find which mini heap this pointer belongs to
        for mini_heap_opt in &mut self.mini_heaps {
            if let Some(ref mut mini_heap) = mini_heap_opt {
                if mini_heap.deallocate(ptr) {
                    return true;
                }
            }
        }

        // Fall back to global heap
        unsafe { (*self.global_heap).deallocate(ptr) }
    }

    fn size_to_class(&self, size: usize) -> usize {
        match size {
            0..=8 => 0,
            9..=16 => 1,
            17..=32 => 2,
            33..=64 => 3,
            65..=128 => 4,
            129..=256 => 5,
            _ => 5, // Fallback to largest small size class
        }
    }
}

/// Global heap that manages spans and performs meshing
struct GlobalHeap {
    spans: Vec<Span>,
    mini_heaps: HashMap<usize, VecDeque<MiniHeap>>,
    stats: MeshStats,
}

#[derive(Debug, Default)]
struct MeshStats {
    mesh_count: usize,
    meshed_mb_total: f64,
    meshed_pages_hwm: usize,
    alloc_count: usize,
    free_count: usize,
}

impl GlobalHeap {
    fn new() -> Self {
        Self {
            spans: Vec::new(),
            mini_heaps: HashMap::new(),
            stats: MeshStats::default(),
        }
    }

    fn get_mini_heap(&mut self, size_class: usize) -> Option<MiniHeap> {
        if let Some(queue) = self.mini_heaps.get_mut(&size_class) {
            if let Some(mini_heap) = queue.pop_front() {
                return Some(mini_heap);
            }
        }

        // Create new span and mini heap
        self.create_mini_heap(size_class)
    }

    fn create_mini_heap(&mut self, size_class: usize) -> Option<MiniHeap> {
        let span_size = PAGE_SIZE * 4; // 16KB spans for example

        // Allocate memory using system allocator
        let layout = Layout::from_size_align(span_size, PAGE_SIZE).ok()?;
        let span_ptr = unsafe { std::alloc::alloc(layout) };

        if span_ptr.is_null() {
            return None;
        }

        let span_non_null = NonNull::new(span_ptr)?;
        let mini_heap = MiniHeap::new(span_non_null, size_class, span_size);

        // Store span for tracking
        self.spans.push(Span {
            start: span_ptr,
            size: span_size,
            mini_heap: Some(mini_heap.clone()),
        });

        // Update stats
        self.stats.alloc_count += 1;

        Some(mini_heap)
    }

    fn deallocate(&mut self, ptr: NonNull<u8>) -> bool {
        for span in &mut self.spans {
            if let Some(ref mut mini_heap) = span.mini_heap {
                if mini_heap.deallocate(ptr) {
                    self.try_mesh();
                    return true;
                }
            }
        }
        false
    }

    fn try_mesh(&mut self) {
        let mut meshable_pairs = Vec::new();

        // Find pairs of mini heaps that can be meshed
        for (i, span1) in self.spans.iter().enumerate() {
            if let Some(ref mini_heap1) = span1.mini_heap {
                for (j, span2) in self.spans.iter().enumerate().skip(i + 1) {
                    if let Some(ref mini_heap2) = span2.mini_heap {
                        if mini_heap1.can_mesh_with(mini_heap2) {
                            meshable_pairs.push((i, j));
                        }
                    }
                }
            }
        }

        // Perform meshing on identified pairs
        for (i, j) in meshable_pairs {
            self.mesh_spans(i, j);
        }
    }

    fn mesh_spans(&mut self, _span1_idx: usize, _span2_idx: usize) {
        // Placeholder for actual meshing implementation
        // This would involve:
        // 1. Copying live objects from both spans to one span
        // 2. Updating pointers and metadata
        // 3. Freeing the unused span
        // 4. Using madvise/VirtualAlloc to actually reclaim memory

        self.stats.mesh_count += 1;
        self.stats.meshed_mb_total += PAGE_SIZE as f64 / (1024.0 * 1024.0);
    }

    fn print_stats(&self) {
        println!("MESH COUNT: {}", self.stats.mesh_count);
        println!("Meshed MB (total): {:.1}", self.stats.meshed_mb_total);
        println!("Meshed pages HWM: {}", self.stats.meshed_pages_hwm);
        println!("MH Alloc Count: {}", self.stats.alloc_count);
        println!("MH Free Count: {}", self.stats.free_count);
    }
}

/// The main Mesh allocator struct
pub struct MeshAllocator {
    global_heap: GlobalHeap,
    local_heap: Cell<LocalHeap>,
}

impl MeshAllocator {
    pub fn new() -> Self {
        let mut allocator = Self {
            global_heap: GlobalHeap::new(),
            local_heap: LocalHeap {
                mini_heaps: vec![None; MAX_SIZE_CLASS],
                global_heap: ptr::null_mut(),
            }
            .into(),
        };

        // Set up the pointer to global heap
        allocator.local_heap.get_mut().global_heap = &mut allocator.global_heap as *mut GlobalHeap;
        allocator
    }

    pub fn print_stats(&self) {
        self.global_heap.print_stats();
    }

    /// Effectively the `update` method that has not yet stabilized for `Cell`
    fn update_local<F, T>(&self, f: F) -> T
    where
        F: Fn(&mut LocalHeap) -> T,
    {
        let mut local = Cell::new(LocalHeap {
            mini_heaps: Vec::new(),
            global_heap: std::ptr::null_mut(),
        });
        self.local_heap.swap(&local);
        let ret = f(local.get_mut());
        self.local_heap.set(local.into_inner());
        ret
    }

    fn allocate_local(&self, layout: Layout) -> Option<*mut u8> {
        self.update_local(|l| l.allocate(layout.size()))
            .map(|nn| nn.as_ptr())
    }
    fn deallocate_local(&self, nn: NonNull<u8>) -> bool {
        self.update_local(|l| l.deallocate(nn))
    }
}

impl Default for MeshAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for MeshAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        println!("attempting to allocate {layout:?}");
        // For large allocations, fall back to system allocator
        if layout.size() > MAX_SIZE_CLASS {
            println!("outside of size classes, falling back to system allocator");
            return std::alloc::alloc(layout);
        }
        println!("attempting to allocate on the local heap");
        self.allocate_local(layout).unwrap_or_else(|| {
            println!("failed to allocate on the local heap, falling back to system allocator");
            std::alloc::alloc(layout)
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() > MAX_SIZE_CLASS {
            std::alloc::dealloc(ptr, layout);
            return;
        }
        let Some(non_null_ptr) = NonNull::new(ptr) else {
            // ptr was null, nothing to deallocate!
            return;
        };
        if !self.deallocate_local(non_null_ptr) {
            // Fallback to system dealloc
            std::alloc::dealloc(ptr, layout);
        }
    }
}

// Enable using Mesh as the global allocator
// Uncomment the following line to use Mesh as the global allocator:
// #[global_allocator]
// static GLOBAL: MeshAllocator = MeshAllocator::new();

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::alloc::{GlobalAlloc, Layout};

//     #[test]
//     fn test_basic_allocation() {
//         println!("initing new allocator");
//         let allocator = MeshAllocator::new();
//         println!("creating a new layout");
//         let layout = Layout::from_size_align(64, 8).unwrap();

//         println!("allocating layout");
//         unsafe {
//             let ptr = allocator.alloc(layout);
//             assert!(!ptr.is_null());
//             println!("pointer was not null, assigning");
//             // Write to the memory to ensure it's valid
//             *ptr = 42;
//             assert_eq!(*ptr, 42);
//             println!("pointer assignment matches 42, deallocating");
//             allocator.dealloc(ptr, layout);
//         }
//         println!("test complete...");
//     }

//     #[test]
//     fn test_mini_heap() {
//         let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
//         let span_ptr = unsafe { std::alloc::alloc(layout) };
//         assert!(!span_ptr.is_null());

//         let span_non_null = NonNull::new(span_ptr).unwrap();
//         let mut mini_heap = MiniHeap::new(span_non_null, 2, PAGE_SIZE); // 32-byte objects

//         // Allocate some objects
//         let ptr1 = mini_heap.allocate().unwrap();
//         let ptr2 = mini_heap.allocate().unwrap();

//         // Deallocate one
//         assert!(mini_heap.deallocate(ptr1));

//         // Should be able to allocate again
//         let ptr3 = mini_heap.allocate().unwrap();
//         assert_ne!(ptr2.as_ptr(), ptr3.as_ptr());

//         unsafe {
//             std::alloc::dealloc(span_ptr, layout);
//         }
//     }

//     #[test]
//     fn test_single_threaded_allocator() {
//         let mut allocator = MeshAllocator::new();

//         // Test basic allocation
//         let ptr1 = allocator.local_heap.get_mut().allocate(64).unwrap();

//         let ptr2 = allocator.local_heap.get_mut().allocate(64).unwrap();

//         // Test deallocation
//         assert!(allocator.local_heap.get_mut().deallocate(ptr1));
//         assert!(allocator.local_heap.get_mut().deallocate(ptr2));
//     }
// }

// Example usage and integration
pub fn example_usage() {
    println!("Mesh Allocator Example");

    let allocator = MeshAllocator::new();

    // Simulate some allocations
    let layout = Layout::from_size_align(64, 8).unwrap();
    let mut ptrs = Vec::new();

    unsafe {
        for i in 0..100 {
            let ptr = allocator.alloc(layout);
            if !ptr.is_null() {
                *(ptr as *mut usize) = i;
                ptrs.push(ptr);
            }
        }

        // Free every other allocation to create fragmentation
        for (i, &ptr) in ptrs.iter().enumerate() {
            if i % 2 == 0 {
                allocator.dealloc(ptr, layout);
            }
        }

        // Allocate more to trigger meshing
        for _ in 0..50 {
            let ptr = allocator.alloc(layout);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }

        // Clean up remaining allocations
        for (i, &ptr) in ptrs.iter().enumerate() {
            if i % 2 == 1 {
                // Free the ones we didn't free before
                allocator.dealloc(ptr, layout);
            }
        }
    }

    allocator.print_stats();
}
