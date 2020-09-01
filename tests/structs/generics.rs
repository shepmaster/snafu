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
