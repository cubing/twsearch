use crate::_internal::cli::VerbosityLevel;

// TODO: replace this with something less custom (ideally from the stdlib?)
#[derive(Clone)]
pub struct SearchLogger {
    // TODO: writers for logs and error
    pub verbosity: VerbosityLevel,
}

impl SearchLogger {
    pub fn write_info(&self, s: &str) {
        if match self.verbosity {
            VerbosityLevel::Silent => false,
            VerbosityLevel::Error => false,
            VerbosityLevel::Warning => false,
            VerbosityLevel::Info => true,
        } {
            println!("{}", s)
        }
    }

    pub fn write_warning(&self, s: &str) {
        if match self.verbosity {
            VerbosityLevel::Silent => false,
            VerbosityLevel::Error => false,
            VerbosityLevel::Warning => true,
            VerbosityLevel::Info => true,
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
        } {
            eprintln!("{}", s);
        }
    }
}
