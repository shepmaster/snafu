extern crate snafu;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub(super) enum Error {
        // We'll test both of these attributes inside `#[snafu(...)]`
        #[snafu(visibility(pub(super)), display("Moo"))]
        Alpha,
    }
}

// Confirm `pub(super)` is applied to the generated struct
#[test]
fn is_visible() {
    let _ = error::Alpha;
}

// Confirm `display("Moo")` is applied to the variant
#[test]
fn has_display() {
    let err = error::Error::Alpha;
    assert_eq!(format!("{}", err), "Moo");
}
