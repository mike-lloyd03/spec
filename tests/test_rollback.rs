mod common;

use std::fs;

use assert_cmd::cargo::cargo_bin_cmd;
use color_eyre::eyre::Result;
use tempfile::TempDir;

use crate::common::{assert_files_match, get_fixture_path};

#[test]
fn test_rollback() -> Result<()> {
    let testdir = TempDir::new()?;
    let etc_path = testdir.path().join("etc");
    let state_path = testdir.path().join("state");
    let fixture_path = get_fixture_path("rollback");
    let config_path_run1 = fixture_path.join("config/run1");
    let config_path_run2 = fixture_path.join("config/run2");
    let expected_path = fixture_path.join("expected");

    fs::create_dir_all(&state_path)?;

    let _ = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path_run1.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("run")
        .arg("--noconfirm")
        .assert()
        .success();

    assert_files_match(
        &etc_path.join("ssh/sshd_config"),
        &expected_path.join("sshd_config_run1"),
    );

    let _ = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path_run2.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("run")
        .arg("--noconfirm")
        .assert()
        .success();

    assert_files_match(
        &etc_path.join("ssh/sshd_config"),
        &expected_path.join("sshd_config_run2"),
    );

    let _ = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path_run2.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("rollback")
        .arg("--noconfirm")
        .assert()
        .success();

    assert_files_match(
        &etc_path.join("ssh/sshd_config"),
        &expected_path.join("sshd_config_run2"),
    );

    let _ = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path_run2.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("rollback")
        .arg("--noconfirm")
        .assert()
        .success();

    assert_files_match(
        &etc_path.join("ssh/sshd_config"),
        &expected_path.join("sshd_config_run1"),
    );

    let _ = cargo_bin_cmd!()
        .env("SPEC_CONFIG_DIR", config_path_run2.to_str().unwrap())
        .env("SPEC_STATE_DIR", state_path.to_str().unwrap())
        .env("SPEC_SYSTEM_CONFIG_DIR", etc_path.to_str().unwrap())
        .arg("rollback")
        .arg("--noconfirm")
        .assert()
        .failure();

    Ok(())
}
