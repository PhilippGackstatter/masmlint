extern crate alloc;

mod errors;
pub use errors::LintError;

pub mod lints;

mod linter;
pub use linter::{EarlyLintPass, Linter};
