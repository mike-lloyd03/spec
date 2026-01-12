use cliclack::{intro, log::warning, note, outro};
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use similar::{ChangeTag, TextDiff};
use toml::Table;

use crate::{
    App,
    cli::VerifyArgs,
    services::{FileArtifact, get_service_by_name},
    utils::hash_bytes,
};

pub fn verify(app: &App, _args: &VerifyArgs) -> Result<()> {
    let services = app.load_services()?;
    let files = get_all_files(app, services)?;

    intro("Checking system for modified configuration files")?;
    compare_files(files)?;
    outro("")?;

    Ok(())
}

fn get_all_files(app: &App, services: Table) -> Result<Vec<FileArtifact>> {
    let mut all_files = vec![];

    for (key, value) in services {
        if let Some(service_table) = value.as_table()
            && let Some(s) = get_service_by_name(&key)
        {
            let (files, _) = s.plan(service_table, &app.system_config_dir)?;
            files.into_iter().for_each(|f| all_files.push(f));
        }
    }

    Ok(all_files)
}

fn compare_files(files: Vec<FileArtifact>) -> Result<()> {
    for file in files {
        let actual_file_contents = std::fs::read_to_string(&file.path)?;
        let actual_file_hash = hash_bytes(actual_file_contents.as_bytes());

        let expected_file_hash = hash_bytes(file.content.as_bytes());

        if actual_file_hash != expected_file_hash {
            let diff = TextDiff::from_lines(&actual_file_contents, &file.content);

            let mut full_output = String::new();

            for change in diff.iter_all_changes() {
                let output = match change.tag() {
                    ChangeTag::Delete => format!("- {}", change).red().to_string(),
                    ChangeTag::Insert => format!("+ {}", change).green().to_string(),
                    ChangeTag::Equal => format!("  {}", change),
                };
                full_output += &output;
            }

            warning("Modified File")?;
            let filepath = &file.path.to_str().unwrap().to_string();
            note(filepath, full_output)?;
        }
    }
    Ok(())
}
