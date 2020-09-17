# Struct errors

You may not always need the flexibility of an enum for your error
type. In those cases, you can use the familiar SNAFU attributes with a
struct:

```rust
# use std::convert::TryFrom;
# use snafu::Snafu;
#[derive(Debug, Snafu)]
#[snafu(display("Unable to parse {} as MyEnum", value))]
struct ParseError {
    value: u8,
}

// That's all it takes! The rest is demonstration of how to use it.

#[derive(Debug)]
enum MyEnum {
    Alpha,
    Beta,
    Gamma,
}

impl TryFrom<u8> for MyEnum {
    type Error = ParseError;

    fn try_from(other: u8) -> Result<Self, Self::Error> {
        match other {
            0 => Ok(Self::Alpha),
            1 => Ok(Self::Beta),
            2 => Ok(Self::Gamma),
            value => ParseContext { value }.fail()
        }
    }
}
```

## Differences from enum errors

While each enum error variant creates a context selector that matches
the variant's name, context selectors for structs remove the suffix
`Error` from the name of the error, if present,  and add `Context`:

```rust
# use snafu::Snafu;
#[derive(Debug, Snafu)]
struct StructError;

fn struct_demonstration() -> Result<(), StructError> {
    StructContext.fail() // This differs from the struct name
}

#[derive(Debug, Snafu)]
enum EnumError {
    EnumExample,
}

fn enum_demonstration() -> Result<(), EnumError> {
    EnumExample.fail() // This matches the name of the enum variant
}
```
