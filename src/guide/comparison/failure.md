# SNAFU vs. Failure

This comparison was made against the examples in [the guide for
failure 0.1.5][failure-guide].

[failure-guide]: https://rust-lang-nursery.github.io/failure/guidance.html

## "Strings as errors"

It's unclear what benefit Failure provides here. If you are using this
functionality, we recommend using the standard library's `Box<dyn
Error>`:

```rust
fn example() -> Result<(), Box<dyn std::error::Error>> {
    Err(format!("Something went bad: {}", 1 + 1))?;
    Ok(())
}
```

If you wanted to do something similar with SNAFU, you can create a
single-variant error enum with `String` data:

```rust
use snafu::Snafu;
use std::ops::Range;

#[derive(Debug, Snafu)]
enum Error {
    Any { detail: String },
}

fn check_range(x: usize, range: Range<usize>) -> Result<usize, Error> {
    if x < range.start {
        return Any {
            detail: format!("{} is below {}", x, range.start),
        }
        .fail();
    }
    if x >= range.end {
        return Any {
            detail: format!("{} is above {}", x, range.end),
        }
        .fail();
    }
    Ok(x)
}
```

This could be enhanced in a few ways:

- create methods on your `Error` type
- create a custom macro
- add a [`Backtrace`][Backtrace] to the enum variant

For example:

```rust
use snafu::{Backtrace, Snafu};
use std::ops::Range;

#[derive(Debug, Snafu)]
enum Error {
    Any {
        detail: String,
        backtrace: Backtrace,
    },
}

macro_rules! format_err {
    ($($arg:tt)*) => { Any { detail: format!($($arg)*) }.fail() }
}

fn check_range(x: usize, range: Range<usize>) -> Result<usize, Error> {
    if x < range.start {
        return format_err!("{} is below {}", x, range.start);
    }
    if x >= range.end {
        return format_err!("{} is above {}", x, range.end);
    }
    Ok(x)
}
```

Please see the next section for the recommended pattern for this error.

[Backtrace]: crate::Backtrace

## "A Custom Fail type" and "Using the Error type"

These two idioms from Failure are combined into one primary use case
in SNAFU. Additionally, SNAFU avoids the downsides listed in the
Failure guide.

You can represent multiple types of errors, allocation is not
required, and you can include any extra information relevant to the
error:

```rust
use snafu::{ensure, Snafu};
use std::ops::Range;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("{} is below {}", value, bound))]
    Below { value: usize, bound: usize },

    #[snafu(display("{} is above {}", value, bound))]
    Above { value: usize, bound: usize },
}

fn check_range(value: usize, range: Range<usize>) -> Result<usize, Error> {
    ensure!(value >= range.start, Below { value, bound: range.start });
    ensure!(value < range.end, Above { value, bound: range.end });
    Ok(value)
}
```

You do not have to have a one-to-one relationship between an
underlying error and an error variant:

```rust
use snafu::{ResultExt, Snafu};
use std::num::ParseIntError;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display(r#"Could not parse the area code from "{}": {}"#, value, source))]
    AreaCodeInvalid {
        value: String,
        source: ParseIntError,
    },

    #[snafu(display(r#"Could not parse the phone exchange from "{}": {}"#, value, source))]
    PhoneExchangeInvalid {
        value: String,
        source: ParseIntError,
    },
}

fn two_errors_from_same_underlying_error(
    area_code: &str,
    exchange: &str,
) -> Result<(i32, i32), Error> {
    let area_code: i32 = area_code
        .parse()
        .context(AreaCodeInvalid { value: area_code })?;
    let exchange: i32 = exchange
        .parse()
        .context(PhoneExchangeInvalid { value: exchange })?;
    Ok((area_code, exchange))
}
```

## "An Error and ErrorKind pair"

If you choose to make your error type [opaque][], you can implement
methods on the opaque type, allowing you to selectively choose what
your public API is.

This includes the ability to return a different public enum that
users can match on without knowing the details of your error
implementation.

```rust
use snafu::Snafu;

#[derive(Debug, Snafu)]
enum InnerError {
    MyError1 { username: String },
    MyError2 { username: String },
    MyError3 { address: String },
}

#[derive(Debug, Snafu)]
pub struct Error(InnerError);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Authorization,
    Network,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        use InnerError::*;

        match self.0 {
            MyError1 { .. } | MyError2 { .. } => ErrorKind::Authorization,
            MyError3 { .. } => ErrorKind::Network,
        }
    }

    pub fn username(&self) -> Option<&str> {
        use InnerError::*;

        match &self.0 {
            MyError1 { username } | MyError2 { username } => Some(username),
            _ => None,
        }
    }
}

# fn main() {}
```

[opaque]: crate::guide::opaque
