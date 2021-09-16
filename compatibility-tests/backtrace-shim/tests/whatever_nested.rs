use snafu::{prelude::*, Whatever};

fn inner_outer() -> Result<(), Whatever> {
    not_a_whatever().with_whatever_context(|_| format!("Outer failure"))
}

fn not_a_whatever() -> Result<(), Box<dyn std::error::Error>> {
    inner_whatever().map_err(Into::into)
}

fn inner_whatever() -> Result<(), Whatever> {
    whatever!("Inner failure");
}

#[test]
fn backtrace_method_delegates_to_nested_whatever() {
    let e = inner_outer().unwrap_err();
    let bt = e.backtrace().expect("Must have a backtrace");
    assert!(bt.to_string().contains("::inner_whatever::"));
}
