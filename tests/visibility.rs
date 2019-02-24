extern crate snafu;

// There are also sad-path tests

mod outer {
    pub mod inner {
        use snafu::Snafu;

        #[derive(Debug, Snafu)]
        #[snafu_visibility = "pub(crate)"]
        pub(crate) enum Error {
            PubCrate,
            #[snafu_visibility = "pub(in ::outer)"]
            PubInPath,
            #[snafu_visibility]
            Private,
        }
    }

    #[test]
    fn can_set_default_visibility() {
        let _ = self::inner::PubCrate.fail::<()>();
    }

    #[test]
    fn can_set_visibility() {
        let _ = self::inner::PubInPath.fail::<()>();
    }
}

#[test]
fn can_set_default_visibility() {
    let _ = self::outer::inner::PubCrate.fail::<()>();
}
