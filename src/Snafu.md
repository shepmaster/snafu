The `Snafu` macro is the entrypoint to defining your own error
types. It is designed to require little configuration for the
recommended and typical usecases while still offering flexibility for
unique situations.

- [`backtrace`](#controlling-backtraces)
- [`context`](#controlling-context)
- [`crate_root`](#controlling-how-the-snafu-crate-is-resolved)
- [`display`](#controlling-display)
- [`source`](#controlling-error-sources)
- [`visibility`](#controlling-visibility)
- [`whatever`](#controlling-stringly-typed-errors)

## Controlling `Display`

You can specify how the `Display` trait will be implemented for each
variant. The argument is a format string and the arguments. All of the
fields of the variant will be available and you can call methods on
them, such as `filename.display()`.

**Example**

```rust
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("The user {} could not log in", username))]
    InvalidLogin { username: String, password: String },
}

fn main() {
    assert_eq!(
        InvalidLoginSnafu { username: "Stefani", password: "Germanotta" }.build().to_string(),
        "The user Stefani could not log in",
    );
}
```

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
        MissingUserSnafu.build().to_string(),
        "No user available. You may need to specify one.",
    );
    assert_eq!(
        MissingPasswordSnafu.build().to_string(),
        "MissingPassword",
    );
}
```

## Controlling context

### Changing the context selector suffix

When context selectors are generated, they have the suffix `Snafu`
added by default. If you'd prefer a different suffix, such as `Ctx` or
`Context`, you can specify that with
`#[snafu(context(suffix(SomeIdentifier)))]`. If you'd like to disable
the suffix entirely, you can use `#[snafu(context(suffix(false)))]`.

**Example**

```rust
# use snafu::Snafu;
#
#[derive(Debug, Snafu)]
enum Error {
    UsesTheDefaultSuffix,

    #[snafu(context(suffix(Ctx)))]
    HasAnotherSuffix,

    #[snafu(context(suffix(false)))]
    DoesNotHaveASuffix,
}

fn my_code() -> Result<(), Error> {
    UsesTheDefaultSuffixSnafu.fail()?;

    HasAnotherSuffixCtx.fail()?;

    DoesNotHaveASuffix.fail()?;

    Ok(())
}
```

### Disabling the context selector

Sometimes, an underlying error can only occur in exactly one context
and there's no additional information that can be provided to the
caller. In these cases, you can use `#[snafu(context(false))]` to
indicate that no context selector should be created. This allows using
the `?` operator directly on the underlying error.

Please think about your end users before making liberal use of this
feature. Adding context to an error is often what distinguishes an
actionable error from a frustrating one.

**Example**

```rust
# use snafu::Snafu;
#
#[derive(Debug, Snafu)]
enum Error {
    #[snafu(context(false))]
    NeedsNoIntroduction { source: VeryUniqueError },
}

fn my_code() -> Result<i32, Error> {
    let val = do_something_unique()?;
    Ok(val + 10)
}

# #[derive(Debug, Snafu)]
# enum VeryUniqueError {}
fn do_something_unique() -> Result<i32, VeryUniqueError> {
    // ...
#    Ok(42)
}
```

## Controlling visibility

By default, each of the context selectors and their inherent
methods will be private. It is our opinion that each module should
have one or more error types that are scoped to that module,
reducing the need to deal with unrelated errors when matching and
increasing cohesiveness.

If you need to access the context selectors from outside of their
module, you can use the `#[snafu(visibility)]` attribute. This can
be applied to the error type as a default visibility or to
specific context selectors.

There are multiple forms of the attribute:

- `#[snafu(visibility(X))]`

  `X` is a normal Rust visibility modifier (`pub`, `pub(crate)`,
  `pub(in some::path)`, etc.).

- `#[snafu(visibility)]` will reset back to private visibility.

```
# use snafu::Snafu;
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))] // Sets the default visibility for these context selectors
pub(crate) enum Error {
    IsPubCrate, // Uses the default
    #[snafu(visibility)]
    IsPrivate, // Will be private
}
```

It should be noted that API stability of context selectors is not
guaranteed. Therefore, exporting them in a crate's public API
could cause semver breakage for such crates, should SNAFU internals
change.

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

## Controlling stringly-typed errors

This allows your custom error type to behave like the [`Whatever`][]
error type. Since it is your type, you can implement additional
methods or traits. When placed on a struct or enum variant, you will
be able to use the type with the [`whatever!`][] macro as well as
`whatever_context` methods, such as [`ResultExt::whatever_context`][].

```rust
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum Error {
    SpecificError { username: String },

    #[snafu(whatever, display("{}", message))]
    GenericError {
        message: String,

        // Having a `source` is optional, but if it is present, it must
        // have this specific attribute and type:
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    }
}
```

## Controlling how the `snafu` crate is resolved

If the `snafu` crate is not called `snafu` for some reason, you can
use `#[snafu(crate_root)]` to instruct the macro how to find the crate
root:

```rust
# use snafu as my_custom_naming_of_snafu;
use my_custom_naming_of_snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(crate_root(my_custom_naming_of_snafu))]
enum Error {
    SomeFailureMode,
}

#[derive(Debug, Snafu)]
#[snafu(crate_root(my_custom_naming_of_snafu))]
struct ApiError(Error);
```
