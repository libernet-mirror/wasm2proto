use crate::libernet_wasm::*;
use crate::sections::*;

use anyhow::{Ok, Result};

pub fn from_wasm(bytes: &[u8]) -> Result<ProgramModule> {
    use wasmparser::Parser;
    let mut program_module = ProgramModule {
        protocol_version: Some(1),
        ..Default::default()
    };
    let mut code_entries: Vec<CodeSectionEntry> = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload?;
        if let Some(payload) = Version::from_wasmparser(&payload)? {
            program_module.version = Some(payload);
        } else if let Some(code_entry) = CodeSectionEntry::from_wasmparser(&payload)? {
            code_entries.push(code_entry);
        } else {
            match Section::from_wasmparser(payload)? {
                Some(Section {
                    section: section::Section::TypeSection(type_section),
                }) => program_module.type_section = Some(type_section),
                Some(Section {
                    section: section::Section::ImportSection(import_section),
                }) => program_module.import_section = Some(import_section),
                Some(Section {
                    section: section::Section::FunctionSection(function_section),
                }) => program_module.function_section = Some(function_section),
                Some(Section {
                    section: section::Section::TableSection(table_section),
                }) => program_module.table_section = Some(table_section),
                Some(Section {
                    section: section::Section::MemorySection(memory_section),
                }) => program_module.memory_section = Some(memory_section),
                Some(Section {
                    section: section::Section::GlobalSection(global_section),
                }) => program_module.global_section = Some(global_section),
                Some(Section {
                    section: section::Section::ExportSection(export_section),
                }) => program_module.export_section = Some(export_section),
                Some(Section {
                    section: section::Section::ElementSection(element_section),
                }) => program_module.element_section = Some(element_section),
                Some(Section {
                    section: section::Section::DataSection(data_section),
                }) => program_module.data_section = Some(data_section),
                Some(Section {
                    section: section::Section::TagSection(tag_section),
                }) => program_module.tag_section = Some(tag_section),
                None => {}
            }
        }
    }

    if !code_entries.is_empty() {
        program_module.code_section = Some(CodeSection {
            code_section_entry: code_entries,
        });
    }

    Ok(program_module)
}

