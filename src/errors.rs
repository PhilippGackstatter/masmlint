use miden_assembly::{SourceSpan, ast::Instruction};
use miette::Diagnostic;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum LintError {
    #[error("operand is pushed before an instruction that can take an immediate value")]
    #[diagnostic(help("use the instruction in its immediate form `{alternative}`"))]
    PushImmediate {
        #[label("instruction can be rewritten to take the immediate directly")]
        span: SourceSpan,
        alternative: String,
    },
    #[error("assert without error message")]
    #[diagnostic(help(
        "use the instruction with a helpful error message, e.g. `{assert_with_error}`"
    ))]
    BareAssert {
        #[label("does not include an error message")]
        span: SourceSpan,
        assert_with_error: Instruction,
    },
}

#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("one or more lints failed")]
pub struct LinterError {
    #[related]
    errors: Vec<LintError>,
}

impl LinterError {
    pub fn new(errors: Vec<LintError>) -> Self {
        Self { errors }
    }
}
