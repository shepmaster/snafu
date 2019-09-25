mod inner {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub struct Error(InnerError);

    pub fn api() -> Result<i32, Error> {
        Ok(a()? + b()?)
    }

    #[derive(Debug, Snafu)]
    enum InnerError {
        #[snafu(display = r#"("The value {} is too big", count)"#)]
        TooBig { count: i32 },
    }

    fn a() -> Result<i32, InnerError> {
        TooBig { count: 1 }.fail()
    }

    fn b() -> Result<i32, InnerError> {
        TooBig { count: 2 }.fail()
    }
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<inner::Error>();
    let e = inner::api().unwrap_err();
    assert!(e.to_string().contains("too big"));
}
