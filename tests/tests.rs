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
        insta::assert_debug_snapshot!(basic);
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
        insta::assert_debug_snapshot!(kitchen_sink);
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
        ..Default::default()
    };

    insta::with_settings!({ info => &format_strs }, {
        insta::assert_debug_snapshot!(format_strs)
    });
}

#[test]
fn unit_struct() {
    #[derive(Serialize, SmartDebug)]
    struct Unit;

    let unit = Unit;

    insta::with_settings!({ info => &unit }, {
        insta::assert_debug_snapshot!(unit);
    });
}

#[test]
fn tuple_struct() {
    #[derive(Serialize, SmartDebug, Default)]
    struct Tuple(bool, u8);

    let tuple = Tuple::default();

    insta::with_settings!({info => &tuple}, {
        insta::assert_debug_snapshot!(tuple);
    });

    #[derive(Serialize, SmartDebug, Default)]
    #[debug(ignore)]
    struct GlobalIgnore((), ());

    let global_ignore = GlobalIgnore::default();

    insta::with_settings!({ info => &global_ignore }, {
        insta::assert_debug_snapshot!(global_ignore);
    });

    #[derive(Serialize, SmartDebug)]
    #[debug(ignore)]
    struct GlobalIgnoreWithLocalOverride(#[debug(no_ignore)] &'static str, ());

    let global_w_local = GlobalIgnoreWithLocalOverride("Local override", ());

    insta::with_settings!({ info => &global_w_local }, {
        insta::assert_debug_snapshot!(global_w_local);
    });

    #[derive(Serialize, SmartDebug, Default)]
    struct FirstFieldIgnored(#[debug(ignore)] (), ());

    let first_field_ignored = FirstFieldIgnored::default();

    insta::with_settings!({ info => &first_field_ignored }, {
        insta::assert_debug_snapshot!(first_field_ignored);
    });
}

#[test]
fn container_lit_str() {
    #[derive(Serialize, SmartDebug)]
    #[debug("This is a custom debug repr")]
    struct CustomDebugRepr;

    let custom = CustomDebugRepr;

    insta::with_settings!({ info => &custom }, {
        insta::assert_debug_snapshot!(custom);
    });
}
