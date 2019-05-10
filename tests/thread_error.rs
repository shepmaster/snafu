// This test asserts that errors can be used across threads.
extern crate snafu;

use std::sync::mpsc;
use std::thread;

use snafu::{ResultExt, Snafu};

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
fn implements_thread_safe_error() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(example()).unwrap();
    });

    let item = rx.recv().unwrap();
}
