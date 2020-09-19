mod wrong_error_types {
    use snafu::Snafu;

    #[derive(Snafu)]
    union AUnion {
        _a: i32,
    }

    #[derive(Snafu)]
    enum TupleEnumVariant {
        Alpha(i32),
    }
}

mod other_attributes {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum Error {
        #[serde]
        UnknownVariantAttributeIsIgnored,
    }
}

mod display {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum FailedAttributeParsing {
        #[snafu(display)]
        DisplayWithoutArgument,

        #[snafu(display = 42)]
        StringStyleNotStringLiteral,
    }

    #[derive(Debug, Snafu)]
    enum InvalidGeneratedCode {
        #[snafu(display(foo()))]
        CleanStyleBadFormatString,

        #[snafu(display = "42")]
        StringStyleBadFormatString,
    }
}

mod opaque {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    struct UnitStruct;

    #[derive(Debug, Snafu)]
    struct NamedFieldStruct {
        some_field: i32,
    }

    #[derive(Debug, Snafu)]
    struct ShortTupleStruct();

    #[derive(Debug, Snafu)]
    struct LongTupleStruct(i32, i32);
}

fn main() {}
