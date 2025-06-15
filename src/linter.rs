use std::sync::Arc;

use miden_assembly::{
    SourceFile, Span,
    ast::{Block, Export, Form, Instruction, Op},
    diagnostics::reporting::PrintDiagnostic,
    testing::TestContext,
};

use crate::{LintError, errors::LinterError};

pub struct Linter {
    errors: Vec<LintError>,
}

impl Linter {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn lint(
        mut self,
        early_lints: Vec<Box<dyn EarlyLintPass>>,
        source: Arc<SourceFile>,
    ) -> Result<(), LinterError> {
        self.early_lint(early_lints, Arc::clone(&source))?;

        let errors = core::mem::take(&mut self.errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(LinterError::new_lints(errors))
        }
    }

    fn early_lint(
        &mut self,
        mut lints: Vec<Box<dyn EarlyLintPass>>,
        source_file: Arc<SourceFile>,
    ) -> Result<(), LinterError> {
        // This is abusing the miden-assembly testing feature to be able to parse the forms,
        // but there is no other public API to get the forms, unfortunately.
        let forms = TestContext::new()
            .parse_forms(Arc::clone(&source_file))
            .map_err(|err| LinterError::FormsParsing(PrintDiagnostic::new(err).to_string()))?;

        let mut early_ctx = EarlyContext { linter: self, source_file };

        for form in forms {
            let Form::Procedure(Export::Procedure(proc)) = form else {
                continue;
            };

            early_ctx.lint_block(proc.body(), &mut lints);
        }

        Ok(())
    }

    fn push_error(&mut self, error: LintError) {
        self.errors.push(error);
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EarlyContext<'linter> {
    linter: &'linter mut Linter,
    source_file: Arc<SourceFile>,
}

impl<'linter> EarlyContext<'linter> {
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
        self.linter.push_error(error);
    }

    pub fn source_file(&self) -> Arc<SourceFile> {
        Arc::clone(&self.source_file)
    }
}

pub trait EarlyLintPass {
    fn lint_instruction(
        &mut self,
        early_ctx: &mut EarlyContext<'_>,
        instruction: &Span<Instruction>,
    );
    fn block_changed(&mut self, _block: &Block) {}
}
