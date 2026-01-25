use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
};

use crate::common::TestPaths;

fn shim(paths: &TestPaths, command_name: &str, content: &str) {
    let shim_path = paths.bin.join(command_name);

    let mut file = File::create(&shim_path).unwrap();
    Write::write_all(&mut file, content.as_bytes()).unwrap();

    file.set_permissions(Permissions::from_mode(0o755)).unwrap();
}

pub fn create_shim(paths: &TestPaths, command_name: &str) {
    let log_file = paths.root.path().join("commands.log");

    let content = format!(
        r#"#!/bin/sh
echo "{} $@" >> "{}"
"#,
        command_name,
        log_file.display()
    );

    shim(paths, command_name, &content)
}

pub fn create_sudo_shim(paths: &TestPaths) {
    let log_file = paths.root.path().join("commands.log");

    let content = format!(
        r#"#!/bin/sh
if [ "$1" != "install" ]; then
    echo -n "sudo " >> "{}"
fi
exec "$@"
"#,
        log_file.display()
    );

    shim(paths, "sudo", &content)
}

pub fn create_install_shim(paths: &TestPaths) {
    let content = r#"#!/bin/bash
args=()
skip_next=0

for arg in "$@"; do
    if [ $skip_next -eq 1 ]; then
        skip_next=0
        continue
    fi
    
    if [ "$arg" = "-o" ] || [ "$arg" = "-g" ]; then
        skip_next=1
        continue
    fi
    
    args+=("$arg")
done

exec /usr/bin/install "${args[@]}"
"#;

    shim(paths, "install", content)
}
