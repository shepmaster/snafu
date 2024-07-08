#![cfg(test)]
#![feature(error_generic_member_access)]

use snafu::{prelude::*, Backtrace, IntoError};

// https://github.com/rust-lang/rust/pull/114973
mod error {
    pub use core::error::request_value;
    pub use snafu::error::{request_ref, Request};
}

#[test]
fn provide_shorthand_on_fields_returns_a_reference() {
    #[derive(Debug, Snafu)]
    struct WithFieldShorthandError {
        #[snafu(provide)]
        name: String,
    }

    let e = WithFieldShorthandSnafu { name: "bob" }.build();
    let inner = error::request_ref::<String>(&e);

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
    let inner = error::request_value::<u8>(&e);

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
    let inner = error::request_value::<u8>(&e);

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
    let inner = error::request_ref::<str>(&e);

    assert_eq!(inner, Some("one"));
}

#[test]
fn provide_static_references_as_values() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(&'static str => "static"))]
    struct StaticValueError;

    let e = StaticValueError;
    let inner = error::request_value::<&'static str>(&e);

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
    let inner = error::request_value::<u8>(&e);

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
    let inner = error::request_ref::<u8>(&e);

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
    let inner = error::request_ref::<SomeImplicitData<1>>(&e);

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
    let inner = error::request_ref::<String>(&e);

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
    let inner = error::request_ref::<InnerError>(&e);

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
    let inner = error::request_ref::<InnerError>(&e);

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
    let inner = error::request_value::<&str>(&e);

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
    let inner = error::request_value::<&str>(&e);

    assert_eq!(inner, Some("outer"));
}

#[test]
fn backtraces_are_automatically_provided() {
    #[derive(Debug, Snafu)]
    struct WithBacktraceError {
        backtrace: Backtrace,
    }

    let e = WithBacktraceSnafu.build();
    let bt = error::request_ref::<Backtrace>(&e);

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
    let bt = error::request_ref::<Backtrace>(&e);

    assert!(bt.is_none(), "was {bt:?}");
}

#[test]
fn backtraces_support_conversion_via_as_backtrace() {
    #[derive(Debug, Snafu)]
    struct AsBacktraceError {
        backtrace: SomeBacktrace,
    }

    let e = AsBacktraceSnafu.build();
    let bt = error::request_ref::<Backtrace>(&e);

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

    let provided_bt = error::request_ref::<Backtrace>(&e).unwrap();

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

    let lhs = error::request_value::<u8>(&e);
    assert_eq!(lhs, Some(99));

    let rhs = error::request_value::<bool>(&e);
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

    let lhs = error::request_value::<u8>(&e);
    assert_eq!(lhs, Some(99));

    let rhs = error::request_value::<bool>(&e);
    assert_eq!(rhs, Some(false));
}

#[test]
fn chaining_to_arbitrary_fields_evaluated_once() {
    use std::sync::atomic::{AtomicU8, Ordering};

    static COUNT: AtomicU8 = AtomicU8::new(0);

    fn inc() {
        COUNT.fetch_add(1, Ordering::SeqCst);
    }

    #[derive(Debug, Snafu)]
    #[snafu(provide(ref, chain, SomeProvidedData<u8> => { inc(); val }))]
    struct ErrorWithChildren {
        val: SomeProvidedData<u8>,
    }

    let e = ErrorWithChildren {
        val: SomeProvidedData(99),
    };

    let lhs = error::request_ref::<SomeProvidedData<u8>>(&e);
    assert_eq!(lhs, Some(&SomeProvidedData(99)));
    assert_eq!(COUNT.load(Ordering::SeqCst), 1);
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

    let alpha = error::request_ref::<u8>(&e);
    let omega = error::request_ref::<i8>(&e);

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
    let inner = error::request_value::<u8>(&e);

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
    let inner = error::request_value::<u8>(&e);

    assert_eq!(inner, Some(99));
}

#[test]
fn opaque_errors_can_chain_provided() {
    #[derive(Debug, Snafu)]
    struct InnerError {
        data: SomeProvidedData<u8>,
    }

    #[derive(Debug, Snafu)]
    #[snafu(provide(ref, chain, SomeProvidedData<u8> => &self.0.data))]
    struct OuterError(InnerError);

    let e = OuterError::from(
        InnerSnafu {
            data: SomeProvidedData(99),
        }
        .build(),
    );
    let inner = error::request_value::<u8>(&e);

    assert_eq!(inner, Some(99));
}

#[test]
fn whatever_errors_provide_the_source_error() {
    #[derive(Debug, Snafu)]
    struct InnerError;

    fn make() -> Result<(), snafu::Whatever> {
        whatever!(Err(InnerError), "big boom");
        Ok(())
    }

    let e = make().unwrap_err();
    let inner = error::request_ref::<dyn snafu::Error>(&e);

    let inner = inner.map(ToString::to_string);
    let inner = inner.as_deref();
    assert_eq!(inner, Some("InnerError"));
}

#[test]
fn whatever_errors_chain_to_the_source_error() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(u8 => 0))]
    struct InnerError;

    fn make() -> Result<(), snafu::Whatever> {
        whatever!(Err(InnerError), "big boom");
        Ok(())
    }

    let e = make().unwrap_err();
    let inner = error::request_value::<u8>(&e);

    assert_eq!(inner, Some(0));
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

