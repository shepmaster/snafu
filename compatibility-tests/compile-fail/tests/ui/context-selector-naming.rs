use snafu::prelude::*;

// FIXME: https://github.com/shepmaster/snafu/issues/486 -- needs semver bump
// #[derive(Debug, Snafu)]
// #[snafu(context)]
// struct CannotUseUnqualifiedContext;

// FIXME: https://github.com/shepmaster/snafu/issues/486 -- needs semver bump
// #[derive(Debug, Snafu)]
// #[snafu(context(true))]
// struct CannotUseContextTrue;

#[derive(Debug, Snafu)]
#[snafu(context(name(Bob), suffix(X)))]
struct CannotUseBothNameAndSuffix;

fn main() {}
