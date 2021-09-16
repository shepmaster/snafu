// There are also happy-path tests

mod outer {
    pub mod inner {
        use snafu::prelude::*;

        #[derive(Debug, Snafu)]
        #[snafu(visibility(pub(crate)))]
        pub(crate) enum Error {
            PubCrate,
            #[snafu(visibility(pub(in crate::outer)))]
            PubInPath,
            #[snafu(visibility)]
            Private,
        }
    }

    fn private_is_applied() {
        let _ = self::inner::PrivateSnafu.build();
    }
}

fn pub_in_path_is_applied() {
    let _ = self::outer::inner::PubInPathSnafu.build();
}

fn private_is_applied() {
    let _ = self::outer::inner::PrivateSnafu.build();
}

fn main() {}
