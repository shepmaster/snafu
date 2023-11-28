## Rust version compatibility

SNAFU is tested and compatible back to Rust 1.56, released on
2021-10-21. Compatibility is controlled by Cargo feature flags.

<style>
.snafu-ff-meta>dt {
  font-weight: bold;
}
.snafu-ff-meta>*>p {
  margin: 0;
}
</style>

## `rust_1_61`

<dl class="snafu-ff-meta">
<dt>Default</dt>
<dd>enabled</dd>
</dl>

When enabled, SNAFU will assume that it's safe to target features
available in Rust 1.61. Notably, the [`Termination`][] trait is
implemented for [`Report`][] to allow it to be returned from `main`
and test functions.

[`Termination`]: std::process::Termination
[`Report`]: crate::Report
