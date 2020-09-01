mod lifetimes {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    struct Error<'a> {
        key: &'a i32,
    }

    #[test]
    fn are_allowed() {
        let key = 42;
        let e = Context { key: &key }.build();
        assert_eq!(*e.key, key);
    }
}

mod types {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    struct Error<T> {
        key: T,
    }

    #[test]
    fn are_allowed() {
        let key = 42;
        let e: Error<i32> = Context { key }.build();
        assert_eq!(e.key, key);
    }

    mod with_defaults {
        use snafu::{AsErrorSource, ResultExt, Snafu};
        use std::{error::Error as StdError, fmt::Debug, io};

        #[derive(Debug, Snafu)]
        struct Error<S = io::Error, T = String>
        where
            S: StdError + AsErrorSource,
            T: Debug,
        {
            source: S,
            key: T,
        }

        #[test]
        fn allows_non_default_types() {
            #[derive(Debug, Snafu)]
            struct AnotherError;

            let r = AnotherContext.fail::<()>();
            let _e: Error<_, u8> = r.context(Context { key: 42 }).unwrap_err();
        }
    }
}

mod bounds {
    mod inline {
        use snafu::Snafu;
        use std::fmt::Display;

        #[derive(Debug, Snafu)]
        #[snafu(display("key: {}", key))]
        struct Error<T: Display> {
            key: T,
        }

        #[test]
        fn are_preserved() {
            let e: Error<bool> = Context { key: true }.build();
            let display = e.to_string();
            assert_eq!(display, "key: true");
        }
    }

    mod where_clause {
        use snafu::Snafu;
        use std::fmt::Display;

        #[derive(Debug, Snafu)]
        #[snafu(display("key: {}", key))]
        struct Error<T>
        where
            T: Display,
        {
            key: T,
        }

        #[test]
        fn are_preserved() {
            let e: Error<bool> = Context { key: true }.build();
            let display = e.to_string();
            assert_eq!(display, "key: true");
        }
    }
}
