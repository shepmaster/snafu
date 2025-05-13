mod enum_misuse {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(display("display should not work here"))]
    #[snafu(source(from(XXXX, Box::new)))]
    #[snafu(source(true))]
    #[snafu(backtrace)]
    #[snafu(context)]
    #[snafu(context(false))]
    #[snafu(context(name(Name)))]
    #[snafu(implicit)]
    #[snafu(provide)]
    #[snafu(provide(u8 => 0))]
    #[snafu(transparent)]
    enum EnumError {
        AVariant,
    }
}

mod variant_misuse {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    enum EnumError {
        // Make sure we catch the error in the second attribute
        #[snafu(display("an error variant"), source(from(XXXX, Box::new)))]
        #[snafu(source)]
        #[snafu(backtrace)]
        #[snafu(crate_root(XXXX))]
        #[snafu(implicit)]
        #[snafu(provide)]
        AVariant,
    }
}

mod field_misuse {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    enum EnumError {
        AVariant {
            #[snafu(display("display should not work here"))]
            #[snafu(visibility(pub))]
            #[snafu(source(false))]
            #[snafu(source(from(XXXX, Box::new)))]
            #[snafu(context)]
            #[snafu(context(false))]
            #[snafu(context(suffix(Suffix)))]
            #[snafu(context(name(Name)))]
            #[snafu(crate_root(XXXX))]
            #[snafu(transparent)]
            source: String,

            #[snafu(provide(false))]
            #[snafu(provide(u8 => 0))]
            not_a_source: bool,
        },
    }
}

mod struct_misuse {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    enum UsableError {}

    #[derive(Debug, Snafu)]
    #[snafu(display("display should not work here"))]
    #[snafu(source(from(UsableError, Box::new)))]
    #[snafu(visibility(pub))]
    #[snafu(source(true))]
    #[snafu(backtrace)]
    #[snafu(context)]
    #[snafu(context(false))]
    #[snafu(context(suffix(Suffix)))]
    #[snafu(context(name(Name)))]
    #[snafu(implicit)]
    #[snafu(provide)]
    #[snafu(provide(u8 => 0))]
    #[snafu(transparent)]
    struct StructError(Box<UsableError>);

    mod field_misuse {
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        struct Opaque(
            #[snafu(backtrace)]
            #[snafu(context)]
            #[snafu(context(false))]
            #[snafu(context(suffix(Suffix)))]
            #[snafu(context(name(Name)))]
            #[snafu(crate_root(nowhere))]
            #[snafu(display("display should not work here"))]
            #[snafu(implicit)]
            #[snafu(module)]
            #[snafu(provide)]
            #[snafu(source)]
            #[snafu(transparent)]
            #[snafu(visibility(pub))]
            #[snafu(whatever)]
            Box<InnerError>,
        );

        #[derive(Debug, Snafu)]
        struct InnerError;
    }
}

fn main() {}
