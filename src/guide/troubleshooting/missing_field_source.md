# Missing field source / IntoError is not implemented

This error is encountered in multi-module / multi-file projects when
the error type is defined in one module and constructed in another.

## Failing Example

**project_error.rs**

```rust,ignore
use snafu::Snafu;
use std::io;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ProjectError {
    #[snafu(visibility(pub(crate)))]
    #[snafu(display("Unable to read configuration from {}: {}", path, source))]
    IOConfigError {
        path: &'static str,
        source: io::Error,
    },
}
```

**main.rs**

```rust,ignore
mod project_error;

use project_error::ProjectError;
use snafu::ResultExt;
use std::{fs, io};

const CONFIG_PATH: &str = "/etc/example/conf.conf";

pub fn read_config() -> Result<String, ProjectError> {
    fs::read_to_string(CONFIG_PATH).context(ProjectError::IOConfigError { path: CONFIG_PATH })
}

pub fn main() {
    println!("{}", read_config().unwrap());
}
```

## Errors

```text
error[E0063]: missing field `source` in initializer of `project_error::ProjectError`
  --> src/lib.rs:200:9
   |
27 |         ProjectError::IOConfigError { path: CONFIG_PATH }
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `source`
```

and

```text
error[E0277]: the trait bound `project_error::ProjectError: snafu::IntoError<_>` is not satisfied
  --> src/lib.rs:200:9
   |
27 |         ProjectError::IOConfigError { path: CONFIG_PATH }
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `snafu::IntoError<_>` is not implemented for `project_error::ProjectError`
the trait bound 'project_error::ProjectError: snafu::IntoError<_>' is not satisfied
```

## Solution

Replace the `ProjectError::IOConfigError` in the `read_config()`
function with `project_error::IOConfigError`.

## Explanation

This works because the `#[derive(Snafu)]` macro creates the *context
selector* type `IoConfigError`:

```rust,ignore
#[derive(Debug, Snafu)]
pub enum ProjectError {
    IOConfigError {
        source: io::Error,
        path: &'static str,
    },
}

// some details removed
struct IOConfigError<P> {
    path: P,
}

// Some impls for the IOConfigError struct
```

See [the macro section](guide::the_macro) of the guide for more details.

When you use `ProjectError::IOConfigError`, you're referencing the
enum variant, not the struct that you need. Replacing
`ProjectError::IOConfigError` with `project_error::IOConfigError`
fixes this problem.
