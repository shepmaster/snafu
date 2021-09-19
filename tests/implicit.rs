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
