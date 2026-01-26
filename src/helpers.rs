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
        let mut r = expr.get_binary_reader();
        let mut bytecode = r.read_bytes(r.bytes_remaining())?.to_vec();
        // remove end opcode, wasmencoder adds it automatically
        if bytecode.last() == Some(&0x0B) {
            bytecode.pop();
        }
        if !r.eof() {
            bail!("Unexpected bytes remaining in expression");
        }
        Ok(Expression { bytecode })
    }
}

impl TryFrom<wasmparser::BinaryReader<'_>> for Expression {
    type Error = anyhow::Error;

    fn try_from(mut r: wasmparser::BinaryReader<'_>) -> Result<Self> {
        let n = r.bytes_remaining();
        Ok(Expression {
            bytecode: r.read_bytes(n)?.to_vec(),
        })
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
    fn test_expression_from_const_expr() {
        // Create a WASM module with a global that has a const expression
        let mut module = wasm_encoder::Module::new();
        let mut global_section = wasm_encoder::GlobalSection::new();
        global_section.global(
            wasm_encoder::GlobalType {
                val_type: wasm_encoder::ValType::I32,
                mutable: false,
                shared: false,
            },
            &wasm_encoder::ConstExpr::raw(vec![0x41, 0x2A, 0x0B]), // i32.const 42, end
        );
        module.section(&global_section);
        let wasm_bytes = module.finish();

        // Parse to get the ConstExpr
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::GlobalSection(section) = payload {
                for global_result in section {
                    match global_result {
                        std::result::Result::Ok(global) => {
                            let init_expr = global.init_expr;
                            let expr = Expression::try_from(init_expr).unwrap();
                            // End opcode (0x0B) should be removed
                            assert_eq!(expr.bytecode, vec![0x41, 0x2A]);
                            break;
                        }
                        std::result::Result::Err(_) => continue,
                    }
                }
            }
        }
    }

    #[test]
    fn test_expression_from_binary_reader() {
        let bytecode = vec![0x41, 0x2A, 0x0B]; // i32.const 42, end
        let reader = wasmparser::BinaryReader::new(&bytecode, 0);
        let expr = Expression::try_from(reader).unwrap();
        assert_eq!(expr.bytecode, bytecode);
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
    fn test_element_kind_from_wasmparser() {
        // Test Passive
        let passive = wasmparser::ElementKind::Passive;
        let result = ElementKind::try_from(passive).unwrap();
        assert_eq!(result.ty, ElementKindType::ElPassive as i32);
        assert!(result.table_index.is_none());
        assert!(result.expression.is_none());

        // Test Declared
        let declared = wasmparser::ElementKind::Declared;
        let result = ElementKind::try_from(declared).unwrap();
        assert_eq!(result.ty, ElementKindType::ElDeclared as i32);
        assert!(result.table_index.is_none());
        assert!(result.expression.is_none());

        // Test Active with offset expression - create a WASM module with element section
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
        let mut element_section = wasm_encoder::ElementSection::new();
        element_section.segment(wasm_encoder::ElementSegment {
            mode: wasm_encoder::ElementMode::Active {
                table: Some(0),
                offset: &wasm_encoder::ConstExpr::raw(vec![0x41, 0x00, 0x0B]), // i32.const 0, end
            },
            elements: wasm_encoder::Elements::Functions(vec![].into()),
        });
        module.section(&element_section);
        let wasm_bytes = module.finish();

        // Parse to get the ElementKind
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::ElementSection(section) = payload {
                for element_result in section {
                    match element_result {
                        std::result::Result::Ok(element) => {
                            let kind = element.kind;
                            let result = ElementKind::try_from(kind).unwrap();
                            assert_eq!(result.ty, ElementKindType::ElActive as i32);
                            assert_eq!(result.table_index, Some(0));
                            assert!(result.expression.is_some());
                            // Verify the expression has the end opcode removed
                            assert_eq!(
                                result.expression.unwrap().bytecode,
                                vec![0x41, 0x00]
                            );
                            break;
                        }
                        std::result::Result::Err(_) => continue,
                    }
                }
            }
        }
    }

    #[test]
    fn test_data_kind_from_wasmparser() {
        // Test Passive
        let passive = wasmparser::DataKind::Passive;
        let result = DataKind::try_from(passive).unwrap();
        assert_eq!(result.ty, DataKindType::Passive as i32);
        assert!(result.memory_index.is_none());
        assert!(result.expression.is_none());

        // Test Active with offset expression - create a WASM module with data section
        let mut module = wasm_encoder::Module::new();
        let mut memory_section = wasm_encoder::MemorySection::new();
        memory_section.memory(wasm_encoder::MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memory_section);
        let mut data_section = wasm_encoder::DataSection::new();
        data_section.segment(wasm_encoder::DataSegment {
            mode: wasm_encoder::DataSegmentMode::Active {
                memory_index: 0,
                offset: &wasm_encoder::ConstExpr::raw(vec![0x41, 0x00, 0x0B]), // i32.const 0, end
            },
            data: vec![],
        });
        module.section(&data_section);
        let wasm_bytes = module.finish();

        // Parse to get the DataKind
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let wasmparser::Payload::DataSection(section) = payload {
                for data_result in section {
                    match data_result {
                        std::result::Result::Ok(data) => {
                            let kind = data.kind;
                            let result = DataKind::try_from(kind).unwrap();
                            assert_eq!(result.ty, DataKindType::Active as i32);
                            assert_eq!(result.memory_index, Some(0));
                            assert!(result.expression.is_some());
                            // Verify the expression has the end opcode removed
                            assert_eq!(
                                result.expression.unwrap().bytecode,
                                vec![0x41, 0x00]
                            );
                            break;
                        }
                        std::result::Result::Err(_) => continue,
                    }
                }
            }
        }
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


    #[test]
    fn test_expression_from_binary_reader_preserves_all_bytes() {
        // BinaryReader path should preserve all bytes including end opcode
        let bytecode = vec![0x41, 0x2A, 0x0B, 0xFF]; // i32.const 42, end, extra byte
        let reader = wasmparser::BinaryReader::new(&bytecode, 0);
        let expr = Expression::try_from(reader).unwrap();
        assert_eq!(expr.bytecode, bytecode);
    }
}
