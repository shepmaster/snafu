# Missing field source / IntoError is not implemented
This error is encountered in multi-file projects. Take the below erring code for an example:
```rust
// project_error/mod.rs
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu}
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ProjectError {
    #[snafu(visibility(pub(crate)))]
    #[snafu(display("Unable to read configuration from {}: {}"))]
    IOConfigError {
        path: &'static str
        source: io::Error,
    }
}

// main.rs
mod project_error;

use project_error::ProjectError;
use std::{io, fs};

const CONFIG_PATH: &str = "/etc/example/conf.conf"

pub fn read_config() -> Result<String, ProjectError> {
    fs::read_to_string(CONFIG_PATH).context(
        ProjectError::IOConfigError{ path: CONFIG_PATH }
    )?;
}

pub fn main() {
    println!("{}", read_config().unwrap());
}
```

If you try and compile the above code, you will get an error. No, not the ever helpful but dreaded borrow checked errors, but instead errors that tell you that `the trait bound 'project_error::ProjectError: snafu::IntoError<_>' is not satisfied`, and `missing field 'source' in initializer of 'project_error::ProjectError'`.
However, there is an easy fix to this problem. Simple replace the `ProjectError::IOConfigError` in the `read_config()` function with `project_error::IOConfigError`.
Why does this work? It works because the error enum in `project_error/mod.rs` expands to this (some details and tidbits removed, obviously):
```rust
#[derive(Debug, Snafu)]
pub enum ProjectError {
    IOConfigError {
        source: io::Error,
        path: &'static str,
    },
}

struct IOConfigError {
    source: ...,
    path: ...
}

// With some impls down below for the IOConfigError struct
```
When you use the `ProjectError::IOConfigError`, you're referencing the enum, not the struct that you need. Replacing `ProjectError::IOConfigError` with `project_error::IOConfigError` fixes this problem.
