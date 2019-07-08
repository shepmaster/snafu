mod enum_duplication {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    #[snafu(visibility(pub))]
    enum EnumError {
        #[snafu(display("an error variant"))]
        #[snafu(display("should not allow duplicate display"))]
        #[snafu(visibility(pub))]
        #[snafu(visibility(pub))]
        AVariant {
            #[snafu(source(from(EnumError, Box::new)))]
            #[snafu(backtrace(delegate))]
            source: Box<EnumError>,
            #[snafu(source(from(EnumError, Box::new)))]
            #[snafu(backtrace(delegate))]
            source2: Box<EnumError>,
            #[snafu(source)]
            #[snafu(backtrace(delegate))]
            source3: String,
            #[snafu(source)]
            #[snafu(source)]
            #[snafu(backtrace(delegate))]
            source4: String,

            #[snafu(backtrace)]
            backtrace1: String,
            #[snafu(backtrace)]
            backtrace2: String,
            #[snafu(backtrace)]
            #[snafu(backtrace)]
            #[snafu(backtrace)]
            backtrace3: String,
        },
    }
}

mod struct_duplication {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum UsableError {}

    #[derive(Debug, Snafu)]
    #[snafu(source(from(UsableError, Box::new)))]
    #[snafu(source(from(UsableError, Box::new)))]
    struct StructError(Box<UsableError>);
}

fn main() {}
