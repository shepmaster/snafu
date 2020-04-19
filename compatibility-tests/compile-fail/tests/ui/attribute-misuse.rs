mod enum_misuse {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(display("display should not work here"))]
    #[snafu(source(from(XXXX, Box::new)))]
    #[snafu(source(true))]
    #[snafu(backtrace)]
    #[snafu(context)]
    enum EnumError {
        AVariant,
    }
}

mod variant_misuse {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum EnumError {
        // Make sure we catch the error in the second attribute
        #[snafu(display("an error variant"), source(from(XXXX, Box::new)))]
        #[snafu(source)]
        #[snafu(backtrace)]
        #[snafu(crate_root(XXXX))]
        AVariant,
    }
}

mod field_misuse {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum EnumError {
        AVariant {
            #[snafu(display("display should not work here"))]
            #[snafu(visibility(pub))]
            #[snafu(source(false))]
            #[snafu(source(from(XXXX, Box::new)))]
            #[snafu(context)]
            #[snafu(crate_root(XXXX))]
            source: String,
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
    #[snafu(context)]
    struct StructError(Box<UsableError>);
}

fn main() {}
