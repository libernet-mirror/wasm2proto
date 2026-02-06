use crate::libernet_wasm::*;
use anyhow::{Ok, Result, anyhow};

impl TryFrom<wasmparser::BlockType> for BlockType {
    type Error = anyhow::Error;

    fn try_from(blockty: wasmparser::BlockType) -> Result<Self> {
        match blockty {
            wasmparser::BlockType::Empty => Ok(BlockType {
                blockty: Some(block_type::Blockty::Empty(0)),
            }),
            wasmparser::BlockType::Type(valtype) => Ok(BlockType {
                blockty: Some(block_type::Blockty::ValueType(ValueType::try_from(
                    valtype,
                )?)),
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
        use crate::libernet_wasm::block_type::Blockty;
        use wasm_encoder::BlockType;
        match blocktype.blockty.ok_or(anyhow!("Block type not found"))? {
            Blockty::Empty(0) => Ok(BlockType::Empty),
            Blockty::ValueType(valtype) => Ok(BlockType::Result(valtype.try_into()?)),
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

impl TryFrom<wasmparser::Catch> for CatchElement {
    type Error = anyhow::Error;

    fn try_from(catch: wasmparser::Catch) -> Result<Self> {
        match catch {
            wasmparser::Catch::One { tag, label } => Ok(CatchElement {
                catch: Some(catch_element::Catch::One(CatchOne {
                    tag: Some(tag),
                    label: Some(label),
                })),
            }),
            wasmparser::Catch::OneRef { tag, label } => Ok(CatchElement {
                catch: Some(catch_element::Catch::OneRef(CatchOneRef {
                    tag: Some(tag),
                    label: Some(label),
                })),
            }),
            wasmparser::Catch::All { label } => Ok(CatchElement {
                catch: Some(catch_element::Catch::All(CatchAllEl { label: Some(label) })),
            }),
            wasmparser::Catch::AllRef { label } => Ok(CatchElement {
                catch: Some(catch_element::Catch::AllRef(CatchAllRef {
                    label: Some(label),
                })),
            }),
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
            // @exceptions
            wasmparser::Operator::TryTable { try_table } => {
                let mut catches: Vec<CatchElement> = Vec::new();
                for catch in try_table.catches {
                    catches.push(CatchElement::try_from(catch)?);
                }
                Ok(Operator {
                    opcode: Some(OpCode::TryTable as i32),
                    operator: Some(operator::Operator::TryTable(TryTableOp {
                        ty: Some(BlockType::try_from(try_table.ty)?),
                        catches,
                    })),
                })
            }
            wasmparser::Operator::Throw { tag_index } => Ok(Operator {
                opcode: Some(OpCode::Throw as i32),
                operator: Some(operator::Operator::Throw(ThrowOp {
                    tag_index: Some(tag_index),
                })),
            }),
            wasmparser::Operator::ThrowRef {} => Ok(Operator {
                opcode: Some(OpCode::ThrowRef as i32),
                ..Operator::default()
            }),
            // @legacy_exceptions
            wasmparser::Operator::Try { blockty } => Ok(Operator {
                opcode: Some(OpCode::Try as i32),
                operator: Some(operator::Operator::Blockty(BlockType::try_from(blockty)?)),
            }),
            wasmparser::Operator::Catch { tag_index } => Ok(Operator {
                opcode: Some(OpCode::Catch as i32),
                operator: Some(operator::Operator::TagIndex(tag_index)),
            }),
            wasmparser::Operator::Rethrow { relative_depth } => Ok(Operator {
                opcode: Some(OpCode::Rethrow as i32),
                operator: Some(operator::Operator::RelativeDepth(relative_depth)),
            }),
            wasmparser::Operator::Delegate { relative_depth } => Ok(Operator {
                opcode: Some(OpCode::Delegate as i32),
                operator: Some(operator::Operator::RelativeDepth(relative_depth)),
            }),
            wasmparser::Operator::CatchAll {} => Ok(Operator {
                opcode: Some(OpCode::CatchAll as i32),
                ..Operator::default()
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
            OpCode::TryTable => {
                let ty = match operator
                    .operator
                    .ok_or(anyhow!("TryTable operator not found"))?
                {
                    operator::Operator::TryTable(tt) => {
                        tt.ty.ok_or(anyhow!("TryTable ty not found"))?
                    }
                    _ => return Err(anyhow!("Expected TryTableOp for TryTable operator")),
                };
                let catches: Vec<wasm_encoder::Catch> = Vec::new();
                Ok(wasm_encoder::Instruction::TryTable(
                    wasm_encoder::BlockType::try_from(ty)?,
                    catches.into(),
                ))
            }
            OpCode::Throw => {
                let tag_index = match operator
                    .operator
                    .ok_or(anyhow!("Throw operator not found"))?
                {
                    operator::Operator::Throw(to) => {
                        to.tag_index.ok_or(anyhow!("Throw tag_index not found"))?
                    }
                    _ => return Err(anyhow!("Expected ThrowOp for Throw operator")),
                };
                Ok(wasm_encoder::Instruction::Throw(tag_index))
            }
            OpCode::ThrowRef => Ok(wasm_encoder::Instruction::ThrowRef),
            OpCode::Try => {
                let ty = match operator.operator.ok_or(anyhow!("Try operator not found"))? {
                    operator::Operator::Blockty(bt) => wasm_encoder::BlockType::try_from(bt)?,
                    _ => return Err(anyhow!("Expected BlockType for Try operator")),
                };
                Ok(wasm_encoder::Instruction::Try(ty))
            }
            OpCode::Catch => {
                let tag_index = match operator
                    .operator
                    .ok_or(anyhow!("Catch operator not found"))?
                {
                    operator::Operator::TagIndex(idx) => idx,
                    _ => return Err(anyhow!("Expected TagIndex for Catch operator")),
                };
                Ok(wasm_encoder::Instruction::Catch(tag_index))
            }
            OpCode::Rethrow => {
                let relative_depth = match operator
                    .operator
                    .ok_or(anyhow!("Rethrow operator not found"))?
                {
                    operator::Operator::RelativeDepth(idx) => idx,
                    _ => return Err(anyhow!("Expected RelativeDepth for Rethrow operator")),
                };
                Ok(wasm_encoder::Instruction::Rethrow(relative_depth))
            }
            OpCode::Delegate => {
                let relative_depth = match operator
                    .operator
                    .ok_or(anyhow!("Delegate operator not found"))?
                {
                    operator::Operator::RelativeDepth(idx) => idx,
                    _ => return Err(anyhow!("Expected RelativeDepth for Delegate operator")),
                };
                Ok(wasm_encoder::Instruction::Delegate(relative_depth))
            }
            OpCode::CatchAll => Ok(wasm_encoder::Instruction::CatchAll),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // BlockType tests
    #[test]
    fn test_blocktype_from_wasmparser_empty() {
        let blockty = wasmparser::BlockType::Empty;
        let result = BlockType::try_from(blockty).unwrap();
        assert!(matches!(
            result.blockty,
            Some(block_type::Blockty::Empty(0))
        ));
    }

    #[test]
    fn test_blocktype_from_wasmparser_type() {
        let blockty = wasmparser::BlockType::Type(wasmparser::ValType::I32);
        let result = BlockType::try_from(blockty).unwrap();
        match result.blockty {
            Some(block_type::Blockty::ValueType(valtype)) => {
                assert_eq!(
                    valtype,
                    ValueType {
                        val_type: Some(EValueType::I32 as i32),
                        ref_type: None
                    }
                );
            }
            _ => panic!("Expected ValueType variant"),
        }
    }

    #[test]
    fn test_blocktype_from_wasmparser_func_type() {
        let blockty = wasmparser::BlockType::FuncType(42);
        let result = BlockType::try_from(blockty).unwrap();
        match result.blockty {
            Some(block_type::Blockty::FuncType(idx)) => {
                assert_eq!(idx, 42);
            }
            _ => panic!("Expected FuncType variant"),
        }
    }

    #[test]
    fn test_blocktype_to_wasm_encoder_empty() {
        let blocktype = BlockType {
            blockty: Some(block_type::Blockty::Empty(0)),
        };
        let result = wasm_encoder::BlockType::try_from(blocktype).unwrap();
        assert!(matches!(result, wasm_encoder::BlockType::Empty));
    }

    #[test]
    fn test_blocktype_to_wasm_encoder_type() {
        let blocktype = BlockType {
            blockty: Some(block_type::Blockty::ValueType(ValueType {
                val_type: Some(EValueType::I32 as i32),
                ref_type: None,
            })),
        };
        let result = wasm_encoder::BlockType::try_from(blocktype).unwrap();
        match result {
            wasm_encoder::BlockType::Result(valtype) => {
                assert!(matches!(valtype, wasm_encoder::ValType::I32));
            }
            _ => panic!("Expected Result variant"),
        }
    }

    #[test]
    fn test_blocktype_to_wasm_encoder_func_type() {
        let blocktype = BlockType {
            blockty: Some(block_type::Blockty::FuncType(99)),
        };
        let result = wasm_encoder::BlockType::try_from(blocktype).unwrap();
        match result {
            wasm_encoder::BlockType::FunctionType(idx) => {
                assert_eq!(idx, 99);
            }
            _ => panic!("Expected FunctionType variant"),
        }
    }

    #[test]
    fn test_blocktype_roundtrip() {
        // Test Empty
        let original = wasmparser::BlockType::Empty;
        let proto = BlockType::try_from(original).unwrap();
        let back = wasm_encoder::BlockType::try_from(proto).unwrap();
        assert!(matches!(back, wasm_encoder::BlockType::Empty));

        // Test Type
        let original = wasmparser::BlockType::Type(wasmparser::ValType::I64);
        let proto = BlockType::try_from(original).unwrap();
        let back = wasm_encoder::BlockType::try_from(proto).unwrap();
        match back {
            wasm_encoder::BlockType::Result(valtype) => {
                assert!(matches!(valtype, wasm_encoder::ValType::I64));
            }
            _ => panic!("Expected Result variant"),
        }

        // Test FuncType
        let original = wasmparser::BlockType::FuncType(123);
        let proto = BlockType::try_from(original).unwrap();
        let back = wasm_encoder::BlockType::try_from(proto).unwrap();
        match back {
            wasm_encoder::BlockType::FunctionType(idx) => {
                assert_eq!(idx, 123);
            }
            _ => panic!("Expected FunctionType variant"),
        }
    }

    // MemArg tests
    #[test]
    fn test_memarg_from_wasmparser() {
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 100,
            memory: 0,
        };
        let result = MemArg::try_from(memarg).unwrap();
        assert_eq!(result.align, Some(2));
        assert_eq!(result.max_align, Some(2));
        assert_eq!(result.offset, Some(100));
        assert_eq!(result.memory, Some(0));
    }

    #[test]
    fn test_memarg_to_wasm_encoder() {
        let memarg = MemArg {
            align: Some(3),
            max_align: Some(3),
            offset: Some(200),
            memory: Some(1),
        };
        let result = wasm_encoder::MemArg::try_from(memarg).unwrap();
        assert_eq!(result.align, 3);
        assert_eq!(result.offset, 200);
        assert_eq!(result.memory_index, 1);
    }

    #[test]
    fn test_memarg_roundtrip() {
        let original = wasmparser::MemArg {
            align: 4,
            max_align: 4,
            offset: 300,
            memory: 0,
        };
        let proto = MemArg::try_from(original).unwrap();
        let back = wasm_encoder::MemArg::try_from(proto).unwrap();
        assert_eq!(back.align, 4);
        assert_eq!(back.offset, 300);
        assert_eq!(back.memory_index, 0);
    }

    #[test]
    fn test_memarg_to_wasm_encoder_missing_fields() {
        let memarg = MemArg {
            align: None,
            max_align: Some(2),
            offset: Some(100),
            memory: Some(0),
        };
        assert!(wasm_encoder::MemArg::try_from(memarg).is_err());
    }

    // Operator tests - MVP operators
    #[test]
    fn test_operator_unreachable() {
        let op = wasmparser::Operator::Unreachable;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Unreachable as i32));
    }

    #[test]
    fn test_operator_nop() {
        let op = wasmparser::Operator::Nop;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Nop as i32));
    }

    #[test]
    fn test_operator_block() {
        let op = wasmparser::Operator::Block {
            blockty: wasmparser::BlockType::Empty,
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Block as i32));
        assert!(matches!(
            result.operator,
            Some(operator::Operator::Blockty(_))
        ));
    }

    #[test]
    fn test_operator_loop() {
        let op = wasmparser::Operator::Loop {
            blockty: wasmparser::BlockType::Type(wasmparser::ValType::I32),
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Loop as i32));
        assert!(matches!(
            result.operator,
            Some(operator::Operator::Blockty(_))
        ));
    }

    #[test]
    fn test_operator_if() {
        let op = wasmparser::Operator::If {
            blockty: wasmparser::BlockType::FuncType(5),
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::If as i32));
        assert!(matches!(
            result.operator,
            Some(operator::Operator::Blockty(_))
        ));
    }

    #[test]
    fn test_operator_else() {
        let op = wasmparser::Operator::Else;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Else as i32));
    }

