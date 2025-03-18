use crate::_internal::cli::args::VerbosityLevel;

// TODO: replace this with something less custom (ideally from the stdlib?)
#[derive(Clone, Default)]
pub struct SearchLogger {
    // TODO: writers for logs and error
    pub verbosity: VerbosityLevel,
}

impl SearchLogger {
    // TODO: support using the `write!` macro to avoid unnecessary string formatting in the caller when nothing is actually logged.
    pub fn write_info(&self, s: &str) {
        if match self.verbosity {
            VerbosityLevel::Silent => false,
            VerbosityLevel::Error => false,
            VerbosityLevel::Warning => false,
            VerbosityLevel::Info => true,
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
