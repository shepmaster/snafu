use snafu::prelude::*;

mod basics {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct OccurrenceCount(usize);

    impl snafu::GenerateImplicitData for OccurrenceCount {
        fn generate() -> Self {
            OccurrenceCount(COUNTER.fetch_add(1, Ordering::SeqCst))
        }
    }

    #[derive(Debug, Snafu)]
    enum ErrorOne {
        Alpha {
            #[snafu(implicit)]
            occurrence: OccurrenceCount,
        },
    }

    #[test]
    fn implicit_fields_are_constructed() {
        let ErrorOne::Alpha {
            occurrence: OccurrenceCount(o1),
        } = AlphaSnafu.build();
        let ErrorOne::Alpha {
            occurrence: OccurrenceCount(o2),
        } = AlphaSnafu.build();

        assert_eq!(o1, 0);
        assert_eq!(o2, 1);
    }
}

mod multiple_fields {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct ImplicitData;

    impl snafu::GenerateImplicitData for ImplicitData {
        fn generate() -> Self {
            ImplicitData
        }
    }

    #[derive(Debug, Snafu)]
    struct Error {
        #[snafu(implicit)]
        one: ImplicitData,
        #[snafu(implicit)]
        two: ImplicitData,
    }

    #[test]
    fn multiple_implicit_fields_are_constructed() {
        let Error { one, two } = Snafu.build();

        assert_eq!(one, two);
    }
}

mod with_and_without_source {
    use snafu::{prelude::*, FromString, IntoError};

    #[derive(Debug, PartialEq)]
    enum ItWas {
        Generate,
        GenerateWithSource,
    }

    #[derive(Debug)]
    struct ImplicitData(ItWas);

    impl snafu::GenerateImplicitData for ImplicitData {
        fn generate() -> Self {
            Self(ItWas::Generate)
        }

        fn generate_with_source(_: &dyn snafu::Error) -> Self {
            Self(ItWas::GenerateWithSource)
        }
    }

    #[derive(Debug, Snafu)]
    struct InnerError;

    #[derive(Debug, Snafu)]
    struct HasSource {
        source: InnerError,
        #[snafu(implicit)]
        data: ImplicitData,
    }

    #[derive(Debug, Snafu)]
    struct NoSource {
        #[snafu(implicit)]
        data: ImplicitData,
    }

    #[derive(Debug, Snafu)]
    #[snafu(context(false))]
    struct HasSourceNoContext {
        source: InnerError,
        #[snafu(implicit)]
        data: ImplicitData,
    }

    #[derive(Debug, Snafu)]
    #[snafu(whatever, display("{message}"))]
    struct MyOwnWhatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
        #[snafu(implicit)]
        data: ImplicitData,
    }

    #[test]
    fn calls_generate_for_no_source() {
        let e = NoSourceSnafu.build();
        assert_eq!(e.data.0, ItWas::Generate);
    }

    #[test]
    fn calls_generate_with_source_for_source() {
        let e = HasSourceSnafu.into_error(InnerError);
        assert_eq!(e.data.0, ItWas::GenerateWithSource);
    }

    #[test]
    fn calls_generate_for_none() {
        let e = NoSourceSnafu.into_error(snafu::NoneError);
        assert_eq!(e.data.0, ItWas::Generate);
    }

    #[test]
    fn calls_generate_with_source_for_no_context() {
        let e = HasSourceNoContext::from(InnerError);
        assert_eq!(e.data.0, ItWas::GenerateWithSource);
    }

    #[test]
    fn calls_generate_for_whatever_with_no_source() {
        let e = MyOwnWhatever::without_source("bang".into());
        assert_eq!(e.data.0, ItWas::Generate);
    }

    #[test]
    fn calls_generate_with_source_for_whatever_with_source() {
        let e = MyOwnWhatever::with_source(Box::new(InnerError), "bang".into());
        assert_eq!(e.data.0, ItWas::GenerateWithSource);
    }
}
