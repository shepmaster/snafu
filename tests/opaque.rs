fn check<T: std::error::Error>() {}

mod inner {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    pub struct Error(InnerError);

    pub fn api() -> Result<i32, Error> {
        Ok(a()? + b()?)
    }

    pub fn not_positive(value: i32) -> Result<i32, Error> {
        ensure!(value < 1, TooBigSnafu { count: value });
        Ok(value)
    }

    pub fn boxed_inner(value: i32) -> Result<i32, Box<dyn std::error::Error>> {
        ensure!(value < 1, TooBigSnafu { count: value });
        Ok(value)
    }

    #[derive(Debug, Snafu)]
    enum InnerError {
        #[snafu(display("The value {count} is too big"))]
        TooBig { count: i32 },
    }

    fn a() -> Result<i32, InnerError> {
        TooBigSnafu { count: 1 }.fail()
    }

    fn b() -> Result<i32, InnerError> {
        TooBigSnafu { count: 2 }.fail()
    }
}

#[test]
fn implements_error() {
    check::<inner::Error>();
    let e = inner::api().unwrap_err();
    assert!(e.to_string().contains("too big"));
}

#[test]
fn ensure_opaque() {
    assert!(inner::not_positive(-1).is_ok());

    let e = inner::not_positive(2).unwrap_err();
    assert!(e.to_string().contains("too big"));
}

#[test]
fn ensure_boxed() {
    assert!(inner::boxed_inner(-1).is_ok());

    let e = inner::boxed_inner(2).unwrap_err();
    assert!(e.to_string().contains("too big"));
}

mod with_exact_source {
    use super::*;
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(display("The inner error"))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct MiddleError(InnerError);

    #[derive(Debug, Snafu)]
    #[snafu(source(from(exact)))]
    struct OuterError(MiddleError);

    trait LocalTrait {}
    impl LocalTrait for i32 {}

    impl<T> From<T> for OuterError
    where
        T: LocalTrait,
    {
        fn from(_: T) -> Self {
            OuterError(MiddleError(InnerError))
        }
    }

    #[test]
    fn usage() {
        check::<OuterError>();
        let e: OuterError = 42.into();
        assert_eq!(e.to_string(), "The inner error");
    }
}

mod with_generic_source {
    use super::*;
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(display("The inner error"))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct MiddleError(InnerError);

    #[derive(Debug, Snafu)]
    #[snafu(source(from(generic)))]
    struct OuterError(MiddleError);

    fn make_inner() -> Result<(), InnerError> {
        InnerSnafu.fail()
    }

    fn make_outer() -> Result<(), OuterError> {
        Ok(make_inner()?)
    }

    #[test]
    fn usage() {
        check::<OuterError>();
        let e: OuterError = make_outer().unwrap_err();
        assert_eq!(e.to_string(), "The inner error");
    }
}