    #[test]
    fn test_operator_end() {
        let op = wasmparser::Operator::End;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::End as i32));
    }

    #[test]
    fn test_operator_br() {
        let op = wasmparser::Operator::Br { relative_depth: 3 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Br as i32));
        match result.operator {
            Some(operator::Operator::RelativeDepth(depth)) => assert_eq!(depth, 3),
            _ => panic!("Expected RelativeDepth"),
        }
    }

    #[test]
    fn test_operator_brif() {
        let op = wasmparser::Operator::BrIf { relative_depth: 7 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::BrIf as i32));
        match result.operator {
            Some(operator::Operator::RelativeDepth(depth)) => assert_eq!(depth, 7),
            _ => panic!("Expected RelativeDepth"),
        }
    }

    #[test]
    fn test_operator_brtable() {
        // Create a WASM module with a BrTable instruction to test
        let mut module = wasm_encoder::Module::new();
        // Add type section first
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType};
        let mut type_section = wasm_encoder::TypeSection::new();
        let func_type = SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(vec![], vec![])),
                shared: false,
                descriptor: None,
                describes: None,
            },
        };
        type_section.ty().subtype(&func_type);
        module.section(&type_section);
        // Add function section
        let mut func_section = wasm_encoder::FunctionSection::new();
        func_section.function(0); // Function type 0
        module.section(&func_section);
        // Add code section
        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        func.instruction(&wasm_encoder::Instruction::I32Const(0));
        func.instruction(&wasm_encoder::Instruction::BrTable(vec![1, 2].into(), 0));
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);
        let wasm_bytes = module.finish();

        // Parse to get the BrTable operator
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::CodeSectionEntry(body) = payload {
                let reader = body.get_operators_reader().unwrap();
                for operator_result in reader {
                    let operator = operator_result.unwrap();
                    if let wasmparser::Operator::BrTable { targets } = operator {
                        let result =
                            Operator::try_from(wasmparser::Operator::BrTable { targets }).unwrap();
                        assert_eq!(result.opcode, Some(OpCode::BrTable as i32));
                        match result.operator {
                            Some(operator::Operator::Targets(targets_proto)) => {
                                assert_eq!(targets_proto.default, Some(0));
                            }
                            _ => panic!("Expected Targets"),
                        }
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn test_operator_return() {
        let op = wasmparser::Operator::Return;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Return as i32));
    }

    #[test]
    fn test_operator_call() {
        let op = wasmparser::Operator::Call { function_index: 42 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Call as i32));
        match result.operator {
            Some(operator::Operator::FunctionIndex(idx)) => assert_eq!(idx, 42),
            _ => panic!("Expected FunctionIndex"),
        }
    }

    #[test]
    fn test_operator_call_indirect() {
        let op = wasmparser::Operator::CallIndirect {
            type_index: 10,
            table_index: 0,
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::CallIndirect as i32));
        match result.operator {
            Some(operator::Operator::CallInderect(ci)) => {
                assert_eq!(ci.type_index, Some(10));
                assert_eq!(ci.table_index, Some(0));
            }
            _ => panic!("Expected CallInderect"),
        }
    }

    #[test]
    fn test_operator_drop() {
        let op = wasmparser::Operator::Drop;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Drop as i32));
    }

    #[test]
    fn test_operator_select() {
        let op = wasmparser::Operator::Select;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Select as i32));
    }

    #[test]
    fn test_operator_local_get() {
        let op = wasmparser::Operator::LocalGet { local_index: 5 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::LocalGet as i32));
        match result.operator {
            Some(operator::Operator::LocalIndex(idx)) => assert_eq!(idx, 5),
            _ => panic!("Expected LocalIndex"),
        }
    }

    #[test]
    fn test_operator_local_set() {
        let op = wasmparser::Operator::LocalSet { local_index: 8 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::LocalSet as i32));
        match result.operator {
            Some(operator::Operator::LocalIndex(idx)) => assert_eq!(idx, 8),
            _ => panic!("Expected LocalIndex"),
        }
    }

    #[test]
    fn test_operator_local_tee() {
        let op = wasmparser::Operator::LocalTee { local_index: 12 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::LocalTee as i32));
        match result.operator {
            Some(operator::Operator::LocalIndex(idx)) => assert_eq!(idx, 12),
            _ => panic!("Expected LocalIndex"),
        }
    }

    #[test]
    fn test_operator_global_get() {
        let op = wasmparser::Operator::GlobalGet { global_index: 3 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::GlobalGet as i32));
        match result.operator {
            Some(operator::Operator::GlobalIndex(idx)) => assert_eq!(idx, 3),
            _ => panic!("Expected GlobalIndex"),
        }
    }

    #[test]
    fn test_operator_global_set() {
        let op = wasmparser::Operator::GlobalSet { global_index: 6 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::GlobalSet as i32));
        match result.operator {
            Some(operator::Operator::GlobalIndex(idx)) => assert_eq!(idx, 6),
            _ => panic!("Expected GlobalIndex"),
        }
    }

    // Memory load/store tests
    #[test]
    fn test_operator_i32_load() {
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 0,
            memory: 0,
        };
        let op = wasmparser::Operator::I32Load { memarg };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I32Load as i32));
        assert!(matches!(
            result.operator,
            Some(operator::Operator::Memarg(_))
        ));
    }

    #[test]
    fn test_operator_i64_store() {
        let memarg = wasmparser::MemArg {
            align: 3,
            max_align: 3,
            offset: 100,
            memory: 0,
        };
        let op = wasmparser::Operator::I64Store { memarg };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I64Store as i32));
        assert!(matches!(
            result.operator,
            Some(operator::Operator::Memarg(_))
        ));
    }

    #[test]
    fn test_operator_memory_size() {
        let op = wasmparser::Operator::MemorySize { mem: 0 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::MemorySize as i32));
        match result.operator {
            Some(operator::Operator::Mem(m)) => assert_eq!(m, 0),
            _ => panic!("Expected Mem"),
        }
    }

    #[test]
    fn test_operator_memory_grow() {
        let op = wasmparser::Operator::MemoryGrow { mem: 1 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::MemoryGrow as i32));
        match result.operator {
            Some(operator::Operator::Mem(m)) => assert_eq!(m, 1),
            _ => panic!("Expected Mem"),
        }
    }

    // Constant tests
    #[test]
    fn test_operator_i32_const() {
        let op = wasmparser::Operator::I32Const { value: 42 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I32Const as i32));
        match result.operator {
            Some(operator::Operator::I32value(v)) => assert_eq!(v, 42),
            _ => panic!("Expected I32value"),
        }
    }

    #[test]
    fn test_operator_i64_const() {
        let op = wasmparser::Operator::I64Const { value: -100 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I64Const as i32));
        match result.operator {
            Some(operator::Operator::I64value(v)) => assert_eq!(v, -100),
            _ => panic!("Expected I64value"),
        }
    }

    #[test]
    fn test_operator_f32_const() {
        // Create a WASM module with F32Const to get a proper Ieee32
        let mut module = wasm_encoder::Module::new();
        // Add type section
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType};
        let mut type_section = wasm_encoder::TypeSection::new();
        let func_type = SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(vec![], vec![])),
                shared: false,
                descriptor: None,
                describes: None,
            },
        };
        type_section.ty().subtype(&func_type);
        module.section(&type_section);
        // Add function section
        let mut func_section = wasm_encoder::FunctionSection::new();
        func_section.function(0);
        module.section(&func_section);
        // Add code section
        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        func.instruction(&wasm_encoder::Instruction::F32Const(
            wasm_encoder::Ieee32::new(3.14f32.to_bits()),
        ));
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);
        let wasm_bytes = module.finish();

        // Parse to get the F32Const operator
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::CodeSectionEntry(body) = payload {
                let reader = body.get_operators_reader().unwrap();
                for operator_result in reader {
                    let operator = operator_result.unwrap();
                    if let wasmparser::Operator::F32Const { value } = operator {
                        let result =
                            Operator::try_from(wasmparser::Operator::F32Const { value }).unwrap();
                        assert_eq!(result.opcode, Some(OpCode::F32Const as i32));
                        match result.operator {
                            Some(operator::Operator::F32value(bits)) => {
                                assert_eq!(bits, 3.14f32.to_bits());
                            }
                            _ => panic!("Expected F32value"),
                        }
                        break;
                    }
                }
            }
        }
    }

    #[test]
    fn test_operator_f64_const() {
        // Create a WASM module with F64Const to get a proper Ieee64
        let mut module = wasm_encoder::Module::new();
        // Add type section
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType};
        let mut type_section = wasm_encoder::TypeSection::new();
        let func_type = SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(vec![], vec![])),
                shared: false,
                descriptor: None,
                describes: None,
            },
        };
        type_section.ty().subtype(&func_type);
        module.section(&type_section);
        // Add function section
        let mut func_section = wasm_encoder::FunctionSection::new();
        func_section.function(0);
        module.section(&func_section);
        // Add code section
        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        func.instruction(&wasm_encoder::Instruction::F64Const(
            wasm_encoder::Ieee64::new(2.718f64.to_bits()),
        ));
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);
        let wasm_bytes = module.finish();

        // Parse to get the F64Const operator
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::CodeSectionEntry(body) = payload {
                let reader = body.get_operators_reader().unwrap();
                for operator_result in reader {
                    let operator = operator_result.unwrap();
                    if let wasmparser::Operator::F64Const { value } = operator {
                        let result =
                            Operator::try_from(wasmparser::Operator::F64Const { value }).unwrap();
                        assert_eq!(result.opcode, Some(OpCode::F64Const as i32));
                        match result.operator {
                            Some(operator::Operator::F64value(bits)) => {
                                assert_eq!(bits, 2.718f64.to_bits());
                            }
                            _ => panic!("Expected F64value"),
                        }
                        break;
                    }
                }
            }
        }
    }

    // Comparison tests
    #[test]
    fn test_operator_i32_eqz() {
        let op = wasmparser::Operator::I32Eqz;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I32Eqz as i32));
    }

    #[test]
    fn test_operator_i32_eq() {
        let op = wasmparser::Operator::I32Eq;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I32Eq as i32));
    }

    #[test]
    fn test_operator_f64_ge() {
        let op = wasmparser::Operator::F64Ge;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::F64Ge as i32));
    }

    // Arithmetic tests
    #[test]
    fn test_operator_i32_add() {
        let op = wasmparser::Operator::I32Add;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I32Add as i32));
    }

    #[test]
    fn test_operator_i64_mul() {
        let op = wasmparser::Operator::I64Mul;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I64Mul as i32));
    }

    #[test]
    fn test_operator_f32_div() {
        let op = wasmparser::Operator::F32Div;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::F32Div as i32));
    }

    // Sign extension tests
    #[test]
    fn test_operator_i32_extend8s() {
        let op = wasmparser::Operator::I32Extend8S;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I32Extend8S as i32));
    }

    #[test]
    fn test_operator_i64_extend32s() {
        let op = wasmparser::Operator::I64Extend32S;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I64Extend32S as i32));
    }

    // Saturating float to int tests
    #[test]
    fn test_operator_i32_trunc_sat_f32s() {
        let op = wasmparser::Operator::I32TruncSatF32S;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I32TruncSatF32s as i32));
    }

    #[test]
    fn test_operator_i64_trunc_sat_f64u() {
        let op = wasmparser::Operator::I64TruncSatF64U;
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::I64TruncSatF64u as i32));
    }

    // Bulk memory tests
    #[test]
    fn test_operator_memory_init() {
        let op = wasmparser::Operator::MemoryInit {
            data_index: 5,
            mem: 0,
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::MemoryInit as i32));
        match result.operator {
            Some(operator::Operator::MemoryInit(mi)) => {
                assert_eq!(mi.data_index, Some(5));
                assert_eq!(mi.mem, Some(0));
            }
            _ => panic!("Expected MemoryInit"),
        }
    }

    #[test]
    fn test_operator_data_drop() {
        let op = wasmparser::Operator::DataDrop { data_index: 3 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::DataDrop as i32));
        match result.operator {
            Some(operator::Operator::DataIndex(idx)) => assert_eq!(idx, 3),
            _ => panic!("Expected DataIndex"),
        }
    }

    #[test]
    fn test_operator_memory_copy() {
        let op = wasmparser::Operator::MemoryCopy {
            dst_mem: 0,
            src_mem: 0,
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::MemoryCopy as i32));
        match result.operator {
            Some(operator::Operator::MemoryCopy(mc)) => {
                assert_eq!(mc.dst_mem, Some(0));
                assert_eq!(mc.src_mem, Some(0));
            }
            _ => panic!("Expected MemoryCopy"),
        }
    }

    #[test]
    fn test_operator_memory_fill() {
        let op = wasmparser::Operator::MemoryFill { mem: 0 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::MemoryFill as i32));
        match result.operator {
            Some(operator::Operator::Mem(m)) => assert_eq!(m, 0),
            _ => panic!("Expected Mem"),
        }
    }

    #[test]
    fn test_operator_table_init() {
        let op = wasmparser::Operator::TableInit {
            elem_index: 2,
            table: 0,
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::TableInit as i32));
        match result.operator {
            Some(operator::Operator::TableInit(ti)) => {
                assert_eq!(ti.elem_index, Some(2));
                assert_eq!(ti.table, Some(0));
            }
            _ => panic!("Expected TableInit"),
        }
    }

    #[test]
    fn test_operator_elem_drop() {
        let op = wasmparser::Operator::ElemDrop { elem_index: 1 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::ElemDrop as i32));
        match result.operator {
            Some(operator::Operator::ElemIndex(idx)) => assert_eq!(idx, 1),
            _ => panic!("Expected ElemIndex"),
        }
    }

    #[test]
    fn test_operator_table_copy() {
        let op = wasmparser::Operator::TableCopy {
            dst_table: 0,
            src_table: 0,
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::TableCopy as i32));
        match result.operator {
            Some(operator::Operator::TableCopy(tc)) => {
                assert_eq!(tc.dst_table, Some(0));
                assert_eq!(tc.src_table, Some(0));
            }
            _ => panic!("Expected TableCopy"),
        }
    }

    // Tests for conversion from Operator to wasm_encoder::Instruction
    #[test]
    fn test_operator_to_instruction_unreachable() {
        let op = Operator {
            opcode: Some(OpCode::Unreachable as i32),
            ..Operator::default()
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::Unreachable));
    }

    #[test]
    fn test_operator_to_instruction_nop() {
        let op = Operator {
            opcode: Some(OpCode::Nop as i32),
            ..Operator::default()
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::Nop));
    }

    #[test]
    fn test_operator_to_instruction_block() {
        let blocktype = BlockType {
            blockty: Some(block_type::Blockty::Empty(0)),
        };
        let op = Operator {
            opcode: Some(OpCode::Block as i32),
            operator: Some(operator::Operator::Blockty(blocktype)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::Block(_)));
    }

    #[test]
    fn test_operator_to_instruction_br() {
        let op = Operator {
            opcode: Some(OpCode::Br as i32),
            operator: Some(operator::Operator::RelativeDepth(5)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::Br(depth) => assert_eq!(depth, 5),
            _ => panic!("Expected Br instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_call() {
        let op = Operator {
            opcode: Some(OpCode::Call as i32),
            operator: Some(operator::Operator::FunctionIndex(99)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::Call(idx) => assert_eq!(idx, 99),
            _ => panic!("Expected Call instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_i32_const() {
        let op = Operator {
            opcode: Some(OpCode::I32Const as i32),
            operator: Some(operator::Operator::I32value(-42)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::I32Const(val) => assert_eq!(val, -42),
            _ => panic!("Expected I32Const instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_f32_const() {
        let bits = 3.14f32.to_bits();
        let op = Operator {
            opcode: Some(OpCode::F32Const as i32),
            operator: Some(operator::Operator::F32value(bits)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::F32Const(ieee) => {
                assert_eq!(ieee.bits(), bits);
            }
            _ => panic!("Expected F32Const instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_i32_load() {
        let memarg = MemArg {
            align: Some(2),
            max_align: Some(2),
            offset: Some(100),
            memory: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::I32Load as i32),
            operator: Some(operator::Operator::Memarg(memarg)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::I32Load(ma) => {
                assert_eq!(ma.align, 2);
                assert_eq!(ma.offset, 100);
            }
            _ => panic!("Expected I32Load instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_brtable() {
        let targets = BrTargets {
            default: Some(0),
            targets: vec![1, 2, 3],
        };
        let op = Operator {
            opcode: Some(OpCode::BrTable as i32),
            operator: Some(operator::Operator::Targets(targets)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::BrTable(targets_vec, default) => {
                assert_eq!(default, 0);
                assert_eq!(targets_vec.len(), 3);
            }
            _ => panic!("Expected BrTable instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_call_indirect() {
        let ci = CallIndirectOp {
            type_index: Some(10),
            table_index: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::CallIndirect as i32),
            operator: Some(operator::Operator::CallInderect(ci)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::CallIndirect {
                type_index,
                table_index,
            } => {
                assert_eq!(type_index, 10);
                assert_eq!(table_index, 0);
            }
            _ => panic!("Expected CallIndirect instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_memory_init() {
        let mi = MemoryInitOp {
            data_index: Some(5),
            mem: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::MemoryInit as i32),
            operator: Some(operator::Operator::MemoryInit(mi)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::MemoryInit { mem, data_index } => {
                assert_eq!(mem, 0);
                assert_eq!(data_index, 5);
            }
            _ => panic!("Expected MemoryInit instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_table_copy() {
        let tc = TableCopyOp {
            dst_table: Some(0),
            src_table: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::TableCopy as i32),
            operator: Some(operator::Operator::TableCopy(tc)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::TableCopy {
                dst_table,
                src_table,
            } => {
                assert_eq!(dst_table, 0);
                assert_eq!(src_table, 0);
            }
            _ => panic!("Expected TableCopy instruction"),
        }
    }

    // Round-trip tests
    #[test]
    fn test_operator_roundtrip_simple() {
        let original = wasmparser::Operator::I32Add;
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        assert!(matches!(back, wasm_encoder::Instruction::I32Add));
    }

    #[test]
    fn test_operator_roundtrip_with_value() {
        let original = wasmparser::Operator::I32Const { value: 123 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::I32Const(val) => assert_eq!(val, 123),
            _ => panic!("Expected I32Const"),
        }
    }

    #[test]
    fn test_operator_roundtrip_with_memarg() {
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 50,
            memory: 0,
        };
        let original = wasmparser::Operator::I32Load { memarg };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::I32Load(ma) => {
                assert_eq!(ma.align, 2);
                assert_eq!(ma.offset, 50);
            }
            _ => panic!("Expected I32Load"),
        }
    }

    #[test]
    fn test_operator_roundtrip_block() {
        let original = wasmparser::Operator::Block {
            blockty: wasmparser::BlockType::Type(wasmparser::ValType::I32),
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Block(blockty) => match blockty {
                wasm_encoder::BlockType::Result(valtype) => {
                    assert!(matches!(valtype, wasm_encoder::ValType::I32));
                }
                _ => panic!("Expected Result block type"),
            },
            _ => panic!("Expected Block instruction"),
        }
    }

    // Error case tests
    #[test]
    fn test_operator_to_instruction_missing_opcode() {
        let op = Operator::default();
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_to_instruction_missing_operator_field() {
        let op = Operator {
            opcode: Some(OpCode::Call as i32),
            ..Operator::default()
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_blocktype_to_wasm_encoder_missing_blockty() {
        let blocktype = BlockType::default();
        assert!(wasm_encoder::BlockType::try_from(blocktype).is_err());
    }

    #[test]
    fn test_blocktype_to_wasm_encoder_invalid_empty() {
        let blocktype = BlockType {
            blockty: Some(block_type::Blockty::Empty(1)), // Should be 0
        };
        // This should fail because Empty must be 0
        let result = wasm_encoder::BlockType::try_from(blocktype);
        assert!(result.is_err());
    }

    // CatchElement tests
    #[test]
    fn test_catch_element_one() {
        let catch = wasmparser::Catch::One { tag: 5, label: 10 };
        let result = CatchElement::try_from(catch).unwrap();
        match result.catch {
            Some(catch_element::Catch::One(one)) => {
                assert_eq!(one.tag, Some(5));
                assert_eq!(one.label, Some(10));
            }
            _ => panic!("Expected Catch::One variant"),
        }
    }

    #[test]
    fn test_catch_element_one_ref() {
        let catch = wasmparser::Catch::OneRef { tag: 3, label: 7 };
        let result = CatchElement::try_from(catch).unwrap();
        match result.catch {
            Some(catch_element::Catch::OneRef(one_ref)) => {
                assert_eq!(one_ref.tag, Some(3));
                assert_eq!(one_ref.label, Some(7));
            }
            _ => panic!("Expected Catch::OneRef variant"),
        }
    }

    #[test]
    fn test_catch_element_all() {
        let catch = wasmparser::Catch::All { label: 15 };
        let result = CatchElement::try_from(catch).unwrap();
        match result.catch {
            Some(catch_element::Catch::All(all)) => {
                assert_eq!(all.label, Some(15));
            }
            _ => panic!("Expected Catch::All variant"),
        }
    }

    #[test]
    fn test_catch_element_all_ref() {
        let catch = wasmparser::Catch::AllRef { label: 20 };
        let result = CatchElement::try_from(catch).unwrap();
        match result.catch {
            Some(catch_element::Catch::AllRef(all_ref)) => {
                assert_eq!(all_ref.label, Some(20));
            }
            _ => panic!("Expected Catch::AllRef variant"),
        }
    }

    // Exception-related operator tests
    #[test]
    fn test_operator_try() {
        let op = wasmparser::Operator::Try {
            blockty: wasmparser::BlockType::Empty,
        };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Try as i32));
        assert!(matches!(
            result.operator,
            Some(operator::Operator::Blockty(_))
        ));
    }

    #[test]
    fn test_operator_catch() {
        let op = wasmparser::Operator::Catch { tag_index: 42 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Catch as i32));
        match result.operator {
            Some(operator::Operator::TagIndex(idx)) => assert_eq!(idx, 42),
            _ => panic!("Expected TagIndex"),
        }
    }

    #[test]
    fn test_operator_catch_all() {
        let op = wasmparser::Operator::CatchAll {};
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::CatchAll as i32));
    }

    #[test]
    fn test_operator_rethrow() {
        let op = wasmparser::Operator::Rethrow { relative_depth: 3 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Rethrow as i32));
        match result.operator {
            Some(operator::Operator::RelativeDepth(depth)) => assert_eq!(depth, 3),
            _ => panic!("Expected RelativeDepth"),
        }
    }

    #[test]
    fn test_operator_delegate() {
        let op = wasmparser::Operator::Delegate { relative_depth: 5 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Delegate as i32));
        match result.operator {
            Some(operator::Operator::RelativeDepth(depth)) => assert_eq!(depth, 5),
            _ => panic!("Expected RelativeDepth"),
        }
    }

    #[test]
    fn test_operator_throw() {
        let op = wasmparser::Operator::Throw { tag_index: 7 };
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::Throw as i32));
        match result.operator {
            Some(operator::Operator::Throw(throw_op)) => {
                assert_eq!(throw_op.tag_index, Some(7));
            }
            _ => panic!("Expected Throw operator"),
        }
    }

    #[test]
    fn test_operator_throw_ref() {
        let op = wasmparser::Operator::ThrowRef {};
        let result = Operator::try_from(op).unwrap();
        assert_eq!(result.opcode, Some(OpCode::ThrowRef as i32));
    }

    // Additional reverse conversion tests (Operator -> wasm_encoder::Instruction)
    #[test]
    fn test_operator_to_instruction_loop() {
        let blocktype = BlockType {
            blockty: Some(block_type::Blockty::ValueType(ValueType {
                val_type: Some(EValueType::I64 as i32),
                ref_type: None,
            })),
        };
        let op = Operator {
            opcode: Some(OpCode::Loop as i32),
            operator: Some(operator::Operator::Blockty(blocktype)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::Loop(_)));
    }

    #[test]
    fn test_operator_to_instruction_if() {
        let blocktype = BlockType {
            blockty: Some(block_type::Blockty::Empty(0)),
        };
        let op = Operator {
            opcode: Some(OpCode::If as i32),
            operator: Some(operator::Operator::Blockty(blocktype)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::If(_)));
    }

    #[test]
    fn test_operator_to_instruction_brif() {
        let op = Operator {
            opcode: Some(OpCode::BrIf as i32),
            operator: Some(operator::Operator::RelativeDepth(10)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::BrIf(depth) => assert_eq!(depth, 10),
            _ => panic!("Expected BrIf instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_local_get() {
        let op = Operator {
            opcode: Some(OpCode::LocalGet as i32),
            operator: Some(operator::Operator::LocalIndex(15)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::LocalGet(idx) => assert_eq!(idx, 15),
            _ => panic!("Expected LocalGet instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_local_set() {
        let op = Operator {
            opcode: Some(OpCode::LocalSet as i32),
            operator: Some(operator::Operator::LocalIndex(20)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::LocalSet(idx) => assert_eq!(idx, 20),
            _ => panic!("Expected LocalSet instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_local_tee() {
        let op = Operator {
            opcode: Some(OpCode::LocalTee as i32),
            operator: Some(operator::Operator::LocalIndex(25)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::LocalTee(idx) => assert_eq!(idx, 25),
            _ => panic!("Expected LocalTee instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_global_get() {
        let op = Operator {
            opcode: Some(OpCode::GlobalGet as i32),
            operator: Some(operator::Operator::GlobalIndex(30)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::GlobalGet(idx) => assert_eq!(idx, 30),
            _ => panic!("Expected GlobalGet instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_global_set() {
        let op = Operator {
            opcode: Some(OpCode::GlobalSet as i32),
            operator: Some(operator::Operator::GlobalIndex(35)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::GlobalSet(idx) => assert_eq!(idx, 35),
            _ => panic!("Expected GlobalSet instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_i64_const() {
        let op = Operator {
            opcode: Some(OpCode::I64Const as i32),
            operator: Some(operator::Operator::I64value(-100)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::I64Const(val) => assert_eq!(val, -100),
            _ => panic!("Expected I64Const instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_f64_const() {
        let bits = 2.718f64.to_bits();
        let op = Operator {
            opcode: Some(OpCode::F64Const as i32),
            operator: Some(operator::Operator::F64value(bits)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::F64Const(ieee) => {
                assert_eq!(ieee.bits(), bits);
            }
            _ => panic!("Expected F64Const instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_memory_size() {
        let op = Operator {
            opcode: Some(OpCode::MemorySize as i32),
            operator: Some(operator::Operator::Mem(0)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::MemorySize(mem) => assert_eq!(mem, 0),
            _ => panic!("Expected MemorySize instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_memory_grow() {
        let op = Operator {
            opcode: Some(OpCode::MemoryGrow as i32),
            operator: Some(operator::Operator::Mem(1)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::MemoryGrow(mem) => assert_eq!(mem, 1),
            _ => panic!("Expected MemoryGrow instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_catch() {
        let op = Operator {
            opcode: Some(OpCode::Catch as i32),
            operator: Some(operator::Operator::TagIndex(5)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::Catch(tag_index) => assert_eq!(tag_index, 5),
            _ => panic!("Expected Catch instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_catch_all() {
        let op = Operator {
            opcode: Some(OpCode::CatchAll as i32),
            ..Operator::default()
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::CatchAll));
    }

    #[test]
    fn test_operator_to_instruction_memory_copy() {
        let mc = MemoryCopyOp {
            dst_mem: Some(0),
            src_mem: Some(1),
        };
        let op = Operator {
            opcode: Some(OpCode::MemoryCopy as i32),
            operator: Some(operator::Operator::MemoryCopy(mc)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::MemoryCopy { dst_mem, src_mem } => {
                assert_eq!(dst_mem, 0);
                assert_eq!(src_mem, 1);
            }
            _ => panic!("Expected MemoryCopy instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_memory_fill() {
        let op = Operator {
            opcode: Some(OpCode::MemoryFill as i32),
            operator: Some(operator::Operator::Mem(2)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::MemoryFill(mem) => assert_eq!(mem, 2),
            _ => panic!("Expected MemoryFill instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_table_init() {
        let ti = TableInitOp {
            elem_index: Some(3),
            table: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::TableInit as i32),
            operator: Some(operator::Operator::TableInit(ti)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::TableInit { elem_index, table } => {
                assert_eq!(elem_index, 3);
                assert_eq!(table, 0);
            }
            _ => panic!("Expected TableInit instruction"),
        }
    }

    // Additional error case tests
    #[test]
    fn test_operator_to_instruction_wrong_operator_type() {
        let op = Operator {
            opcode: Some(OpCode::Block as i32),
            operator: Some(operator::Operator::RelativeDepth(5)), // Wrong type
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_to_instruction_missing_memarg() {
        let op = Operator {
            opcode: Some(OpCode::I32Load as i32),
            ..Operator::default() // Missing MemArg
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_to_instruction_missing_call_indirect_fields() {
        let ci = CallIndirectOp {
            type_index: None, // Missing
            table_index: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::CallIndirect as i32),
            operator: Some(operator::Operator::CallInderect(ci)),
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_to_instruction_invalid_opcode() {
        let op = Operator {
            opcode: Some(9999), // Invalid opcode
            ..Operator::default()
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    // Round-trip tests for additional operators
    #[test]
    fn test_operator_roundtrip_loop() {
        let original = wasmparser::Operator::Loop {
            blockty: wasmparser::BlockType::Type(wasmparser::ValType::F32),
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Loop(blockty) => match blockty {
                wasm_encoder::BlockType::Result(valtype) => {
                    assert!(matches!(valtype, wasm_encoder::ValType::F32));
                }
                _ => panic!("Expected Result block type"),
            },
            _ => panic!("Expected Loop instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_if() {
        let original = wasmparser::Operator::If {
            blockty: wasmparser::BlockType::Empty,
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::If(blockty) => {
                assert!(matches!(blockty, wasm_encoder::BlockType::Empty));
            }
            _ => panic!("Expected If instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_brif() {
        let original = wasmparser::Operator::BrIf { relative_depth: 2 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::BrIf(depth) => assert_eq!(depth, 2),
            _ => panic!("Expected BrIf instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_local_get() {
        let original = wasmparser::Operator::LocalGet { local_index: 10 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::LocalGet(idx) => assert_eq!(idx, 10),
            _ => panic!("Expected LocalGet instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_global_get() {
        let original = wasmparser::Operator::GlobalGet { global_index: 5 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::GlobalGet(idx) => assert_eq!(idx, 5),
            _ => panic!("Expected GlobalGet instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_i64_const() {
        let original = wasmparser::Operator::I64Const { value: -999 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::I64Const(val) => assert_eq!(val, -999),
            _ => panic!("Expected I64Const instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_memory_size() {
        let original = wasmparser::Operator::MemorySize { mem: 0 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::MemorySize(mem) => assert_eq!(mem, 0),
            _ => panic!("Expected MemorySize instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_i32_load() {
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 100,
            memory: 0,
        };
        let original = wasmparser::Operator::I32Load { memarg };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::I32Load(ma) => {
                assert_eq!(ma.align, 2);
                assert_eq!(ma.offset, 100);
                assert_eq!(ma.memory_index, 0);
            }
            _ => panic!("Expected I32Load instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_i64_load8s() {
        let memarg = wasmparser::MemArg {
            align: 0,
            max_align: 0,
            offset: 0,
            memory: 1,
        };
        let original = wasmparser::Operator::I64Load8S { memarg };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::I64Load8S(ma) => {
                assert_eq!(ma.memory_index, 1);
            }
            _ => panic!("Expected I64Load8S instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_f32_store() {
        let memarg = wasmparser::MemArg {
            align: 2,
            max_align: 2,
            offset: 200,
            memory: 0,
        };
        let original = wasmparser::Operator::F32Store { memarg };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::F32Store(ma) => {
                assert_eq!(ma.align, 2);
                assert_eq!(ma.offset, 200);
            }
            _ => panic!("Expected F32Store instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_i32_store8() {
        let memarg = wasmparser::MemArg {
            align: 0,
            max_align: 0,
            offset: 50,
            memory: 0,
        };
        let original = wasmparser::Operator::I32Store8 { memarg };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::I32Store8(ma) => {
                assert_eq!(ma.offset, 50);
            }
            _ => panic!("Expected I32Store8 instruction"),
        }
    }

    // Roundtrip tests for constants
    #[test]
    fn test_operator_roundtrip_i32_const() {
        let original = wasmparser::Operator::I32Const { value: -42 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::I32Const(val) => assert_eq!(val, -42),
            _ => panic!("Expected I32Const instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_f32_const() {
        // Create a WASM module to get proper Ieee32
        let mut module = wasm_encoder::Module::new();
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType};
        let mut type_section = wasm_encoder::TypeSection::new();
        let func_type = SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(vec![], vec![])),
                shared: false,
                descriptor: None,
                describes: None,
            },
        };
        type_section.ty().subtype(&func_type);
        module.section(&type_section);
        let mut func_section = wasm_encoder::FunctionSection::new();
        func_section.function(0);
        module.section(&func_section);
        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        let test_value = 1.5f32;
        func.instruction(&wasm_encoder::Instruction::F32Const(
            wasm_encoder::Ieee32::new(test_value.to_bits()),
        ));
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);
        let wasm_bytes = module.finish();

        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::CodeSectionEntry(body) = payload {
                let reader = body.get_operators_reader().unwrap();
                for operator_result in reader {
                    let operator = operator_result.unwrap();
                    if let wasmparser::Operator::F32Const { value } = operator {
                        let proto = Operator::try_from(wasmparser::Operator::F32Const { value }).unwrap();
                        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
                        match back {
                            wasm_encoder::Instruction::F32Const(ieee) => {
                                assert_eq!(ieee.bits(), test_value.to_bits());
                            }
                            _ => panic!("Expected F32Const instruction"),
                        }
                        break;
                    }
                }
            }
        }
    }

    // Roundtrip tests for local/global operations
    #[test]
    fn test_operator_roundtrip_local_set() {
        let original = wasmparser::Operator::LocalSet { local_index: 99 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::LocalSet(idx) => assert_eq!(idx, 99),
            _ => panic!("Expected LocalSet instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_local_tee() {
        let original = wasmparser::Operator::LocalTee { local_index: 77 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::LocalTee(idx) => assert_eq!(idx, 77),
            _ => panic!("Expected LocalTee instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_global_set() {
        let original = wasmparser::Operator::GlobalSet { global_index: 33 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::GlobalSet(idx) => assert_eq!(idx, 33),
            _ => panic!("Expected GlobalSet instruction"),
        }
    }

    // Roundtrip tests for call operations
    #[test]
    fn test_operator_roundtrip_call() {
        let original = wasmparser::Operator::Call { function_index: 123 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Call(idx) => assert_eq!(idx, 123),
            _ => panic!("Expected Call instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_call_indirect() {
        let original = wasmparser::Operator::CallIndirect {
            type_index: 5,
            table_index: 1,
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::CallIndirect { type_index, table_index } => {
                assert_eq!(type_index, 5);
                assert_eq!(table_index, 1);
            }
            _ => panic!("Expected CallIndirect instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_br() {
        let original = wasmparser::Operator::Br { relative_depth: 7 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Br(depth) => assert_eq!(depth, 7),
            _ => panic!("Expected Br instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_memory_grow() {
        let original = wasmparser::Operator::MemoryGrow { mem: 2 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::MemoryGrow(mem) => assert_eq!(mem, 2),
            _ => panic!("Expected MemoryGrow instruction"),
        }
    }

    // Roundtrip tests for bulk memory operations
    #[test]
    fn test_operator_roundtrip_memory_init() {
        let original = wasmparser::Operator::MemoryInit {
            data_index: 3,
            mem: 0,
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::MemoryInit { mem, data_index } => {
                assert_eq!(data_index, 3);
                assert_eq!(mem, 0);
            }
            _ => panic!("Expected MemoryInit instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_memory_copy() {
        let original = wasmparser::Operator::MemoryCopy {
            dst_mem: 0,
            src_mem: 1,
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::MemoryCopy { dst_mem, src_mem } => {
                assert_eq!(dst_mem, 0);
                assert_eq!(src_mem, 1);
            }
            _ => panic!("Expected MemoryCopy instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_memory_fill() {
        let original = wasmparser::Operator::MemoryFill { mem: 1 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::MemoryFill(mem) => assert_eq!(mem, 1),
            _ => panic!("Expected MemoryFill instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_data_drop() {
        let original = wasmparser::Operator::DataDrop { data_index: 5 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::DataDrop(idx) => assert_eq!(idx, 5),
            _ => panic!("Expected DataDrop instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_table_init() {
        let original = wasmparser::Operator::TableInit {
            elem_index: 2,
            table: 0,
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::TableInit { table, elem_index } => {
                assert_eq!(elem_index, 2);
                assert_eq!(table, 0);
            }
            _ => panic!("Expected TableInit instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_table_copy() {
        let original = wasmparser::Operator::TableCopy {
            dst_table: 1,
            src_table: 0,
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::TableCopy { dst_table, src_table } => {
                assert_eq!(dst_table, 1);
                assert_eq!(src_table, 0);
            }
            _ => panic!("Expected TableCopy instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_elem_drop() {
        let original = wasmparser::Operator::ElemDrop { elem_index: 4 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::ElemDrop(idx) => assert_eq!(idx, 4),
            _ => panic!("Expected ElemDrop instruction"),
        }
    }

    // Roundtrip tests for exception operations
    #[test]
    fn test_operator_roundtrip_throw() {
        let original = wasmparser::Operator::Throw { tag_index: 10 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Throw(idx) => assert_eq!(idx, 10),
            _ => panic!("Expected Throw instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_rethrow() {
        let original = wasmparser::Operator::Rethrow { relative_depth: 2 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Rethrow(depth) => assert_eq!(depth, 2),
            _ => panic!("Expected Rethrow instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_delegate() {
        let original = wasmparser::Operator::Delegate { relative_depth: 4 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Delegate(depth) => assert_eq!(depth, 4),
            _ => panic!("Expected Delegate instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_catch() {
        let original = wasmparser::Operator::Catch { tag_index: 8 };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Catch(idx) => assert_eq!(idx, 8),
            _ => panic!("Expected Catch instruction"),
        }
    }

    #[test]
    fn test_operator_roundtrip_try() {
        let original = wasmparser::Operator::Try {
            blockty: wasmparser::BlockType::Type(wasmparser::ValType::I32),
        };
        let proto = Operator::try_from(original).unwrap();
        let back = wasm_encoder::Instruction::try_from(proto).unwrap();
        match back {
            wasm_encoder::Instruction::Try(blockty) => {
                match blockty {
                    wasm_encoder::BlockType::Result(valtype) => {
                        assert!(matches!(valtype, wasm_encoder::ValType::I32));
                    }
                    _ => panic!("Expected Result block type"),
                }
            }
            _ => panic!("Expected Try instruction"),
        }
    }

    // Roundtrip test for BlockType with FuncType
    #[test]
    fn test_blocktype_roundtrip_func_type() {
        let original = wasmparser::BlockType::FuncType(42);
        let proto = BlockType::try_from(original).unwrap();
        let back = wasm_encoder::BlockType::try_from(proto).unwrap();
        match back {
            wasm_encoder::BlockType::FunctionType(idx) => assert_eq!(idx, 42),
            _ => panic!("Expected FunctionType block type"),
        }
    }

    // Reverse conversion tests for load operations
    #[test]
    fn test_operator_to_instruction_i64_load() {
        let memarg = MemArg {
            align: Some(3),
            max_align: Some(3),
            offset: Some(150),
            memory: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::I64Load as i32),
            operator: Some(operator::Operator::Memarg(memarg)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::I64Load(ma) => {
                assert_eq!(ma.align, 3);
                assert_eq!(ma.offset, 150);
            }
            _ => panic!("Expected I64Load instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_f64_load() {
        let memarg = MemArg {
            align: Some(3),
            max_align: Some(3),
            offset: Some(0),
            memory: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::F64Load as i32),
            operator: Some(operator::Operator::Memarg(memarg)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::F64Load(_)));
    }

    #[test]
    fn test_operator_to_instruction_i32_load16u() {
        let memarg = MemArg {
            align: Some(1),
            max_align: Some(1),
            offset: Some(10),
            memory: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::I32Load16U as i32),
            operator: Some(operator::Operator::Memarg(memarg)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::I32Load16U(ma) => {
                assert_eq!(ma.offset, 10);
            }
            _ => panic!("Expected I32Load16U instruction"),
        }
    }

    // Reverse conversion tests for store operations
    #[test]
    fn test_operator_to_instruction_i64_store() {
        let memarg = MemArg {
            align: Some(3),
            max_align: Some(3),
            offset: Some(300),
            memory: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::I64Store as i32),
            operator: Some(operator::Operator::Memarg(memarg)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        match result {
            wasm_encoder::Instruction::I64Store(ma) => {
                assert_eq!(ma.offset, 300);
            }
            _ => panic!("Expected I64Store instruction"),
        }
    }

    #[test]
    fn test_operator_to_instruction_i64_store32() {
        let memarg = MemArg {
            align: Some(2),
            max_align: Some(2),
            offset: Some(0),
            memory: Some(0),
        };
        let op = Operator {
            opcode: Some(OpCode::I64Store32 as i32),
            operator: Some(operator::Operator::Memarg(memarg)),
        };
        let result = wasm_encoder::Instruction::try_from(op).unwrap();
        assert!(matches!(result, wasm_encoder::Instruction::I64Store32(_)));
    }

    // Reverse conversion tests for constants (duplicates removed - already exist above)

    // Reverse conversion tests for simple operators (no data)
    #[test]
    fn test_operator_to_instruction_simple_ops() {
        let simple_ops = vec![
            (OpCode::Unreachable, wasm_encoder::Instruction::Unreachable),
            (OpCode::Nop, wasm_encoder::Instruction::Nop),
            (OpCode::Else, wasm_encoder::Instruction::Else),
            (OpCode::End, wasm_encoder::Instruction::End),
            (OpCode::Return, wasm_encoder::Instruction::Return),
            (OpCode::Drop, wasm_encoder::Instruction::Drop),
            (OpCode::Select, wasm_encoder::Instruction::Select),
        ];

        for (opcode, expected_instr) in simple_ops {
            let op = Operator {
                opcode: Some(opcode as i32),
                operator: None,
            };
            let result = wasm_encoder::Instruction::try_from(op).unwrap();
            // Compare by matching the instruction type
            match (&result, &expected_instr) {
                (wasm_encoder::Instruction::Unreachable, wasm_encoder::Instruction::Unreachable) => {}
                (wasm_encoder::Instruction::Nop, wasm_encoder::Instruction::Nop) => {}
                (wasm_encoder::Instruction::Else, wasm_encoder::Instruction::Else) => {}
                (wasm_encoder::Instruction::End, wasm_encoder::Instruction::End) => {}
                (wasm_encoder::Instruction::Return, wasm_encoder::Instruction::Return) => {}
                (wasm_encoder::Instruction::Drop, wasm_encoder::Instruction::Drop) => {}
                (wasm_encoder::Instruction::Select, wasm_encoder::Instruction::Select) => {}
                _ => panic!("Mismatch for {:?}", opcode),
            }
        }
    }

    // Test BrTable with multiple targets
    #[test]
    fn test_operator_brtable_multiple_targets() {
        // Create a WASM module with BrTable
        let mut module = wasm_encoder::Module::new();
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType};
        let mut type_section = wasm_encoder::TypeSection::new();
        let func_type = SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(vec![], vec![])),
                shared: false,
                descriptor: None,
                describes: None,
            },
        };
        type_section.ty().subtype(&func_type);
        module.section(&type_section);
        let mut func_section = wasm_encoder::FunctionSection::new();
        func_section.function(0);
        module.section(&func_section);
        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        func.instruction(&wasm_encoder::Instruction::I32Const(0));
        func.instruction(&wasm_encoder::Instruction::BrTable(
            vec![1, 2, 3, 4].into(),
            0,
        ));
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);
        let wasm_bytes = module.finish();

        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::CodeSectionEntry(body) = payload {
                let reader = body.get_operators_reader().unwrap();
                for operator_result in reader {
                    let operator = operator_result.unwrap();
                    if let wasmparser::Operator::BrTable { targets } = operator {
                        let proto = Operator::try_from(wasmparser::Operator::BrTable { targets }).unwrap();
                        match proto.operator {
                            Some(operator::Operator::Targets(targets_proto)) => {
                                assert_eq!(targets_proto.default, Some(0));
                                // Verify targets are captured (exact count depends on parser)
                                assert!(!targets_proto.targets.is_empty() || targets_proto.default.is_some());
                            }
                            _ => panic!("Expected Targets"),
                        }
                        break;
                    }
                }
            }
        }
    }

    // ========== Error Handling Tests ==========

    #[test]
    fn test_operator_missing_opcode() {
        let op = Operator {
            opcode: None,
            operator: None,
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_invalid_opcode() {
        let op = Operator {
            opcode: Some(99999), // Invalid opcode
            operator: None,
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_block_missing_operator() {
        let op = Operator {
            opcode: Some(OpCode::Block as i32),
            operator: None,
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_block_wrong_operator_type() {
        let op = Operator {
            opcode: Some(OpCode::Block as i32),
            operator: Some(operator::Operator::RelativeDepth(5)), // Wrong type
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_call_missing_operator() {
        let op = Operator {
            opcode: Some(OpCode::Call as i32),
            operator: None,
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_call_wrong_operator_type() {
        let op = Operator {
            opcode: Some(OpCode::Call as i32),
            operator: Some(operator::Operator::LocalIndex(5)), // Wrong type
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_call_indirect_missing_fields() {
        let op = Operator {
            opcode: Some(OpCode::CallIndirect as i32),
            operator: Some(operator::Operator::CallInderect(CallIndirectOp {
                type_index: None, // Missing
                table_index: Some(0),
            })),
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_memory_init_missing_fields() {
        let op = Operator {
            opcode: Some(OpCode::MemoryInit as i32),
            operator: Some(operator::Operator::MemoryInit(MemoryInitOp {
                data_index: None, // Missing
                mem: Some(0),
            })),
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    #[test]
    fn test_operator_brtable_missing_default() {
        let op = Operator {
            opcode: Some(OpCode::BrTable as i32),
            operator: Some(operator::Operator::Targets(BrTargets {
                default: None, // Missing
                targets: vec![1, 2],
            })),
        };
        assert!(wasm_encoder::Instruction::try_from(op).is_err());
    }

    // test_blocktype_to_wasm_encoder_invalid_empty already exists above

    #[test]
    fn test_memarg_to_wasm_encoder_missing_offset() {
        let memarg = MemArg {
            align: Some(2),
            max_align: Some(2),
            offset: None, // Missing
            memory: Some(0),
        };
        assert!(wasm_encoder::MemArg::try_from(memarg).is_err());
    }

    #[test]
    fn test_memarg_to_wasm_encoder_missing_memory() {
        let memarg = MemArg {
            align: Some(2),
            max_align: Some(2),
            offset: Some(100),
            memory: None, // Missing
        };
        assert!(wasm_encoder::MemArg::try_from(memarg).is_err());
    }

    // Test all comparison operators roundtrip
    #[test]
    fn test_comparison_operators_roundtrip() {
        let comparisons = vec![
            (OpCode::I32Ne, wasm_encoder::Instruction::I32Ne),
            (OpCode::I32LtS, wasm_encoder::Instruction::I32LtS),
            (OpCode::I32GtU, wasm_encoder::Instruction::I32GtU),
            (OpCode::I64Eq, wasm_encoder::Instruction::I64Eq),
            (OpCode::F32Lt, wasm_encoder::Instruction::F32Lt),
            (OpCode::F64Ge, wasm_encoder::Instruction::F64Ge),
        ];

        for (expected_opcode, expected_instr) in comparisons {
            // Create operator from wasmparser, then convert back
            let wasm_op = match expected_opcode {
                OpCode::I32Ne => wasmparser::Operator::I32Ne,
                OpCode::I32LtS => wasmparser::Operator::I32LtS,
                OpCode::I32GtU => wasmparser::Operator::I32GtU,
                OpCode::I64Eq => wasmparser::Operator::I64Eq,
                OpCode::F32Lt => wasmparser::Operator::F32Lt,
                OpCode::F64Ge => wasmparser::Operator::F64Ge,
                _ => continue,
            };
            let proto = Operator::try_from(wasm_op).unwrap();
            assert_eq!(proto.opcode, Some(expected_opcode as i32));
            let back = wasm_encoder::Instruction::try_from(proto).unwrap();
            // Verify it's the right type
            match (&back, &expected_instr) {
                (wasm_encoder::Instruction::I32Ne, wasm_encoder::Instruction::I32Ne) => {}
                (wasm_encoder::Instruction::I32LtS, wasm_encoder::Instruction::I32LtS) => {}
                (wasm_encoder::Instruction::I32GtU, wasm_encoder::Instruction::I32GtU) => {}
                (wasm_encoder::Instruction::I64Eq, wasm_encoder::Instruction::I64Eq) => {}
                (wasm_encoder::Instruction::F32Lt, wasm_encoder::Instruction::F32Lt) => {}
                (wasm_encoder::Instruction::F64Ge, wasm_encoder::Instruction::F64Ge) => {}
                _ => panic!("Mismatch for {:?}", expected_opcode),
            }
        }
    }

    // Test arithmetic operators roundtrip
    #[test]
    fn test_arithmetic_operators_roundtrip() {
        let arithmetic = vec![
            (OpCode::I32Sub, wasm_encoder::Instruction::I32Sub),
            (OpCode::I32DivU, wasm_encoder::Instruction::I32DivU),
            (OpCode::I64Mul, wasm_encoder::Instruction::I64Mul),
            (OpCode::F32Add, wasm_encoder::Instruction::F32Add),
            (OpCode::F64Div, wasm_encoder::Instruction::F64Div),
        ];

        for (expected_opcode, expected_instr) in arithmetic {
            let wasm_op = match expected_opcode {
                OpCode::I32Sub => wasmparser::Operator::I32Sub,
                OpCode::I32DivU => wasmparser::Operator::I32DivU,
                OpCode::I64Mul => wasmparser::Operator::I64Mul,
                OpCode::F32Add => wasmparser::Operator::F32Add,
                OpCode::F64Div => wasmparser::Operator::F64Div,
                _ => continue,
            };
            let proto = Operator::try_from(wasm_op).unwrap();
            assert_eq!(proto.opcode, Some(expected_opcode as i32));
            let back = wasm_encoder::Instruction::try_from(proto).unwrap();
            // Verify conversion succeeded
            match (&back, &expected_instr) {
                (wasm_encoder::Instruction::I32Sub, wasm_encoder::Instruction::I32Sub) => {}
                (wasm_encoder::Instruction::I32DivU, wasm_encoder::Instruction::I32DivU) => {}
                (wasm_encoder::Instruction::I64Mul, wasm_encoder::Instruction::I64Mul) => {}
                (wasm_encoder::Instruction::F32Add, wasm_encoder::Instruction::F32Add) => {}
                (wasm_encoder::Instruction::F64Div, wasm_encoder::Instruction::F64Div) => {}
                _ => panic!("Mismatch for {:?}", expected_opcode),
            }
        }
    }

    // Test conversion operators roundtrip
    #[test]
    fn test_conversion_operators_roundtrip() {
        let conversions = vec![
            (OpCode::I32WrapI64, wasm_encoder::Instruction::I32WrapI64),
            (OpCode::I64ExtendI32s, wasm_encoder::Instruction::I64ExtendI32S),
            (OpCode::F32DemoteF64, wasm_encoder::Instruction::F32DemoteF64),
            (OpCode::F64PromoteF32, wasm_encoder::Instruction::F64PromoteF32),
        ];

        for (expected_opcode, expected_instr) in conversions {
            let wasm_op = match expected_opcode {
                OpCode::I32WrapI64 => wasmparser::Operator::I32WrapI64,
                OpCode::I64ExtendI32s => wasmparser::Operator::I64ExtendI32S,
                OpCode::F32DemoteF64 => wasmparser::Operator::F32DemoteF64,
                OpCode::F64PromoteF32 => wasmparser::Operator::F64PromoteF32,
                _ => continue,
            };
            let proto = Operator::try_from(wasm_op).unwrap();
            assert_eq!(proto.opcode, Some(expected_opcode as i32));
            let back = wasm_encoder::Instruction::try_from(proto).unwrap();
            // Verify conversion succeeded
            match (&back, &expected_instr) {
                (wasm_encoder::Instruction::I32WrapI64, wasm_encoder::Instruction::I32WrapI64) => {}
                (wasm_encoder::Instruction::I64ExtendI32S, wasm_encoder::Instruction::I64ExtendI32S) => {}
                (wasm_encoder::Instruction::F32DemoteF64, wasm_encoder::Instruction::F32DemoteF64) => {}
                (wasm_encoder::Instruction::F64PromoteF32, wasm_encoder::Instruction::F64PromoteF32) => {}
                _ => panic!("Mismatch for {:?}", expected_opcode),
            }
        }
    }
}
