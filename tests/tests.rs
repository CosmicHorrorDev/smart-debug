use serde::Serialize;
use smart_debug::SmartDebug;

#[test]
fn basic() {
    #[derive(Serialize, SmartDebug, Default)]
    #[debug(ignore_defaults)]
    struct Basic {
        default: bool,
        not_default: bool,
    }

    let basic = Basic {
        default: false,
        not_default: true,
    };

    insta::with_settings!({ info => &basic }, {
        insta::assert_debug_snapshot!(basic)
    });
}

#[test]
fn kitchen_sink() {
    struct Wrapper<'a>(&'a str);

    impl<'a> std::fmt::Debug for Wrapper<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Wrapped")
        }
    }

    #[derive(Serialize, SmartDebug, Default)]
    #[debug(ignore_defaults)]
    struct KitchenSink {
        default_ignored: (),
        #[debug(ignore)]
        ignore_ignored: (),
        #[debug(no_ignore)]
        no_ignore_displayed: (),
        #[debug(ignore_if = false)]
        ignore_if_ignored: bool,
        #[debug(ignore_if = true)]
        ignore_if_displayed: bool,
        #[debug("<hidden>")]
        password_is_hidden: &'static str,
        #[debug(wrapper = Wrapper)]
        wrapped_display: &'static str,
    }

    let kitchen_sink = KitchenSink {
        password_is_hidden: "hunter2",
        wrapped_display: "NotWrapped",
        ..Default::default()
    };

    insta::with_settings!({ info => &kitchen_sink }, {
        insta::assert_debug_snapshot!(kitchen_sink)
    });
}

#[test]
fn format_str() {
    #[derive(Serialize, SmartDebug, Default)]
    struct FormatStrs {
        #[debug("{:#x}")]
        hex: u64,
        #[debug("{}")]
        display: &'static str,
        #[debug("I am the text")]
        just_text: (),
    }

    let format_strs = FormatStrs {
        hex: 0x1234abcd,
        display: "You'll see unescaped quotes -> \"\"\"",
        just_text: (),
    };

    insta::with_settings!({ info => &format_strs }, {
        insta::assert_debug_snapshot!(format_strs)
    });
}
