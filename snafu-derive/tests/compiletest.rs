extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();

    config.mode = mode.parse().expect("Invalid mode");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.link_deps();
    config.clean_rmeta();

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail");
}
