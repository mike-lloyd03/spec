mod common;

use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;

use crate::common::{TestPaths, assert_files_match, get_fixture_path, setup_cmd};

#[test]
fn test_systemctl() -> Result<()> {
    let fixtures = get_fixture_path("systemctl");
    let expected = fixtures.join("expected");
    let paths = TestPaths::new(&fixtures.join("config"));

    let command_log = paths.get_root().join("commands.log");

    let cmd = &mut cargo_bin_cmd!();
    let out = setup_cmd(cmd, &paths, "run", Some(vec!["--noconfirm"]))
        .assert()
        .success()
        .to_string();

    println!("{}", out);

    assert_files_match(&command_log, &expected.join("systemctl_output"));

    Ok(())
}
