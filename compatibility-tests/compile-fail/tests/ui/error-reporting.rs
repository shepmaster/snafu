mod wrong_error_types {
    use snafu::Snafu;

    #[derive(Snafu)]
    union AUnion { _a: i32 }

    #[derive(Snafu)]
    enum TupleEnumVariant {
        Alpha(i32),
    }
}

mod other_attributes {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum UnknownVariantAttributeIsIgnored {
        #[serde]
        Alpha
    }
}

mod display {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum DisplayWithoutArgument {
        #[snafu(display)]
        Alpha
    }

    mod clean_style {
        use snafu::Snafu;

        #[derive(Debug, Snafu)]
        enum BadFormatString {
            #[snafu(display(foo()))]
            Alpha { a: i32 },
        }
    }

    mod string_style {
        use snafu::Snafu;

        #[derive(Debug, Snafu)]
        enum NotStringLiteral {
            #[snafu(display = 42)]
            Alpha { a: i32 },
        }

        #[derive(Debug, Snafu)]
        enum BadFormatString {
            #[snafu(display = "42")]
            Alpha { a: i32 },
        }
    }
}

mod opaque {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    struct UnitStruct;

    #[derive(Debug, Snafu)]
    struct NamedFieldStruct { alpha: i32 }

    #[derive(Debug, Snafu)]
    struct ShortTupleStruct();

    #[derive(Debug, Snafu)]
    struct LongTupleStruct(i32, i32);
}

fn main() {}
