mod and_context {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(context, whatever)]
    struct Error;
}

mod with_arguments {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(whatever(true))]
    struct Error;
}

mod missing_message {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(whatever)]
    struct Error;
}

mod double_message {
    use snafu::prelude::*;

    #[derive(Snafu)]
    #[snafu(whatever)]
    struct Error {
        message: String,
        message: String,
    }
}

mod with_context_fields {
    use snafu::prelude::*;

    #[derive(Debug, Snafu)]
    #[snafu(whatever)]
    struct Error {
        message: String,
        user_id: i32,
    }
}

fn main() {}
