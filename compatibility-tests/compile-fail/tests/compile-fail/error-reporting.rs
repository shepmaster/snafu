extern crate snafu;

use snafu::Snafu;

#[derive(Snafu)]
struct AStruct;
//~^ ERROR Can only derive `Snafu` for an enum

#[derive(Snafu)]
union AUnion { _a: i32 }
//~^ ERROR Can only derive `Snafu` for an enum

#[derive(Debug, Snafu)]
enum UnknownVariantAttributeIsIgnored {
    #[serde]
    //~^ ERROR The attribute `serde` is currently unknown
    Alpha
}

#[derive(Snafu)]
enum TupleEnumVariant {
    Alpha(i32),
    //~^ ERROR Only struct-like and unit enum variants are supported
}

#[derive(Snafu)]
enum SnafuDisplayWrongKindOfExpression {
    #[snafu::display {}]
    //~^ ERROR A parenthesized format string with optional values is expected
    //~^^ expected one of `(` or `=`
    Alpha(i32),
}

#[derive(Snafu)]
enum OldSnafuDisplayWithoutArgument {
    #[snafu_display]
    //~^ ERROR `snafu_display` requires an argument
    Alpha
}

#[derive(Snafu)]
enum OldSnafuDisplayNonLiteral {
    #[snafu_display(foo())]
    //~^ ERROR A list of string literals is expected
    Alpha(i32),
}

#[derive(Snafu)]
enum OldSnafuDisplayNonStringLiteral {
    #[snafu_display(42)]
    //~^ ERROR A list of string literals is expected
    Alpha(i32),
}

#[derive(Snafu)]
enum OldOldSnafuDisplayNonStringLiteral {
    #[snafu_display = 42]
    //~^ ERROR A string literal is expected
    Alpha(i32),
}

#[derive(Snafu)]
enum OldOldSnafuDisplayNonExpression {
    #[snafu_display = "42"]
    //~^ ERROR A parenthesized format string with optional values is expected
    Alpha(i32),
}


fn main() {}
