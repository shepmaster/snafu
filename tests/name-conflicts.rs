use snafu::Snafu;

#[derive(Debug, Snafu)]
enum VariantNamedNone {
    None,
}

#[derive(Debug, Snafu)]
enum VariantNamedSome<T> {
    Some { value: T },
}

#[derive(Debug, Snafu)]
enum VariantNamedOk<T> {
    Ok { value: T },
}

#[derive(Debug, Snafu)]
enum VariantNamedErr<T> {
    Err { value: T },
}
