#![cfg(test)]
#![feature(error_generic_member_access, provide_any)]

use snafu::{prelude::*, Backtrace, IntoError};
use std::any;

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
fn sources_can_be_not_automatically_provided() {
    #[derive(Debug, Snafu, PartialEq)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct WithSourceError {
        #[snafu(provide(false))]
        source: InnerError,
    }

    let e = WithSourceSnafu.into_error(InnerError);
    let e = &e as &dyn snafu::Error;
    let inner = e.request_ref::<InnerError>();

    assert_eq!(inner, None);
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

#[test]
fn sources_provided_values_can_be_superseded() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(&'static str => "inner"))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    #[snafu(provide(priority, &'static str => "outer"))]
    struct OuterError {
        source: InnerError,
    }

    let e = OuterSnafu.into_error(InnerError);
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<&str>();

    assert_eq!(inner, Some("outer"));
}

#[test]
fn backtraces_are_automatically_provided() {
    #[derive(Debug, Snafu)]
    struct WithBacktraceError {
        backtrace: Backtrace,
    }

    let e = WithBacktraceSnafu.build();
    let e = &e as &dyn snafu::Error;
    let bt = e.request_ref::<Backtrace>();

    assert!(bt.is_some(), "was {bt:?}");
}

#[test]
fn backtraces_can_be_not_automatically_provided() {
    #[derive(Debug, Snafu)]
    struct WithBacktraceError {
        #[snafu(provide(false))]
        backtrace: Backtrace,
    }

    let e = WithBacktraceSnafu.build();
    let e = &e as &dyn snafu::Error;
    let bt = e.request_ref::<Backtrace>();

    assert!(bt.is_none(), "was {bt:?}");
}

#[test]
fn backtraces_support_conversion_via_as_backtrace() {
    #[derive(Debug, Snafu)]
    struct AsBacktraceError {
        backtrace: SomeBacktrace,
    }

    let e = AsBacktraceSnafu.build();
    let e = &e as &dyn snafu::Error;
    let bt = e.request_ref::<Backtrace>();

    assert!(bt.is_some(), "was {bt:?}");
}

#[test]
fn backtraces_pick_deepest_by_default() {
    #[derive(Debug, Snafu)]
    struct InnerError {
        backtrace: SomeBacktrace,
    }

    #[derive(Debug, Snafu)]
    struct OuterError {
        source: InnerError,
        backtrace: SomeBacktrace,
    }

    let e = OuterSnafu.into_error(InnerSnafu.build());
    let outer_bt = &e.backtrace.backtrace;
    let inner_bt = &e.source.backtrace.backtrace;

    let e = &e as &dyn snafu::Error;
    let provided_bt = e.request_ref::<Backtrace>().unwrap();

    assert!(
        std::ptr::eq(inner_bt, provided_bt),
        "Inner backtrace was {inner_bt:p}, but provided was {provided_bt:p}",
    );

    assert!(
        !std::ptr::eq(outer_bt, provided_bt),
        "Outer backtrace was {outer_bt:p}, but provided was {provided_bt:p}",
    );
}

#[test]
fn can_chain_to_arbitrary_fields() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(ref, chain, SomeProvidedData<u8> => lhs))]
    #[snafu(provide(ref, chain, SomeProvidedData<bool> => rhs))]
    struct ErrorWithChildren {
        lhs: SomeProvidedData<u8>,
        rhs: SomeProvidedData<bool>,
    }

    let e = ErrorWithChildren {
        lhs: SomeProvidedData(99),
        rhs: SomeProvidedData(false),
    };
    let e = &e as &dyn snafu::Error;

    let lhs = e.request_value::<u8>();
    assert_eq!(lhs, Some(99));

    let rhs = e.request_value::<bool>();
    assert_eq!(rhs, Some(false));
}

#[test]
fn can_chain_to_arbitrary_optional_fields() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(ref, opt, chain, SomeProvidedData<u8> => lhs))]
    #[snafu(provide(ref, opt, chain, SomeProvidedData<bool> => rhs))]
    struct ErrorWithChildren {
        lhs: Option<SomeProvidedData<u8>>,
        rhs: Option<SomeProvidedData<bool>>,
    }

    let e = ErrorWithChildren {
        lhs: Some(SomeProvidedData(99)),
        rhs: Some(SomeProvidedData(false)),
    };
    let e = &e as &dyn snafu::Error;

    let lhs = e.request_value::<u8>();
    assert_eq!(lhs, Some(99));

    let rhs = e.request_value::<bool>();
    assert_eq!(rhs, Some(false));
}

#[test]
fn order_of_flags_does_not_matter() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(ref, opt, u8 => alpha.as_ref()))]
    #[snafu(provide(opt, ref, i8 => omega.as_ref()))]
    struct MaybeProvideError {
        alpha: Option<u8>,
        omega: Option<i8>,
    }

    let e = MaybeProvideSnafu {
        alpha: Some(255),
        omega: Some(-1),
    }
    .build();
    let e = &e as &dyn snafu::Error;

    let alpha = e.request_ref::<u8>();
    let omega = e.request_ref::<i8>();

    assert_eq!(alpha, Some(&255));
    assert_eq!(omega, Some(&-1));
}

#[test]
fn opaque_errors_chain_to_inner_errors() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(u8 => 42))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct OuterError(InnerError);

    let e = OuterError::from(InnerError);
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<u8>();

    assert_eq!(inner, Some(42));
}

#[test]
fn opaque_errors_can_supersede_provided_values() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(u8 => 1))]
    struct InnerError;

    #[derive(Debug, Snafu)]
    #[snafu(provide(priority, u8 => 99))]
    struct OuterError(InnerError);

    let e = OuterError::from(InnerError);
    let e = &e as &dyn snafu::Error;
    let inner = e.request_value::<u8>();

    assert_eq!(inner, Some(99));
}

#[derive(Debug, PartialEq)]
struct SomeImplicitData<const V: u8>(u8);

impl<const V: u8> snafu::GenerateImplicitData for SomeImplicitData<V> {
    fn generate() -> Self {
        Self(V)
    }
}

#[derive(Debug)]
struct SomeBacktrace {
    // Exists only to make this type a non-ZST
    _dummy: u8,
    backtrace: Backtrace,
}

impl snafu::GenerateImplicitData for SomeBacktrace {
    fn generate() -> Self {
        Self {
            _dummy: 1,
            backtrace: Backtrace::generate(),
        }
    }
}

impl snafu::AsBacktrace for SomeBacktrace {
    fn as_backtrace(&self) -> Option<&Backtrace> {
        Some(&self.backtrace)
    }
}

#[derive(Debug, PartialEq)]
struct SomeProvidedData<T>(T);

impl<T> any::Provider for SomeProvidedData<T>
where
    T: Copy + 'static,
{
    fn provide(&self, demand: &mut any::Demand<'_>) {
        demand.provide_value::<T>(self.0);
    }
}