pub fn render_wasm(program: ProgramModule) -> Result<Vec<u8>> {
    use wasm_encoder::Module;
    let mut module: Module = Module::new();
    program
        .type_section
        .map(|section| section.render_wasm(&mut module));
    program
        .import_section
        .map(|section| section.render_wasm(&mut module));
    program
        .function_section
        .map(|section| section.render_wasm(&mut module));
    program
        .table_section
        .map(|section| section.render_wasm(&mut module));
    program
        .memory_section
        .map(|section| section.render_wasm(&mut module));
    program
        .tag_section
        .map(|section| section.render_wasm(&mut module));
    program
        .global_section
        .map(|section| section.render_wasm(&mut module));
    program
        .export_section
        .map(|section| section.render_wasm(&mut module));
    program
        .element_section
        .map(|section| section.render_wasm(&mut module));
    program
        .code_section
        .map(|section| section.render_wasm(&mut module));
    program
        .data_section
        .map(|section| section.render_wasm(&mut module));
    Ok(module.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{CodeSection, Function, Instruction, Module, TypeSection, ValType};
    use wasmparser;

    /// Creates a minimal valid WASM module for testing
    fn create_minimal_wasm_module() -> Vec<u8> {
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType};
        let mut module = Module::new();

        // Type section: (func)
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
        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        // Code section
        let mut code = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::End);
        code.function(&func);
        module.section(&code);

        module.finish()
    }

    /// Creates a WASM module with exports for testing
    fn create_wasm_module_with_exports() -> Vec<u8> {
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType};
        let mut module = Module::new();

        // Type section: (func) -> i32
        let mut types = TypeSection::new();
        types.ty().subtype(&SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(
                    vec![],
                    vec![ValType::I32],
                )),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });
        module.section(&types);

        // Function section
        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        // Export section
        let mut exports = wasm_encoder::ExportSection::new();
        exports.export("test", wasm_encoder::ExportKind::Func, 0);
        module.section(&exports);

        // Code section
        let mut code = CodeSection::new();
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::I32Const(42));
        func.instruction(&Instruction::End);
        code.function(&func);
        module.section(&code);

        module.finish()
    }

    #[test]
    fn test_from_wasm_valid_module() {
        let wasm_bytes = create_minimal_wasm_module();
        let result = from_wasm(&wasm_bytes);

        assert!(result.is_ok());
        let program = result.unwrap();

        // Check protocol version
        assert_eq!(program.protocol_version, Some(1));

        // Check version
        assert!(program.version.is_some());
        let version = program.version.unwrap();
        assert_eq!(version.r#number, Some(1));
        assert_eq!(version.encoding, Some(wasmparser::Encoding::Module as i32));

        // Check sections
        assert!(program.type_section.is_some());
        assert!(program.function_section.is_some());
        assert!(program.code_section.is_some());
    }

    #[test]
    fn test_from_wasm_empty_bytes() {
        let empty_bytes = vec![];
        let result = from_wasm(&empty_bytes);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Version not found") || error_msg.contains("unexpected end"));
    }

    #[test]
    fn test_from_wasm_invalid_bytes() {
        let invalid_bytes = vec![0x00, 0x01, 0x02, 0x03];
        let result = from_wasm(&invalid_bytes);

        // Should fail parsing invalid WASM
        assert!(result.is_err());
    }

    #[test]
    fn test_from_wasm_module_with_exports() {
        let wasm_bytes = create_wasm_module_with_exports();
        let result = from_wasm(&wasm_bytes);

        assert!(result.is_ok());
        let program = result.unwrap();

        // Should have version
        assert!(program.version.is_some());

        // Should have sections including export section
        assert!(program.export_section.is_some());
    }

    #[test]
    fn test_render_wasm_valid_program() {
        let wasm_bytes = create_minimal_wasm_module();
        let program = from_wasm(&wasm_bytes).unwrap();

        let result = render_wasm(program);

        assert!(result.is_ok());
        let rendered_bytes = result.unwrap();
        assert!(!rendered_bytes.is_empty());

        // Rendered WASM should be valid
        assert!(
            wasmparser::Parser::new(0)
                .parse_all(&rendered_bytes)
                .next()
                .is_some()
        );
    }

    #[test]
    fn test_render_wasm_empty_sections() {
        let mut program = ProgramModule::default();
        program.protocol_version = Some(1);
        program.version = Some(Version {
            r#number: Some(1),
            encoding: Some(1),
        });

        let result = render_wasm(program);

        assert!(result.is_ok());
        let rendered_bytes = result.unwrap();
        assert!(!rendered_bytes.is_empty());
    }

    #[test]
    fn test_round_trip_conversion() {
        let original_wasm = create_minimal_wasm_module();

        // Convert WASM -> ProgramModule
        let program = from_wasm(&original_wasm).unwrap();

        // Convert ProgramModule -> WASM
        let rendered_wasm = render_wasm(program).unwrap();

        // Verify rendered WASM is valid
        assert!(!rendered_wasm.is_empty());

        // Convert back to ProgramModule to verify it's still valid
        let round_trip_program = from_wasm(&rendered_wasm);
        assert!(round_trip_program.is_ok());

        let round_trip = round_trip_program.unwrap();
        assert_eq!(round_trip.protocol_version, Some(1));
        assert!(round_trip.version.is_some());
    }

    #[test]
    fn test_round_trip_with_exports() {
        let original_wasm = create_wasm_module_with_exports();

        // Convert WASM -> ProgramModule
        let program = from_wasm(&original_wasm).unwrap();

        // Convert ProgramModule -> WASM
        let rendered_wasm = render_wasm(program).unwrap();

        // Verify rendered WASM is valid and can be parsed again
        let round_trip_program = from_wasm(&rendered_wasm);
        assert!(round_trip_program.is_ok());

        let round_trip = round_trip_program.unwrap();
        assert!(round_trip.export_section.is_some());
    }

    #[test]
    fn test_program_module_structure() {
        let wasm_bytes = create_minimal_wasm_module();
        let program = from_wasm(&wasm_bytes).unwrap();

        // Verify structure
        assert_eq!(program.protocol_version, Some(1));
        assert!(program.version.is_some());

        let version = program.version.unwrap();
        assert_eq!(version.r#number, Some(1));
        assert_eq!(version.encoding, Some(wasmparser::Encoding::Module as i32));
    }
}
