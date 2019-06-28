# Upgrading from previous releases

## Version 0.3 → 0.4

### `Context` vs. `IntoError`

The `Context` type and related `From` implementations have been
removed in favor of the [`IntoError`](crate::IntoError) trait. If
you were making use of this for custom conversions, you will need
to update your trait bounds:

#### Before

```rust,ignore
fn example<C, E>(context: C) -> MyType<E>
where
    snafu::Context<SomeError, C>: Into<E>;
```

#### After

```rust,ignore
fn example<C, E>(context: C) -> MyType<E>
where
    C: snafu::IntoError<E, Source = SomeError>,
    E: std::error::Error + snafu::ErrorCompat;
```

### `Borrow<std::error::Error>`

SNAFU no longer generates `Borrow<std::error::Error>`
implementations for SNAFU error types (sorry for the whiplash if
you were affected by this when upgrading to 0.3).

## Version 0.2 → 0.3

Minimal changes should be required: if you previously implemented
`Borrow<std::error::Error>` for a SNAFU error type, you should
remove that implementation and allow SNAFU to implement it for
you.

## Version 0.1 → 0.2

Support for the `snafu::display` attribute was removed as this
type of attribute was [never intended to be
supported][oops]. Since this required a SemVer-incompatible
version, the attribute format has also been updated and
normalized.

1. Attributes have been renamed
    - `snafu_display` and `snafu::display` became `snafu(display)`.
    - `snafu_visibility` became `snafu(visibility)`
    - `snafu_backtrace` became `snafu(backtrace)`

1. Support for `snafu_display` with individually-quoted format
   arguments was removed. Migrate to either the "clean" or "all
   one string" styles, depending on what version of Rust you are
   targeting.

[oops]: https://github.com/rust-lang/rust/pull/58899

### Before

```rust,ignore
#[derive(Debug, Snafu)]
enum DisplayUpdate {
    #[snafu::display("Format and {}", argument)]
    CleanStyle { argument: i32 },

    #[snafu_display("Format and {}", "argument")]
    QuotedArgumentStyle { argument: i32 },

    #[snafu_display = r#"("Format and {}", argument)"#]
    AllOneStringStyle { argument: i32 },
}
```

```rust,ignore
#[derive(Debug, Snafu)]
enum VisibilityUpdate {
    #[snafu_visibility(pub(crate))]
    CleanStyle,

    #[snafu_visibility = "pub(crate)"]
    AllOneStringStyle,
}
```

### After

```rust,ignore
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum DisplayUpdate {
    #[snafu(display("Format and {}", argument))]
    CleanStyle { argument: i32 },

    #[snafu(display = r#"("Format and {}", argument)"#)]
    QuotedArgumentStyle { argument: i32 },

    #[snafu(display = r#"("Format and {}", argument)"#)]
    AllOneStringStyle { argument: i32 },
}
```

```rust,ignore
# use snafu::Snafu;
#[derive(Debug, Snafu)]
enum VisibilityUpdate {
    #[snafu(visibility(pub(crate)))]
    CleanStyle,

    #[snafu(visibility = "pub(crate)")]
    AllOneStringStyle,
}
```
