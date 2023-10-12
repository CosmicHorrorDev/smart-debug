/// Returns `true` is the string should be run through `format_args!()`
pub fn needs_formatting(s: &str) -> bool {
    // We used to use more complex logic here, but it got hard to track both as authors and users
    // of this crate. Just return true if there is a curly brace anywhere in the target string. If
    // the user wants a literal curly then they can escape it
    s.contains(['{', '}'])
}
