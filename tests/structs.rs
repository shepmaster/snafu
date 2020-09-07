// Using #[path] to work around a https://github.com/rust-lang/rustfmt/issues/4404
// Once fixed and released, switch to a `mod structs { ... }`

#[path = "structs/backtrace.rs"]
mod backtrace;
#[path = "structs/backtrace_attributes.rs"]
mod backtrace_attributes;
#[path = "structs/display.rs"]
mod display;
#[path = "structs/from_option.rs"]
mod from_option;
#[path = "structs/generics.rs"]
mod generics;
#[path = "structs/no_context.rs"]
mod no_context;
#[path = "structs/single_use_lifetimes.rs"]
mod single_use_lifetimes;
#[path = "structs/source_attributes.rs"]
mod source_attributes;
#[path = "structs/visibility.rs"]
mod visibility;
#[path = "structs/with_source.rs"]
mod with_source;
#[path = "structs/without_source.rs"]
mod without_source;
