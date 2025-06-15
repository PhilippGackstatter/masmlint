extern crate alloc;

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Parser;
use masmlint::{self, LintSelector, Linter};
use miden_assembly::{SourceFile, SourceId};
use miette::Report;

/// A linter for Miden Assembly.
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to a MASM file to lint or a directory of MASM files. If a directory is given, it is
    /// searched recursively and lints all MASM files that are found.
    path: String,

    /// Comma-separated list of lint names to exclude. These will be excluded from the default list
    /// of lints.
    #[arg(short, long, value_delimiter = ',', use_value_delimiter = true)]
    exclude: Vec<String>,

    /// Comma-separated list of lint names to run. This list is exhaustive; no other lints will be
    /// run.
    #[arg(short, long, value_delimiter = ',', use_value_delimiter = true)]
    select: Vec<String>,
}

fn main() -> miette::Result<()> {
    let args = Args::parse();
    let source_path = args.path;
    let source_path = Path::new(&source_path)
        .canonicalize()
        .map_err(|err| Report::msg(format!("{err}")))?;

    let masm_files = if source_path.is_dir() {
        get_masm_files(source_path.as_path()).map_err(|err| {
            Report::msg(format!(
                "failed to get masm files from directory {}: {err}",
                source_path.display()
            ))
        })?
    } else {
        vec![source_path.to_owned()]
    };

    if !args.exclude.is_empty() && !args.select.is_empty() {
        return Err(Report::msg("cannot use include and exclude argument at the same time"));
    }

    let selector = if !args.exclude.is_empty() {
        LintSelector::Exclude(args.exclude)
    } else if !args.select.is_empty() {
        LintSelector::Select(args.select)
    } else {
        LintSelector::default()
    };

    let lints = selector.select()?;

    let mut linter = Linter::new(lints);

    for (file_idx, file) in masm_files.into_iter().enumerate() {
        let source = std::fs::read(&file)
            .map_err(|err| Report::msg(format!("failed to open file {}: {err}", file.display())))?;
        let source_content = String::from_utf8(source)
            .map_err(|err| Report::msg(format!("failed to decode file as UTF-8: {err}")))?;

        let relative_file_path = file
            .strip_prefix(source_path.as_path())
            .expect("file should contain source path as a prefix");
        let file_name = format!("{}", relative_file_path.display());
        let id = SourceId::try_from(file_idx)
            .expect("system limit: source manager has exhausted its supply of source ids");
        let source_file = SourceFile::new(id, file_name, source_content);

        linter.lint(Arc::new(source_file))?;
    }

    linter.finish().map_err(Report::from)
}

/// Returns a vector with paths to all MASM files in the specified directory and recursive
/// directories.
///
/// All non-MASM files are skipped.
fn get_masm_files<P: AsRef<Path>>(dir_path: P) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    match fs::read_dir(dir_path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        if entry_path.is_dir() {
                            files.extend(get_masm_files(entry_path)?);
                        } else if is_masm_file(&entry_path)? {
                            files.push(entry_path);
                        }
                    },
                    Err(e) => {
                        return Err(io::Error::other(format!(
                            "Error reading directory entry: {}",
                            e
                        )));
                    },
                }
            }
        },
        Err(e) => {
            return Err(io::Error::other(format!("Error reading directory: {}", e)));
        },
    }

    Ok(files)
}

/// Returns true if the provided path resolves to a file with `.masm` extension.
///
/// # Errors
/// Returns an error if the path could not be converted to a UTF-8 string.
fn is_masm_file(path: &Path) -> io::Result<bool> {
    if let Some(extension) = path.extension() {
        let extension = extension
            .to_str()
            .ok_or_else(|| io::Error::other("invalid UTF-8 filename"))?
            .to_lowercase();
        Ok(extension == "masm")
    } else {
        Ok(false)
    }
}
