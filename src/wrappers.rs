// NONE OF THIS IS PART OF THE PUBLIC API :P
#[doc(hidden)]
pub mod __private {
    use std::fmt;

    pub struct __DebugArgs<'args>(pub fmt::Arguments<'args>);

    impl fmt::Debug for __DebugArgs<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_fmt(self.0)
        }
    }
}
