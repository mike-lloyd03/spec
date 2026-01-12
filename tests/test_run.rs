mod common;

use std::{fs, path::Path};

use assert_cmd::cargo::cargo_bin_cmd;
use color_eyre::eyre::Result;
use predicates::prelude::*;
use tempfile::TempDir;

use crate::common::{assert_files_match, get_fixture_path};

#[test]
fn test_run() -> Result<()> {
    let testdir = TempDir::new()?;
    let etc_path = testdir.path().join("etc");
    let state_path = testdir.path().join("state");
    let fixture_path = get_fixture_path("run");
    let config_path = fixture_path.join("config/run1");
    let expected_path = fixture_path.join("expected");

    fs::create_dir_all(&state_path)?;

    let out = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("run")
        .arg("--noconfirm")
        .assert()
        .success()
        .to_string();

    println!("{out}");

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

#[test]
fn test_run_rm_old_files() -> Result<()> {
    let testdir = TempDir::new()?;
    let etc_path = testdir.path().join("etc");
    let state_path = testdir.path().join("state");
    let fixture_path = get_fixture_path("run");
    let config_path_run1 = fixture_path.join("config/run1");
    let config_path_run2 = fixture_path.join("config/run2");

    fs::create_dir_all(&state_path)?;

    let out = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path_run1.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("run")
        .arg("--noconfirm")
        .assert()
        .success()
        .to_string();

    println!("{out}");

    assert!(etc_path.join("ssh/sshd_config").exists());

    let out = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path_run2.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("run")
        .arg("--noconfirm")
        .assert()
        .success()
        .to_string();

    println!("{out}");

    assert!(!etc_path.join("ssh/sshd_config").exists());

    Ok(())
}
