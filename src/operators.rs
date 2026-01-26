use crate::program::*;
use anyhow::{Ok, Result, anyhow};

impl TryFrom<wasmparser::BlockType> for BlockType {
    type Error = anyhow::Error;

    fn try_from(blockty: wasmparser::BlockType) -> Result<Self> {
        match blockty {
            wasmparser::BlockType::Empty => Ok(BlockType {
                blockty: Some(block_type::Blockty::Empty(0)),
            }),
            wasmparser::BlockType::Type(valtype) => Ok(BlockType {
                blockty: Some(block_type::Blockty::ValueType(
                    ValueType::try_from(valtype)? as i32,
                )),
            }),
            wasmparser::BlockType::FuncType(funcidx) => Ok(BlockType {
                blockty: Some(block_type::Blockty::FuncType(funcidx)),
            }),
        }
    }
}

impl TryFrom<BlockType> for wasm_encoder::BlockType {
    type Error = anyhow::Error;

    fn try_from(blocktype: BlockType) -> Result<Self> {
        use crate::program::block_type::Blockty;
        use wasm_encoder::BlockType;
        match blocktype.blockty.ok_or(anyhow!("Block type not found"))? {
            Blockty::Empty(0) => Ok(BlockType::Empty),
            Blockty::ValueType(valtype) => {
                Ok(BlockType::Result(ValueType::try_from(valtype)?.try_into()?))
            }
            Blockty::FuncType(funcidx) => Ok(BlockType::FunctionType(funcidx)),
            _ => Err(anyhow!("Unsupported block type: {:?}", blocktype)),
        }
    }
}

impl TryFrom<wasmparser::MemArg> for MemArg {
    type Error = anyhow::Error;

    fn try_from(memarg: wasmparser::MemArg) -> Result<Self> {
        Ok(MemArg {
            align: Some(memarg.align as u32),
            max_align: Some(memarg.max_align as u32),
            offset: Some(memarg.offset),
            memory: Some(memarg.memory),
        })
    }
}

impl TryFrom<MemArg> for wasm_encoder::MemArg {
    type Error = anyhow::Error;

    fn try_from(memarg: MemArg) -> Result<Self> {
        match memarg {
            MemArg {
                align: Some(align),
                max_align: _,
                offset: Some(offset),
                memory: Some(memory),
            } => Ok(wasm_encoder::MemArg {
                align,
                offset,
                memory_index: memory,
            }),
            _ => Err(anyhow!("Unsupported memarg: {:?}", memarg)),
        }
    }
}

