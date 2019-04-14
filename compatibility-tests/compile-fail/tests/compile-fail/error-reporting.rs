extern crate snafu;

mod wrong_error_types {
    use snafu::Snafu;

    #[derive(Snafu)]
    union AUnion { _a: i32 }
    //~^ ERROR Can only derive `Snafu` for an enum or a newtype

    #[derive(Snafu)]
    enum TupleEnumVariant {
        Alpha(i32),
        //~^ ERROR Only struct-like and unit enum variants are supported
    }
}

mod other_attributes {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum UnknownVariantAttributeIsIgnored {
        #[serde]
        //~^ ERROR The attribute `serde` is currently unknown
        Alpha
    }
}

mod display {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    enum DisplayWithoutArgument {
        #[snafu(display)]
        //~^ ERROR `snafu(display)` requires an argument
        Alpha
    }

    mod clean_style {
        use snafu::Snafu;

        #[derive(Debug, Snafu)]
        enum BadFormatString {
            #[snafu(display(foo()))]
            //~^ ERROR format argument must be a string literal
            Alpha { a: i32 },
        }
    }

    mod string_style {
        use snafu::Snafu;

        #[derive(Debug, Snafu)]
        enum NotStringLiteral {
            #[snafu(display = 42)]
            //~^ ERROR expected string literal
            Alpha { a: i32 },
        }

        #[derive(Debug, Snafu)]
        enum BadFormatString {
            #[snafu(display = "42")]
            //~^ ERROR format argument must be a string literal
            Alpha { a: i32 },
        }
    }
}

mod opaque {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    struct UnitStruct;
    //~^ ERROR tuple structs

    #[derive(Debug, Snafu)]
    struct NamedFieldStruct { alpha: i32 }
    //~^ ERROR tuple structs

    #[derive(Debug, Snafu)]
    struct ShortTupleStruct();
    //~^ ERROR exactly one field

    #[derive(Debug, Snafu)]
    struct LongTupleStruct(i32, i32);
    //~^ ERROR exactly one field
}

fn main() {}
