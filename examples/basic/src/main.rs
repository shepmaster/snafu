#![feature(unrestricted_attribute_tokens)]

use my_error::ResultExt;
use my_error_derive::MyError;
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Debug, MyError)]
enum Error {
    #[my_error::display("Could not open config file at {}: {}", self.filename.display(), self.source)]
    OpenConfig { filename: PathBuf, source: io::Error },
    SaveConfig { source: io::Error },
    MissingUser,
}

type Result<T, E = Error> = std::result::Result<T, E>;

const CONFIG_FILENAME: &str = "/tmp/config";

fn do_it(root: impl AsRef<Path>, username: &str) -> Result<()> {
    let root = root.as_ref();
    let filename = &root.join(CONFIG_FILENAME);

    let config = fs::read(filename).context(OpenConfig { filename })?;

    if username.is_empty() {
        // MissingUser::fail()?;
        return Err(Error::MissingUser);
    }

    fs::write(filename, config).context(SaveConfig)?;

    Ok(())
}

fn main() {
    match do_it("/some/directory/that/does/not/exist", "") {
        Ok(_) => panic!("Should always fail"),
        Err(e) => panic!("{}", e),
    }
}
