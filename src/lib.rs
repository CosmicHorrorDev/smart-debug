// TODO: include compile tests to test for error messages when it fails to build. dtolnay has some
//       library for it already
// TODO: should `debug("Blah")` be `Blah` or `"Blah"` when formatted?
// TODO: rename ignore -> skip
// TODO: definitely need to check error messages to make sure that the error spans for things make
//       sense
// TODO: Get wrapper working as a container attr
#![doc = include_str!("../README.md")]

#[doc(inline)]
pub use smart_debug_derive::SmartDebug;

/// NOT PART OF THE PUBLIC API
#[doc(hidden)]
pub mod internal {
    use std::fmt;

    pub struct __IgnoredTupleField;

    impl fmt::Debug for __IgnoredTupleField {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            // Specifics of the formatting target shouldn't matter here since it's always `_`
            write!(f, "{:?}", __LiteralField(format_args!("_")))
        }
    }

    pub struct __LiteralField<'args>(pub fmt::Arguments<'args>);

    impl fmt::Debug for __LiteralField<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(self.0)
        }
    }
}
