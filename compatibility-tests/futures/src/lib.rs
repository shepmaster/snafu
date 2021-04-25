#![cfg(test)]

mod api {
    use futures::{stream, StreamExt, TryStream};
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub enum Error {
        InvalidUrl { url: String },
    }

    pub async fn fetch_page(url: &str) -> Result<String, Error> {
        InvalidUrlSnafu { url }.fail()
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
    UnableToLoadAppleStock {
        source: api::Error,
    },

    UnableToLoadGoogleStock {
        source: api::Error,
        name: String,
    },

    #[snafu(whatever, display("{}", message))]
    UnableToLoadOtherStock {
        message: String,

        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

// Normal `Result` code works with `await`
async fn load_stock_data_sequential() -> Result<String, Error> {
    let apple = api::fetch_page("apple")
        .await
        .context(UnableToLoadAppleStockSnafu)?;

    let google = api::fetch_page("google")
        .await
        .with_context(|| UnableToLoadGoogleStockSnafu {
            name: String::from("sequential"),
        })?;

    let other_1 = api::fetch_page("other_1")
        .await
        .whatever_context("Oh no!")?;

    let symbol = "other_2";
    let other_2 = api::fetch_page(symbol)
        .await
        .with_whatever_context(|_| format!("Unable to get stock prices for: {}", symbol))?;

    Ok(format!("{}+{}+{}+{}", apple, google, other_1, other_2))
}

// Can be used as a `Future` combinator
async fn load_stock_data_concurrent() -> Result<String, Error> {
    let apple = api::fetch_page("apple").context(UnableToLoadAppleStockSnafu);
    let google = api::fetch_page("google").with_context(|| UnableToLoadGoogleStockSnafu {
        name: String::from("concurrent"),
    });
    let other_1 = api::fetch_page("other_1").whatever_context::<_, Error>("Oh no!");
    let symbol = "other_2";
    let other_2 = api::fetch_page(symbol)
        .with_whatever_context(|_| format!("Unable to get stock prices for: {}", symbol));

    let (apple, google, other_1, other_2) =
        future::try_join4(apple, google, other_1, other_2).await?;

    Ok(format!("{}+{}+{}+{}", apple, google, other_1, other_2))
}

// Return values of the combinators implement `Future`
async fn load_stock_data_sequential_again() -> Result<String, Error> {
    let apple = api::fetch_page("apple")
        .context(UnableToLoadAppleStockSnafu)
        .await?;

    let google = api::fetch_page("google")
        .with_context(|| UnableToLoadGoogleStockSnafu {
            name: String::from("sequential"),
        })
        .await?;

    let other_1 = api::fetch_page("other_1")
        .whatever_context("Oh no!")
        .await?;

    let symbol = "other_2";
    let other_2 = api::fetch_page(symbol)
        .with_whatever_context(|_| format!("Unable to get stock prices for: {}", symbol))
        .await?;

    Ok(format!("{}+{}+{}+{}", apple, google, other_1, other_2))
}

// Can be used as a `Stream` combinator
async fn load_stock_data_series() -> Result<String, Error> {
    let apple = api::keep_fetching_page("apple").context(UnableToLoadAppleStockSnafu);
    let google = api::keep_fetching_page("google").with_context(|| UnableToLoadGoogleStockSnafu {
        name: String::from("stream"),
    });
    let other_1 = api::keep_fetching_page("other_1").whatever_context("Oh no!");
    let symbol = "other_2";
    let other_2 = api::keep_fetching_page(symbol)
        .with_whatever_context(|_| format!("Unable to get stock prices for: {}", symbol));

    let together = apple.zip(google).zip(other_1).zip(other_2);

    // No try_zip?
    let together = together.map(|(((a, g), o1), o2)| Ok((a?, g?, o1?, o2?)));

    together
        .take(10)
        .try_fold(String::new(), |mut acc, (a, g, o1, o2)| {
            use std::fmt::Write;
            writeln!(&mut acc, "{}+{}+{}+{}", a, g, o1, o2).expect("Could not format");
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

    let c = block_on(load_stock_data_sequential_again());
    c.unwrap_err();

    let d = block_on(load_stock_data_series());
    d.unwrap_err();
}
