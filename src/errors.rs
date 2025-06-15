use std::sync::Arc;

use miden_assembly::{SourceFile, SourceSpan, ast::Instruction};
use miette::Diagnostic;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum LintError {
    #[error("operand is pushed before an instruction that can take an immediate value")]
    #[diagnostic(help("use the instruction in its immediate form `{alternative}`"))]
    PushImmediate {
        #[label("instruction can be rewritten to take the immediate directly")]
        span: SourceSpan,
        alternative: String,
        #[source_code]
        source_file: Arc<SourceFile>,
    },
    #[error("assert without error message")]
    #[diagnostic(help(
        "use the instruction with a helpful error message, e.g. `{assert_with_error}`"
    ))]
    BareAssert {
        #[label("does not include an error message")]
        span: SourceSpan,
        assert_with_error: Instruction,
        #[source_code]
        source_file: Arc<SourceFile>,
    },
}

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum LinterError {
    #[error("one or more lints failed")]
    Lints {
        #[related]
        errors: Vec<LintError>,
    },
    #[error("failed to parse MASM into forms: {0}")]
    FormsParsing(String),
    #[error("failed to select unknown lint `{0}`")]
    UnknownSelectedLint(String),
    #[error("failed to exclude unknown lint `{0}`")]
    UnknownExcludedLint(String),
}

impl LinterError {
    pub fn new_lints(errors: Vec<LintError>) -> Self {
        Self::Lints { errors }
    }
}
