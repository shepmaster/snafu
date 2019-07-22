#![cfg(test)]

use snafu::{Snafu, ResultExt};
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Debug, Snafu)]
struct PublicError(Error);

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
enum Error {
    #[snafu(display("Could not open config file at {}: {}", filename.display(), source))]
    OpenConfig { filename: PathBuf, source: io::Error },
    #[snafu(display("Could not open config file at {}", source))]
    SaveConfig { source: io::Error },
    #[snafu(display("No user available"))]
    #[snafu(visibility)]
    MissingUser,
}

type Result<T, E = Error> = std::result::Result<T, E>;

const CONFIG_FILENAME: &str = "/tmp/config";

fn example(root: impl AsRef<Path>, username: &str) -> Result<()> {
    let root = root.as_ref();
    let filename = &root.join(CONFIG_FILENAME);

    let config = fs::read(filename).context(OpenConfig { filename })?;

    if username.is_empty() {
        MissingUser.fail()?;
    }

    fs::write(filename, config).context(SaveConfig)?;

    Ok(())
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();
    check::<PublicError>();

    example("/some/directory/that/does/not/exist", "").unwrap_err();
}
