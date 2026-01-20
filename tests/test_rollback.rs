mod common;

use crate::common::{TestPaths, assert_files_match, get_fixture_path, setup_cmd};
use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn test_rollback() -> Result<()> {
    let fixtures = get_fixture_path("rollback");
    let expected = fixtures.join("expected");
    let config_run1 = &fixtures.join("config/run1");
    let config_run2 = &fixtures.join("config/run2");
    let mut paths = TestPaths::new(config_run1);

    let run_cmd = |paths: &TestPaths| {
        let cmd = &mut cargo_bin_cmd!();
        let out = setup_cmd(cmd, paths, "run", Some(vec!["--noconfirm"]))
            .assert()
            .success()
            .to_string();
        println!("{out}");
    };

    let rollback_cmd = |paths: &TestPaths| {
        let cmd = &mut cargo_bin_cmd!();
        let out = setup_cmd(cmd, paths, "rollback", Some(vec!["--noconfirm"]))
            .assert()
            .success()
            .to_string();
        println!("{out}");
    };

    run_cmd(&paths);

    assert_files_match(
        &paths.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run1"),
    );

    paths.config_dir(config_run2);

    run_cmd(&paths);

    assert_files_match(
        &paths.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run2"),
    );

    rollback_cmd(&paths);

    assert_files_match(
        &paths.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run2"),
    );

    rollback_cmd(&paths);

    assert_files_match(
        &paths.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run1"),
    );

    let cmd = &mut cargo_bin_cmd!();
    let out = setup_cmd(cmd, &paths, "rollback", Some(vec!["--noconfirm"]))
        .assert()
        .failure()
        .to_string();
    println!("{out}");

    Ok(())
}
