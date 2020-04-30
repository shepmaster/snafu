#![cfg(test)]

mod api {
    use futures::{future, stream, Future, Stream, Sink};
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub enum Error {
        InvalidUrl { url: String },
        InfallibleFailed,
    }

    impl From<()> for Error {
        fn from(_: ()) -> Self {
            Error::InfallibleFailed
        }
    }

    pub fn fetch_page(url: &str) -> impl Future<Item = String, Error = Error> {
        future::result(InvalidUrl { url }.fail())
    }

    pub fn keep_fetching_page<'u>(url: &'u str) -> impl Stream<Item = String, Error = Error> + 'u {
        stream::repeat::<_, ()>(()).then(move |_| fetch_page(url))
    }

    pub fn upload_str(url: &str, _: &str) -> Result<String, Error> {
        InvalidUrl { url }.fail()
    }

    pub fn upload<'u>(url: &'u str) -> impl Sink<SinkItem = String, SinkError = Error> + 'u {
        Vec::new().with(move |s: String| { upload_str(url, &s) })
    }
}

use futures::{future, stream, Future, Stream, Sink};
use snafu::{
    futures01::{future::FutureExt as _, stream::StreamExt as _, sink::SinkExt as _},
    Snafu,
};

#[derive(Debug, Snafu)]
enum Error {
    UnableToLoadAppleStock { source: api::Error },
    UnableToLoadGoogleStock { source: api::Error, name: String },
    UnableToUploadApple { source: api::Error },
    UnableToUploadGoogle { source: api::Error, name: String },
}

// Can be used as a `Future` combinator
fn load_stock_data_concurrent() -> impl Future<Item = String, Error = Error> {
    let apple = api::fetch_page("apple").context(UnableToLoadAppleStock);
    let google = api::fetch_page("google").with_context(|| UnableToLoadGoogleStock {
        name: String::from("concurrent"),
    });

    apple
        .join(google)
        .map(|(apple, google)| format!("{}+{}", apple, google))
}

// Can be used as a `Stream` combinator
fn load_stock_data_series() -> impl Future<Item = String, Error = Error> {
    let apple = api::keep_fetching_page("apple").context(UnableToLoadAppleStock);
    let google = api::keep_fetching_page("google").with_context(|| UnableToLoadGoogleStock {
        name: String::from("stream"),
    });

    apple
        .zip(google)
        .take(10)
        .fold(String::new(), |mut acc, (a, g)| {
            use std::fmt::Write;
            writeln!(&mut acc, "{}+{}", a, g).expect("Could not format");
            future::ok(acc)
        })
}

// Can be used as a `SinkExt` combinator
fn upload_strings() -> impl Future<Item=(), Error=Error> {
    let apple = api::upload("apple").context(UnableToUploadApple);
    let google = api::upload("google").with_context(|| UnableToUploadGoogle {
        name: String::from("sink"),
    });

    let together = apple.fanout(google);

    stream::repeat("str".to_owned())
        .take(10)
        .forward(together)
        .map(|_| ())
}

#[test]
fn implements_error() {
    fn check<T: std::error::Error>() {}
    check::<Error>();

    let b = load_stock_data_concurrent().wait();
    b.unwrap_err();

    let c = load_stock_data_series().wait();
    c.unwrap_err();

    let d = upload_strings().wait();
    d.unwrap_err();
}
