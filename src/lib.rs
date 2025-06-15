extern crate alloc;

mod errors;
pub use errors::{LintError, LinterError};

pub mod lints;

mod linter;
pub use linter::{EarlyContext, EarlyLintPass, Linter};
