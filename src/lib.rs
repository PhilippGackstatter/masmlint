extern crate alloc;

mod errors;
pub use errors::LintError;

mod push_before_imm_variant_instr;
pub use push_before_imm_variant_instr::PushBeforeImmVariantInstr;

mod bare_assert;
pub use bare_assert::BareAssert;

mod linter;
pub use linter::{EarlyLintPass, LateLintPass, Linter};
