use anyhow::{Ok, Result, anyhow, bail};

use crate::program::*;

impl TryFrom<wasmparser::RefType> for ERefType {
    type Error = anyhow::Error;

    fn try_from(ref_type: wasmparser::RefType) -> Result<Self> {
        use wasmparser::{AbstractHeapType, HeapType};
        match ref_type.heap_type() {
            HeapType::Abstract {
                shared: false,
                ty: AbstractHeapType::Func,
            } => Ok(ERefType::RefFunc),
            _ => {
                bail!("Only reffunc elements are supported");
            }
        }
    }
}

impl TryFrom<ERefType> for wasm_encoder::RefType {
    type Error = anyhow::Error;

    fn try_from(_ref_type: ERefType) -> Result<Self> {
        Ok(wasm_encoder::RefType::FUNCREF)
    }
}

impl TryFrom<wasmparser::ValType> for ValueType {
    type Error = anyhow::Error;

    fn try_from(val_type: wasmparser::ValType) -> Result<Self> {
        use wasmparser::ValType;
        match val_type {
            ValType::I32 => Ok(ValueType {
                val_type: Some(EValueType::I32 as i32),
                ref_type: None,
            }),
            ValType::I64 => Ok(ValueType {
                val_type: Some(EValueType::I64 as i32),
                ref_type: None,
            }),
            ValType::F32 => Ok(ValueType {
                val_type: Some(EValueType::F32 as i32),
                ref_type: None,
            }),
            ValType::F64 => Ok(ValueType {
                val_type: Some(EValueType::F64 as i32),
                ref_type: None,
            }),
            ValType::V128 => Ok(ValueType {
                val_type: Some(EValueType::V128 as i32),
                ref_type: None,
            }),
            ValType::Ref(ref_type) => Ok(ValueType {
                val_type: Some(EValueType::Ref as i32),
                ref_type: Some(ERefType::try_from(ref_type)? as i32),
            }),
        }
    }
}

impl TryFrom<ValueType> for wasm_encoder::ValType {
    type Error = anyhow::Error;

    fn try_from(value_type: ValueType) -> Result<Self> {
        let ty: EValueType = value_type
            .val_type
            .ok_or(anyhow!("Value type not found"))?
            .try_into()?;
        match ty {
            EValueType::I32 => Ok(wasm_encoder::ValType::I32),
            EValueType::I64 => Ok(wasm_encoder::ValType::I64),
            EValueType::F32 => Ok(wasm_encoder::ValType::F32),
            EValueType::F64 => Ok(wasm_encoder::ValType::F64),
            EValueType::V128 => Ok(wasm_encoder::ValType::V128),
            EValueType::Ref => Ok(wasm_encoder::ValType::Ref(ERefType::try_from(value_type.ref_type.ok_or(anyhow!("Ref type not found"))?)?.try_into()?)),
        }
    }
}

impl TryFrom<wasmparser::ConstExpr<'_>> for Expression {
    type Error = anyhow::Error;

    fn try_from(expr: wasmparser::ConstExpr<'_>) -> Result<Self> {
        let reader = expr.get_operators_reader();
        let mut operators: Vec<Operator> = Vec::new();
        for operator in reader {
            operators.push(Operator::try_from(operator?)?);
        }
        Ok(Expression { operators })
    }
}

impl TryFrom<Expression> for wasm_encoder::ConstExpr {
    type Error = anyhow::Error;

    fn try_from(expression: Expression) -> Result<Self> {
        use wasm_encoder::ConstExpr;
        let mut instructions: Vec<wasm_encoder::Instruction> = Vec::new();
        for operator in expression.operators {
            instructions.push(wasm_encoder::Instruction::try_from(operator)?);
        }
        // drop last operator if it is end
        if let Some(last) = instructions.last()
            && matches!(last, wasm_encoder::Instruction::End)
        {
            instructions.pop();
        }
        Ok(ConstExpr::extended(instructions))
    }
}

impl TryFrom<wasmparser::ExternalKind> for ExternalKind {
    type Error = anyhow::Error;

    fn try_from(external_kind: wasmparser::ExternalKind) -> Result<Self> {
        match external_kind {
            wasmparser::ExternalKind::Func => Ok(ExternalKind::ExtFunc),
            wasmparser::ExternalKind::Table => Ok(ExternalKind::ExtTable),
            wasmparser::ExternalKind::Memory => Ok(ExternalKind::ExtMemory),
            wasmparser::ExternalKind::Global => Ok(ExternalKind::ExtGlobal),
            wasmparser::ExternalKind::Tag => Ok(ExternalKind::ExtTag),
            wasmparser::ExternalKind::FuncExact => Ok(ExternalKind::ExtFuncExact),
        }
    }
}

impl TryFrom<ExternalKind> for wasm_encoder::ExportKind {
    type Error = anyhow::Error;

    fn try_from(external_kind: ExternalKind) -> Result<wasm_encoder::ExportKind> {
        match external_kind {
            ExternalKind::ExtFunc => Ok(wasm_encoder::ExportKind::Func),
            ExternalKind::ExtTable => Ok(wasm_encoder::ExportKind::Table),
            ExternalKind::ExtMemory => Ok(wasm_encoder::ExportKind::Memory),
            ExternalKind::ExtGlobal => Ok(wasm_encoder::ExportKind::Global),
            ExternalKind::ExtTag => Ok(wasm_encoder::ExportKind::Tag),
            _ => Err(anyhow!("ExternalKind is not supported")),
        }
    }
}

