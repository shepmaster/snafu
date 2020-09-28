mod unknown_attributes {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(unknown_attribute)]
    struct Error {}
}

mod invalid_on_struct {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(source(true))]
    #[snafu(backtrace)]
    struct Error {}
}

mod invalid_on_field {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    struct Error {
        #[snafu(display("display should not work here"))]
        #[snafu(visibility(pub))]
        #[snafu(context)]
        id: i32,
    }
}

fn main() {}
