use snafu::{ensure, Snafu, ResultExt};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("{}", r#type))]
    Example { r#type: String },

    r#Awesome {
        #[snafu(source(from(Error, Box::new)))]
        r#mod: Box<Error>
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();
}

#[test]
fn creates_context_selectors() {
    fn one(success: bool) -> Result<()> {
        ensure!(success, Example { r#type: "boom" });
        Ok(())
    }

    fn two(success: bool) -> Result<()> {
        one(success).context(r#Awesome)?;
        Ok(())
    }

    assert!(two(true).is_ok());
    assert!(two(false).is_err());
}