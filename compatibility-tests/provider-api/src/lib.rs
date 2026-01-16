#![cfg(test)]
#![feature(error_generic_member_access)]

use core::error;
use snafu::{prelude::*, Backtrace, IntoError};

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
fn sources_are_not_automatically_provided() {
    #[derive(Debug, Snafu, PartialEq)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct WithSourceError {
        source: InnerError,
    }

    let e = WithSourceSnafu.into_error(InnerError);
    let inner = error::request_ref::<InnerError>(&e);

    assert_eq!(inner, None);
}

#[test]
fn sources_can_be_provided() {
    #[derive(Debug, Snafu, PartialEq)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct WithSourceError {
        #[snafu(provide)]
        source: InnerError,
    }

    let e = WithSourceSnafu.into_error(InnerError);
    let inner = error::request_ref::<InnerError>(&e);

    assert_eq!(inner, Some(&InnerError));
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
fn whatever_errors_do_not_provide_the_source_error() {
    #[derive(Debug, Snafu)]
    struct InnerError;

    fn make() -> Result<(), snafu::Whatever> {
        whatever!(Err(InnerError), "big boom");
        Ok(())
    }

    let e = make().unwrap_err();
    let inner = error::request_ref::<dyn snafu::Error + Send + Sync>(&e);

    assert!(inner.is_none());
}

#[test]
fn whatever_errors_provide_backtrace() {
    #[derive(Debug, Snafu)]
    #[snafu(provide(u8 => 0))]
    struct InnerError;

    fn make() -> Result<(), snafu::Whatever> {
        whatever!(Err(InnerError), "big boom");
        Ok(())
    }

    let e = make().unwrap_err();
    let bt = error::request_ref::<Backtrace>(&e);
    assert!(bt.is_some());
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

mod doctests {
    #[test]
    fn example_1() {
        use core::error;
        use snafu::prelude::*;

        #[derive(Debug)]
        struct UserId(u8);

        #[derive(Debug, Snafu)]
        enum ApiError {
            Login {
                #[snafu(provide)]
                user_id: UserId,
            },

            Logout {
                #[snafu(provide)]
                user_id: UserId,
            },

            #[expect(dead_code)]
            NetworkUnreachable { source: std::io::Error },
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
        use core::error;
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        struct AuthorizationError {
            backtrace: snafu::Backtrace,
        }

        let e = AuthorizationSnafu.build();

        // We can get the backtrace
        error::request_ref::<snafu::Backtrace>(&e).expect("Must have a backtrace");
    }

    #[test]
    fn example_3() {
        use core::error;
        use snafu::{prelude::*, ErrorCompat, IntoError};

        #[derive(Debug, Snafu)]
        struct InnerError {
            backtrace: snafu::Backtrace,
        }

        #[derive(Debug, Snafu)]
        struct OuterError {
            source: InnerError,
            backtrace: snafu::Backtrace,
        }

        let e = OuterSnafu.into_error(InnerSnafu.build());

        // Get the deepest backtrace
        ErrorCompat::iter_chain(&e)
            .filter_map(error::request_ref::<snafu::Backtrace>)
            .last()
            .expect("Must have a backtrace");
    }

    #[test]
    fn example_4() {
        use core::error;
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
    fn example_5() {
        use core::error;
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
    fn example_6() {
        use core::error;
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
    fn example_7() {
        use core::error;
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        #[snafu(provide(opt, char => char::from_u32(*char_code)))]
        struct OptFlagExampleError {
            char_code: u32,
        }

        let e = OptFlagExampleSnafu { char_code: b'x' }.build();

        assert_eq!(Some('x'), error::request_value::<char>(&e));
    }
}
