# Attributes understood by the `Snafu` macro

## Controlling `Display`

There are a number of ways you can specify how the `Display` trait
will be implemented for each variant:

- `#[snafu(display("a format string with arguments: {}", info))]`

  The argument is a format string and the arguments. Available in Rust 1.34.

- `#[snafu(display = r#"("a format string with arguments: {}", info)"#)]`

  The same argument as above, but wrapped in a raw string to
  support previous Rust versions.

Each choice has the same capabilities. All of the fields of the
variant will be available and you can call methods on them, such
as `filename.display()`.

### The default `Display` implementation

It is recommended that you provide a value for `snafu(display)`, but
if it is omitted, the summary of the documentation comment will be
used. If that is not present, the name of the variant will be used.

```rust
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum Error {
    /// No user available.
    /// You may need to specify one.
    MissingUser,
    MissingPassword,
}

fn main() {
    assert_eq!(
        Error::MissingUser.to_string(),
        "No user available. You may need to specify one.",
    );
    assert_eq!(
        Error::MissingPassword.to_string(),
        "MissingPassword",
    );
}
```

## Controlling visibility

By default, each of the context selectors and their inherent
methods will be private. It is our opinion that each module should
have one or more error types that are scoped to that module,
reducing the need to deal with unrelated errors when matching and
increasing cohesiveness.

If you need access the context selectors from outside of their
module, you can use the `#[snafu(visibility)]` attribute. This can
be applied to the error type as a default visibility or to
specific context selectors.

There are a number of forms of the attribute:

- `#[snafu(visibility(X))]`

  `X` is a normal Rust visibility modifier (`pub`, `pub(crate)`,
  `pub(in some::path)`, etc.). Supported in Rust 1.34.

- `#[snafu(visibility = "X")]`

  The same argument as above, but wrapped in a string to support
  previous Rust versions.

- `#[snafu(visibility)]` will reset back to private visibility.

```
# use snafu::Snafu;
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")] // Default
enum Error {
    IsPubCrate, // Uses the default
    #[snafu(visibility)]
    IsPrivate, // Will be private
}
```

## Controlling error sources

### Selecting the source field

If your error enum variant contains other errors but the field
cannot be named `source`, or if it contains a field named `source`
which is not actually an error, you can use `#[snafu(source)]` to
indicate if a field is an underlying cause or not:

```rust
# mod another {
#     use snafu::Snafu;
#     #[derive(Debug, Snafu)]
#     pub enum Error {}
# }
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum Error {
    SourceIsNotAnError {
        #[snafu(source(false))]
        source: String,
    },

    CauseIsAnError {
        #[snafu(source)]
        cause: another::Error,
    },
}
```

### Transforming the source

If your error type contains an underlying cause that needs to be
transformed, you can use `#[snafu(source(from(...)))]`. This takes
two arguments: the real type and an expression to transform from
that type to the type held by the error.

```rust
# mod another {
#     use snafu::Snafu;
#     #[derive(Debug, Snafu)]
#     pub enum Error {}
# }
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum Error {
    SourceNeedsToBeBoxed {
        #[snafu(source(from(another::Error, Box::new)))]
        source: Box<another::Error>,
    },
}

#[derive(Debug, Snafu)]
#[snafu(source(from(Error, Box::new)))]
struct ApiError(Box<Error>);
```

Note: If you specify `#[snafu(source(from(...)))]` then the field
will be treated as a source, even if it's not named "source" - in
other words, `#[snafu(source(from(...)))]` implies
`#[snafu(source)]`.

## Controlling backtraces

If your error enum variant contains a backtrace but the field
cannot be named `backtrace`, or if it contains a field named
`backtrace` which is not actually a backtrace, you can use
`#[snafu(backtrace)]` to indicate if a field is actually a
 backtrace or not:

```rust
# use snafu::{Backtrace, Snafu};
#[derive(Debug, Snafu)]
enum Error {
    BacktraceIsNotABacktrace {
        #[snafu(backtrace(false))]
        backtrace: bool,
    },

    TraceIsABacktrace {
        #[snafu(backtrace)]
        trace: Backtrace,
    },
}
```

If your error contains other SNAFU errors which can report
backtraces, you may wish to delegate returning a backtrace to
those errors. To specify this, use `#[snafu(backtrace)]` on the
source field representing the other error:

```rust
# mod another {
#     use snafu::Snafu;
#     #[derive(Debug, Snafu)]
#     pub enum Error {}
# }
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum Error {
    MyError {
        #[snafu(backtrace)]
        source: another::Error,
    },
}
```
