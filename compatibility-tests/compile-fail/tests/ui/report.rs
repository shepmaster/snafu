use std::process::ExitCode;

#[snafu::report]
const NOT_HERE: u8 = 42;

#[snafu::report]
fn cannot_add_report_macro_with_no_return_value() {}

#[snafu::report]
fn cannot_add_report_macro_with_non_result_return_value() -> ExitCode {
    ExitCode::SUCCESS
}

fn main() {}
