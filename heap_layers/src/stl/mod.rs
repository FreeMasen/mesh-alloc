pub mod allocator;

extern crate alloc;

#[cfg(not(feature = "nightly"))]
pub(crate) use allocator_api2::{alloc::{AllocError, Allocator, handle_alloc_error}, boxed::Box};
#[cfg(feature = "nightly")]
use core::alloc::{AllocError, Allocator, handle_alloc_error};
#[cfg(feature = "nightly")]
use alloc::boxed::Box;



pub trait FlexableAllocator: Allocator + allocator::RawAllocator {}
