#![cfg(test)]

mod api {
    use futures::{sink, stream, Sink, SinkExt, StreamExt, TryStream};
    use snafu::Snafu;
    use std::convert::Infallible;

    #[derive(Debug, Clone, Snafu)]
    pub enum Error {
        InvalidUrl { url: String },
    }

    impl From<Infallible> for Error {
        fn from(e: Infallible) -> Self {
            match e {}
        }
    }

    pub async fn fetch_page(url: &str) -> Result<String, Error> {
        InvalidUrl { url }.fail()
    }

    pub fn keep_fetching_page<'u>(url: &'u str) -> impl TryStream<Ok = String, Error = Error> + 'u {
        stream::repeat(()).then(move |_| fetch_page(url))
    }

    pub async fn upload_str(url: &str, _: &str) -> Result<String, Error> {
        InvalidUrl { url }.fail()
    }

    pub fn upload<'u>(url: &'u str) -> impl Sink<String, Error = Error> + 'u {
        sink::drain().with(move |s: String| async move { upload_str(url, &s).await })
    }
}

use futures::future::ok;
use futures::{
    future,
    stream::{self, StreamExt as _, TryStreamExt as _},
    SinkExt,
};
use snafu::{
    futures::{SnafuSinkExt as _, TryFutureExt as _, TryStreamExt as _},
    ResultExt, Snafu,
};

#[derive(Debug, Clone, Snafu)]
enum Error {
    UnableToLoadAppleStock { source: api::Error },
    UnableToLoadGoogleStock { source: api::Error, name: String },
    UnableToUploadApple { source: api::Error },
    UnableToUploadGoogle { source: api::Error, name: String },
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

// Return values of the combinators implement `Future`
async fn load_stock_data_sequential_again() -> Result<String, Error> {
    let apple = api::fetch_page("apple")
        .context(UnableToLoadAppleStock)
        .await?;

    let google = api::fetch_page("google")
        .with_context(|| UnableToLoadGoogleStock {
            name: String::from("sequential"),
        })
        .await?;

    Ok(format!("{}+{}", apple, google))
}

// Can be used as a `Stream` combinator
async fn load_stock_data_series() -> Result<String, Error> {
    let apple = api::keep_fetching_page("apple").context(UnableToLoadAppleStock);
    let google = api::keep_fetching_page("google").with_context(|| UnableToLoadGoogleStock {
        name: String::from("stream"),
    });

    let together = apple.zip(google);

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

// Can be used as a `SinkExt` combinator
async fn upload_strings() -> Result<(), Error> {
    let apple = api::upload("apple").context(UnableToUploadApple);
    let google = api::upload("google").with_context(|| UnableToUploadGoogle {
        name: String::from("sink"),
    });

    let together = apple.fanout(google);

    stream::repeat(Ok("str".to_owned()))
        .take(10)
        .forward(together)
        .await?;
    Ok(())
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

    let c = block_on(load_stock_data_sequential_again());
    c.unwrap_err();

    let d = block_on(load_stock_data_series());
    d.unwrap_err();

    let d = block_on(upload_strings());
    d.unwrap_err();
}
