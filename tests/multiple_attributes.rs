use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum InnerError {
    InnerVariant,
}

mod error {
    use super::InnerError;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub(super) enum Error {
        // We'll test both of these attributes inside `#[snafu(...)]`
        #[snafu(visibility(pub(super)), display("Moo"))]
        Alpha {
            // Ensure we can have multiple field attributes as well
            #[snafu(source, backtrace)]
            cause: InnerError,
        },
    }
}

// Confirm `pub(super)` is applied to the generated struct
#[test]
fn is_visible() {
    let _ = error::Alpha;
}

fn example() -> Result<u8, InnerError> {
    InnerVariant.fail()
}

// Confirm `display("Moo")` is applied to the variant
#[test]
fn has_display() {
    let err = example().context(error::Alpha).unwrap_err();
    assert_eq!(format!("{}", err), "Moo");
}
