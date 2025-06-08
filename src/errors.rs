use miden_assembly::{SourceSpan, ast::Instruction};
use miette::Diagnostic;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum LintError {
    #[error("push before instruction that takes an immediate value")]
    #[diagnostic(help("use the instruction in its immediate form, e.g. `{alternative}`"))]
    PushBeforeNonImmediateInstruction {
        #[label]
        span: SourceSpan,
        alternative: Instruction,
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
