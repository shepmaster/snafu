use snafu::{prelude::*, Report};

#[derive(Debug, Snafu)]
struct Error;

#[test]
fn tokio_main_attribute_first() {
    #[tokio::main(flavor = "current_thread")]
    #[snafu::report]
    async fn mainlike() -> Result<(), Error> {
        Snafu.fail()
    }

    let _: Report<_> = mainlike();
}

#[test]
fn tokio_main_attribute_last() {
    #[snafu::report]
    #[tokio::main(flavor = "current_thread")]
    async fn mainlike() -> Result<(), Error> {
        Snafu.fail()
    }

    let _: Report<_> = mainlike();
}

#[tokio::test]
#[snafu::report]
async fn tokio_test_attribute_first() -> Result<(), Error> {
    Ok(())
}

#[snafu::report]
#[tokio::test]
async fn tokio_test_attribute_last() -> Result<(), Error> {
    Ok(())
}

#[test]
fn async_std_main_attribute_last() {
    #[derive(Debug, Snafu)]
    struct Error;

    #[snafu::report]
    #[async_std::main]
    async fn main() -> Result<(), Error> {
        Snafu.fail()
    }

    let _: Report<_> = main();
}

#[snafu::report]
#[async_std::test]
async fn async_std_test_attribute_last() -> Result<(), Error> {
    Ok(())
}
