//! The tidy style checking engine.

use config::Config;

use glob::glob;

use std::path::PathBuf;
use std::fmt;
use std::error::Error;

/// Information about a check failure.
///
/// Each enabled check can produce any number of failures per file.
#[derive(Debug)]
pub struct CheckError {
    /// Path to the file that failed a check.
    path: PathBuf,
    /// Byte position inside the file where the failure occurred.
    bytepos: usize,
    /// Message describing what went wrong.
    msg: String,
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "file '{}' failed a tidy check: {}", self.path.display(), self.msg));

        Ok(())
    }
}

impl Error for CheckError {
    fn description(&self) -> &str {
        &self.msg
    }
}

fn run_all_checks_on(config: &Config, path: PathBuf) -> Result<(), Vec<CheckError>> {
    debug!("checking {}", path.display());
    unimplemented!();
}

pub fn run_checks(config: &Config) -> Result<(), Vec<CheckError>> {
    let errors: Vec<_> = config.include.iter()
        .flat_map(|include_glob| glob(include_glob.as_str()).unwrap())
        .filter_map(|glob_result| {
            let path_buf = glob_result.unwrap();    // FIXME unlikely case, but can fail
            debug!("include set matched path: {}", path_buf.display());

            // If the path matches any exclude glob, skip
            if config.exclude.iter().any(|exclude_pat| exclude_pat.matches_path(&path_buf)) {
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
