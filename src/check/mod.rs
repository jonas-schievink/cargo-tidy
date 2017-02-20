//! The tidy style checking engine.

pub mod forbidden_content;
pub mod indentation_style;
pub mod max_line_length;

use config::Config;

use glob::glob;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::fmt;
use std::error::Error;

/// Information about a check failure.
///
/// Each enabled check can produce any number of failures per file.
#[derive(Debug)]
pub struct CheckError {
    /// Path to the file that failed a check.
    path: PathBuf,
    /// Line inside the file where the failure occurred (0-based!).
    line: usize,
    /// Column in the line where the failure occurred. This might not make sense. In that case, this
    /// can be set to 0.
    column: usize,
    /// Message describing what went wrong.
    msg: String,
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error at {}:{}:{}: {}",
               self.path.display(),
               self.line + 1,
               self.column + 1,
               self.msg)?;

        Ok(())
    }
}

impl Error for CheckError {
    fn description(&self) -> &str {
        &self.msg
    }
}

/// The `TidyContext` holds information about a check performed on a file. It is passed to all
/// checks and simplifies error construction.
pub struct TidyContext<'a> {
    pub config: &'a Config,
    /// Relative path to the file being checked.
    pub path: &'a Path,
    /// The complete content of the file to-be-checked.
    pub content: &'a str,
    /// The file's contents split into lines. Note that, unlike `str::lines()`, this preserves the
    /// line ending found at the end of all lines (except the last).
    pub lines_with_endings: &'a [&'a str],
    errors: Vec<CheckError>,
}

impl<'a> TidyContext<'a> {
    /// Pushes a new error into the error buffer of this context.
    pub fn error(&mut self, pos: (usize, usize), msg: String) {
        self.errors.push(CheckError {
            path: self.path.to_path_buf(),
            line: pos.0,
            column: pos.1,
            msg: msg,
        });
    }
}

fn run_all_checks_on(config: &Config, path: PathBuf) -> Result<(), Vec<CheckError>> {
    debug!("checking {}", path.display());

    // Load file into `String`
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    // Split contents into lines, keeping line endings untouched
    let mut lines_with_endings = Vec::new();
    let mut last_idx = 0;
    for (idx, _) in content.match_indices('\n') {
        lines_with_endings.push(&content[last_idx..idx+1]);
        last_idx = idx;
    }

    let mut cx = TidyContext {
        config: config,
        path: &path,
        content: &content,
        lines_with_endings: &lines_with_endings,
        errors: Vec::new(),
    };

    // Run all checks on the contents
    max_line_length::check(&mut cx);
    forbidden_content::check(&mut cx);
    indentation_style::check(&mut cx);

    if cx.errors.is_empty() {
        Ok(())
    } else {
        Err(cx.errors)
    }
}

pub fn run_checks(config: &Config) -> Result<(), Vec<CheckError>> {
    let errors: Vec<_> = config.include.iter()
        .flat_map(|include_glob| glob(include_glob.as_str()).unwrap())
        .filter_map(|glob_result| {
            let path_buf = glob_result.unwrap();    // FIXME unlikely case, but can fail
            debug!("include set matched path: {}", path_buf.display());

            // If the path matches any exclude glob, skip
            if config.exclude.iter().any(|exclude_pat| exclude_pat.matches_path(&path_buf)) {
                debug!("path in exclude set, skipping");
                None
            } else {
                Some(path_buf)
            }
        }).flat_map(|path_buf| {
            if let Err(errs) = run_all_checks_on(config, path_buf) {
                errs.into_iter()
            } else {
                Vec::new().into_iter()
            }
        }).collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
