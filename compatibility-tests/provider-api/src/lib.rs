#![cfg(test)]
#![feature(error_generic_member_access, provide_any)]

use snafu::prelude::*;

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

#[derive(Debug)]
struct SomeImplicitData<const V: u8>(u8);

impl<const V: u8> snafu::GenerateImplicitData for SomeImplicitData<V> {
    fn generate() -> Self {
        Self(V)
    }
}
