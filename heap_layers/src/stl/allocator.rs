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
// #ifndef HL_STLALLOCATOR_H
// #define HL_STLALLOCATOR_H

// #include <cstdio>
// #include <new>
// #include <cstdlib>
// #include <limits>

// #include <memory> // STL

// // Somewhere someone is defining a max macro (on Windows),
// // and this is a problem -- solved by undefining it.

// #if defined(max)
// #undef max
// #endif

// using namespace std;

// /**
//  * @class STLAllocator
//  * @brief An allocator adapter for STL.
//  *
//  * This mixin lets you use any Heap Layers allocator as the allocator
//  * for an STL container.
//  *
//  * Example:
//  * <TT>
//  *   typedef STLAllocator<int, MyHeapType> MyAllocator;<BR>
//  *   list<int, MyAllocator> l;<BR>
//  * </TT>
//  */
// namespace HL {

// template <class T, class Super>
// class STLAllocator : public Super {

// // public:
// pub trait STLAllocator<T> {
//     pub type Alloc: RawAllocator;
//     //   typedef T value_type;
//     pub type ValueType = T;
//     //   typedef T * pointer;
//     pub type Pointer = *mut Self::ValueType;
//     //   typedef const T * const_pointer;
//     pub type ConstPointer = *const Self::ValueType;
//     //   typedef T& reference;
//     pub type Reference = &mut Self::ValueType;
//     //   typedef const T& const_reference;
//     pub type ConstReference = &Self::ValueType;
//     //   typedef std::size_t size_type;

//     //   typedef std::ptrdiff_t difference_type;
//     //   typedef std::true_type propagate_on_container_move_assignment;

//     //   template <class U>
//     //   struct rebind {
//     //     typedef STLAllocator<U,Super> other;
//     //   };

//     //   pointer address (reference x) const {
//     fn address(x: Self::Reference) -> Self::Pointer {
//         x.as_mut_ptr()
//     }

//     //   const_pointer address (const_reference x) const {
//     pub fn address_const(x: Self::ConstReference) -> Self::ConstPointer {
//         x.as_ptr()
//     }

//     //   STLAllocator() throw() {
//     //   }

//     //   STLAllocator (const STLAllocator& s) throw()
//     //     : Super (s)
//     //   {}

//     //   virtual ~STLAllocator() throw() {
//     //   }

//     //   /// Make the maximum size be the largest possible object.
//     //   size_type max_size() const
//     const fn max_size() -> usize {
//         usize::MAX / core::mem::size_of::<T>()
//     }
//     //   {
//     //     return std::numeric_limits<std::size_t>::max() / sizeof(T);
//     //   }

//     // #if defined(_WIN32)
//     //   char * _Charalloc (size_type n) {
//     //     return (char *) allocate (n);
//     //   }
//     // #endif

//     // #if defined(__SUNPRO_CC)
//     //   inline void * allocate (size_type n,
//     // 			  const void * = 0) {

//     //     if (n) {
//     //       return reinterpret_cast<void *>(Super::malloc (sizeof(T) * n));
//     //     } else {
//     //       return (void *) 0;
//     //     }
//     //   }
//     // #else
//     //   inline pointer allocate (size_type n,
//     // 			  const void * = 0) {
//         fn allocate(n: usize) -> Self::Pointer {
//             if n == 0 {
//                 return core::ptr::null_mut()
//             }
//             Self::Alloc::malloc(core::mem::size_of::<T>() * n).cast()
//     //     if (n) {
//     //       return reinterpret_cast<pointer>(Super::malloc (sizeof(T) * n));
//     //     } else {
//     //       return 0;
//     //     }
//     //   }
//         }
//     // #endif

//     //   inline void deallocate (void * p, size_type) {
//         fn deallocate(p: Self::Pointer, _: usize) {
//     //     Super::free (p);
//             Self::Alloc::free(p)
//     //   }
//         }

//     //   inline void deallocate (pointer p, size_type) {
//     //     Super::free (p);
//     //   }

//     // #if __cplusplus >= 201103L
//     //   // Updated API from C++11 to work with move constructors
//     //   template<class U, class... Args>
//     //   void construct (U* p, Args&&... args) {
//     //     new((void *) p) U(std::forward<Args>(args)...);
//     //   }

//     //   template<class U>
//     //   void destroy (U* p) {
//     //     p->~U();
//     //   }

//     // #else
//     //   // Legacy API for pre-C++11
//     //   void construct (pointer p, const T& val) {
//     //     new ((void *) p) T (val);
//     //   }

//     //   void destroy (pointer p) {
//     //     ((T*)p)->~T();
//     //   }

//     // #endif

//     //   template <class U> STLAllocator (const STLAllocator<U, Super> &) throw()
//     //   {
//     //   }

//     // };

//     //   template <typename T, class S>
//     //   inline bool operator!=(const STLAllocator<T,S>& a, const STLAllocator<T,S>& b) {
//     //     return (&a != &b);
//     //   }

//     //   template <typename T, class S>
//     //   inline bool operator==(const STLAllocator<T,S>& a, const STLAllocator<T,S>& b) {
//     //     return (&a == &b);
//     //   }

//     // }

//     // #endif
// }

pub unsafe trait RawAllocator: super::Allocator {
    const ALLIGNMENT: usize;
    fn malloc(&self, n: usize) -> *mut u8;
    fn free(&self, ptr: *mut u8, sz: usize);
    fn memalign(&self, alignment: usize, sz: usize) -> *mut u8;
    fn get_size(&self, ptr: *mut u8) -> usize;
}
