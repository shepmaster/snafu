mod enum_misuse {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(display("display should not work here"))]
    #[snafu(source(from(XXXX, Box::new)))]
    #[snafu(source(true))]
    #[snafu(backtrace)]
    enum EnumError {
        #[snafu(display("an error variant"))]
        #[snafu(source(from(XXXX, Box::new)))]
        #[snafu(backtrace)]
        AVariant {
            #[snafu(display("display should not work here"))]
            #[snafu(visibility(pub))]
            #[snafu(source(false))]
            #[snafu(source(from(XXXX, Box::new)))]
            a: String,
        },
    }
}

mod struct_misuse {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum UsableError {}

    #[derive(Debug, Snafu)]
    #[snafu(display("display should not work here"))]
    #[snafu(source(from(UsableError, Box::new)))]
    #[snafu(visibility(pub))]
    #[snafu(source(true))]
    #[snafu(backtrace)]
    struct StructError(Box<UsableError>);
}

fn main() {}
