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
