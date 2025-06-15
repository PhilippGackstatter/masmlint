use std::sync::Arc;

use miden_assembly::{
    SourceFile, Span,
    ast::{Block, Export, Form, Instruction, Op},
    diagnostics::reporting::PrintDiagnostic,
    testing::TestContext,
};

use crate::{LintError, errors::LinterError};

pub struct Linter {
    lints: Vec<Box<dyn EarlyLintPass>>,
    errors: Vec<LintError>,
}

impl Linter {
    pub fn new(lints: Vec<Box<dyn EarlyLintPass>>) -> Self {
        Self { lints, errors: Vec::new() }
    }

    pub fn lint(&mut self, source: Arc<SourceFile>) -> Result<(), LinterError> {
        self.early_lint(Arc::clone(&source))?;

        let errors = core::mem::take(&mut self.errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(LinterError::new_lints(errors))
        }
    }

    fn early_lint(&mut self, source_file: Arc<SourceFile>) -> Result<(), LinterError> {
        // This is abusing the miden-assembly testing feature to be able to parse the forms,
        // but there is no other public API to get the forms, unfortunately.
        let forms = TestContext::new()
            .parse_forms(Arc::clone(&source_file))
            .map_err(|err| LinterError::FormsParsing(PrintDiagnostic::new(err).to_string()))?;

        let errors = core::mem::take(&mut self.errors);

        let mut early_ctx = EarlyContext { errors, source_file };

        for form in forms {
            let Form::Procedure(Export::Procedure(proc)) = form else {
                continue;
            };

            early_ctx.lint_block(proc.body(), self.lints.as_mut_slice());
        }

        // Put the errors back into the field.
        core::mem::swap(&mut early_ctx.errors, &mut self.errors);

        Ok(())
    }
}

pub struct EarlyContext {
    errors: Vec<LintError>,
    source_file: Arc<SourceFile>,
}

impl EarlyContext {
    fn lint_block(&mut self, block: &Block, lints: &mut [Box<dyn EarlyLintPass>]) {
        for lint in lints.iter_mut() {
            lint.block_changed(block);
        }

        for op in block.iter() {
            match op {
                Op::If { then_blk, else_blk, .. } => {
                    self.lint_block(then_blk, lints);
                    self.lint_block(else_blk, lints);
                },
                Op::While { body, .. } => {
                    self.lint_block(body, lints);
                },
                Op::Repeat { body, .. } => {
                    self.lint_block(body, lints);
                },
                Op::Inst(instr) => {
                    for lint in lints.iter_mut() {
                        lint.lint_instruction(self, instr);
                    }
                },
            }
        }
    }

    pub fn push_error(&mut self, error: LintError) {
        self.errors.push(error);
    }

    pub fn source_file(&self) -> Arc<SourceFile> {
        Arc::clone(&self.source_file)
    }
}

pub trait EarlyLintPass {
    fn lint_instruction(&mut self, early_ctx: &mut EarlyContext, instruction: &Span<Instruction>);
    fn block_changed(&mut self, _block: &Block) {}
}
