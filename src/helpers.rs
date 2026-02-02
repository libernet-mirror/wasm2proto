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
            HeapType::Abstract {
                shared: false,
                ty: AbstractHeapType::Extern,
            } => Ok(ERefType::ExternRef),
            _ => {
                bail!("Only reffunc elements are supported");
            }
        }
    }
}

impl TryFrom<ERefType> for wasm_encoder::RefType {
    type Error = anyhow::Error;

    fn try_from(ref_type: ERefType) -> Result<Self> {
        match ref_type {
            ERefType::RefFunc => Ok(wasm_encoder::RefType::FUNCREF),
            ERefType::ExternRef => Ok(wasm_encoder::RefType::EXTERNREF),
        }
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
            EValueType::Ref => Ok(wasm_encoder::ValType::Ref(
                ERefType::try_from(value_type.ref_type.ok_or(anyhow!("Ref type not found"))?)?
                    .try_into()?,
            )),
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
                ty: Some(ElementKindType::ElPassive as i32),
                table_index: None,
                expression: None,
            }),
            wasmparser::ElementKind::Active {
                table_index,
                offset_expr,
            } => Ok(ElementKind {
                ty: Some(ElementKindType::ElActive as i32),
                table_index,
                expression: Some(Expression::try_from(offset_expr)?),
            }),
            wasmparser::ElementKind::Declared => Ok(ElementKind {
                ty: Some(ElementKindType::ElDeclared as i32),
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
                ty: Some(DataKindType::Passive as i32),
                memory_index: None,
                expression: None,
            }),
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => Ok(DataKind {
                ty: Some(DataKindType::Active as i32),
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
            ValueType {
                val_type: Some(1),
                ref_type: None
            }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::I64).unwrap(),
            ValueType {
                val_type: Some(2),
                ref_type: None
            }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::F32).unwrap(),
            ValueType {
                val_type: Some(3),
                ref_type: None
            }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::F64).unwrap(),
            ValueType {
                val_type: Some(4),
                ref_type: None
            }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::V128).unwrap(),
            ValueType {
                val_type: Some(5),
                ref_type: None
            }
        ));
        assert!(matches!(
            ValueType::try_from(ValType::Ref(wasmparser::RefType::FUNCREF)).unwrap(),
            ValueType {
                val_type: Some(6),
                ref_type: Some(1)
            }
        ));
    }

    #[test]
    fn test_valtype_to_wasm_encoder() {
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType {
                val_type: Some(EValueType::I32 as i32),
                ref_type: None
            })
            .unwrap(),
            wasm_encoder::ValType::I32
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType {
                val_type: Some(EValueType::I64 as i32),
                ref_type: None
            })
            .unwrap(),
            wasm_encoder::ValType::I64
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType {
                val_type: Some(EValueType::F32 as i32),
                ref_type: None
            })
            .unwrap(),
            wasm_encoder::ValType::F32
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType {
                val_type: Some(EValueType::F64 as i32),
                ref_type: None
            })
            .unwrap(),
            wasm_encoder::ValType::F64
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType {
                val_type: Some(EValueType::V128 as i32),
                ref_type: None
            })
            .unwrap(),
            wasm_encoder::ValType::V128
        ));
        assert!(matches!(
            wasm_encoder::ValType::try_from(ValueType {
                val_type: Some(EValueType::Ref as i32),
                ref_type: Some(ERefType::RefFunc as i32)
            })
            .unwrap(),
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

    #[test]
    fn test_reftype_from_wasmparser_funcref() {
        let ref_type = wasmparser::RefType::FUNCREF;
        let result = ERefType::try_from(ref_type).unwrap();
        assert_eq!(result, ERefType::RefFunc);
    }

    #[test]
    fn test_reftype_from_wasmparser_externref() {
        let ref_type = wasmparser::RefType::EXTERNREF;
        let result = ERefType::try_from(ref_type).unwrap();
        assert_eq!(result, ERefType::ExternRef);
    }

    #[test]
    fn test_reftype_to_wasm_encoder_funcref() {
        let result = wasm_encoder::RefType::try_from(ERefType::RefFunc).unwrap();
        assert_eq!(result, wasm_encoder::RefType::FUNCREF);
    }

    #[test]
    fn test_reftype_to_wasm_encoder_externref() {
        let result = wasm_encoder::RefType::try_from(ERefType::ExternRef).unwrap();
        assert_eq!(result, wasm_encoder::RefType::EXTERNREF);
    }

    #[test]
    fn test_expression_from_wasmparser_constexpr() {
        use wasm_encoder::{ConstExpr, Instruction, Module};
        use wasmparser::Parser;

        // Create a simple const expression: i32.const 42
        let mut module = Module::new();
        let mut types = wasm_encoder::TypeSection::new();
        types.ty().subtype(&wasm_encoder::SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: wasm_encoder::CompositeType {
                inner: wasm_encoder::CompositeInnerType::Func(wasm_encoder::FuncType::new(
                    vec![],
                    vec![],
                )),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });
        module.section(&types);

        let mut globals = wasm_encoder::GlobalSection::new();
        let expr = ConstExpr::extended(vec![Instruction::I32Const(42), Instruction::End]);
        globals.global(
            wasm_encoder::GlobalType {
                val_type: wasm_encoder::ValType::I32,
                mutable: false,
                shared: false,
            },
            &expr,
        );
        module.section(&globals);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::GlobalSection(section) = payload {
                for global in section {
                    let global = global.unwrap();
                    let result = Expression::try_from(global.init_expr).unwrap();
                    assert!(!result.operators.is_empty());
                    return;
                }
            }
        }
        panic!("GlobalSection not found");
    }

    #[test]
    fn test_expression_to_wasm_encoder_constexpr() {
        // Create an expression with operators
        let expr = Expression {
            operators: vec![Operator {
                opcode: Some(OpCode::I32Const as i32),
                operator: Some(operator::Operator::I32value(42)),
            }],
        };

        // Verify conversion succeeds
        let result = wasm_encoder::ConstExpr::try_from(expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expression_to_wasm_encoder_constexpr_with_end() {
        // Create an expression with End operator (should be removed)
        let expr = Expression {
            operators: vec![
                Operator {
                    opcode: Some(OpCode::I32Const as i32),
                    operator: Some(operator::Operator::I32value(42)),
                },
                Operator {
                    opcode: Some(OpCode::End as i32),
                    ..Operator::default()
                },
            ],
        };

        // Verify conversion succeeds (End should be removed automatically)
        let result = wasm_encoder::ConstExpr::try_from(expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_element_kind_from_wasmparser_passive() {
        use wasmparser::ElementKind;

        let element_kind = ElementKind::Passive;
        let result = crate::program::ElementKind::try_from(element_kind).unwrap();
        assert_eq!(result.ty, Some(ElementKindType::ElPassive as i32));
        assert_eq!(result.table_index, None);
        assert_eq!(result.expression, None);
    }

    #[test]
    fn test_element_kind_from_wasmparser_active() {
        use wasm_encoder::{
            CompositeInnerType, CompositeType, ConstExpr, ElementMode, ElementSection,
            ElementSegment, Elements, FunctionSection, Instruction, Module, RefType, SubType,
            TableSection, TableType, TypeSection,
        };
        use wasmparser::Parser;

        // Create a complete WASM module with an element section
        let mut module = Module::new();

        // Type section
        let mut types = TypeSection::new();
        types.ty().subtype(&SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(vec![], vec![])),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });
        module.section(&types);

        // Function section
        let mut functions = FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        // Table section
        let mut tables = TableSection::new();
        tables.table(TableType {
            element_type: RefType::FUNCREF,
            table64: false,
            minimum: 10,
            maximum: None,
            shared: false,
        });
        module.section(&tables);

        // Element section with Active mode
        let mut elements = ElementSection::new();
        let offset = ConstExpr::extended(vec![Instruction::I32Const(0)]);
        elements.segment(ElementSegment {
            mode: ElementMode::Active {
                table: Some(0),
                offset: &offset,
            },
            elements: Elements::Functions(vec![0].into()),
        });
        module.section(&elements);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::ElementSection(section) = payload {
                for element in section {
                    let element = element.unwrap();
                    let result = crate::program::ElementKind::try_from(element.kind).unwrap();
                    assert_eq!(result.ty, Some(ElementKindType::ElActive as i32));
                    assert_eq!(result.table_index, Some(0));
                    assert!(result.expression.is_some());
                    return;
                }
            }
        }
        panic!("ElementSection not found");
    }

    #[test]
    fn test_element_kind_from_wasmparser_declared() {
        use wasmparser::ElementKind;

        let element_kind = ElementKind::Declared;
        let result = crate::program::ElementKind::try_from(element_kind).unwrap();
        assert_eq!(result.ty, Some(ElementKindType::ElDeclared as i32));
        assert_eq!(result.table_index, None);
        assert_eq!(result.expression, None);
    }

    #[test]
    fn test_data_kind_from_wasmparser_passive() {
        use wasmparser::DataKind;

        let data_kind = DataKind::Passive;
        let result = crate::program::DataKind::try_from(data_kind).unwrap();
        assert_eq!(result.ty, Some(DataKindType::Passive as i32));
        assert_eq!(result.memory_index, None);
        assert_eq!(result.expression, None);
    }

    #[test]
    fn test_data_kind_from_wasmparser_active() {
        use wasm_encoder::{
            ConstExpr, DataSection, DataSegment, DataSegmentMode, Instruction, MemorySection,
            Module,
        };
        use wasmparser::Parser;

        // Create a WASM module with a data section to get a real Active DataKind
        let mut module = Module::new();

        // Memory section (required for Active data segments)
        let mut memories = MemorySection::new();
        memories.memory(wasm_encoder::MemoryType {
            memory64: false,
            shared: false,
            minimum: 1,
            maximum: None,
            page_size_log2: None,
        });
        module.section(&memories);

        // Data section with Active mode
        let mut data = DataSection::new();
        let offset = ConstExpr::extended(vec![Instruction::I32Const(0)]);
        data.segment(DataSegment {
            mode: DataSegmentMode::Active {
                memory_index: 0,
                offset: &offset,
            },
            data: b"hello".to_vec(),
        });
        module.section(&data);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::DataSection(section) = payload {
                for data in section {
                    let data = data.unwrap();
                    let result = crate::program::DataKind::try_from(data.kind).unwrap();
                    assert_eq!(result.ty, Some(DataKindType::Active as i32));
                    assert_eq!(result.memory_index, Some(0));
                    assert!(result.expression.is_some());
                    return;
                }
            }
        }
        panic!("DataSection not found");
    }

    #[test]
    fn test_valtype_to_wasm_encoder_missing_val_type() {
        let result = wasm_encoder::ValType::try_from(ValueType {
            val_type: None,
            ref_type: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_valtype_to_wasm_encoder_ref_missing_ref_type() {
        let result = wasm_encoder::ValType::try_from(ValueType {
            val_type: Some(EValueType::Ref as i32),
            ref_type: None,
        });
        assert!(result.is_err());
    }
}
