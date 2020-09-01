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
