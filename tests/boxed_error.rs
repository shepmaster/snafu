// This test asserts that a boxed error can be used as a source.

use snafu::{Snafu, ResultExt};

mod api {
    pub type Error = Box<dyn std::error::Error + 'static>;

    pub fn function() -> Result<i32, Error> {
        Ok(42)
    }
}

#[derive(Debug, Snafu)]
enum Error {
    Authenticating { user_id: i32, source: api::Error },
}

fn example() -> Result<(), Error> {
    api::function().context(Authenticating { user_id: 42 })?;
    Ok(())
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();
    example().unwrap();
}
