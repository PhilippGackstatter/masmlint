use std::sync::Arc;

use masmlint::{
    EarlyLintPass, Linter,
    lints::{BareAssert, PushBeforeImmVariantInstr},
};
use miden_assembly::{SourceFile, SourceId};

fn main() -> miette::Result<()> {
    let source_path = std::env::args().nth(1).unwrap();

    let source = std::fs::read(&source_path).unwrap();
    let source_content = String::from_utf8(source).unwrap();

    let source_file = SourceFile::new(SourceId::new(5), source_path, source_content);

    let early_lints: Vec<Box<dyn EarlyLintPass>> =
        vec![Box::new(PushBeforeImmVariantInstr::new()), Box::new(BareAssert)];

    Linter::new().lint(early_lints, Arc::new(source_file))
}