#[derive(PartialEq)]
struct SomeProvidedData<T>(T);

use std::fmt;

impl<T> fmt::Debug for SomeProvidedData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "SomeProvidedData(...)".fmt(f)
    }
}

impl<T> fmt::Display for SomeProvidedData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<T> snafu::Error for SomeProvidedData<T>
where
    T: Copy + 'static,
{
    fn provide(&self, request: &mut error::Request<'_>) {
        request.provide_value::<T>(self.0);
    }
}

mod doctests {
    use crate::error;

    #[test]
    fn example_1() {
        use snafu::prelude::*;

        #[derive(Debug)]
        struct UserId(u8);

        #[derive(Debug, Snafu)]
        enum ApiError {
            Login {
                #[snafu(provide)]
                user_id: UserId,
            },

            _Logout {
                #[snafu(provide)]
                user_id: UserId,
            },

            _NetworkUnreachable {
                source: std::io::Error,
            },
        }

        let e = LoginSnafu { user_id: UserId(0) }.build();
        match error::request_ref::<UserId>(&e) {
            // Present when ApiError::Login or ApiError::Logout
            Some(UserId(user_id)) => {
                println!("{user_id} experienced an error");
            }
            // Absent when ApiError::NetworkUnreachable
            None => {
                println!("An error occurred for an unknown user");
            }
        }
    }

    #[test]
    fn example_2() {
        use snafu::{prelude::*, IntoError};

        #[derive(Debug)]
        struct UserId(());

        #[derive(Debug, Snafu)]
        struct InnerError {
            #[snafu(provide)]
            user_id: UserId,
            backtrace: snafu::Backtrace,
        }

        #[derive(Debug, Snafu)]
        struct OuterError {
            source: InnerError,
        }

        let user_id = UserId(());
        let inner = InnerSnafu { user_id }.build();
        let outer = OuterSnafu.into_error(inner);

        // We can get the source error and downcast it at once
        error::request_ref::<InnerError>(&outer).expect("Must have a source");

        // We can get the deepest backtrace
        error::request_ref::<snafu::Backtrace>(&outer).expect("Must have a backtrace");

        // We can get arbitrary values from sources as well
        error::request_ref::<UserId>(&outer).expect("Must have a user id");
    }

    #[test]
    fn example_3() {
        use snafu::prelude::*;

        #[derive(Debug, PartialEq)]
        struct HttpCode(u16);

        const HTTP_NOT_FOUND: HttpCode = HttpCode(404);

        #[derive(Debug, Snafu)]
        #[snafu(provide(HttpCode => HTTP_NOT_FOUND))]
        struct WebserverError;

        let e = WebserverError;
        assert_eq!(Some(HTTP_NOT_FOUND), error::request_value::<HttpCode>(&e));
    }

    #[test]
    fn example_4() {
        use snafu::prelude::*;

        #[derive(Debug, PartialEq)]
        struct Summation(u8);

        #[derive(Debug, Snafu)]
        #[snafu(provide(Summation => Summation(left_side + right_side)))]
        struct AdditionError {
            left_side: u8,
            right_side: u8,
        }

        let e = AdditionSnafu {
            left_side: 1,
            right_side: 2,
        }
        .build();
        assert_eq!(Some(Summation(3)), error::request_value::<Summation>(&e));
    }

    #[test]
    fn example_5() {
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        #[snafu(provide(ref, str => name))]
        struct RefFlagExampleError {
            name: String,
        }

        let e = RefFlagExampleSnafu { name: "alice" }.build();

        assert_eq!(Some("alice"), error::request_ref::<str>(&e));
    }

    #[test]
    fn example_6() {
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        #[snafu(provide(opt, char => char::from_u32(*char_code)))]
        struct OptFlagExampleError {
            char_code: u32,
        }

        let e = OptFlagExampleSnafu { char_code: b'x' }.build();

        assert_eq!(Some('x'), error::request_value::<char>(&e));
    }

    #[test]
    fn example_7() {
        use snafu::{prelude::*, IntoError};

        #[derive(Debug, PartialEq)]
        struct Fatal(bool);

        #[derive(Debug, Snafu)]
        #[snafu(provide(Fatal => Fatal(true)))]
        struct InnerError;

        #[derive(Debug, Snafu)]
        #[snafu(provide(priority, Fatal => Fatal(false)))]
        struct PriorityFlagExampleError {
            source: InnerError,
        }

        let e = PriorityFlagExampleSnafu.into_error(InnerError);

        assert_eq!(Some(Fatal(false)), error::request_value::<Fatal>(&e));
    }

    #[test]
    fn example_8() {
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        #[snafu(provide(u8 => 1))]
        struct NotTheSourceError;

        #[derive(Debug, Snafu)]
        #[snafu(provide(ref, chain, NotTheSourceError => data))]
        struct ChainFlagExampleError {
            data: NotTheSourceError,
        }

        let e = ChainFlagExampleSnafu {
            data: NotTheSourceError,
        }
        .build();

        assert_eq!(Some(1), error::request_value::<u8>(&e));
    }
}
