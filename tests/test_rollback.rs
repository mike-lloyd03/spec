mod common;

use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::TempDir;

use crate::common::{TestPaths, assert_files_match, get_fixture_path, setup_cmd};

#[test]
fn test_rollback() -> Result<()> {
    let testdir = TempDir::new()?;
    let fixtures = get_fixture_path("rollback");
    let expected = fixtures.join("expected");
    let paths_r1 = TestPaths::new(&testdir, &fixtures.join("config/run1"));
    let paths_r2 = TestPaths::new(&testdir, &fixtures.join("config/run2"));

    let run_cmd = |paths: &TestPaths| {
        let cmd = &mut cargo_bin_cmd!();
        let out = setup_cmd(cmd, paths, "run", Some(vec!["--noconfirm"]))
            .assert()
            .success()
            .to_string();
        println!("{out}");
    };

    let rollback_cmd = || {
        let cmd = &mut cargo_bin_cmd!();
        let out = setup_cmd(cmd, &paths_r1, "rollback", Some(vec!["--noconfirm"]))
            .assert()
            .success()
            .to_string();
        println!("{out}");
    };

    run_cmd(&paths_r1);

    assert_files_match(
        &paths_r1.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run1"),
    );

    run_cmd(&paths_r2);

    assert_files_match(
        &paths_r1.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run2"),
    );

    rollback_cmd();

    assert_files_match(
        &paths_r1.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run2"),
    );

    rollback_cmd();

    assert_files_match(
        &paths_r1.etc.join("ssh/sshd_config"),
        &expected.join("sshd_config_run1"),
    );

    let cmd = &mut cargo_bin_cmd!();
    let out = setup_cmd(cmd, &paths_r1, "rollback", Some(vec!["--noconfirm"]))
        .assert()
        .failure()
        .to_string();
    println!("{out}");

    Ok(())
}
