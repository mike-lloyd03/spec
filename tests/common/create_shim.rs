use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

use crate::common::TestPaths;

pub fn create_shim(paths: &TestPaths, command_name: &str) -> PathBuf {
    let log_file = paths.root.path().join("commands.log");

    let script_content = format!(
        r#"#!/bin/sh
echo "{} $@" >> "{}"
"#,
        command_name,
        log_file.display()
    );

    let shim_path = paths.bin.join(command_name);

    let mut file = File::create(&shim_path).unwrap();
    Write::write_all(&mut file, script_content.as_bytes()).unwrap();

    file.set_permissions(Permissions::from_mode(0o755)).unwrap();

    shim_path
}
