// Using #[path] to work around a https://github.com/rust-lang/rustfmt/issues/4404
// Once fixed and released, switch to a `mod structs { ... }`

#[path = "structs/display.rs"]
mod display;
#[path = "structs/with_source.rs"]
mod with_source;
#[path = "structs/without_source.rs"]
mod without_source;
