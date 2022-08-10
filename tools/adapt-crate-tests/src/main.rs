#![feature(exit_status_error)]

use std::{fs, io, path::Path, process::Command};

use color_eyre::eyre::{self, eyre, WrapErr as _};
use sha3::{Digest as _, Sha3_256};
use tempfile::TempDir;
use tracing::info;
use walkdir::WalkDir;

mod args;

fn contains_async_fn(path: &Path) -> io::Result<bool> {
    fs::read(path).map(|contents| twoway::find_bytes(&contents, b"async fn").is_some())
}

fn convert_file(input: String) -> String {
    let mut res = "#![allow(non_camel_case_types)]\n".to_owned();
    let iter = input.lines().enumerate();
    for (idx, line) in iter {
        let mut line = line.to_owned();
        if line.contains("async fn") {
            let mut hasher = Sha3_256::new();
            hasher.update(&line);
            hasher.update(&idx.to_le_bytes());
            let type_name = hex::encode(hasher.finalize());
            res.push_str(&format!("#[::name_it::name_it(t{type_name})]\n"));

            let mut n = 0;
            while line.contains("_:") {
                line = line.replacen("_:", &format!("_t{n}:"), 1);
                n += 1;
            }
        }
        res.push_str(&line);
        res.push('\n');
    }
    res
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let config = args::Config::parse_args()?;

    info!("Creating output directory...");
    match fs::create_dir(&config.output) {
        Ok(()) => {}
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => {}
            _ => Err(err).wrap_err("failed to create output directory")?,
        },
    }

    let repo_dir = TempDir::new()?;

    info!(
        "Cloning {repo_url} into {repo_dir:?}...",
        repo_url = config.git_url
    );
    Command::new("git")
        .args(["clone", "--depth=1", &config.git_url])
        .arg(repo_dir.path())
        .status()?
        .exit_ok()?;
    info!("Cloned sucessfuly!");

    if !config.ignore.is_empty() {
        info!("Combined ignore list:");
        for pattern in &config.ignore {
            info!("- {pattern}");
        }
    }

    for subdir in &config.subdirs {
        let tests_dir = repo_dir.path().join(subdir);
        info!("Traversing {tests_dir:?}...");
        for entry in WalkDir::new(tests_dir) {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type().is_file() && !config.ignored(path) && contains_async_fn(path)? {
                let contents = convert_file(fs::read_to_string(&path)?);
                let mut file_name = config.prefix.as_ref().cloned().unwrap_or_default();
                file_name.push(
                    path.file_name()
                        .ok_or_else(|| eyre!("failed to determine file name for path"))?,
                );
                fs::write(config.output.join(&file_name), contents)?;
                info!("Copied {file_name:?}");
            }
        }
    }

    Ok(())
}
