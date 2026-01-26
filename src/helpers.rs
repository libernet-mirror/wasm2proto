use anyhow::{Ok, Result, anyhow, bail};

use crate::program::*;

impl TryFrom<wasmparser::ValType> for ValueType {
    type Error = anyhow::Error;

    fn try_from(val_type: wasmparser::ValType) -> Result<Self> {
        use wasmparser::ValType;
        match val_type {
            ValType::I32 => Ok(ValueType::I32),
            ValType::I64 => Ok(ValueType::I64),
            ValType::F32 => Ok(ValueType::F32),
            ValType::F64 => Ok(ValueType::F64),
            ValType::V128 => Ok(ValueType::V128),
            ValType::Ref(_) => Err(anyhow!("Ref types are not supported")),
        }
    }
}

impl TryFrom<ValueType> for wasm_encoder::ValType {
    type Error = anyhow::Error;

    fn try_from(value_type: ValueType) -> Result<Self> {
        match value_type {
            ValueType::I32 => Ok(wasm_encoder::ValType::I32),
            ValueType::I64 => Ok(wasm_encoder::ValType::I64),
            ValueType::F32 => Ok(wasm_encoder::ValType::F32),
            ValueType::F64 => Ok(wasm_encoder::ValType::F64),
            ValueType::V128 => Ok(wasm_encoder::ValType::V128),
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

impl TryFrom<wasmparser::RefType> for RefType {
    type Error = anyhow::Error;

    fn try_from(ref_type: wasmparser::RefType) -> Result<Self> {
        use wasmparser::{AbstractHeapType, HeapType};
        match ref_type.heap_type() {
            HeapType::Abstract {
                shared: false,
                ty: AbstractHeapType::Func,
            } => {
                // Ok
            }
            _ => {
                bail!("Only funcref elements are supported");
            }
        };

        Ok(RefType {
            shared: false,
            nullable: ref_type.is_nullable(),
            ty: AbstractRefType::RefFunc as i32,
        })
    }
}

impl TryFrom<RefType> for wasm_encoder::RefType {
    type Error = anyhow::Error;

    fn try_from(ref_type: RefType) -> Result<Self> {
        use wasm_encoder::{AbstractHeapType, HeapType};
        Ok(wasm_encoder::RefType {
            nullable: ref_type.nullable,
            heap_type: HeapType::Abstract {
                shared: ref_type.shared,
                ty: AbstractHeapType::Func,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valtype_from_wasmparser() {
        use wasmparser::ValType;

        // Test all supported value types
        assert!(matches!(
            ValueType::try_from(ValType::I32).unwrap(),
            ValueType::I32
        ));
        assert!(matches!(
            ValueType::try_from(ValType::I64).unwrap(),
            ValueType::I64
        ));
        assert!(matches!(
            ValueType::try_from(ValType::F32).unwrap(),
            ValueType::F32
        ));
        assert!(matches!(
            ValueType::try_from(ValType::F64).unwrap(),
            ValueType::F64
        ));
        assert!(matches!(
            ValueType::try_from(ValType::V128).unwrap(),
            ValueType::V128
        ));

        // Test unsupported Ref type - create a funcref type
        // We need to parse a WASM module with a ref type to get a proper RefType
        let mut module = wasm_encoder::Module::new();
        let mut table_section = wasm_encoder::TableSection::new();
        table_section.table(wasm_encoder::TableType {
            element_type: wasm_encoder::RefType::FUNCREF,
            minimum: 0,
            maximum: None,
            shared: false,
            table64: false,
        });
        module.section(&table_section);
        let wasm_bytes = module.finish();

        // Parse to get the RefType
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::TableSection(section) = payload {
                for table_result in section {
                    match table_result {
                        std::result::Result::Ok(table) => {
                            // Try to convert the ref type to ValueType - should fail
                            let element_type = table.ty.element_type;
                            let ref_val_type = ValType::Ref(element_type);
                            let result = ValueType::try_from(ref_val_type);
                            assert!(result.is_err());
                            break;
                        }
                        std::result::Result::Err(_) => continue,
                    }
                }
            }
        }
    }

    #[test]
    fn test_valtype_to_wasm_encoder() {
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType::I32).unwrap(),
            wasm_encoder::ValType::I32
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType::I64).unwrap(),
            wasm_encoder::ValType::I64
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType::F32).unwrap(),
            wasm_encoder::ValType::F32
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType::F64).unwrap(),
            wasm_encoder::ValType::F64
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType::V128).unwrap(),
            wasm_encoder::ValType::V128
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

    #[test]
    fn test_ref_type_from_wasmparser() {
        // Create a WASM module with a funcref table to get a proper RefType
        let mut module = wasm_encoder::Module::new();
        let mut table_section = wasm_encoder::TableSection::new();
        table_section.table(wasm_encoder::TableType {
            element_type: wasm_encoder::RefType::FUNCREF,
            minimum: 0,
            maximum: None,
            shared: false,
            table64: false,
        });
        module.section(&table_section);
        let wasm_bytes = module.finish();

        // Parse to get the RefType
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::TableSection(section) = payload {
                for table_result in section {
                    match table_result {
                        std::result::Result::Ok(table) => {
                            // Test valid funcref (non-shared, nullable)
                            let ref_type = table.ty.element_type;
                            let is_nullable = ref_type.is_nullable();
                            let result = RefType::try_from(ref_type).unwrap();
                            assert_eq!(result.shared, false);
                            assert_eq!(result.nullable, is_nullable);
                            assert_eq!(result.ty, AbstractRefType::RefFunc as i32);
                            break;
                        }
                        std::result::Result::Err(_) => continue,
                    }
                }
            }
        }

        // Test invalid: externref - create a module with externref
        let mut module = wasm_encoder::Module::new();
        let mut table_section = wasm_encoder::TableSection::new();
        table_section.table(wasm_encoder::TableType {
            element_type: wasm_encoder::RefType::EXTERNREF,
            minimum: 0,
            maximum: None,
            shared: false,
            table64: false,
        });
        module.section(&table_section);
        let wasm_bytes = module.finish();

        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::TableSection(section) = payload {
                for table_result in section {
                    match table_result {
                        std::result::Result::Ok(table) => {
                            // Should fail for externref
                            let element_type = table.ty.element_type;
                            let result = RefType::try_from(element_type);
                            assert!(result.is_err());
                            break;
                        }
                        std::result::Result::Err(_) => continue,
                    }
                }
            }
        }
    }

    #[test]
    fn test_ref_type_to_wasm_encoder() {
        // Test nullable funcref
        let ref_type = RefType {
            shared: false,
            nullable: true,
            ty: AbstractRefType::RefFunc as i32,
        };
        let result = wasm_encoder::RefType::try_from(ref_type).unwrap();
        assert_eq!(result.nullable, true);
        if let wasm_encoder::HeapType::Abstract { shared, ty } = result.heap_type {
            assert_eq!(shared, false);
            assert!(matches!(ty, wasm_encoder::AbstractHeapType::Func));
        } else {
            panic!("Expected Abstract heap type");
        }

        // Test non-nullable funcref
        let ref_type = RefType {
            shared: false,
            nullable: false,
            ty: AbstractRefType::RefFunc as i32,
        };
        let result = wasm_encoder::RefType::try_from(ref_type).unwrap();
        assert_eq!(result.nullable, false);
        if let wasm_encoder::HeapType::Abstract { shared, ty } = result.heap_type {
            assert_eq!(shared, false);
            assert!(matches!(ty, wasm_encoder::AbstractHeapType::Func));
        } else {
            panic!("Expected Abstract heap type");
        }
    }
}
