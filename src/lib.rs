// TODO: include compile tests to test for error messages when it fails to build. dtolnay has some
//       library for it already
// TODO: should `debug("Blah")` be `Blah` or `"Blah"` when formatted?
// TODO: It's currently impossible to have a debug format with just `{}` due to interpol detection.
//       Maybe have it so that it always uses `format_args!()` when `{` is used?
// TODO: switch back to single crate. No need for wrapper now
// TODO: rename ignore -> skip
// TODO: definitely need to check error messages to make sure that the error spans for things make
//       sense
#![doc = include_str!("../README.md")]

use std::fmt::{self, Write};

#[doc(inline)]
pub use smart_debug_derive::SmartDebug;

/// NOT PART OF THE PUBLIC API
// Has the debug repr `_`
#[doc(hidden)]
pub struct __IgnoredTupleStructField;

impl fmt::Debug for __IgnoredTupleStructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('_')
    }
}
