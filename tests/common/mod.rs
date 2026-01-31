#![allow(dead_code)]

mod shims;
use shims::create_shim;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use tempfile::TempDir;

use crate::common::shims::{create_install_shim, create_sudo_shim};

pub struct TestPaths {
    pub root: TempDir,
    pub config: PathBuf,
    pub state: PathBuf,
    pub etc: PathBuf,
    pub bin: PathBuf,
}

impl TestPaths {
    pub fn new(config_dir: &Path) -> Self {
        let root = TempDir::new().expect("failed to create temp dir");
        let config = config_dir.to_owned();
        let state = root.path().join("state");
        let etc = root.path().join("etc");
        let bin = root.path().join("bin");

        fs::create_dir_all(&config).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::create_dir_all(&etc).unwrap();
        fs::create_dir_all(&bin).unwrap();

        Self {
            root,
            config,
            state,
            etc,
            bin,
        }
    }

    pub fn config_dir(&mut self, dir: &Path) {
        self.config = dir.to_owned();
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
    let current_path = env::var_os("PATH").unwrap_or_default();
    let new_path =
        env::join_paths(std::iter::once(paths.bin.clone()).chain(env::split_paths(&current_path)))
            .expect("Failed to construct new PATH");

    create_sudo_shim(paths);
    create_install_shim(paths);
    create_shim(paths, "systemctl");
    create_shim(paths, "udevadm");

    let cmd = cmd
        .env("SPEC_CONFIG_DIR", &paths.config)
        .env("SPEC_STATE_DIR", &paths.state)
        .env("SPEC_SYSTEM_CONFIG_DIR", &paths.etc)
        .env("PATH", new_path)
        .arg(subcommand);

    if let Some(a) = args {
        cmd.args(a);
    }

    cmd
}
