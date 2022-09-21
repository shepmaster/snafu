#![cfg(test)]

// There are not yet any feature differences that require testing for Rust 1.34 compatibility.

use std::io;

fn io_failure() -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other, "arbitrary failure"))
}

fn implements_error<T: std::error::Error>() {}

mod enum_style {
    use super::*;
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    enum Error {
        #[snafu(display("Without a source: {}", id))]
        WithoutSource { id: i32 },

        #[snafu(display("With a source: {}", source))]
        WithSource { id: i32, source: io::Error },
    }

    type Result<T, E = Error> = std::result::Result<T, E>;

    fn create_without_source() -> Result<()> {
        WithoutSourceSnafu { id: 42 }.fail()
    }

    fn create_with_source() -> Result<()> {
        io_failure().context(WithSourceSnafu { id: 42 })
    }

    #[test]
    fn it_works() {
        implements_error::<Error>();
        let _ = create_without_source();
        let _ = create_with_source();
    }
}

mod struct_style {
    use super::*;
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(display("Without a source: {}", id))]
    struct WithoutSource {
        id: i32,
    }

    #[derive(Debug, Snafu)]
    #[snafu(display("With a source: {}", source))]
    struct WithSource {
        id: i32,
        source: io::Error,
    }

    fn create_without_source() -> Result<(), WithoutSource> {
        WithoutSourceSnafu { id: 42 }.fail()
    }

    fn create_with_source() -> Result<(), WithSource> {
        io_failure().context(WithSourceSnafu { id: 42 })
    }

    #[test]
    fn it_works() {
        implements_error::<WithoutSource>();
        implements_error::<WithSource>();
        let _ = create_without_source();
        let _ = create_with_source();
    }
}

mod opaque_style {
    use super::*;
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    struct Dummy;

    #[derive(Debug, Snafu)]
    struct Opaque(Dummy);

    fn create() -> Result<(), Opaque> {
        Ok(DummySnafu.fail()?)
    }

    #[test]
    fn it_works() {
        implements_error::<Opaque>();
        let _ = create();
    }
}

mod report {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    struct Error;

    #[test]
    #[snafu::report]
    fn it_works() -> Result<(), Error> {
        Ok(())
    }
}
