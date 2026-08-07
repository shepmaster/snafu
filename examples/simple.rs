use std::io;

#[derive(Debug, snafu::Snafu)]
enum OuterError {
    #[snafu(display("Get failed"))]
    Get { source: GetError },
}

#[derive(Debug, snafu::Snafu)]
enum GetError {
    #[snafu(display("IO failure"))]
    Io { source: io::Error },
}

fn main() {
    let err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe is broken");
    let err = GetError::Io { source: err };
    let err = OuterError::Get { source: err };

    println!("display normal: {err}");
    println!("display alt   : {err:#}");
}
