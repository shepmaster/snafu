## Rust version compatibility

SNAFU is tested and compatible back to Rust 1.65, released on
2022-11-03. Compatibility is controlled by Cargo feature flags.

<style>
.snafu-ff-meta>dt {
  font-weight: bold;
}
.snafu-ff-meta>*>p {
  margin: 0;
}
</style>

## `rust_1_81`

<dl class="snafu-ff-meta">
<dt>Default</dt>
<dd>disabled</dd>
</dl>

When enabled, SNAFU will assume that it's safe to target features
available in Rust 1.81. Notably, the [`core::error::Error`][] trait is
used instead of [`std::error::Error`][].
