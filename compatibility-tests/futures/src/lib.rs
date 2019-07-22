#![cfg(test)]
#![feature(async_await)]

mod api {
    use futures::{stream, StreamExt, TryStream};
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub enum Error {
        InvalidUrl { url: String },
    }

    pub async fn fetch_page(url: &str) -> Result<String, Error> {
        InvalidUrl { url }.fail()
    }

    pub fn keep_fetching_page<'u>(url: &'u str) -> impl TryStream<Ok = String, Error = Error> + 'u {
        stream::repeat(()).then(move |_| fetch_page(url))
    }
}

use futures::{
    future,
    stream::{StreamExt as _, TryStreamExt as _},
};
use snafu::{
    futures::{TryFutureExt as _, TryStreamExt as _},
    ResultExt, Snafu,
};

#[derive(Debug, Snafu)]
enum Error {
    UnableToLoadAppleStock { source: api::Error },
    UnableToLoadGoogleStock { source: api::Error, name: String },
}

// Normal `Result` code works with `await`
async fn load_stock_data_sequential() -> Result<String, Error> {
    let apple = api::fetch_page("apple")
        .await
        .context(UnableToLoadAppleStock)?;

    let google = api::fetch_page("google")
        .await
        .with_context(|| UnableToLoadGoogleStock {
            name: String::from("sequential"),
        })?;

    Ok(format!("{}+{}", apple, google))
}

// Can be used as a `Future` combinator
async fn load_stock_data_concurrent() -> Result<String, Error> {
    let apple = api::fetch_page("apple").context(UnableToLoadAppleStock);
    let google = api::fetch_page("google").with_context(|| UnableToLoadGoogleStock {
        name: String::from("concurrent"),
    });

    let (apple, google) = future::try_join(apple, google).await?;

    Ok(format!("{}+{}", apple, google))
}

// Can be used as a `Stream` combinator
async fn load_stock_data_series() -> Result<String, Error> {
    let apple = api::keep_fetching_page("apple").context(UnableToLoadAppleStock);
    let google = api::keep_fetching_page("google").with_context(|| UnableToLoadGoogleStock {
        name: String::from("stream"),
    });

    let together = apple.into_stream().zip(google.into_stream());

    // No try_zip?
    let together = together.map(|(a, g)| Ok((a?, g?)));

    together
        .take(10)
        .try_fold(String::new(), |mut acc, (a, g)| {
            use std::fmt::Write;
            writeln!(&mut acc, "{}+{}", a, g).expect("Could not format");
            future::ready(Ok(acc))
        })
        .await
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();

    use futures::executor::block_on;

    let a = block_on(load_stock_data_sequential());
    a.unwrap_err();

    let b = block_on(load_stock_data_concurrent());
    b.unwrap_err();

    let c = block_on(load_stock_data_series());
    c.unwrap_err();
}
