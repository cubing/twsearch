use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum VerbosityLevel {
    Silent,
    Error,
    #[default]
    Warning,
    Info,
    Extra,
}

impl FromStr for VerbosityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "silent" => VerbosityLevel::Silent,
            "error" => VerbosityLevel::Error,
            "warning" => VerbosityLevel::Warning,
            "info" => VerbosityLevel::Info,
            "extra" => VerbosityLevel::Extra,
            _ => Err("Invalid verbosity level name".to_owned())?,
        })
    }
}

// TODO: replace this with something less custom (ideally from the stdlib?)
#[derive(Clone, Default)]
pub struct SearchLogger {
    // TODO: writers for logs and error
    pub verbosity: VerbosityLevel,
}

impl SearchLogger {
    // TODO: support using the `write!` macro to avoid unnecessary string formatting in the caller when nothing is actually logged.
    pub fn write_extra(&self, s: &str) {
        if match self.verbosity {
            VerbosityLevel::Silent => false,
            VerbosityLevel::Error => false,
            VerbosityLevel::Warning => false,
            VerbosityLevel::Info => false,
            VerbosityLevel::Extra => true,
        } {
            eprintln!("{}", s)
        }
    }

    // TODO: support using the `write!` macro to avoid unnecessary string formatting in the caller when nothing is actually logged.
    pub fn write_info(&self, s: &str) {
        if match self.verbosity {
            VerbosityLevel::Silent => false,
            VerbosityLevel::Error => false,
            VerbosityLevel::Warning => false,
            VerbosityLevel::Info => true,
            VerbosityLevel::Extra => true,
        } {
            eprintln!("{}", s)
        }
    }

    pub fn write_warning(&self, s: &str) {
        if match self.verbosity {
            VerbosityLevel::Silent => false,
            VerbosityLevel::Error => false,
            VerbosityLevel::Warning => true,
            VerbosityLevel::Info => true,
            VerbosityLevel::Extra => true,
        } {
            eprintln!("{}", s);
        }
    }

    pub fn write_error(&self, s: &str) {
        if match self.verbosity {
            VerbosityLevel::Silent => false,
            VerbosityLevel::Error => true,
            VerbosityLevel::Warning => true,
            VerbosityLevel::Info => true,
            VerbosityLevel::Extra => true,
        } {
            eprintln!("{}", s);
        }
    }
}
