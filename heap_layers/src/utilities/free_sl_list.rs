use crate::stl::{Allocator, Box};
use core::{ptr::NonNull, marker::PhantomData};
// // -*- C++ -*-

// #ifndef HL_FREESLLIST_H_
// #define HL_FREESLLIST_H_

// #include <assert.h>

// /**
//  * @class FreeSLList
//  * @brief A "memory neutral" singly-linked list,
//  *
//  * Uses the free space in objects to store
//  * the pointers.
//  */

// class FreeSLList {
// public:
pub struct FreeSLList<
    T,
    A: Allocator,
> {
    pub head: Option<NonNull<Node<T>>>,
    pub alloc: A,
    pub marker: PhantomData<Box<Node<T>, A>>,
}

impl<T, A: Allocator> FreeSLList<T, A> {
    pub fn new(alloc: A) -> Self {
        Self {
            head: None,
            alloc,
            marker: PhantomData,
        }
    }
}

pub struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    element: T,
}

impl<T, A: Allocator> FreeSLList<T, A> 
{


//   inline void clear() {
//     head.next = nullptr;
//   }
    pub fn clear(&mut self) {
        self.head = None;
    }

//   class Entry;

//   /// Get the head of the list.
//   inline Entry * get() {
    pub fn get(&mut self) -> Option<Box<Node<T>, &A>> {
//     const Entry * e = head.next;
    self.head.map(|node| unsafe {
        let node = Box::from_raw_in(node.as_ptr(), &self.alloc);
        self.head = node.next;
        node
    })
//     if (e == nullptr) {
//       return nullptr;
//     }
//     head.next = e->next;
//     return const_cast<Entry *>(e);
//   }
    }

//   inline Entry * remove() {
    pub fn remove(&mut self) -> Option<Box<Node<T>, &A>> {
        self.get()
    }
//     const Entry * e = head.next;
//     if (e == nullptr) {
//       return nullptr;
//     }
//     head.next = e->next;
//     return const_cast<Entry *>(e);
//   }

//   inline void insert (void * e) {
    pub fn insert(&mut self, elt: T) {
        let node = Box::new_in(Node::new(elt), &self.alloc);
        let node: NonNull<Node<T>> = NonNull::from(Box::leak(node));
        // SAFETY: node_ptr is a unique pointer to a node we boxed with self.alloc and leaked
        // This method takes care not to create mutable references to whole nodes,
        // to maintain validity of aliasing pointers into `element`.
        unsafe {
            (*node.as_ptr()).next = self.head.take();
            let node = Some(node);
            self.head = node;
        }
    }
//     Entry * entry = reinterpret_cast<Entry *>(e);
//     entry->next = head.next;
//     head.next = entry;
//   }

//   class Entry {
//   public:
//     Entry()
//       : next (nullptr)
//     {}
//     Entry * next;
//   private:
//     Entry (const Entry&);
//     Entry& operator=(const Entry&);
//   };

// private:
//   Entry head;
// };
}
// #endif
impl<T> Node<T> {
    pub fn new(v: T) -> Self {
        Self {
            element: v,
            next: None,
        }
    }

    pub fn into_element<A: Allocator>(boxed: Box<Self, A>) -> T {
        Box::into_inner(boxed).element
    }
}