impl TryFrom<wasmparser::ElementKind<'_>> for ElementKind {
    type Error = anyhow::Error;

    fn try_from(element_kind: wasmparser::ElementKind) -> Result<Self> {
        match element_kind {
            wasmparser::ElementKind::Passive => Ok(ElementKind {
                ty: ElementKindType::ElPassive as i32,
                table_index: None,
                expression: None,
            }),
            wasmparser::ElementKind::Active {
                table_index,
                offset_expr,
            } => Ok(ElementKind {
                ty: ElementKindType::ElActive as i32,
                table_index,
                expression: Some(Expression::try_from(offset_expr)?),
            }),
            wasmparser::ElementKind::Declared => Ok(ElementKind {
                ty: ElementKindType::ElDeclared as i32,
                table_index: None,
                expression: None,
            }),
        }
    }
}

impl TryFrom<wasmparser::DataKind<'_>> for DataKind {
    type Error = anyhow::Error;

    fn try_from(element_kind: wasmparser::DataKind) -> Result<Self> {
        match element_kind {
            wasmparser::DataKind::Passive => Ok(DataKind {
                ty: DataKindType::Passive as i32,
                memory_index: None,
                expression: None,
            }),
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => Ok(DataKind {
                ty: DataKindType::Active as i32,
                memory_index: Some(memory_index),
                expression: Some(Expression::try_from(offset_expr)?),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valtype_from_wasmparser() {
        use wasmparser::ValType;

        assert!(matches!(
            ValueType::try_from(ValType::I32).unwrap(),
            ValueType { val_type: Some(1), ref_type: None }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::I64).unwrap(),
            ValueType { val_type: Some(2), ref_type: None }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::F32).unwrap(),
            ValueType { val_type: Some(3), ref_type: None }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::F64).unwrap(),
            ValueType { val_type: Some(4), ref_type: None }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::V128).unwrap(),
            ValueType { val_type: Some(5), ref_type: None }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::Ref(wasmparser::RefType::FUNCREF)).unwrap(),
            ValueType { val_type: Some(6), ref_type: Some(1) }
        ));
    }

    #[test]
    fn test_valtype_to_wasm_encoder() {
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType { val_type: Some(EValueType::I32 as i32), ref_type: None }).unwrap(),
            wasm_encoder::ValType::I32
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType { val_type: Some(EValueType::I64 as i32), ref_type: None }).unwrap(),
            wasm_encoder::ValType::I64
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType { val_type: Some(EValueType::F32 as i32), ref_type: None }).unwrap(),
            wasm_encoder::ValType::F32
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType { val_type: Some(EValueType::F64 as i32), ref_type: None }).unwrap(),
            wasm_encoder::ValType::F64
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType { val_type: Some(EValueType::V128 as i32), ref_type: None }).unwrap(),
            wasm_encoder::ValType::V128
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType { val_type: Some(EValueType::Ref as i32), ref_type: Some(ERefType::RefFunc as i32) }).unwrap(),
            wasm_encoder::ValType::Ref(wasm_encoder::RefType::FUNCREF)
        ));
    }

    #[test]
    fn test_external_kind_from_wasmparser() {
        use wasmparser::ExternalKind as WasmParserExternalKind;

        assert_eq!(
            ExternalKind::try_from(WasmParserExternalKind::Func).unwrap(),
            ExternalKind::ExtFunc
        );
        assert_eq!(
            ExternalKind::try_from(WasmParserExternalKind::Table).unwrap(),
            ExternalKind::ExtTable
        );
        assert_eq!(
            ExternalKind::try_from(WasmParserExternalKind::Memory).unwrap(),
            ExternalKind::ExtMemory
        );
        assert_eq!(
            ExternalKind::try_from(WasmParserExternalKind::Global).unwrap(),
            ExternalKind::ExtGlobal
        );
        assert_eq!(
            ExternalKind::try_from(WasmParserExternalKind::Tag).unwrap(),
            ExternalKind::ExtTag
        );
        assert_eq!(
            ExternalKind::try_from(WasmParserExternalKind::FuncExact).unwrap(),
            ExternalKind::ExtFuncExact
        );
    }

    #[test]
    fn test_external_kind_to_wasm_encoder() {
        let result = wasm_encoder::ExportKind::try_from(ExternalKind::ExtFunc).unwrap();
        assert!(matches!(result, wasm_encoder::ExportKind::Func));

        let result = wasm_encoder::ExportKind::try_from(ExternalKind::ExtTable).unwrap();
        assert!(matches!(result, wasm_encoder::ExportKind::Table));

        let result = wasm_encoder::ExportKind::try_from(ExternalKind::ExtMemory).unwrap();
        assert!(matches!(result, wasm_encoder::ExportKind::Memory));

        let result = wasm_encoder::ExportKind::try_from(ExternalKind::ExtGlobal).unwrap();
        assert!(matches!(result, wasm_encoder::ExportKind::Global));

        let result = wasm_encoder::ExportKind::try_from(ExternalKind::ExtTag).unwrap();
        assert!(matches!(result, wasm_encoder::ExportKind::Tag));

        // Test unsupported ExtFuncExact
        assert!(wasm_encoder::ExportKind::try_from(ExternalKind::ExtFuncExact).is_err());
    }

}
