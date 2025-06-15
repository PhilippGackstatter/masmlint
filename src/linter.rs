use std::sync::Arc;

use miden_assembly::{
    Report, SourceFile, Span,
    ast::{Block, Export, Form, Instruction, Op},
    testing::TestContext,
};
use miette::{Context, Result};

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
    ) -> Result<()> {
        self.early_lint(early_lints, Arc::clone(&source))?;

        let errors = core::mem::take(&mut self.errors);

        if errors.is_empty() {
            Ok(())
        } else {
            let error = LinterError::new(errors);
            let error = Report::from(error).with_source_code(source);
            Err(error)
        }
    }

    fn early_lint(
        &mut self,
        mut lints: Vec<Box<dyn EarlyLintPass>>,
        source: Arc<SourceFile>,
    ) -> Result<()> {
        // This is abusing the miden-assembly testing feature to be able to parse the forms,
        // but there is no other public API to get the forms, unfortunately.
        let forms = TestContext::new()
            .parse_forms(source)
            .context("failed to parse source into forms")?;

        for form in forms {
            let Form::Procedure(Export::Procedure(proc)) = form else {
                continue;
            };

            self.lint_block(proc.body(), &mut lints);
        }

        Ok(())
    }

    pub fn lint_block(&mut self, block: &Block, lints: &mut [Box<dyn EarlyLintPass>]) {
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
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EarlyLintPass {
    fn lint_instruction(&mut self, linter: &mut Linter, instruction: &Span<Instruction>);
    fn block_changed(&mut self, _block: &Block) {}
}
