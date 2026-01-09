mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use color_eyre::eyre::Result;
use tempfile::TempDir;

use crate::common::{assert_files_match, get_fixture_path};

#[test]
fn test_run() -> Result<()> {
    let etc_path = TempDir::new()?.path().join("etc");

    let fixture_path = get_fixture_path("run");
    let config_path = fixture_path.join("config");
    let expected_path = fixture_path.join("expected");

    let mut cmd = cargo_bin_cmd!();
    let assert = cmd
        .arg("--config-dir")
        .arg(config_path.to_str().unwrap())
        .arg("run")
        .arg("--sys-config-dir")
        .arg(etc_path.to_str().unwrap())
        .arg("--noconfirm")
        .assert();

    assert.success();

    assert_files_match(
        &etc_path.join("ssh/sshd_config"),
        &expected_path.join("ssh/sshd_config"),
    );

    assert_files_match(
        &etc_path.join("systemd/system/multi-user.target.wants/service1.service"),
        &expected_path.join("systemd/service1.service"),
    );

    assert_files_match(
        &etc_path.join("systemd/system/multi-user.target.wants/service2.service"),
        &expected_path.join("systemd/service2.service"),
    );

    assert_files_match(
        &etc_path.join("udev/rules.d/50-allow-hidraw-keyboard.rules"),
        &expected_path.join("udev/50-allow-hidraw-keyboard.rules"),
    );

    assert_files_match(
        &etc_path.join("udev/udev.conf"),
        &expected_path.join("udev/udev.conf"),
    );

    Ok(())
}
