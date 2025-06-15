extern crate alloc;

mod errors;
pub use errors::{LintError, LinterError};

pub mod lints;

mod lint_selector;
pub use lint_selector::LintSelector;

mod linter;
pub use linter::{EarlyContext, EarlyLintPass, Linter};
