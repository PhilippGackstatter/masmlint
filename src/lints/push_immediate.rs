use miden_assembly::{
    SourceId, SourceSpan, Span,
    ast::{Block, Ident, Immediate, Instruction},
};
use miden_core::Felt;

use crate::{EarlyLintPass, LintError};

pub struct PushImmediate {
    prev_push_instr: Option<(SourceSpan, ImmediateWithoutSpan)>,
}

impl PushImmediate {
    pub fn new() -> Self {
        Self { prev_push_instr: None }
    }
}

impl Default for PushImmediate {
    fn default() -> Self {
        Self::new()
    }
}

impl EarlyLintPass for PushImmediate {
    fn lint_instruction(&mut self, linter: &mut crate::Linter, instruction: &Span<Instruction>) {
        if let (Some((prev_span, prev_imm)), current_instr) =
            (self.prev_push_instr.take(), instruction)
        {
            if let Some(alternative) =
                match_non_immediate_instruction(prev_span, prev_imm, current_instr)
            {
                let full_span = SourceSpan::new(
                    prev_span.source_id(),
                    prev_span.start()..current_instr.span().end(),
                );

                linter.push_error(LintError::PushImmediate { span: full_span, alternative });
            }
        }

        match match_push_instruction(instruction) {
            Some(imm) => {
                self.prev_push_instr = Some((instruction.span(), imm));
            },
            None => {
                self.prev_push_instr = None;
            },
        }
    }

    /// Reset the previous instruction if the block has changed, as the lint might not apply across
    /// block boundaries.
    fn block_changed(&mut self, _block: &Block) {
        self.prev_push_instr = None;
    }
}

fn match_push_instruction(instruction: &Span<Instruction>) -> Option<ImmediateWithoutSpan> {
    match instruction.inner() {
        Instruction::Push(immediate) => Some(immediate.clone().into()),
        Instruction::PushU8(val) => Some(ImmediateWithoutSpan::Value((*val).into())),
        Instruction::PushU16(val) => Some(ImmediateWithoutSpan::Value((*val).into())),
        Instruction::PushU32(val) => Some(ImmediateWithoutSpan::Value((*val).into())),
        Instruction::PushFelt(base_element) => Some(ImmediateWithoutSpan::Value(*base_element)),
        // It looks like these instructions are never used?
        Instruction::PushU8List(_items) => None,
        Instruction::PushU16List(_items) => None,
        Instruction::PushU32List(_items) => None,
        Instruction::PushFeltList(_base_elements) => None,
        Instruction::PushWord(_) => None,
        _ => None,
    }
}

// Some instructions support immediate-style MASM but do not have explicit immediate instruction
// variants, such as `lt`. When writing `lt.2` in MASM, it is rewritten to `push.2 lt` at parsing
// time. To differentiate the case when the MASM code contains `push.2 lt` or `lt.2` we can check if
// the source span of the push instruction is different from the instruction one, which is only the
// case if it was automatically rewritten. This is a quirk of how the instruction is rewritten, see:
// https://github.com/0xMiden/miden-vm/blob/506027aec5ac692c117eeb47f72fadb07d807012/assembly/src/parser/grammar.lalrpop#L782-L793
fn match_non_immediate_instruction(
    prev_span: SourceSpan,
    imm: ImmediateWithoutSpan,
    instruction: &Span<Instruction>,
) -> Option<String> {
    let instr = instruction.inner().clone();
    match instr {
        Instruction::Add => Some(Instruction::AddImm(imm.into_felt()).to_string()),
        Instruction::Sub => Some(Instruction::SubImm(imm.into_felt()).to_string()),
        Instruction::Mul => Some(Instruction::MulImm(imm.into_felt()).to_string()),
        Instruction::Div => Some(Instruction::DivImm(imm.into_felt()).to_string()),
        Instruction::Exp => Some(Instruction::ExpImm(imm.into_felt()).to_string()),
        Instruction::Eq => Some(Instruction::EqImm(imm.into_felt()).to_string()),
        Instruction::Neq => Some(Instruction::NeqImm(imm.into_felt()).to_string()),
        Instruction::Lt if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::Lte if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::Gt if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::Gte if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32WrappingAdd => {
            Some(Instruction::U32WrappingAddImm(imm.into_u32()).to_string())
        },
        Instruction::U32OverflowingAdd => {
            Some(Instruction::U32OverflowingAddImm(imm.into_u32()).to_string())
        },
        Instruction::U32WrappingSub => {
            Some(Instruction::U32WrappingSubImm(imm.into_u32()).to_string())
        },
        Instruction::U32OverflowingSub => {
            Some(Instruction::U32OverflowingSubImm(imm.into_u32()).to_string())
        },
        Instruction::U32WrappingMul => {
            Some(Instruction::U32WrappingMulImm(imm.into_u32()).to_string())
        },
        Instruction::U32OverflowingMul => {
            Some(Instruction::U32OverflowingMulImm(imm.into_u32()).to_string())
        },
        Instruction::U32Div => Some(Instruction::U32DivImm(imm.into_u32()).to_string()),
        Instruction::U32Mod => Some(Instruction::U32ModImm(imm.into_u32()).to_string()),
        Instruction::U32DivMod => Some(Instruction::U32DivModImm(imm.into_u32()).to_string()),
        Instruction::U32And if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Or if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Xor if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Not if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Shr if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Shl if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Rotr if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Rotl if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Lt if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Lte if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Gt if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Gte if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Min if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::U32Max if prev_span != instruction.span() => format_instr(instr, imm),
        Instruction::MemLoad => Some(Instruction::MemLoadImm(imm.into_u32()).to_string()),
        Instruction::MemLoadW => Some(Instruction::MemLoadWImm(imm.into_u32()).to_string()),
        Instruction::MemStore => Some(Instruction::MemStoreImm(imm.into_u32()).to_string()),
        Instruction::MemStoreW => Some(Instruction::MemStoreWImm(imm.into_u32()).to_string()),
        _ => None,
    }
}

enum ImmediateWithoutSpan {
    Value(Felt),
    Constant(Ident),
}

impl ImmediateWithoutSpan {
    pub fn into_felt(self) -> Immediate<Felt> {
        let span = SourceSpan::at(SourceId::new(0), 0);
        match self {
            ImmediateWithoutSpan::Value(base_element) => {
                Immediate::Value(Span::new(span, base_element))
            },
            ImmediateWithoutSpan::Constant(ident) => Immediate::Constant(ident),
        }
    }

    pub fn into_u32(self) -> Immediate<u32> {
        let span = SourceSpan::at(SourceId::new(0), 0);
        match self {
            ImmediateWithoutSpan::Value(base_element) => Immediate::Value(Span::new(
                span,
                base_element.try_into().expect("should fit into u32"),
            )),
            ImmediateWithoutSpan::Constant(ident) => Immediate::Constant(ident),
        }
    }
}

impl From<Immediate<Felt>> for ImmediateWithoutSpan {
    fn from(value: Immediate<Felt>) -> Self {
        match value {
            Immediate::Value(span) => ImmediateWithoutSpan::Value(span.into_inner()),
            Immediate::Constant(ident) => ImmediateWithoutSpan::Constant(ident),
        }
    }
}

fn format_instr(instr: Instruction, imm: ImmediateWithoutSpan) -> Option<String> {
    Some(format!("{}.{}", instr, imm.into_felt()))
}
