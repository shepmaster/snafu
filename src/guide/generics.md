# Using generic types

Error types enhanced by SNAFU may contain generic type and lifetime parameters.

## Types

```rust
# use snafu::{Snafu, ensure};
#
#[derive(Debug, Snafu)]
enum Error<T>
where
    T: std::fmt::Display,
{
    #[snafu(display("The value {} was too large", value))]
    TooLarge { value: T, limit: u32 },

    #[snafu(display("The value {} was too small", value))]
    TooSmall { value: T, limit: u32 },
}

fn validate_number(value: u8) -> Result<u8, Error<u8>> {
    ensure!(value <= 200, TooLarge { value, limit: 100u32 });
    ensure!(value >= 100, TooSmall { value, limit: 200u32 });
    Ok(value)
}

fn validate_string(value: &str) -> Result<&str, Error<String>> {
    ensure!(value.len() <= 20, TooLarge { value, limit: 10u32 });
    ensure!(value.len() >= 10, TooSmall { value, limit: 20u32 });
    Ok(value)
}
```

## Lifetimes

```rust
# use snafu::{Snafu, ensure};
#
#[derive(Debug, Snafu)]
enum Error<'a> {
    #[snafu(display("The username {} contains the bad word {}", value, word))]
    BadWord { value: &'a str, word: &'static str },
}

fn validate_username<'a>(value: &'a str) -> Result<&'a str, Error<'a>> {
    ensure!(!value.contains("stinks"), BadWord { value, word: "stinks" });
    ensure!(!value.contains("smells"), BadWord { value, word: "smells" });
    Ok(value)
}
```

## Caveats

A SNAFU [opaque type](crate::guide::opaque) requires that the
contained type implements several traits, such as
`Display`. However, type constraints cannot be automatically added
to the opaque type because they are not allowed to reference the
inner type without also exposing it publicly.

The best option is to avoid using a generic opaque error. If you
choose to expose a generic opaque error, you will likely need to add
explicit duplicate type constraints:

```rust
use snafu::Snafu;

#[derive(Debug, Snafu)]
struct ApiError<T>(Error<T>)
where                        // These lines are required to
   T: std::fmt::Debug;       // ensure that delegation can work.

#[derive(Debug, Snafu)]
enum Error<T>
where
    T: std::fmt::Debug,
{
    #[snafu(display("Boom: {:?}", value))]
    Boom { value: T },
}
```
