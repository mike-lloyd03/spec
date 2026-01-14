mod common;

use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use tempfile::TempDir;

use crate::common::{TestPaths, assert_files_match, get_fixture_path, setup_cmd};

#[test]
fn test_run() -> Result<()> {
    let testdir = TempDir::new()?;
    let fixtures = get_fixture_path("run");
    let expected = fixtures.join("expected");
    let paths = TestPaths::new(&testdir, &fixtures.join("config/run1"));

    let cmd = &mut cargo_bin_cmd!();
    let out = setup_cmd(cmd, &paths, "run", Some(vec!["--noconfirm"]))
        .assert()
        .success()
        .to_string();

    println!("{out}");

    assert_files_match(
        &paths.etc.join("ssh/sshd_config"),
        &expected.join("ssh/sshd_config"),
    );

    assert_files_match(
        &paths
            .etc
            .join("systemd/system/multi-user.target.wants/service1.service"),
        &expected.join("systemd/service1.service"),
    );

    assert_files_match(
        &paths
            .etc
            .join("systemd/system/multi-user.target.wants/service2.service"),
        &expected.join("systemd/service2.service"),
    );

    assert_files_match(
        &paths
            .etc
            .join("udev/rules.d/50-allow-hidraw-keyboard.rules"),
        &expected.join("udev/50-allow-hidraw-keyboard.rules"),
    );

    assert_files_match(
        &paths.etc.join("udev/udev.conf"),
        &expected.join("udev/udev.conf"),
    );

    Ok(())
}

#[test]
fn test_run_rm_old_files() -> Result<()> {
    let testdir = TempDir::new()?;
    let fixtures = get_fixture_path("run");
    let paths_r1 = TestPaths::new(&testdir, &fixtures.join("config/run1"));
    let paths_r2 = TestPaths::new(&testdir, &fixtures.join("config/run2"));

    let run_cmd = |test_paths: &TestPaths| {
        let cmd = &mut cargo_bin_cmd!();
        let out = setup_cmd(cmd, test_paths, "run", Some(vec!["--noconfirm"]))
            .assert()
            .success()
            .to_string();
        println!("{out}");
    };

    run_cmd(&paths_r1);

    assert!(paths_r1.etc.join("ssh/sshd_config").exists());

    run_cmd(&paths_r2);

    assert!(!paths_r1.etc.join("ssh/sshd_config").exists());

    Ok(())
}