impl TryFrom<wasmparser::Operator<'_>> for Operator {
    type Error = anyhow::Error;

    fn try_from(operator: wasmparser::Operator<'_>) -> Result<Self> {
        match operator {
            // @mvp
            wasmparser::Operator::Unreachable => Ok(Operator {
                opcode: Some(OpCode::Unreachable as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::Nop => Ok(Operator {
                opcode: Some(OpCode::Nop as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::Block { blockty } => Ok(Operator {
                opcode: Some(OpCode::Block as i32),
                operator: Some(operator::Operator::Blockty(BlockType::try_from(blockty)?)),
            }),
            wasmparser::Operator::Loop { blockty } => Ok(Operator {
                opcode: Some(OpCode::Loop as i32),
                operator: Some(operator::Operator::Blockty(BlockType::try_from(blockty)?)),
            }),
            wasmparser::Operator::If { blockty } => Ok(Operator {
                opcode: Some(OpCode::If as i32),
                operator: Some(operator::Operator::Blockty(BlockType::try_from(blockty)?)),
            }),
            wasmparser::Operator::Else => Ok(Operator {
                opcode: Some(OpCode::Else as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::End => Ok(Operator {
                opcode: Some(OpCode::End as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::Br { relative_depth } => Ok(Operator {
                opcode: Some(OpCode::Br as i32),
                operator: Some(operator::Operator::RelativeDepth(relative_depth)),
            }),
            wasmparser::Operator::BrIf { relative_depth } => Ok(Operator {
                opcode: Some(OpCode::BrIf as i32),
                operator: Some(operator::Operator::RelativeDepth(relative_depth)),
            }),
            wasmparser::Operator::BrTable { targets } => {
                let mut brtargets: Vec<u32> = Vec::new();
                for target in targets.targets() {
                    brtargets.push(target?);
                }
                Ok(Operator {
                    opcode: Some(OpCode::BrTable as i32),
                    operator: Some(operator::Operator::Targets(BrTargets {
                        default: Some(targets.default()),
                        targets: brtargets,
                    })),
                })
            }
            wasmparser::Operator::Return => Ok(Operator {
                opcode: Some(OpCode::Return as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::Call { function_index } => Ok(Operator {
                opcode: Some(OpCode::Call as i32),
                operator: Some(operator::Operator::FunctionIndex(function_index)),
            }),
            wasmparser::Operator::CallIndirect {
                type_index,
                table_index,
            } => Ok(Operator {
                opcode: Some(OpCode::CallIndirect as i32),
                operator: Some(operator::Operator::CallInderect(CallIndirectOp {
                    type_index: Some(type_index),
                    table_index: Some(table_index),
                })),
            }),
            wasmparser::Operator::Drop => Ok(Operator {
                opcode: Some(OpCode::Drop as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::Select => Ok(Operator {
                opcode: Some(OpCode::Select as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::LocalGet { local_index } => Ok(Operator {
                opcode: Some(OpCode::LocalGet as i32),
                operator: Some(operator::Operator::LocalIndex(local_index)),
            }),
            wasmparser::Operator::LocalSet { local_index } => Ok(Operator {
                opcode: Some(OpCode::LocalSet as i32),
                operator: Some(operator::Operator::LocalIndex(local_index)),
            }),
            wasmparser::Operator::LocalTee { local_index } => Ok(Operator {
                opcode: Some(OpCode::LocalTee as i32),
                operator: Some(operator::Operator::LocalIndex(local_index)),
            }),
            wasmparser::Operator::GlobalGet { global_index } => Ok(Operator {
                opcode: Some(OpCode::GlobalGet as i32),
                operator: Some(operator::Operator::GlobalIndex(global_index)),
            }),
            wasmparser::Operator::GlobalSet { global_index } => Ok(Operator {
                opcode: Some(OpCode::GlobalSet as i32),
                operator: Some(operator::Operator::GlobalIndex(global_index)),
            }),
            wasmparser::Operator::I32Load { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Load as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Load { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Load as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::F32Load { memarg } => Ok(Operator {
                opcode: Some(OpCode::F32Load as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::F64Load { memarg } => Ok(Operator {
                opcode: Some(OpCode::F64Load as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I32Load8S { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Load8S as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I32Load8U { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Load8U as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I32Load16S { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Load16S as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I32Load16U { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Load16U as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Load8S { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Load8S as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Load8U { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Load8U as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Load16S { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Load16S as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Load16U { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Load16U as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Load32S { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Load32S as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Load32U { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Load32U as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I32Store { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Store as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Store { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Store as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::F32Store { memarg } => Ok(Operator {
                opcode: Some(OpCode::F32Store as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::F64Store { memarg } => Ok(Operator {
                opcode: Some(OpCode::F64Store as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I32Store8 { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Store8 as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I32Store16 { memarg } => Ok(Operator {
                opcode: Some(OpCode::I32Store16 as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Store8 { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Store8 as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Store16 { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Store16 as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::I64Store32 { memarg } => Ok(Operator {
                opcode: Some(OpCode::I64Store32 as i32),
                operator: Some(operator::Operator::Memarg(MemArg::try_from(memarg)?)),
            }),
            wasmparser::Operator::MemorySize { mem } => Ok(Operator {
                opcode: Some(OpCode::MemorySize as i32),
                operator: Some(operator::Operator::Mem(mem)),
            }),
            wasmparser::Operator::MemoryGrow { mem } => Ok(Operator {
                opcode: Some(OpCode::MemoryGrow as i32),
                operator: Some(operator::Operator::Mem(mem)),
            }),
            wasmparser::Operator::I32Const { value } => Ok(Operator {
                opcode: Some(OpCode::I32Const as i32),
                operator: Some(operator::Operator::I32value(value)),
            }),
            wasmparser::Operator::I64Const { value } => Ok(Operator {
                opcode: Some(OpCode::I64Const as i32),
                operator: Some(operator::Operator::I64value(value)),
            }),
            wasmparser::Operator::F32Const { value } => Ok(Operator {
                opcode: Some(OpCode::F32Const as i32),
                operator: Some(operator::Operator::F32value(value.bits())),
            }),
            wasmparser::Operator::F64Const { value } => Ok(Operator {
                opcode: Some(OpCode::F64Const as i32),
                operator: Some(operator::Operator::F64value(value.bits())),
            }),
            wasmparser::Operator::I32Eqz => Ok(Operator {
                opcode: Some(OpCode::I32Eqz as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Eq => Ok(Operator {
                opcode: Some(OpCode::I32Eq as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Ne => Ok(Operator {
                opcode: Some(OpCode::I32Ne as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32LtS => Ok(Operator {
                opcode: Some(OpCode::I32LtS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32LtU => Ok(Operator {
                opcode: Some(OpCode::I32LtU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32GtS => Ok(Operator {
                opcode: Some(OpCode::I32GtS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32GtU => Ok(Operator {
                opcode: Some(OpCode::I32GtU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32LeS => Ok(Operator {
                opcode: Some(OpCode::I32LeS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32LeU => Ok(Operator {
                opcode: Some(OpCode::I32LeU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32GeS => Ok(Operator {
                opcode: Some(OpCode::I32GeS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32GeU => Ok(Operator {
                opcode: Some(OpCode::I32GeU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Eqz => Ok(Operator {
                opcode: Some(OpCode::I64Eqz as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Eq => Ok(Operator {
                opcode: Some(OpCode::I64Eq as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Ne => Ok(Operator {
                opcode: Some(OpCode::I64Ne as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64LtS => Ok(Operator {
                opcode: Some(OpCode::I64LtS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64LtU => Ok(Operator {
                opcode: Some(OpCode::I64LtU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64GtS => Ok(Operator {
                opcode: Some(OpCode::I64GtS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64GtU => Ok(Operator {
                opcode: Some(OpCode::I64GtU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64LeS => Ok(Operator {
                opcode: Some(OpCode::I64LeS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64LeU => Ok(Operator {
                opcode: Some(OpCode::I64LeU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64GeS => Ok(Operator {
                opcode: Some(OpCode::I64GeS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64GeU => Ok(Operator {
                opcode: Some(OpCode::I64GeU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Eq => Ok(Operator {
                opcode: Some(OpCode::F32Eq as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Ne => Ok(Operator {
                opcode: Some(OpCode::F32Ne as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Lt => Ok(Operator {
                opcode: Some(OpCode::F32Lt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Gt => Ok(Operator {
                opcode: Some(OpCode::F32Gt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Le => Ok(Operator {
                opcode: Some(OpCode::F32Le as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Ge => Ok(Operator {
                opcode: Some(OpCode::F32Ge as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Eq => Ok(Operator {
                opcode: Some(OpCode::F64Eq as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Ne => Ok(Operator {
                opcode: Some(OpCode::F64Ne as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Lt => Ok(Operator {
                opcode: Some(OpCode::F64Lt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Gt => Ok(Operator {
                opcode: Some(OpCode::F64Gt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Le => Ok(Operator {
                opcode: Some(OpCode::F64Le as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Ge => Ok(Operator {
                opcode: Some(OpCode::F64Ge as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Clz => Ok(Operator {
                opcode: Some(OpCode::I32Clz as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Ctz => Ok(Operator {
                opcode: Some(OpCode::I32Ctz as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Popcnt => Ok(Operator {
                opcode: Some(OpCode::I32Popcnt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Add => Ok(Operator {
                opcode: Some(OpCode::I32Add as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Sub => Ok(Operator {
                opcode: Some(OpCode::I32Sub as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Mul => Ok(Operator {
                opcode: Some(OpCode::I32Mul as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32DivS => Ok(Operator {
                opcode: Some(OpCode::I32DivS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32DivU => Ok(Operator {
                opcode: Some(OpCode::I32DivU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32RemS => Ok(Operator {
                opcode: Some(OpCode::I32RemS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32RemU => Ok(Operator {
                opcode: Some(OpCode::I32RemU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32And => Ok(Operator {
                opcode: Some(OpCode::I32And as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Or => Ok(Operator {
                opcode: Some(OpCode::I32Or as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Xor => Ok(Operator {
                opcode: Some(OpCode::I32Xor as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Shl => Ok(Operator {
                opcode: Some(OpCode::I32Shl as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32ShrS => Ok(Operator {
                opcode: Some(OpCode::I32ShrS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32ShrU => Ok(Operator {
                opcode: Some(OpCode::I32ShrU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Rotl => Ok(Operator {
                opcode: Some(OpCode::I32Rotl as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Rotr => Ok(Operator {
                opcode: Some(OpCode::I32Rotr as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Clz => Ok(Operator {
                opcode: Some(OpCode::I64Clz as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Ctz => Ok(Operator {
                opcode: Some(OpCode::I64Ctz as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Popcnt => Ok(Operator {
                opcode: Some(OpCode::I64Popcnt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Add => Ok(Operator {
                opcode: Some(OpCode::I64Add as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Sub => Ok(Operator {
                opcode: Some(OpCode::I64Sub as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Mul => Ok(Operator {
                opcode: Some(OpCode::I64Mul as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64DivS => Ok(Operator {
                opcode: Some(OpCode::I64DivS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64DivU => Ok(Operator {
                opcode: Some(OpCode::I64DivU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64RemS => Ok(Operator {
                opcode: Some(OpCode::I64RemS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64RemU => Ok(Operator {
                opcode: Some(OpCode::I64RemU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64And => Ok(Operator {
                opcode: Some(OpCode::I64And as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Or => Ok(Operator {
                opcode: Some(OpCode::I64Or as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Xor => Ok(Operator {
                opcode: Some(OpCode::I64Xor as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Shl => Ok(Operator {
                opcode: Some(OpCode::I64Shl as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64ShrS => Ok(Operator {
                opcode: Some(OpCode::I64ShrS as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64ShrU => Ok(Operator {
                opcode: Some(OpCode::I64ShrU as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Rotl => Ok(Operator {
                opcode: Some(OpCode::I64Rotl as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Rotr => Ok(Operator {
                opcode: Some(OpCode::I64Rotr as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Abs => Ok(Operator {
                opcode: Some(OpCode::F32Abs as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Neg => Ok(Operator {
                opcode: Some(OpCode::F32Neg as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Ceil => Ok(Operator {
                opcode: Some(OpCode::F32Ceil as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Floor => Ok(Operator {
                opcode: Some(OpCode::F32Floor as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Trunc => Ok(Operator {
                opcode: Some(OpCode::F32Trunc as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Nearest => Ok(Operator {
                opcode: Some(OpCode::F32Nearest as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Sqrt => Ok(Operator {
                opcode: Some(OpCode::F32Sqrt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Add => Ok(Operator {
                opcode: Some(OpCode::F32Add as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Sub => Ok(Operator {
                opcode: Some(OpCode::F32Sub as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Mul => Ok(Operator {
                opcode: Some(OpCode::F32Mul as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Div => Ok(Operator {
                opcode: Some(OpCode::F32Div as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Min => Ok(Operator {
                opcode: Some(OpCode::F32Min as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Max => Ok(Operator {
                opcode: Some(OpCode::F32Max as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32Copysign => Ok(Operator {
                opcode: Some(OpCode::F32Copysign as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Abs => Ok(Operator {
                opcode: Some(OpCode::F64Abs as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Neg => Ok(Operator {
                opcode: Some(OpCode::F64Neg as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Ceil => Ok(Operator {
                opcode: Some(OpCode::F64Ceil as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Floor => Ok(Operator {
                opcode: Some(OpCode::F64Floor as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Trunc => Ok(Operator {
                opcode: Some(OpCode::F64Trunc as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Nearest => Ok(Operator {
                opcode: Some(OpCode::F64Nearest as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Sqrt => Ok(Operator {
                opcode: Some(OpCode::F64Sqrt as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Add => Ok(Operator {
                opcode: Some(OpCode::F64Add as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Sub => Ok(Operator {
                opcode: Some(OpCode::F64Sub as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Mul => Ok(Operator {
                opcode: Some(OpCode::F64Mul as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Div => Ok(Operator {
                opcode: Some(OpCode::F64Div as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Min => Ok(Operator {
                opcode: Some(OpCode::F64Min as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Max => Ok(Operator {
                opcode: Some(OpCode::F64Max as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64Copysign => Ok(Operator {
                opcode: Some(OpCode::F64Copysign as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32WrapI64 => Ok(Operator {
                opcode: Some(OpCode::I32WrapI64 as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32TruncF32S => Ok(Operator {
                opcode: Some(OpCode::I32TruncF32s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32TruncF32U => Ok(Operator {
                opcode: Some(OpCode::I32TruncF32u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32TruncF64S => Ok(Operator {
                opcode: Some(OpCode::I32TruncF64s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32TruncF64U => Ok(Operator {
                opcode: Some(OpCode::I32TruncF64u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64ExtendI32S => Ok(Operator {
                opcode: Some(OpCode::I64ExtendI32s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64ExtendI32U => Ok(Operator {
                opcode: Some(OpCode::I64ExtendI32u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncF32S => Ok(Operator {
                opcode: Some(OpCode::I64TruncF32s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncF32U => Ok(Operator {
                opcode: Some(OpCode::I64TruncF32u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncF64S => Ok(Operator {
                opcode: Some(OpCode::I64TruncF64s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncF64U => Ok(Operator {
                opcode: Some(OpCode::I64TruncF64u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32ConvertI32S => Ok(Operator {
                opcode: Some(OpCode::F32ConvertI32s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32ConvertI32U => Ok(Operator {
                opcode: Some(OpCode::F32ConvertI32u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32ConvertI64S => Ok(Operator {
                opcode: Some(OpCode::F32ConvertI64s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32ConvertI64U => Ok(Operator {
                opcode: Some(OpCode::F32ConvertI64u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32DemoteF64 => Ok(Operator {
                opcode: Some(OpCode::F32DemoteF64 as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64ConvertI32S => Ok(Operator {
                opcode: Some(OpCode::F64ConvertI32s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64ConvertI32U => Ok(Operator {
                opcode: Some(OpCode::F64ConvertI32u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64ConvertI64S => Ok(Operator {
                opcode: Some(OpCode::F64ConvertI64s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64ConvertI64U => Ok(Operator {
                opcode: Some(OpCode::F64ConvertI64u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64PromoteF32 => Ok(Operator {
                opcode: Some(OpCode::F64PromoteF32 as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32ReinterpretF32 => Ok(Operator {
                opcode: Some(OpCode::I32ReinterpretF32 as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64ReinterpretF64 => Ok(Operator {
                opcode: Some(OpCode::I64ReinterpretF64 as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F32ReinterpretI32 => Ok(Operator {
                opcode: Some(OpCode::F32ReinterpretI32 as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::F64ReinterpretI64 => Ok(Operator {
                opcode: Some(OpCode::F64ReinterpretI64 as i32),
                ..Operator::default()
            }),

            // @sign_extension
            wasmparser::Operator::I32Extend8S => Ok(Operator {
                opcode: Some(OpCode::I32Extend8S as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32Extend16S => Ok(Operator {
                opcode: Some(OpCode::I32Extend16S as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Extend8S => Ok(Operator {
                opcode: Some(OpCode::I64Extend8S as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Extend16S => Ok(Operator {
                opcode: Some(OpCode::I64Extend16S as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64Extend32S => Ok(Operator {
                opcode: Some(OpCode::I64Extend32S as i32),
                ..Operator::default()
            }),

            // @saturating_float_to_int
            wasmparser::Operator::I32TruncSatF32S => Ok(Operator {
                opcode: Some(OpCode::I32TruncSatF32s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32TruncSatF32U => Ok(Operator {
                opcode: Some(OpCode::I32TruncSatF32u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32TruncSatF64S => Ok(Operator {
                opcode: Some(OpCode::I32TruncSatF64s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I32TruncSatF64U => Ok(Operator {
                opcode: Some(OpCode::I32TruncSatF64u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncSatF32S => Ok(Operator {
                opcode: Some(OpCode::I64TruncSatF32s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncSatF32U => Ok(Operator {
                opcode: Some(OpCode::I64TruncSatF32u as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncSatF64S => Ok(Operator {
                opcode: Some(OpCode::I64TruncSatF64s as i32),
                ..Operator::default()
            }),
            wasmparser::Operator::I64TruncSatF64U => Ok(Operator {
                opcode: Some(OpCode::I64TruncSatF64u as i32),
                ..Operator::default()
            }),

            // @bulk_memory
            wasmparser::Operator::MemoryInit { data_index, mem } => Ok(Operator {
                opcode: Some(OpCode::MemoryInit as i32),
                operator: Some(operator::Operator::MemoryInit(MemoryInitOp {
                    data_index: Some(data_index),
                    mem: Some(mem),
                })),
            }),
            wasmparser::Operator::DataDrop { data_index } => Ok(Operator {
                opcode: Some(OpCode::DataDrop as i32),
                operator: Some(operator::Operator::DataIndex(data_index)),
            }),
            wasmparser::Operator::MemoryCopy { dst_mem, src_mem } => Ok(Operator {
                opcode: Some(OpCode::MemoryCopy as i32),
                operator: Some(operator::Operator::MemoryCopy(MemoryCopyOp {
                    dst_mem: Some(dst_mem),
                    src_mem: Some(src_mem),
                })),
            }),
            wasmparser::Operator::MemoryFill { mem } => Ok(Operator {
                opcode: Some(OpCode::MemoryFill as i32),
                operator: Some(operator::Operator::Mem(mem)),
            }),
            wasmparser::Operator::TableInit { elem_index, table } => Ok(Operator {
                opcode: Some(OpCode::TableInit as i32),
                operator: Some(operator::Operator::TableInit(TableInitOp {
                    elem_index: Some(elem_index),
                    table: Some(table),
                })),
            }),
            wasmparser::Operator::ElemDrop { elem_index } => Ok(Operator {
                opcode: Some(OpCode::ElemDrop as i32),
                operator: Some(operator::Operator::ElemIndex(elem_index)),
            }),
            wasmparser::Operator::TableCopy {
                dst_table,
                src_table,
            } => Ok(Operator {
                opcode: Some(OpCode::TableCopy as i32),
                operator: Some(operator::Operator::TableCopy(TableCopyOp {
                    dst_table: Some(dst_table),
                    src_table: Some(src_table),
                })),
            }),

            _ => Err(anyhow!("Got unsupported operator: {:?}", operator)),
        }
    }
}

impl TryFrom<Operator> for wasm_encoder::Instruction<'_> {
    type Error = anyhow::Error;

    fn try_from(operator: Operator) -> Result<Self> {
        let opcode = operator.opcode.ok_or(anyhow!("Opcode not found"))?;
        let opcode = OpCode::try_from(opcode)?;
        match opcode {
            OpCode::Unreachable => Ok(wasm_encoder::Instruction::Unreachable),
            OpCode::Nop => Ok(wasm_encoder::Instruction::Nop),
            OpCode::Block => {
                let blockty = match operator
                    .operator
                    .ok_or(anyhow!("Block operator not found"))?
                {
                    operator::Operator::Blockty(bt) => wasm_encoder::BlockType::try_from(bt)?,
                    _ => return Err(anyhow!("Expected BlockType for Block operator")),
                };
                Ok(wasm_encoder::Instruction::Block(blockty))
            }
            OpCode::Loop => {
                let blockty = match operator
                    .operator
                    .ok_or(anyhow!("Loop operator not found"))?
                {
                    operator::Operator::Blockty(bt) => wasm_encoder::BlockType::try_from(bt)?,
                    _ => return Err(anyhow!("Expected BlockType for Loop operator")),
                };
                Ok(wasm_encoder::Instruction::Loop(blockty))
            }
            OpCode::If => {
                let blockty = match operator.operator.ok_or(anyhow!("If operator not found"))? {
                    operator::Operator::Blockty(bt) => wasm_encoder::BlockType::try_from(bt)?,
                    _ => return Err(anyhow!("Expected BlockType for If operator")),
                };
                Ok(wasm_encoder::Instruction::If(blockty))
            }
            OpCode::Else => Ok(wasm_encoder::Instruction::Else),
            OpCode::End => Ok(wasm_encoder::Instruction::End),
            OpCode::Br => {
                let relative_depth =
                    match operator.operator.ok_or(anyhow!("Br operator not found"))? {
                        operator::Operator::RelativeDepth(depth) => depth,
                        _ => return Err(anyhow!("Expected RelativeDepth for Br operator")),
                    };
                Ok(wasm_encoder::Instruction::Br(relative_depth))
            }
            OpCode::BrIf => {
                let relative_depth = match operator
                    .operator
                    .ok_or(anyhow!("BrIf operator not found"))?
                {
                    operator::Operator::RelativeDepth(depth) => depth,
                    _ => return Err(anyhow!("Expected RelativeDepth for BrIf operator")),
                };
                Ok(wasm_encoder::Instruction::BrIf(relative_depth))
            }
            OpCode::BrTable => {
                let (targets_vec, default) = match operator
                    .operator
                    .ok_or(anyhow!("BrTable operator not found"))?
                {
                    operator::Operator::Targets(targets) => {
                        let default = targets
                            .default
                            .ok_or(anyhow!("BrTable default target not found"))?;
                        (targets.targets, default)
                    }
                    _ => return Err(anyhow!("Expected Targets for BrTable operator")),
                };
                Ok(wasm_encoder::Instruction::BrTable(
                    targets_vec.into(),
                    default,
                ))
            }
            OpCode::Return => Ok(wasm_encoder::Instruction::Return),
            OpCode::Call => {
                let function_index = match operator
                    .operator
                    .ok_or(anyhow!("Call operator not found"))?
                {
                    operator::Operator::FunctionIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected FunctionIndex for Call operator")),
                };
                Ok(wasm_encoder::Instruction::Call(function_index))
            }
            OpCode::CallIndirect => {
                let (type_index, table_index) = match operator
                    .operator
                    .ok_or(anyhow!("CallIndirect operator not found"))?
                {
                    operator::Operator::CallInderect(ci) => {
                        let type_index = ci
                            .type_index
                            .ok_or(anyhow!("CallIndirect type_index not found"))?;
                        let table_index = ci
                            .table_index
                            .ok_or(anyhow!("CallIndirect table_index not found"))?;
                        (type_index, table_index)
                    }
                    _ => return Err(anyhow!("Expected CallIndirectOp for CallIndirect operator")),
                };
                Ok(wasm_encoder::Instruction::CallIndirect {
                    type_index,
                    table_index,
                })
            }
            OpCode::Drop => Ok(wasm_encoder::Instruction::Drop),
            OpCode::Select => Ok(wasm_encoder::Instruction::Select),
            OpCode::LocalGet => {
                let local_index = match operator
                    .operator
                    .ok_or(anyhow!("LocalGet operator not found"))?
                {
                    operator::Operator::LocalIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected LocalIndex for LocalGet operator")),
                };
                Ok(wasm_encoder::Instruction::LocalGet(local_index))
            }
            OpCode::LocalSet => {
                let local_index = match operator
                    .operator
                    .ok_or(anyhow!("LocalSet operator not found"))?
                {
                    operator::Operator::LocalIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected LocalIndex for LocalSet operator")),
                };
                Ok(wasm_encoder::Instruction::LocalSet(local_index))
            }
            OpCode::LocalTee => {
                let local_index = match operator
                    .operator
                    .ok_or(anyhow!("LocalTee operator not found"))?
                {
                    operator::Operator::LocalIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected LocalIndex for LocalTee operator")),
                };
                Ok(wasm_encoder::Instruction::LocalTee(local_index))
            }
            OpCode::GlobalGet => {
                let global_index = match operator
                    .operator
                    .ok_or(anyhow!("GlobalGet operator not found"))?
                {
                    operator::Operator::GlobalIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected GlobalIndex for GlobalGet operator")),
                };
                Ok(wasm_encoder::Instruction::GlobalGet(global_index))
            }
            OpCode::GlobalSet => {
                let global_index = match operator
                    .operator
                    .ok_or(anyhow!("GlobalSet operator not found"))?
                {
                    operator::Operator::GlobalIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected GlobalIndex for GlobalSet operator")),
                };
                Ok(wasm_encoder::Instruction::GlobalSet(global_index))
            }
            OpCode::I32Load => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Load operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Load operator")),
                };
                Ok(wasm_encoder::Instruction::I32Load(memarg))
            }
            OpCode::I64Load => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Load operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Load operator")),
                };
                Ok(wasm_encoder::Instruction::I64Load(memarg))
            }
            OpCode::F32Load => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("F32Load operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for F32Load operator")),
                };
                Ok(wasm_encoder::Instruction::F32Load(memarg))
            }
            OpCode::F64Load => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("F64Load operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for F64Load operator")),
                };
                Ok(wasm_encoder::Instruction::F64Load(memarg))
            }
            OpCode::I32Load8S => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Load8S operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Load8S operator")),
                };
                Ok(wasm_encoder::Instruction::I32Load8S(memarg))
            }
            OpCode::I32Load8U => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Load8U operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Load8U operator")),
                };
                Ok(wasm_encoder::Instruction::I32Load8U(memarg))
            }
            OpCode::I32Load16S => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Load16S operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Load16S operator")),
                };
                Ok(wasm_encoder::Instruction::I32Load16S(memarg))
            }
            OpCode::I32Load16U => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Load16U operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Load16U operator")),
                };
                Ok(wasm_encoder::Instruction::I32Load16U(memarg))
            }
            OpCode::I64Load8S => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Load8S operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Load8S operator")),
                };
                Ok(wasm_encoder::Instruction::I64Load8S(memarg))
            }
            OpCode::I64Load8U => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Load8U operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Load8U operator")),
                };
                Ok(wasm_encoder::Instruction::I64Load8U(memarg))
            }
            OpCode::I64Load16S => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Load16S operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Load16S operator")),
                };
                Ok(wasm_encoder::Instruction::I64Load16S(memarg))
            }
            OpCode::I64Load16U => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Load16U operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Load16U operator")),
                };
                Ok(wasm_encoder::Instruction::I64Load16U(memarg))
            }
            OpCode::I64Load32S => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Load32S operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Load32S operator")),
                };
                Ok(wasm_encoder::Instruction::I64Load32S(memarg))
            }
            OpCode::I64Load32U => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Load32U operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Load32U operator")),
                };
                Ok(wasm_encoder::Instruction::I64Load32U(memarg))
            }
            OpCode::I32Store => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Store operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Store operator")),
                };
                Ok(wasm_encoder::Instruction::I32Store(memarg))
            }
            OpCode::I64Store => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Store operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Store operator")),
                };
                Ok(wasm_encoder::Instruction::I64Store(memarg))
            }
            OpCode::F32Store => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("F32Store operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for F32Store operator")),
                };
                Ok(wasm_encoder::Instruction::F32Store(memarg))
            }
            OpCode::F64Store => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("F64Store operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for F64Store operator")),
                };
                Ok(wasm_encoder::Instruction::F64Store(memarg))
            }
            OpCode::I32Store8 => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Store8 operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Store8 operator")),
                };
                Ok(wasm_encoder::Instruction::I32Store8(memarg))
            }
            OpCode::I32Store16 => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I32Store16 operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I32Store16 operator")),
                };
                Ok(wasm_encoder::Instruction::I32Store16(memarg))
            }
            OpCode::I64Store8 => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Store8 operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Store8 operator")),
                };
                Ok(wasm_encoder::Instruction::I64Store8(memarg))
            }
            OpCode::I64Store16 => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Store16 operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Store16 operator")),
                };
                Ok(wasm_encoder::Instruction::I64Store16(memarg))
            }
            OpCode::I64Store32 => {
                let memarg = match operator
                    .operator
                    .ok_or(anyhow!("I64Store32 operator not found"))?
                {
                    operator::Operator::Memarg(ma) => wasm_encoder::MemArg::try_from(ma)?,
                    _ => return Err(anyhow!("Expected MemArg for I64Store32 operator")),
                };
                Ok(wasm_encoder::Instruction::I64Store32(memarg))
            }
            OpCode::MemorySize => {
                let mem = match operator
                    .operator
                    .ok_or(anyhow!("MemorySize operator not found"))?
                {
                    operator::Operator::Mem(m) => m,
                    _ => return Err(anyhow!("Expected Mem for MemorySize operator")),
                };
                Ok(wasm_encoder::Instruction::MemorySize(mem))
            }
            OpCode::MemoryGrow => {
                let mem = match operator
                    .operator
                    .ok_or(anyhow!("MemoryGrow operator not found"))?
                {
                    operator::Operator::Mem(m) => m,
                    _ => return Err(anyhow!("Expected Mem for MemoryGrow operator")),
                };
                Ok(wasm_encoder::Instruction::MemoryGrow(mem))
            }
            OpCode::I32Const => {
                let value = match operator
                    .operator
                    .ok_or(anyhow!("I32Const operator not found"))?
                {
                    operator::Operator::I32value(v) => v,
                    _ => return Err(anyhow!("Expected I32value for I32Const operator")),
                };
                Ok(wasm_encoder::Instruction::I32Const(value))
            }
            OpCode::I64Const => {
                let value = match operator
                    .operator
                    .ok_or(anyhow!("I64Const operator not found"))?
                {
                    operator::Operator::I64value(v) => v,
                    _ => return Err(anyhow!("Expected I64value for I64Const operator")),
                };
                Ok(wasm_encoder::Instruction::I64Const(value))
            }
            OpCode::F32Const => {
                let value = match operator
                    .operator
                    .ok_or(anyhow!("F32Const operator not found"))?
                {
                    operator::Operator::F32value(bits) => wasm_encoder::Ieee32::new(bits),
                    _ => return Err(anyhow!("Expected F32value for F32Const operator")),
                };
                Ok(wasm_encoder::Instruction::F32Const(value))
            }
            OpCode::F64Const => {
                let value = match operator
                    .operator
                    .ok_or(anyhow!("F64Const operator not found"))?
                {
                    operator::Operator::F64value(bits) => wasm_encoder::Ieee64::new(bits),
                    _ => return Err(anyhow!("Expected F64value for F64Const operator")),
                };
                Ok(wasm_encoder::Instruction::F64Const(value))
            }
            OpCode::I32Eqz => Ok(wasm_encoder::Instruction::I32Eqz),
            OpCode::I32Eq => Ok(wasm_encoder::Instruction::I32Eq),
            OpCode::I32Ne => Ok(wasm_encoder::Instruction::I32Ne),
            OpCode::I32LtS => Ok(wasm_encoder::Instruction::I32LtS),
            OpCode::I32LtU => Ok(wasm_encoder::Instruction::I32LtU),
            OpCode::I32GtS => Ok(wasm_encoder::Instruction::I32GtS),
            OpCode::I32GtU => Ok(wasm_encoder::Instruction::I32GtU),
            OpCode::I32LeS => Ok(wasm_encoder::Instruction::I32LeS),
            OpCode::I32LeU => Ok(wasm_encoder::Instruction::I32LeU),
            OpCode::I32GeS => Ok(wasm_encoder::Instruction::I32GeS),
            OpCode::I32GeU => Ok(wasm_encoder::Instruction::I32GeU),
            OpCode::I64Eqz => Ok(wasm_encoder::Instruction::I64Eqz),
            OpCode::I64Eq => Ok(wasm_encoder::Instruction::I64Eq),
            OpCode::I64Ne => Ok(wasm_encoder::Instruction::I64Ne),
            OpCode::I64LtS => Ok(wasm_encoder::Instruction::I64LtS),
            OpCode::I64LtU => Ok(wasm_encoder::Instruction::I64LtU),
            OpCode::I64GtS => Ok(wasm_encoder::Instruction::I64GtS),
            OpCode::I64GtU => Ok(wasm_encoder::Instruction::I64GtU),
            OpCode::I64LeS => Ok(wasm_encoder::Instruction::I64LeS),
            OpCode::I64LeU => Ok(wasm_encoder::Instruction::I64LeU),
            OpCode::I64GeS => Ok(wasm_encoder::Instruction::I64GeS),
            OpCode::I64GeU => Ok(wasm_encoder::Instruction::I64GeU),
            OpCode::F32Eq => Ok(wasm_encoder::Instruction::F32Eq),
            OpCode::F32Ne => Ok(wasm_encoder::Instruction::F32Ne),
            OpCode::F32Lt => Ok(wasm_encoder::Instruction::F32Lt),
            OpCode::F32Gt => Ok(wasm_encoder::Instruction::F32Gt),
            OpCode::F32Le => Ok(wasm_encoder::Instruction::F32Le),
            OpCode::F32Ge => Ok(wasm_encoder::Instruction::F32Ge),
            OpCode::F64Eq => Ok(wasm_encoder::Instruction::F64Eq),
            OpCode::F64Ne => Ok(wasm_encoder::Instruction::F64Ne),
            OpCode::F64Lt => Ok(wasm_encoder::Instruction::F64Lt),
            OpCode::F64Gt => Ok(wasm_encoder::Instruction::F64Gt),
            OpCode::F64Le => Ok(wasm_encoder::Instruction::F64Le),
            OpCode::F64Ge => Ok(wasm_encoder::Instruction::F64Ge),
            OpCode::I32Clz => Ok(wasm_encoder::Instruction::I32Clz),
            OpCode::I32Ctz => Ok(wasm_encoder::Instruction::I32Ctz),
            OpCode::I32Popcnt => Ok(wasm_encoder::Instruction::I32Popcnt),
            OpCode::I32Add => Ok(wasm_encoder::Instruction::I32Add),
            OpCode::I32Sub => Ok(wasm_encoder::Instruction::I32Sub),
            OpCode::I32Mul => Ok(wasm_encoder::Instruction::I32Mul),
            OpCode::I32DivS => Ok(wasm_encoder::Instruction::I32DivS),
            OpCode::I32DivU => Ok(wasm_encoder::Instruction::I32DivU),
            OpCode::I32RemS => Ok(wasm_encoder::Instruction::I32RemS),
            OpCode::I32RemU => Ok(wasm_encoder::Instruction::I32RemU),
            OpCode::I32And => Ok(wasm_encoder::Instruction::I32And),
            OpCode::I32Or => Ok(wasm_encoder::Instruction::I32Or),
            OpCode::I32Xor => Ok(wasm_encoder::Instruction::I32Xor),
            OpCode::I32Shl => Ok(wasm_encoder::Instruction::I32Shl),
            OpCode::I32ShrS => Ok(wasm_encoder::Instruction::I32ShrS),
            OpCode::I32ShrU => Ok(wasm_encoder::Instruction::I32ShrU),
            OpCode::I32Rotl => Ok(wasm_encoder::Instruction::I32Rotl),
            OpCode::I32Rotr => Ok(wasm_encoder::Instruction::I32Rotr),
            OpCode::I64Clz => Ok(wasm_encoder::Instruction::I64Clz),
            OpCode::I64Ctz => Ok(wasm_encoder::Instruction::I64Ctz),
            OpCode::I64Popcnt => Ok(wasm_encoder::Instruction::I64Popcnt),
            OpCode::I64Add => Ok(wasm_encoder::Instruction::I64Add),
            OpCode::I64Sub => Ok(wasm_encoder::Instruction::I64Sub),
            OpCode::I64Mul => Ok(wasm_encoder::Instruction::I64Mul),
            OpCode::I64DivS => Ok(wasm_encoder::Instruction::I64DivS),
            OpCode::I64DivU => Ok(wasm_encoder::Instruction::I64DivU),
            OpCode::I64RemS => Ok(wasm_encoder::Instruction::I64RemS),
            OpCode::I64RemU => Ok(wasm_encoder::Instruction::I64RemU),
            OpCode::I64And => Ok(wasm_encoder::Instruction::I64And),
            OpCode::I64Or => Ok(wasm_encoder::Instruction::I64Or),
            OpCode::I64Xor => Ok(wasm_encoder::Instruction::I64Xor),
            OpCode::I64Shl => Ok(wasm_encoder::Instruction::I64Shl),
            OpCode::I64ShrS => Ok(wasm_encoder::Instruction::I64ShrS),
            OpCode::I64ShrU => Ok(wasm_encoder::Instruction::I64ShrU),
            OpCode::I64Rotl => Ok(wasm_encoder::Instruction::I64Rotl),
            OpCode::I64Rotr => Ok(wasm_encoder::Instruction::I64Rotr),
            OpCode::F32Abs => Ok(wasm_encoder::Instruction::F32Abs),
            OpCode::F32Neg => Ok(wasm_encoder::Instruction::F32Neg),
            OpCode::F32Ceil => Ok(wasm_encoder::Instruction::F32Ceil),
            OpCode::F32Floor => Ok(wasm_encoder::Instruction::F32Floor),
            OpCode::F32Trunc => Ok(wasm_encoder::Instruction::F32Trunc),
            OpCode::F32Nearest => Ok(wasm_encoder::Instruction::F32Nearest),
            OpCode::F32Sqrt => Ok(wasm_encoder::Instruction::F32Sqrt),
            OpCode::F32Add => Ok(wasm_encoder::Instruction::F32Add),
            OpCode::F32Sub => Ok(wasm_encoder::Instruction::F32Sub),
            OpCode::F32Mul => Ok(wasm_encoder::Instruction::F32Mul),
            OpCode::F32Div => Ok(wasm_encoder::Instruction::F32Div),
            OpCode::F32Min => Ok(wasm_encoder::Instruction::F32Min),
            OpCode::F32Max => Ok(wasm_encoder::Instruction::F32Max),
            OpCode::F32Copysign => Ok(wasm_encoder::Instruction::F32Copysign),
            OpCode::F64Abs => Ok(wasm_encoder::Instruction::F64Abs),
            OpCode::F64Neg => Ok(wasm_encoder::Instruction::F64Neg),
            OpCode::F64Ceil => Ok(wasm_encoder::Instruction::F64Ceil),
            OpCode::F64Floor => Ok(wasm_encoder::Instruction::F64Floor),
            OpCode::F64Trunc => Ok(wasm_encoder::Instruction::F64Trunc),
            OpCode::F64Nearest => Ok(wasm_encoder::Instruction::F64Nearest),
            OpCode::F64Sqrt => Ok(wasm_encoder::Instruction::F64Sqrt),
            OpCode::F64Add => Ok(wasm_encoder::Instruction::F64Add),
            OpCode::F64Sub => Ok(wasm_encoder::Instruction::F64Sub),
            OpCode::F64Mul => Ok(wasm_encoder::Instruction::F64Mul),
            OpCode::F64Div => Ok(wasm_encoder::Instruction::F64Div),
            OpCode::F64Min => Ok(wasm_encoder::Instruction::F64Min),
            OpCode::F64Max => Ok(wasm_encoder::Instruction::F64Max),
            OpCode::F64Copysign => Ok(wasm_encoder::Instruction::F64Copysign),
            OpCode::I32WrapI64 => Ok(wasm_encoder::Instruction::I32WrapI64),
            OpCode::I32TruncF32s => Ok(wasm_encoder::Instruction::I32TruncF32S),
            OpCode::I32TruncF32u => Ok(wasm_encoder::Instruction::I32TruncF32U),
            OpCode::I32TruncF64s => Ok(wasm_encoder::Instruction::I32TruncF64S),
            OpCode::I32TruncF64u => Ok(wasm_encoder::Instruction::I32TruncF64U),
            OpCode::I64ExtendI32s => Ok(wasm_encoder::Instruction::I64ExtendI32S),
            OpCode::I64ExtendI32u => Ok(wasm_encoder::Instruction::I64ExtendI32U),
            OpCode::I64TruncF32s => Ok(wasm_encoder::Instruction::I64TruncF32S),
            OpCode::I64TruncF32u => Ok(wasm_encoder::Instruction::I64TruncF32U),
            OpCode::I64TruncF64s => Ok(wasm_encoder::Instruction::I64TruncF64S),
            OpCode::I64TruncF64u => Ok(wasm_encoder::Instruction::I64TruncF64U),
            OpCode::F32ConvertI32s => Ok(wasm_encoder::Instruction::F32ConvertI32S),
            OpCode::F32ConvertI32u => Ok(wasm_encoder::Instruction::F32ConvertI32U),
            OpCode::F32ConvertI64s => Ok(wasm_encoder::Instruction::F32ConvertI64S),
            OpCode::F32ConvertI64u => Ok(wasm_encoder::Instruction::F32ConvertI64U),
            OpCode::F32DemoteF64 => Ok(wasm_encoder::Instruction::F32DemoteF64),
            OpCode::F64ConvertI32s => Ok(wasm_encoder::Instruction::F64ConvertI32S),
            OpCode::F64ConvertI32u => Ok(wasm_encoder::Instruction::F64ConvertI32U),
            OpCode::F64ConvertI64s => Ok(wasm_encoder::Instruction::F64ConvertI64S),
            OpCode::F64ConvertI64u => Ok(wasm_encoder::Instruction::F64ConvertI64U),
            OpCode::F64PromoteF32 => Ok(wasm_encoder::Instruction::F64PromoteF32),
            OpCode::I32ReinterpretF32 => Ok(wasm_encoder::Instruction::I32ReinterpretF32),
            OpCode::I64ReinterpretF64 => Ok(wasm_encoder::Instruction::I64ReinterpretF64),
            OpCode::F32ReinterpretI32 => Ok(wasm_encoder::Instruction::F32ReinterpretI32),
            OpCode::F64ReinterpretI64 => Ok(wasm_encoder::Instruction::F64ReinterpretI64),
            OpCode::I32Extend8S => Ok(wasm_encoder::Instruction::I32Extend8S),
            OpCode::I32Extend16S => Ok(wasm_encoder::Instruction::I32Extend16S),
            OpCode::I64Extend8S => Ok(wasm_encoder::Instruction::I64Extend8S),
            OpCode::I64Extend16S => Ok(wasm_encoder::Instruction::I64Extend16S),
            OpCode::I64Extend32S => Ok(wasm_encoder::Instruction::I64Extend32S),
            OpCode::I32TruncSatF32s => Ok(wasm_encoder::Instruction::I32TruncSatF32S),
            OpCode::I32TruncSatF32u => Ok(wasm_encoder::Instruction::I32TruncSatF32U),
            OpCode::I32TruncSatF64s => Ok(wasm_encoder::Instruction::I32TruncSatF64S),
            OpCode::I32TruncSatF64u => Ok(wasm_encoder::Instruction::I32TruncSatF64U),
            OpCode::I64TruncSatF32s => Ok(wasm_encoder::Instruction::I64TruncSatF32S),
            OpCode::I64TruncSatF32u => Ok(wasm_encoder::Instruction::I64TruncSatF32U),
            OpCode::I64TruncSatF64s => Ok(wasm_encoder::Instruction::I64TruncSatF64S),
            OpCode::I64TruncSatF64u => Ok(wasm_encoder::Instruction::I64TruncSatF64U),
            OpCode::MemoryInit => {
                let (data_index, mem) = match operator
                    .operator
                    .ok_or(anyhow!("MemoryInit operator not found"))?
                {
                    operator::Operator::MemoryInit(mi) => {
                        let data_index = mi
                            .data_index
                            .ok_or(anyhow!("MemoryInit data_index not found"))?;
                        let mem = mi.mem.ok_or(anyhow!("MemoryInit mem not found"))?;
                        (data_index, mem)
                    }
                    _ => return Err(anyhow!("Expected MemoryInitOp for MemoryInit operator")),
                };
                Ok(wasm_encoder::Instruction::MemoryInit { mem, data_index })
            }
            OpCode::DataDrop => {
                let data_index = match operator
                    .operator
                    .ok_or(anyhow!("DataDrop operator not found"))?
                {
                    operator::Operator::DataIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected DataIndex for DataDrop operator")),
                };
                Ok(wasm_encoder::Instruction::DataDrop(data_index))
            }
            OpCode::MemoryCopy => {
                let (dst_mem, src_mem) = match operator
                    .operator
                    .ok_or(anyhow!("MemoryCopy operator not found"))?
                {
                    operator::Operator::MemoryCopy(mc) => {
                        let dst_mem = mc.dst_mem.ok_or(anyhow!("MemoryCopy dst_mem not found"))?;
                        let src_mem = mc.src_mem.ok_or(anyhow!("MemoryCopy src_mem not found"))?;
                        (dst_mem, src_mem)
                    }
                    _ => return Err(anyhow!("Expected MemoryCopyOp for MemoryCopy operator")),
                };
                Ok(wasm_encoder::Instruction::MemoryCopy { dst_mem, src_mem })
            }
            OpCode::MemoryFill => {
                let mem = match operator
                    .operator
                    .ok_or(anyhow!("MemoryFill operator not found"))?
                {
                    operator::Operator::Mem(m) => m,
                    _ => return Err(anyhow!("Expected Mem for MemoryFill operator")),
                };
                Ok(wasm_encoder::Instruction::MemoryFill(mem))
            }
            OpCode::TableInit => {
                let (elem_index, table) = match operator
                    .operator
                    .ok_or(anyhow!("TableInit operator not found"))?
                {
                    operator::Operator::TableInit(ti) => {
                        let elem_index = ti
                            .elem_index
                            .ok_or(anyhow!("TableInit elem_index not found"))?;
                        let table = ti.table.ok_or(anyhow!("TableInit table not found"))?;
                        (elem_index, table)
                    }
                    _ => return Err(anyhow!("Expected TableInitOp for TableInit operator")),
                };
                Ok(wasm_encoder::Instruction::TableInit { table, elem_index })
            }
            OpCode::ElemDrop => {
                let elem_index = match operator
                    .operator
                    .ok_or(anyhow!("ElemDrop operator not found"))?
                {
                    operator::Operator::ElemIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected ElemIndex for ElemDrop operator")),
                };
                Ok(wasm_encoder::Instruction::ElemDrop(elem_index))
            }
            OpCode::TableCopy => {
                let (dst_table, src_table) = match operator
                    .operator
                    .ok_or(anyhow!("TableCopy operator not found"))?
                {
                    operator::Operator::TableCopy(tc) => {
                        let dst_table = tc
                            .dst_table
                            .ok_or(anyhow!("TableCopy dst_table not found"))?;
                        let src_table = tc
                            .src_table
                            .ok_or(anyhow!("TableCopy src_table not found"))?;
                        (dst_table, src_table)
                    }
                    _ => return Err(anyhow!("Expected TableCopyOp for TableCopy operator")),
                };
                Ok(wasm_encoder::Instruction::TableCopy {
                    dst_table,
                    src_table,
                })
            }
        }
    }
}
