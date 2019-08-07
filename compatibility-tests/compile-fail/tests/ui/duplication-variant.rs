use snafu::Snafu;

#[derive(Debug, Snafu)]
enum EnumError {
    #[snafu(display("an error variant"))]
    #[snafu(display("should not allow duplicate display"))]
    AVariant,
}

fn main() {}
