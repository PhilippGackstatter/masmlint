use std::sync::Arc;

use miden_assembly::{
    LibraryPath, ModuleParser, Report, SourceFile,
    ast::{Form, Module, ModuleKind},
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
        late_lints: Vec<Box<dyn LateLintPass>>,
        kind: ModuleKind,
        source: Arc<SourceFile>,
    ) -> Result<()> {
        self.early_lint(early_lints, Arc::clone(&source))?;
        self.late_lint(late_lints, kind, Arc::clone(&source))?;

        let errors = core::mem::take(&mut self.errors);

        if errors.is_empty() {
            Ok(())
        } else {
            let error = LinterError::new(errors);
            let error = Report::from(error).with_source_code(source.as_bytes().to_vec());
            Err(error)
        }
    }

    fn early_lint(
        &mut self,
        lints: Vec<Box<dyn EarlyLintPass>>,
        source: Arc<SourceFile>,
    ) -> Result<()> {
        // This is abusing the miden-assembly testing feature to be able to parse the forms,
        // but there is not other public API to get the forms, unfortunately.
        let forms = TestContext::new()
            .parse_forms(source)
            .context("failed to parse source into forms")?;

        for mut lint in lints {
            lint.lint(self, &forms);
        }

        Ok(())
    }

    fn late_lint(
        &mut self,
        lints: Vec<Box<dyn LateLintPass>>,
        kind: ModuleKind,
        source: Arc<SourceFile>,
    ) -> Result<()> {
        let path = LibraryPath::new("library_path").unwrap();
        let module = ModuleParser::new(kind).parse(path, Arc::clone(&source))?;

        for mut lint in lints {
            lint.lint(self, &module);
        }

        Ok(())
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

pub trait LateLintPass {
    fn lint(&mut self, linter: &mut Linter, module: &Module);
}

pub trait EarlyLintPass {
    fn lint(&mut self, linter: &mut Linter, forms: &[Form]);
}
