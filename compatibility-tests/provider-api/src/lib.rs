#![cfg(test)]
#![feature(error_generic_member_access, provide_any)]

use snafu::{prelude::*, IntoError};

#[test]
fn provide_shorthand_on_fields_returns_a_reference() {
    #[derive(Debug, Snafu)]
    struct WithFieldShorthandError {
        #[snafu(provide)]
        name: String,
    }

    let e = WithFieldShorthandSnafu { name: "bob" }.build();
    let e = &e as &dyn snafu::Error;
    let inner = e.request_ref::<String>();

    let inner = inner.map(String::as_str);
    assert_eq!(inner, Some("bob"));
}

#[test]
fn provide_value_from_expression() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(u8 => 1 + ALPHA + beta::gamma() + Delta::default().epsilon()))]
    struct WithExpressionError;

    const ALPHA: u8 = 1;
    mod beta {
        pub fn gamma() -> u8 {
            1
        }
    }
    #[derive(Default)]
    struct Delta;
    impl Delta {
        fn epsilon(&self) -> u8 {
            1
        }
    }

    let e = WithExpressionSnafu.build();
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<u8>();

    assert_eq!(inner, Some(4));
}

#[test]
fn provide_value_expressions_can_use_fields() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(u8 => base.0 + secret + code))]
    struct WithExpressionError {
        #[snafu(implicit)]
        base: SomeImplicitData<1>,
        secret: u8,
        code: u8,
    }

    let e = WithExpressionSnafu { secret: 2, code: 3 }.build();
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<u8>();

    assert_eq!(inner, Some(6));
}

#[test]
fn provide_reference_expressions() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(ref, str => self.choose_one()))]
    struct WithExpressionError {
        which: bool,
        one: String,
        two: String,
    }

    impl WithExpressionError {
        fn choose_one(&self) -> &str {
            if self.which {
                &self.one
            } else {
                &self.two
            }
        }
    }

    let e = WithExpressionSnafu {
        which: true,
        one: "one",
        two: "two",
    }
    .build();
    let e = &e as &dyn snafu::Error;
    let inner = e.request_ref::<str>();

    assert_eq!(inner, Some("one"));
}

#[test]
fn provide_static_references_as_values() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(&'static str => "static"))]
    struct StaticValueError;

    let e = StaticValueError;
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<&'static str>();

    assert_eq!(inner, Some("static"));
}

#[test]
fn provide_optional_value() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(opt, u8 => *thing))]
    struct MaybeProvideError {
        thing: Option<u8>,
    }

    let e = MaybeProvideSnafu { thing: Some(42) }.build();
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<u8>();

    assert_eq!(inner, Some(42));
}

#[test]
fn provide_optional_reference() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(ref, opt, u8 => thing.as_ref()))]
    struct MaybeProvideError {
        thing: Option<u8>,
    }

    let e = MaybeProvideSnafu { thing: Some(42) }.build();
    let e = &e as &dyn snafu::Error;
    let inner = e.request_ref::<u8>();

    assert_eq!(inner, Some(&42));
}

#[test]
fn implicit_fields_can_be_provided() {
    #[derive(Debug, Snafu)]
    struct WithImplicitDataError {
        #[snafu(implicit, provide)]
        implicit: SomeImplicitData<1>,
    }

    let e = WithImplicitDataSnafu.build();
    let e = &e as &dyn snafu::Error;
    let inner = e.request_ref::<SomeImplicitData<1>>();

    assert_eq!(inner, Some(&SomeImplicitData(1)));
}

#[test]
fn message_fields_can_be_provided() {
    use snafu::FromString;

    #[derive(Debug, Snafu)]
    #[snafu(whatever)]
    struct WhateverError {
        #[snafu(provide)]
        message: String,
    }

    let e = WhateverError::without_source("Bad stuff".into());
    let e = &e as &dyn snafu::Error;
    let inner = e.request_ref::<String>();

    let inner = inner.map(String::as_str);
    assert_eq!(inner, Some("Bad stuff"));
}

#[test]
fn sources_are_automatically_provided() {
    #[derive(Debug, Snafu, PartialEq)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct WithSourceError {
        source: InnerError,
    }

    let e = WithSourceSnafu.into_error(InnerError);
    let e = &e as &dyn snafu::Error;
    let inner = e.request_ref::<InnerError>();

    assert_eq!(inner, Some(&InnerError));
}

#[test]
fn sources_provided_values_are_chained() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(&'static str => "inner"))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    #[snafu(provide(&'static str => "outer"))]
    struct OuterError {
        source: InnerError,
    }

    let e = OuterSnafu.into_error(InnerError);
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<&str>();

    assert_eq!(inner, Some("inner"));
}

#[derive(Debug, PartialEq)]
struct SomeImplicitData<const V: u8>(u8);

impl<const V: u8> snafu::GenerateImplicitData for SomeImplicitData<V> {
    fn generate() -> Self {
        Self(V)
    }
}
