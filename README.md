**⚠️ !!This is currently pre-alpha software!! ⚠️**

Allows for easy manipulation of debug formatting through a derive macro

```rust
use smart_debug::SmartDebug;
use std::fmt;

#[derive(SmartDebug, Default)]
#[debug(ignore_defaults)]
struct Text {
    #[debug(no_ignore)]
    text: &'static str,
    #[debug(wrapper = DebugInline)]
    hyperlink: Option<&'static str>,
    is_bold: bool,
    is_italics: bool,
}

// Wrapper that displays inline even when using pretty formatting (`{:#?}`)
struct DebugInline<'inner, T>(pub &'inner T);

impl<T: fmt::Debug> fmt::Debug for DebugInline<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

const EXPECTED: &str = r#"
Text {
    text: "Look! A link",
    hyperlink: Some("https://example.org"),
    ..
}
"#;

fn main() {
    let text = Text {
        text: "Look! A link",
        hyperlink: Some("https://example.org"),
        ..Text::default()
    };

    let formatted = format!("\n{text:#?}\n");
    assert_eq!(formatted, EXPECTED);
}
```
