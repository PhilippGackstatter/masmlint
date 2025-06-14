use std::sync::Arc;

use miden_assembly::{
    SourceId, SourceSpan, Span,
    ast::{Block, Export, Form, Immediate, Instruction, Op},
};

use crate::{LintError, Linter, linter::EarlyLintPass};

pub struct BareAssert;

impl EarlyLintPass for BareAssert {
    fn lint(&mut self, linter: &mut Linter, forms: &[Form]) {
        for form in forms {
            let Form::Procedure(Export::Procedure(proc)) = form else {
                continue;
            };

            lint_block(linter, proc.body());
        }
    }
}

pub fn lint_block(linter: &mut Linter, block: &Block) {
    for op in block.iter() {
        match op {
            Op::If { then_blk, else_blk, .. } => {
                lint_block(linter, then_blk);
                lint_block(linter, else_blk);
            },
            Op::While { body, .. } => {
                lint_block(linter, body);
            },
            Op::Repeat { body, .. } => {
                lint_block(linter, body);
            },
            Op::Inst(instr) => {
                if let Some(assert_with_error) = match_assert_instruction(instr) {
                    linter.push_error(LintError::BareAssert {
                        span: instr.span(),
                        assert_with_error,
                    });
                }
            },
        }
    }
}

fn match_assert_instruction(instruction: &Span<Instruction>) -> Option<Instruction> {
    let example_msg = "helpful error message";
    let span = SourceSpan::at(SourceId::new(0), 0);
    let example_msg: Arc<str> = Arc::from(example_msg);
    let example_msg = Immediate::Value(Span::new(span, example_msg));

    match instruction.inner() {
        Instruction::Assert => Some(Instruction::AssertWithError(example_msg)),
        Instruction::AssertEq => Some(Instruction::AssertEqWithError(example_msg)),
        Instruction::AssertEqw => Some(Instruction::AssertEqwWithError(example_msg)),
        Instruction::Assertz => Some(Instruction::AssertzWithError(example_msg)),
        Instruction::U32Assert => Some(Instruction::U32AssertWithError(example_msg)),
        Instruction::U32Assert2 => Some(Instruction::U32Assert2WithError(example_msg)),
        Instruction::U32AssertW => Some(Instruction::U32AssertWWithError(example_msg)),
        _ => None,
    }
}
