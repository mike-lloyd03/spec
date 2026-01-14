use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use tempfile::TempDir;

pub struct TestPaths {
    pub config: PathBuf,
    pub state: PathBuf,
    pub etc: PathBuf,
}

impl TestPaths {
    pub fn new(temp_dir: &TempDir, config_dir: &Path) -> Self {
        Self {
            config: config_dir.to_owned(),
            state: temp_dir.path().join("state"),
            etc: temp_dir.path().join("etc"),
        }
    }
}

pub fn assert_files_match(actual_path: &Path, expected_path: &Path) {
    if !actual_path.exists() {
        panic!("Actual file missing: {:?}", actual_path);
    }

    let actual = fs::read_to_string(actual_path).expect("Failed to read actual output file");
    let actual = actual.trim_end();

    let expected = fs::read_to_string(expected_path).expect("Failed to read expected output file");
    let expected = expected.trim_end();

    assert_eq!(
        actual, expected,
        "Generated output file does not match expected file"
    );
}

pub fn get_fixture_path(test_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(test_name)
}

pub fn setup_cmd<'a>(
    cmd: &'a mut Command,
    paths: &TestPaths,
    subcommand: &str,
    args: Option<Vec<&str>>,
) -> &'a mut Command {
    let cmd = cmd
        .env("SPEC_CONFIG_DIR", &paths.config)
        .env("SPEC_STATE_DIR", &paths.state)
        .env("SPEC_SYSTEM_CONFIG_DIR", &paths.etc)
        .env("SPEC_SYSTEMCTL_CMD", "echo")
        .arg(subcommand);

    if let Some(a) = args {
        for arg in a {
            cmd.arg(arg);
        }
    }
    cmd
}

pub fn p_to_str(p: &Path) -> &str {
    let err_msg = format!("Failed to convert Path '{:?}' to string", p);
    p.to_str().expect(&err_msg)
}
