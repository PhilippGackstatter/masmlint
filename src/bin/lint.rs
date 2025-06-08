use std::sync::Arc;

use masmlint::{EarlyLintPass, LateLintPass, Linter, PushBeforeImmVariantInstr};
use miden_assembly::{SourceFile, SourceId, ast::ModuleKind};

fn main() -> Result<(), miden_assembly::Report> {
    let source_path = std::env::args().nth(1).unwrap();

    let source = std::fs::read(&source_path).unwrap();
    let source_content = String::from_utf8(source).unwrap();

    let source_file = SourceFile::new(SourceId::new(5), source_path, source_content);

    let late_lints: Vec<Box<dyn LateLintPass>> = vec![];
    let early_lints: Vec<Box<dyn EarlyLintPass>> = vec![Box::new(PushBeforeImmVariantInstr)];

    Linter::new().lint(early_lints, late_lints, ModuleKind::Library, Arc::new(source_file))
}
