//! Configuration file parsing.

use check::indentation_style::IndentationStyle;

use glob::{Pattern, PatternError};
use toml;
use toml::de::Error as TomlError;
use regex::{self, RegexSet};
use serde::Deserialize;

use std::path::Path;
use std::io::{self, Read};
use std::fs::File;
use std::fmt;
use std::error::Error;
use std::str::FromStr;

// FIXME return a better error when the config file does not exist

// FIXME this macro does not allow doc comments on the fields

macro_rules! make_config {
    ( $( $fld:ident : $t:ty => |$raw_ident:ident : $raw_ty:ty| { $load:expr },)+ ) => {
        #[derive(Deserialize)]
        struct RawConfig {
            $( $fld : $raw_ty, )+
        }

        #[derive(Debug)]
        pub struct Config {
            $( pub $fld : $t, )+
            _priv: (),
        }

        impl Config {
            fn from_raw(raw: RawConfig) -> Result<Config, LoadError> {
                Ok(Config {
                    $( $fld : {
                        let $raw_ident = raw.$fld;
                        $load
                    } ,)+
                    _priv: (),
                })
            }
        }
    };
}

make_config! {
    // List of file globs to check.
    //
    // Default: `vec!["**/*.rs"]`, which checks all .rs-files in all directories but nothing else.
    include: Vec<Pattern> => |raw: Option<Vec<String>>| {
        raw.unwrap_or(vec!["**/*.rs".to_string()])
            .iter().map(|s| Pattern::new(s)).collect::<Result<Vec<_>, _>>()?
    },

    // List of file globs to exclude from checking.
    //
    // When a file should be checked according to the `include` globs, it is matched against this
    // list of globs. If it matches, it's skipped.
    exclude: Vec<Pattern> => |raw: Vec<String>| {
        raw.iter().map(|s| Pattern::new(s)).collect::<Result<Vec<_>, _>>()?
    },


    // Maximum number of `char`s in a single line of code.
    //
    // A line containing more characters fails the tidy check.
    max_line_length: u64 => |raw: u64| { raw },

    // List of regular expressions matching "forbidden" content of lines inside checked files.
    //
    // Any checked file that contains a line matching any regex fails the check.
    //
    // This is a very flexible check and can be used to check for a few different things:
    //
    // * Windows/Mac OS line endings
    // * Trailing whitespace
    // * Tab characters used for indentation
    // * Spaces used for indentation
    //
    // Currently, `RegexSet` does not seem to have a method for getting the string a regex inside it
    // was created with (despite printing all regexes in its `Debug` impl), so we store them as a
    // `Vec<String>` next to it.
    forbidden_content: (RegexSet, Vec<String>) => |raw: Vec<String>| {
        (RegexSet::new(&raw)?, raw)
    },

    // The indentation style to enforce in the checked files.
    //
    // See `IndentationStyle` for more info about the accepted values.
    indentation_style: Option<IndentationStyle> => |raw: Option<String>| {
        if let Some(raw) = raw {
            Some(IndentationStyle::from_str(&raw)
                .map_err(|e| LoadError::InvalidIndentationStyle(e))?)
        } else {
            None
        }
    },
}

// FIXME this could use one of the many error macro crates

#[derive(Debug)]
pub enum LoadError {
    /// I/O error while reading/opening config file.
    IoError(io::Error),
    /// TOML syntax/decoding errors.
    TomlError(TomlError),
    /// Invalid glob.
    GlobError(PatternError),
    /// Invalid regex.
    RegexError(regex::Error),
    /// An `indentation-style` key could not be parsed.
    InvalidIndentationStyle(&'static str),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadError::IoError(ref e) => write!(f, "IO error: {}", e)?,
            LoadError::TomlError(ref e) => write!(f, "{}", e)?,
            LoadError::GlobError(ref e) => write!(f, "{}", e)?,
            LoadError::RegexError(ref e) => write!(f, "{}", e)?,
            LoadError::InvalidIndentationStyle(ref e) => write!(f, "{}", e)?,
        }

        Ok(())
    }
}

impl Error for LoadError {
    fn description(&self) -> &str {
        match *self {
            LoadError::IoError(ref e) => e.description(),
            LoadError::TomlError(ref e) => e.description(),
            LoadError::GlobError(ref e) => e.description(),
            LoadError::RegexError(ref e) => e.description(),
            LoadError::InvalidIndentationStyle(ref e) => e,
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> Self {
        LoadError::IoError(e)
    }
}

impl From<TomlError> for LoadError {
    fn from(e: TomlError) -> Self {
        LoadError::TomlError(e)
    }
}

impl From<PatternError> for LoadError {
    fn from(e: PatternError) -> Self {
        LoadError::GlobError(e)
    }
}

impl From<regex::Error> for LoadError {
    fn from(e: regex::Error) -> Self {
        LoadError::RegexError(e)
    }
}

/// Decodes a type from TOML pulled from a reader.
fn decode_toml<T: Deserialize, R: Read>(reader: &mut R) -> Result<T, LoadError> {
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(toml::from_str(&content)?)
}

impl Config {
    /// Load `Config` from a file, given as a path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, LoadError> {
        debug!("loading configuration from {}", path.as_ref().display());

        let mut file = File::open(path)?;
        let raw: RawConfig = decode_toml(&mut file)?;

        Ok(Config::from_raw(raw)?)
    }
}
