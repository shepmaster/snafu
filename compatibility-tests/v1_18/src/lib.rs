extern crate snafu;
#[macro_use]
extern crate snafu_derive;

use snafu::{ResultExt, Backtrace, ErrorCompat};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Snafu)]
struct PublicError(Error);

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display = r#"("Could not open config file at {}: {}", filename.display(), source)"#)]
    OpenConfig { filename: PathBuf, source: io::Error },
    #[snafu(display = r#"("Could not open config file at {}", source)"#)]
    SaveConfig { source: io::Error },
    #[snafu(display = r#"("User ID {} is invalid", user_id)"#)]
    InvalidUser { user_id: i32, backtrace: Backtrace },
    #[snafu(display = r#"("No user available")"#)]
    MissingUser,
}

type Result<T, E = Error> = std::result::Result<T, E>;

const CONFIG_FILENAME: &str = "/tmp/config";

fn example<P>(root: P, user_id: Option<i32>) -> Result<()>
where
    P: AsRef<Path>,
{
    let root = root.as_ref();
    let filename = &root.join(CONFIG_FILENAME);

    let config = read(filename).context(OpenConfig { filename })?;

    let _user_id = match user_id {
        None => MissingUser.fail()?,
        Some(user_id) if user_id != 42 => InvalidUser { user_id }.fail()?,
        Some(user_id) => user_id,
    };

    write(filename, &config).context(SaveConfig)?;

    Ok(())
}

fn read<P>(path: P) -> io::Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Read;

    let mut f = File::open(path)?;
    let mut v = Vec::new();
    f.read_to_end(&mut v)?;
    Ok(v)
}

fn write<P>(path: P, data: &[u8]) -> io::Result<()>
where
    P: AsRef<Path>,
{
    use std::fs::File;
    use std::io::Write;

    let mut f = File::open(path)?;
    f.write_all(data)?;
    Ok(())
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();
    check::<PublicError>();

    let e = example("/some/directory/that/does/not/exist", None).unwrap_err();
    ErrorCompat::backtrace(&e);
}
