use miden_assembly::{
    SourceId, SourceSpan, Span,
    ast::{Block, Export, Form, Ident, Immediate, Instruction, Op},
};
use miden_core::Felt;

use crate::{LintError, linter::EarlyLintPass};

pub struct PushBeforeImmVariantInstr;

impl EarlyLintPass for PushBeforeImmVariantInstr {
    fn lint(&mut self, linter: &mut crate::Linter, forms: &[Form]) {
        for form in forms {
            let Form::Procedure(Export::Procedure(proc)) = form else {
                continue;
            };

            if let Err(err) = lint_block(proc.body()) {
                linter.push_error(err);
            }
        }
    }
}

pub fn lint_block(block: &Block) -> Result<(), LintError> {
    let mut prev_instr: Option<(SourceSpan, ImmediateWithoutSpan)> = None;

    for op in block.iter() {
        if let (Some((prev_span, prev_imm)), Op::Inst(current_instr)) = (prev_instr.take(), op) {
            if let Some(alternative) = match_non_immediate_instruction(prev_imm, current_instr) {
                let full_span = SourceSpan::new(
                    prev_span.source_id(),
                    prev_span.start()..current_instr.span().end(),
                );

                return Err(LintError::PushBeforeNonImmediateInstruction {
                    span: full_span,
                    alternative,
                });
            }
        }

        if let Op::Inst(instr) = op {
            match match_push_instruction(instr) {
                Some(imm) => {
                    prev_instr = Some((instr.span(), imm));
                },
                None => {
                    prev_instr = None;
                },
            }
        }
    }

    Ok(())
}

fn match_push_instruction(instruction: &Span<Instruction>) -> Option<ImmediateWithoutSpan> {
    match instruction.inner() {
        Instruction::Push(immediate) => Some(immediate.clone().into()),
        Instruction::PushU8(val) => Some(ImmediateWithoutSpan::Value((*val).into())),
        Instruction::PushU16(val) => Some(ImmediateWithoutSpan::Value((*val).into())),
        Instruction::PushU32(val) => Some(ImmediateWithoutSpan::Value((*val).into())),
        Instruction::PushFelt(base_element) => Some(ImmediateWithoutSpan::Value(*base_element)),
        // Instruction::PushU8List(items) => todo!(),
        // Instruction::PushU16List(items) => todo!(),
        // Instruction::PushU32List(items) => todo!(),
        // Instruction::PushFeltList(base_elements) => todo!(),
        Instruction::PushWord(_) => None,
        _ => None,
    }
}

fn match_non_immediate_instruction(
    imm: ImmediateWithoutSpan,
    instruction: &Span<Instruction>,
) -> Option<Instruction> {
    match instruction.inner() {
        Instruction::Add => Some(Instruction::AddImm(imm.into_felt())),
        Instruction::Sub => Some(Instruction::SubImm(imm.into_felt())),
        Instruction::Mul => Some(Instruction::MulImm(imm.into_felt())),
        Instruction::Div => Some(Instruction::DivImm(imm.into_felt())),
        Instruction::Exp => Some(Instruction::ExpImm(imm.into_felt())),
        Instruction::Eq => Some(Instruction::EqImm(imm.into_felt())),
        Instruction::Neq => Some(Instruction::NeqImm(imm.into_felt())),
        // Instruction::Lt => todo!(),
        // Instruction::Lte => todo!(),
        // Instruction::Gt => todo!(),
        // Instruction::Gte => todo!(),
        Instruction::U32WrappingAdd => Some(Instruction::U32WrappingAddImm(imm.into_u32())),
        Instruction::U32OverflowingAdd => Some(Instruction::U32OverflowingAddImm(imm.into_u32())),
        Instruction::U32WrappingSub => Some(Instruction::U32WrappingSubImm(imm.into_u32())),
        Instruction::U32OverflowingSub => Some(Instruction::U32OverflowingSubImm(imm.into_u32())),
        Instruction::U32WrappingMul => Some(Instruction::U32WrappingMulImm(imm.into_u32())),
        Instruction::U32OverflowingMul => Some(Instruction::U32OverflowingMulImm(imm.into_u32())),
        Instruction::U32Div => Some(Instruction::U32DivImm(imm.into_u32())),
        Instruction::U32Mod => Some(Instruction::U32ModImm(imm.into_u32())),
        Instruction::U32DivMod => Some(Instruction::U32DivModImm(imm.into_u32())),
        // Instruction::U32And => todo!()
        // Instruction::U32Or => todo!(),
        // Instruction::U32Xor => todo!(),
        // Instruction::U32Not => todo!(),
        // Instruction::U32Shr => todo!(),
        // Instruction::U32Shl => todo!(),
        // Instruction::U32Rotr => todo!(),
        // Instruction::U32Rotl => todo!(),
        // Instruction::U32Lt => todo!(),
        // Instruction::U32Lte => todo!(),
        // Instruction::U32Gt => todo!(),
        // Instruction::U32Gte => todo!(),
        // Instruction::U32Min => todo!(),
        // Instruction::U32Max => todo!(),
        Instruction::MemLoad => Some(Instruction::MemLoadImm(imm.into_u32())),
        Instruction::MemLoadW => Some(Instruction::MemLoadWImm(imm.into_u32())),
        Instruction::MemStore => Some(Instruction::MemStoreImm(imm.into_u32())),
        Instruction::MemStoreW => Some(Instruction::MemStoreWImm(imm.into_u32())),
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
