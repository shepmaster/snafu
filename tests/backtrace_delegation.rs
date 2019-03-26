extern crate snafu;

use snafu::{ErrorCompat, ResultExt, Snafu};

mod house {
    use snafu::{Backtrace, Snafu};

    #[derive(Debug, Snafu)]
    pub enum Error {
        Fatal { backtrace: Backtrace },
    }

    pub fn answer_telephone() -> Result<(), Error> {
        Fatal.fail()
    }
}

#[derive(Debug, Snafu)]
enum Error {
    MovieTrope {
        #[snafu(backtrace(delegate))]
        source: house::Error,
    },
}

fn example() -> Result<(), Error> {
    house::answer_telephone().context(MovieTrope)?;

    Ok(())
}

#[test]
fn backtrace_comes_from_inside_the_house() {
    let e = example().unwrap_err();
    let text = ErrorCompat::backtrace(&e)
        .map(ToString::to_string)
        .unwrap_or_default();
    assert!(
        text.contains("answer_telephone"),
        "{:?} does not contain `answer_telephone`",
        text
    );
}
