#![doc = include_str!("../README.md")]

// TODO: unhide after we have wrappers for people to use
#[doc(hidden)]
pub mod wrappers;

#[doc(inline)]
pub use smart_debug_derive::SmartDebug;
