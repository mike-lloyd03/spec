mod common;

use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;

use crate::common::{TestPaths, assert_files_match, get_fixture_path, setup_cmd};

#[test]
fn test_run() -> Result<()> {
    let fixtures = get_fixture_path("run");
    let expected = fixtures.join("expected");
    let paths = TestPaths::new(&fixtures.join("config/run1"));

    let cmd = &mut cargo_bin_cmd!();
    let out = setup_cmd(cmd, &paths, "run", Some(vec!["--noconfirm"]))
        .assert()
        .success()
        .to_string();

    println!("{out}");

    assert_files_match(
        &paths.etc.join("ssh/ssh_config"),
        &expected.join("ssh/ssh_config"),
    );

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
    let fixtures = get_fixture_path("run");
    let config_run1 = &fixtures.join("config/run1");
    let config_run2 = &fixtures.join("config/run2");
    let mut paths = TestPaths::new(config_run1);

    let run_cmd = |test_paths: &TestPaths| {
        let cmd = &mut cargo_bin_cmd!();
        let out = setup_cmd(cmd, test_paths, "run", Some(vec!["--noconfirm"]))
            .assert()
            .success()
            .to_string();
        println!("{out}");
    };

    run_cmd(&paths);

    assert!(paths.etc.join("ssh/sshd_config").exists());

    paths.config_dir(config_run2);
    run_cmd(&paths);

    assert!(!paths.etc.join("ssh/sshd_config").exists());

    Ok(())
}
