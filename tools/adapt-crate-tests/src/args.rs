use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::eyre::{self, WrapErr as _};

#[derive(Parser, Debug)]
struct Args {
    /// Glob patterns to ignore
    #[clap(long)]
    ignore: Vec<String>,
    /// Files to take glob patterns from (one per line)
    #[clap(long)]
    ignore_file: Vec<PathBuf>,
    /// Prefix to add to filenames
    #[clap(long)]
    prefix: Option<OsString>,
    /// Git URL to fetch
    git_url: String,
    /// Output directory (will be created if not exists)
    output: PathBuf,
    /// Subdirs inside of repo
    subdirs: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct Config {
    pub ignore: Vec<glob::Pattern>,
    pub git_url: String,
    pub prefix: Option<OsString>,
    pub subdirs: Vec<PathBuf>,
    pub output: PathBuf,
}

impl Config {
    pub fn parse_args() -> eyre::Result<Self> {
        Args::parse().try_into()
    }

    pub fn ignored(&self, path: &Path) -> bool {
        self.ignore.iter().any(|pattern| {
            pattern.matches_path_with(path, glob::MatchOptions {
                case_sensitive: true,
                require_literal_separator: true,
                require_literal_leading_dot: false,
            })
        })
    }
}

impl TryFrom<Args> for Config {
    type Error = eyre::Error;

    fn try_from(
        Args {
            ignore,
            ignore_file,
            git_url,
            subdirs,
            output,
            prefix,
        }: Args,
    ) -> eyre::Result<Self> {
        let ignore = ignore
            .into_iter()
            .map(Ok)
            .chain(ignore_file.into_iter().flat_map(|filename| {
                match fs::read_to_string(filename) {
                    Ok(contents) => contents
                        .lines()
                        .map(|line| Ok(line.trim().to_owned()))
                        .collect(),
                    Err(err) => vec![Err(err)],
                }
            }))
            .map(|res| {
                res.wrap_err("failed to read ignore file")
                    .and_then(|pattern| {
                        glob::Pattern::new(&pattern).wrap_err("failed to parse glob pattern")
                    })
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            ignore,
            git_url,
            subdirs,
            output,
            prefix,
        })
    }
}
