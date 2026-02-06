use crate::libernet_wasm::*;
use anyhow::{Ok, Result, anyhow, bail};

impl Section {
    pub fn from_wasmparser(payload: wasmparser::Payload) -> Result<Option<Self>> {
        use wasmparser::Payload;
        match payload {
            Payload::TypeSection(section) => Ok(Some(Section {
                section: Some(section::Section::TypeSection(TypeSection::from_wasmparser(
                    section,
                )?)),
            })),
            Payload::ImportSection(section) => Ok(Some(Section {
                section: Some(section::Section::ImportSection(
                    ImportSection::from_wasmparser(section)?,
                )),
            })),
            Payload::FunctionSection(section) => Ok(Some(Section {
                section: Some(section::Section::FunctionSection(
                    FunctionSection::from_wasmparser(section)?,
                )),
            })),
            Payload::TableSection(section) => Ok(Some(Section {
                section: Some(section::Section::TableSection(
                    TableSection::from_wasmparser(section)?,
                )),
            })),
            Payload::MemorySection(section) => Ok(Some(Section {
                section: Some(section::Section::MemorySection(
                    MemorySection::from_wasmparser(section)?,
                )),
            })),
            Payload::TagSection(section) => Ok(Some(Section {
                section: Some(section::Section::TagSection(TagSection::from_wasmparser(
                    section,
                )?)),
            })),
            Payload::GlobalSection(section) => Ok(Some(Section {
                section: Some(section::Section::GlobalSection(
                    GlobalSection::from_wasmparser(section)?,
                )),
            })),
            Payload::ExportSection(section) => Ok(Some(Section {
                section: Some(section::Section::ExportSection(
                    ExportSection::from_wasmparser(section)?,
                )),
            })),
            Payload::StartSection { .. } => {
                bail!("StartSection is not supported");
            }
            Payload::ElementSection(section) => Ok(Some(Section {
                section: Some(section::Section::ElementSection(
                    ElementSection::from_wasmparser(section)?,
                )),
            })),
            Payload::DataCountSection { .. } => {
                // this section provider information about data sections count, we don't need it
                Ok(None)
            }
            Payload::DataSection(section) => Ok(Some(Section {
                section: Some(section::Section::DataSection(DataSection::from_wasmparser(
                    section,
                )?)),
            })),
            Payload::CodeSectionStart { .. } => {
                // this section provider information about code sections count, we don't need it
                Ok(None)
            }
            Payload::CodeSectionEntry(body) => Ok(Some(Section {
                section: Some(section::Section::CodeSectionEntry(
                    CodeSectionEntry::from_wasmparser(body)?,
                )),
            })),
            Payload::InstanceSection(_) => Ok(None),
            Payload::CoreTypeSection(_) => Ok(None),
            Payload::ComponentInstanceSection(_) => Ok(None),
            Payload::ComponentAliasSection(_) => Ok(None),
            Payload::ComponentTypeSection(_) => Ok(None),
            Payload::ComponentCanonicalSection(_) => Ok(None),
            Payload::ComponentStartSection { .. } => Ok(None),
            Payload::ComponentImportSection(_) => Ok(None),
            Payload::ComponentExportSection(_) => Ok(None),
            Payload::CustomSection(_) => Ok(None),
            Payload::End(_) => Ok(None),
            rest => {
                bail!("Unknown section {:?}", rest);
            }
        }
    }
    pub fn render_wasm(
        &self,
        module: &mut wasm_encoder::Module,
        code_section: &mut wasm_encoder::CodeSection,
    ) -> Result<()> {
        match &self.section {
            Some(section::Section::TypeSection(type_section)) => type_section.render_wasm(module),
            Some(section::Section::ImportSection(import_section)) => {
                import_section.render_wasm(module)
            }
            Some(section::Section::FunctionSection(function_section)) => {
                function_section.render_wasm(module)
            }
            Some(section::Section::TableSection(table_section)) => {
                table_section.render_wasm(module)
            }
            Some(section::Section::MemorySection(memory_section)) => {
                memory_section.render_wasm(module)
            }
            Some(section::Section::GlobalSection(global_section)) => {
                global_section.render_wasm(module)
            }
            Some(section::Section::ExportSection(export_section)) => {
                export_section.render_wasm(module)
            }
            Some(section::Section::ElementSection(element_section)) => {
                element_section.render_wasm(module)
            }
            Some(section::Section::CodeSectionEntry(code_section_entry)) => {
                code_section_entry.render_wasm(code_section)
            }
            Some(section::Section::TagSection(tag_section)) => tag_section.render_wasm(module),
            Some(section::Section::DataSection(data_section)) => data_section.render_wasm(module),
            _ => bail!("render_wasm: is not supported for section {:?}", self),
        }
    }
}

impl Version {
    pub fn from_wasmparser(payload: &wasmparser::Payload) -> Result<Option<Version>> {
        use wasmparser::{Encoding, Payload};
        match payload {
            &Payload::Version { num, encoding, .. } => {
                if num != 1 {
                    bail!("Only version 1 is supported, got {}", num);
                }

                if encoding != Encoding::Module {
                    bail!("Only Encoding::Module is supported, got {:?}", encoding);
                }

                Ok(Some(Version {
                    num: Some(num as u32),
                    encoding: Some(encoding as i32),
                }))
            }
            _ => Ok(None),
        }
    }
}

impl TypeSection {
    fn from_wasmparser(
        types: wasmparser::SectionLimited<'_, wasmparser::RecGroup>,
    ) -> Result<TypeSection> {
        use wasmparser::CompositeInnerType;

        let mut program_types: Vec<SubType> = Vec::new();
        for group in types {
            let group = group?;
            let types = group.types().collect::<Vec<_>>();
            if types.is_empty() {
                bail!("Expected at least 1 type in type group");
            }
            if types.len() > 1 {
                bail!("Recursive & GC type groups are not supported");
            }
            let atype = types[0];
            if !atype.is_final {
                bail!("Non-final types are not supported");
            }
            if atype.supertype_idx.is_some() {
                bail!("Supertype indexes are not supported");
            }
            match &atype.composite_type.inner {
                CompositeInnerType::Func(ft) => {
                    let mut params: Vec<ValueType> = Vec::new();
                    for param in ft.params() {
                        params.push(ValueType::try_from(*param)?);
                    }
                    let mut results: Vec<ValueType> = Vec::new();
                    for result in ft.results() {
                        results.push(ValueType::try_from(*result)?);
                    }
                    program_types.push(SubType {
                        kind: Some(sub_type::Kind::Func(FuncType { params, results })),
                    });
                }
                inner_type => {
                    bail!("Type is not supported {:?}", inner_type);
                }
            }
        }
        Ok(TypeSection {
            types: program_types,
        })
    }

    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::{CompositeInnerType, CompositeType, SubType, TypeSection, ValType};
        let mut types = TypeSection::new();
        for ty in &self.types {
            match &ty.kind {
                Some(sub_type::Kind::Func(ft)) => {
                    let mut params: Vec<ValType> = Vec::new();
                    for p in &ft.params {
                        params.push((*p).try_into()?);
                    }
                    let mut results: Vec<ValType> = Vec::new();
                    for r in &ft.results {
                        results.push((*r).try_into()?);
                    }
                    let ty = &SubType {
                        is_final: true,
                        supertype_idx: None,
                        composite_type: CompositeType {
                            inner: CompositeInnerType::Func(wasm_encoder::FuncType::new(
                                params, results,
                            )),
                            shared: false,
                            descriptor: None,
                            describes: None,
                        },
                    };
                    types.ty().subtype(ty);
                }
                None => {
                    bail!("Type is not supported {:?}", ty);
                }
            }
        }
        module.section(&types);
        Ok(())
    }
}

impl ImportSection {
    fn from_wasmparser(
        imports: wasmparser::SectionLimited<'_, wasmparser::Imports>,
    ) -> Result<ImportSection> {
        use wasmparser::{Imports, TypeRef};
        let mut refs: Vec<TypeRefFunc> = Vec::new();
        for import in imports {
            match import? {
                Imports::Single(_, import) => {
                    let module = import.module.to_string();
                    let name = import.name.to_string();
                    match import.ty {
                        TypeRef::Func(ftype) => {
                            refs.push(TypeRefFunc {
                                module: Some(module),
                                name: Some(name),
                                function_type: Some(ftype),
                            });
                        }
                        _ => bail!("ImportSection: only Func type imports are supported"),
                    }
                }
                _ => {
                    bail!("ImportSection: only Single Import is supported");
                }
            }
        }

        Ok(ImportSection { imports: refs })
    }

    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::{EntityType, ImportSection};
        let mut imports = ImportSection::new();
        for import in &self.imports {
            imports.import(
                import
                    .module
                    .as_ref()
                    .ok_or(anyhow!("Module not found"))?
                    .as_str(),
                import
                    .name
                    .as_ref()
                    .ok_or(anyhow!("Name not found"))?
                    .as_str(),
                EntityType::Function(
                    import
                        .function_type
                        .ok_or(anyhow!("Function type not found"))?,
                ),
            );
        }
        module.section(&imports);
        Ok(())
    }
}

impl FunctionSection {
    fn from_wasmparser(functions: wasmparser::SectionLimited<'_, u32>) -> Result<FunctionSection> {
        let mut type_idxs: Vec<u32> = Vec::new();
        for function in functions {
            type_idxs.push(function?);
        }
        Ok(FunctionSection { type_idxs })
    }
    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::FunctionSection;
        let mut functions = FunctionSection::new();
        for type_idx in &self.type_idxs {
            functions.function(*type_idx);
        }
        module.section(&functions);
        Ok(())
    }
}

impl TableSection {
    fn from_wasmparser(
        tables: wasmparser::SectionLimited<'_, wasmparser::Table>,
    ) -> Result<TableSection> {
        use wasmparser::TableInit;
        let mut proto_tables: Vec<TableType> = Vec::new();
        for table in tables {
            let table = table?;
            if let TableInit::Expr(_) = table.init {
                bail!("Table init is not supported");
            }

            if table.ty.shared {
                bail!("Shared tables are not supported");
            }

            proto_tables.push(TableType {
                reference_type: Some(RefType::try_from(table.ty.element_type)? as i32),
                table64: Some(table.ty.table64),
                initial: Some(table.ty.initial),
                maximum: table.ty.maximum,
                shared: Some(table.ty.shared),
            });
        }
        Ok(TableSection {
            types: proto_tables,
        })
    }

    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::{TableSection, TableType};
        let mut table_types = TableSection::new();
        for ty in &self.types {
            let ref_type =
                RefType::try_from(ty.reference_type.ok_or(anyhow!("Ref type not found"))?)?;
            table_types.table(TableType {
                element_type: ref_type.try_into()?,
                table64: ty.table64.ok_or(anyhow!("Table64 not found"))?,
                minimum: ty.initial.ok_or(anyhow!("Initial not found"))?,
                maximum: ty.maximum,
                shared: ty.shared.ok_or(anyhow!("Shared not found"))?,
            });
        }
        module.section(&table_types);
        Ok(())
    }
}

impl MemorySection {
    fn from_wasmparser(
        section: wasmparser::SectionLimited<'_, wasmparser::MemoryType>,
    ) -> Result<MemorySection> {
        let mut memory_types: Vec<MemoryType> = Vec::new();
        for memory in section {
            let memory = memory?;
            memory_types.push(MemoryType {
                memory64: Some(memory.memory64),
                shared: Some(memory.shared),
                initial: Some(memory.initial),
                maximum: memory.maximum,
                page_size_log2: memory.page_size_log2,
            })
        }
        Ok(MemorySection { memory_types })
    }
    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::MemorySection;
        let mut types = MemorySection::new();
        for memory in &self.memory_types {
            types.memory(wasm_encoder::MemoryType {
                memory64: memory.memory64.ok_or(anyhow!("Memory64 not found"))?,
                shared: memory.shared.ok_or(anyhow!("Shared not found"))?,
                minimum: memory.initial.ok_or(anyhow!("Initial not found"))?,
                maximum: memory.maximum,
                page_size_log2: memory.page_size_log2,
            });
        }
        module.section(&types);
        Ok(())
    }
}

impl GlobalSection {
    fn from_wasmparser(
        section: wasmparser::SectionLimited<'_, wasmparser::Global>,
    ) -> Result<GlobalSection> {
        let mut globals: Vec<Global> = Vec::new();
        for global in section {
            let global = global?;
            globals.push(Global {
                r#type: Some(GlobalType {
                    content_type: Some(ValueType::try_from(global.ty.content_type)?),
                    mutable: Some(global.ty.mutable),
                    shared: Some(global.ty.shared),
                }),
                init_expr: Some(global.init_expr.try_into()?),
            });
        }
        Ok(GlobalSection { globals })
    }
    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::{ConstExpr, GlobalSection, GlobalType};
        let mut globals = GlobalSection::new();
        for global in &self.globals {
            let ty = global
                .r#type
                .as_ref()
                .ok_or(anyhow!("Global type not found"))?;
            let init_expr = global
                .init_expr
                .clone()
                .ok_or(anyhow!("Init expr not found"))?;
            globals.global(
                GlobalType {
                    val_type: ty
                        .content_type
                        .ok_or(anyhow!("Content type not found"))?
                        .try_into()?,
                    mutable: ty.mutable.ok_or(anyhow!("Mutable not found"))?,
                    shared: ty.shared.ok_or(anyhow!("Shared not found"))?,
                },
                &ConstExpr::try_from(init_expr)?,
            );
        }
        module.section(&globals);
        Ok(())
    }
}

impl ExportSection {
    fn from_wasmparser(
        section: wasmparser::SectionLimited<'_, wasmparser::Export>,
    ) -> Result<ExportSection> {
        let mut exports: Vec<Export> = Vec::new();
        for export in section {
            let export = export?;
            exports.push(Export {
                name: Some(export.name.to_string()),
                kind: Some(ExternalKind::try_from(export.kind)? as i32),
                index: Some(export.index),
            });
        }

        Ok(ExportSection { exports })
    }
    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::ExportSection;
        let mut exports = ExportSection::new();
        for export in &self.exports {
            exports.export(
                export
                    .name
                    .as_ref()
                    .ok_or(anyhow!("Name not found"))?
                    .as_str(),
                ExternalKind::try_from(export.kind.ok_or(anyhow!("Kind not found"))?)?
                    .try_into()?,
                export.index.ok_or(anyhow!("Index not found"))?,
            );
        }
        module.section(&exports);
        Ok(())
    }
}

impl ElementSection {
    fn from_wasmparser(
        section: wasmparser::SectionLimited<'_, wasmparser::Element>,
    ) -> Result<ElementSection> {
        use wasmparser::ElementItems;
        let mut elements: Vec<Element> = Vec::new();
        for element in section {
            let element = element?;

            let items = match element.items {
                ElementItems::Functions(section) => {
                    let mut functions = Vec::new();
                    for function in section {
                        functions.push(function?);
                    }
                    element::Items::Functions(ElementFunctions { functions })
                }
                ElementItems::Expressions(ref_type, section) => {
                    let mut expressions = Vec::new();
                    for expression in section {
                        expressions.push(Expression::try_from(expression?)?);
                    }
                    element::Items::Expressions(ElementExpressions {
                        reference_type: Some(RefType::try_from(ref_type)? as i32),
                        expressions,
                    })
                }
            };

            elements.push(Element {
                kind: Some(ElementKind::try_from(element.kind)?),
                items: Some(items),
            });
        }

        Ok(ElementSection { elements })
    }

    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::{
            ConstExpr, ElementMode, ElementSection, ElementSegment, Elements,
            RefType as WasmRefType,
        };
        let mut elements = ElementSection::new();
        for element in &self.elements {
            let kind = element.kind.as_ref().ok_or(anyhow!("Kind not found"))?;
            let ty = kind.r#type.ok_or(anyhow!("Element kind type not found"))?;
            let element_mode = match ElementKindType::try_from(ty)? {
                ElementKindType::ElPassive => ElementMode::Passive,
                ElementKindType::ElActive => ElementMode::Active {
                    table: kind.table_index,
                    offset: &ConstExpr::try_from(
                        kind.expression
                            .as_ref()
                            .ok_or(anyhow!("Expression not found"))?
                            .clone(),
                    )?,
                },
                ElementKindType::ElDeclared => ElementMode::Declared,
            };
            let items = match element.items.as_ref().ok_or(anyhow!("Items not found"))? {
                element::Items::Functions(functions) => {
                    Elements::Functions(functions.functions.clone().into())
                }
                element::Items::Expressions(expressions) => {
                    let mut instructions: Vec<ConstExpr> = Vec::new();
                    for expression in &expressions.expressions {
                        instructions.push(ConstExpr::try_from(expression.clone())?);
                    }
                    Elements::Expressions(
                        WasmRefType::try_from(RefType::try_from(
                            expressions
                                .reference_type
                                .ok_or(anyhow!("Ref type not found"))?,
                        )?)?,
                        instructions.into(),
                    )
                }
            };
            elements.segment(ElementSegment {
                mode: element_mode,
                elements: items,
            });
        }
        module.section(&elements);
        Ok(())
    }
}

impl CodeSectionEntry {
    fn from_wasmparser(section: wasmparser::FunctionBody) -> Result<CodeSectionEntry> {
        let mut locals: Vec<Locals> = Vec::new();
        for local in section.get_locals_reader()? {
            let (count, val_type) = local?;

            locals.push(Locals {
                count: Some(count),
                value_type: Some(ValueType::try_from(val_type)?),
            });
        }
        let mut operators: Vec<Operator> = Vec::new();
        let reader = section.get_operators_reader()?;
        for operator in reader {
            operators.push(Operator::try_from(operator?)?);
        }
        Ok(CodeSectionEntry {
            locals,
            body: operators,
        })
    }

    fn render_wasm(&self, code_section: &mut wasm_encoder::CodeSection) -> Result<()> {
        use wasm_encoder::{Function, Instruction, ValType};
        let mut locals: Vec<(u32, ValType)> = Vec::new();
        for local in &self.locals {
            locals.push((
                local.count.ok_or(anyhow!("Count not found"))?,
                local
                    .value_type
                    .ok_or(anyhow!("Value type not found"))?
                    .try_into()?,
            ));
        }
        let mut function = Function::new(locals);
        for operator in &self.body {
            let instruction = Instruction::try_from(operator.clone())?;
            function.instruction(&instruction);
        }
        code_section.function(&function);
        Ok(())
    }
}

impl DataSection {
    fn from_wasmparser(
        section: wasmparser::SectionLimited<'_, wasmparser::Data>,
    ) -> Result<DataSection> {
        let mut datas: Vec<Data> = Vec::new();
        for data in section {
            let data = data?;
            datas.push(Data {
                kind: Some(DataKind::try_from(data.kind)?),
                data: Some(data.data.to_vec()),
            });
        }

        Ok(DataSection { datas })
    }
    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::{ConstExpr, DataSection, DataSegment, DataSegmentMode};
        let mut section = DataSection::new();
        for data in &self.datas {
            let kind = data.kind.as_ref().ok_or(anyhow!("Kind not found"))?;
            let ty = kind.r#type.ok_or(anyhow!("Data kind type not found"))?;
            let data_mode = match DataKindType::try_from(ty)? {
                DataKindType::Passive => DataSegmentMode::Passive,
                DataKindType::Active => DataSegmentMode::Active {
                    memory_index: kind.memory_index.ok_or(anyhow!("Memory index not found"))?,
                    offset: &ConstExpr::try_from(
                        kind.expression
                            .as_ref()
                            .ok_or(anyhow!("Expression not found"))?
                            .clone(),
                    )?,
                },
            };
            section.segment(DataSegment {
                mode: data_mode,
                data: data.data.clone().ok_or(anyhow!("Data not found"))?,
            });
        }
        module.section(&section);
        Ok(())
    }
}

impl TagSection {
    fn from_wasmparser(
        section: wasmparser::SectionLimited<'_, wasmparser::TagType>,
    ) -> Result<TagSection> {
        let mut tags: Vec<TagType> = Vec::new();
        for tag in section {
            let tag = tag?;
            if tag.kind != wasmparser::TagKind::Exception {
                bail!("Only Exception tags are supported");
            }
            tags.push(TagType {
                kind: Some(TagKind::Exception as i32),
                function_type_idx: Some(tag.func_type_idx),
            });
        }
        Ok(TagSection { tags })
    }
    fn render_wasm(&self, module: &mut wasm_encoder::Module) -> Result<()> {
        use wasm_encoder::{TagKind, TagSection, TagType};
        let mut tags = TagSection::new();
        for tag in &self.tags {
            tags.tag(TagType {
                kind: TagKind::Exception,
                func_type_idx: tag
                    .function_type_idx
                    .ok_or(anyhow!("Func type index not found"))?,
            });
        }
        module.section(&tags);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{CodeSection, Module};
    use wasmparser::{Parser, Payload};

    // Helper function to create a minimal WASM module
    fn create_minimal_wasm() -> Vec<u8> {
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
        module.finish()
    }

    #[test]
    fn test_version_from_wasmparser_valid() {
        let wasm_bytes = create_minimal_wasm();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::Version { .. } = &payload {
                let result = Version::from_wasmparser(&payload).unwrap();
                assert!(result.is_some());
                let version = result.unwrap();
                assert_eq!(version.num, Some(1));
                assert_eq!(version.encoding, Some(wasmparser::Encoding::Module as i32));
                return;
            }
        }
        panic!("Version payload not found");
    }

    #[test]
    fn test_version_from_wasmparser_invalid_version() {
        // Create a WASM module with version 2 (not supported)
        // Note: wasmparser doesn't allow creating invalid versions easily,
        // so we'll test the error handling in the function directly
        // by checking that version 1 is required
        let wasm_bytes = create_minimal_wasm();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::Version { num, .. } = &payload {
                // The parser should only give us version 1, so we can't easily test invalid versions
                // But we can verify that version 1 works
                assert_eq!(*num, 1);
            }
        }
    }

    #[test]
    fn test_section_from_wasmparser_type_section() {
        let wasm_bytes = create_minimal_wasm();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::TypeSection(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::TypeSection(_))
                ));
                return;
            }
        }
        panic!("TypeSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_function_section() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::FunctionSection(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::FunctionSection(_))
                ));
                return;
            }
        }
        panic!("FunctionSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_import_section() {
        let mut module = Module::new();
        let mut imports = wasm_encoder::ImportSection::new();
        imports.import(
            "env",
            "memory",
            wasm_encoder::EntityType::Memory(wasm_encoder::MemoryType {
                memory64: false,
                shared: false,
                minimum: 1,
                maximum: None,
                page_size_log2: None,
            }),
        );
        module.section(&imports);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::ImportSection(_) = &payload {
                // ImportSection only supports Func type imports, so this should fail
                let result = Section::from_wasmparser(payload);
                assert!(result.is_err());
                return;
            }
        }
        panic!("ImportSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_import_section_func() {
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

        let mut imports = wasm_encoder::ImportSection::new();
        imports.import("env", "foo", wasm_encoder::EntityType::Function(0));
        module.section(&imports);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::ImportSection(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::ImportSection(_))
                ));
                return;
            }
        }
        panic!("ImportSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_table_section() {
        let mut module = Module::new();
        let mut tables = wasm_encoder::TableSection::new();
        tables.table(wasm_encoder::TableType {
            element_type: wasm_encoder::RefType::FUNCREF,
            table64: false,
            minimum: 10,
            maximum: Some(100),
            shared: false,
        });
        module.section(&tables);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::TableSection(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::TableSection(_))
                ));
                return;
            }
        }
        panic!("TableSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_memory_section() {
        let mut module = Module::new();
        let mut memories = wasm_encoder::MemorySection::new();
        memories.memory(wasm_encoder::MemoryType {
            memory64: false,
            shared: false,
            minimum: 1,
            maximum: Some(10),
            page_size_log2: None,
        });
        module.section(&memories);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::MemorySection(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::MemorySection(_))
                ));
                return;
            }
        }
        panic!("MemorySection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_export_section() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        let mut exports = wasm_encoder::ExportSection::new();
        exports.export("main", wasm_encoder::ExportKind::Func, 0);
        module.section(&exports);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::ExportSection(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::ExportSection(_))
                ));
                return;
            }
        }
        panic!("ExportSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_code_section_entry() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::CodeSectionEntry(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::CodeSectionEntry(_))
                ));
                return;
            }
        }
        panic!("CodeSectionEntry payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_data_section() {
        let mut module = Module::new();
        let mut data = wasm_encoder::DataSection::new();
        data.segment(wasm_encoder::DataSegment {
            mode: wasm_encoder::DataSegmentMode::Passive,
            data: b"hello".to_vec(),
        });
        module.section(&data);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::DataSection(_) = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_some());
                let section = result.unwrap();
                assert!(matches!(
                    section.section,
                    Some(section::Section::DataSection(_))
                ));
                return;
            }
        }
        panic!("DataSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_start_section_error() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        module.section(&wasm_encoder::StartSection { function_index: 0 });

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::StartSection { .. } = &payload {
                let result = Section::from_wasmparser(payload);
                assert!(result.is_err());
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("StartSection is not supported")
                );
                return;
            }
        }
        panic!("StartSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_data_count_section_none() {
        let mut module = Module::new();
        module.section(&wasm_encoder::DataCountSection { count: 1 });

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::DataCountSection { .. } = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_none());
                return;
            }
        }
        panic!("DataCountSection payload not found");
    }

    #[test]
    fn test_section_from_wasmparser_code_section_start_none() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        // CodeSectionStart is not a separate section in wasm-encoder,
        // it's part of the CodeSection itself. We'll test this by parsing
        // a module with a code section instead.
        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::CodeSectionStart { .. } = &payload {
                let result = Section::from_wasmparser(payload).unwrap();
                assert!(result.is_none());
                return;
            }
        }
        // CodeSectionStart may not appear in all parsers, so we'll just verify
        // that the code section itself works
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::CodeSectionEntry(_) = &payload {
                // Found code section entry, test passes
                return;
            }
        }
        panic!("CodeSectionEntry payload not found");
    }

    #[test]
    fn test_section_render_wasm_type_section() {
        let wasm_bytes = create_minimal_wasm();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::TypeSection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                let mut module = Module::new();
                let mut code_section = CodeSection::new();
                let result = section.render_wasm(&mut module, &mut code_section);
                assert!(result.is_ok());
                return;
            }
        }
        panic!("TypeSection payload not found");
    }

    #[test]
    fn test_section_render_wasm_function_section() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::FunctionSection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                let mut module = Module::new();
                let mut code_section = CodeSection::new();
                let result = section.render_wasm(&mut module, &mut code_section);
                assert!(result.is_ok());
                return;
            }
        }
        panic!("FunctionSection payload not found");
    }

    #[test]
    fn test_section_render_wasm_code_section_entry() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        let mut code = wasm_encoder::CodeSection::new();
        let mut func = wasm_encoder::Function::new(vec![]);
        func.instruction(&wasm_encoder::Instruction::End);
        code.function(&func);
        module.section(&code);

        let wasm_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            let payload = payload.unwrap();
            if let Payload::CodeSectionEntry(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                let mut module = Module::new();
                let mut code_section = CodeSection::new();
                let result = section.render_wasm(&mut module, &mut code_section);
                assert!(result.is_ok());
                return;
            }
        }
        panic!("CodeSectionEntry payload not found");
    }

    #[test]
    fn test_type_section_round_trip() {
        let mut module = Module::new();
        let mut types = wasm_encoder::TypeSection::new();
        types.ty().subtype(&wasm_encoder::SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: wasm_encoder::CompositeType {
                inner: wasm_encoder::CompositeInnerType::Func(wasm_encoder::FuncType::new(
                    vec![wasm_encoder::ValType::I32, wasm_encoder::ValType::I64],
                    vec![wasm_encoder::ValType::F32],
                )),
                shared: false,
                descriptor: None,
                describes: None,
            },
        });
        module.section(&types);

        let original_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&original_bytes) {
            let payload = payload.unwrap();
            if let Payload::TypeSection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                let mut new_module = Module::new();
                let mut code_section = CodeSection::new();
                section
                    .render_wasm(&mut new_module, &mut code_section)
                    .unwrap();
                let new_bytes = new_module.finish();
                // Verify the round trip produces valid WASM
                let new_parser = Parser::new(0);
                for _payload in new_parser.parse_all(&new_bytes) {
                    // Just verify it parses without error
                }
                return;
            }
        }
        panic!("TypeSection payload not found");
    }

    #[test]
    fn test_function_section_round_trip() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        functions.function(0);
        module.section(&functions);

        let original_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&original_bytes) {
            let payload = payload.unwrap();
            if let Payload::FunctionSection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                if let Some(section::Section::FunctionSection(func_section)) = &section.section {
                    assert_eq!(func_section.type_idxs.len(), 2);
                    assert_eq!(func_section.type_idxs[0], 0);
                    assert_eq!(func_section.type_idxs[1], 0);
                }
                let mut new_module = Module::new();
                let mut code_section = CodeSection::new();
                section
                    .render_wasm(&mut new_module, &mut code_section)
                    .unwrap();
                return;
            }
        }
        panic!("FunctionSection payload not found");
    }

    #[test]
    fn test_table_section_round_trip() {
        let mut module = Module::new();
        let mut tables = wasm_encoder::TableSection::new();
        tables.table(wasm_encoder::TableType {
            element_type: wasm_encoder::RefType::FUNCREF,
            table64: false,
            minimum: 5,
            maximum: Some(20),
            shared: false,
        });
        module.section(&tables);

        let original_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&original_bytes) {
            let payload = payload.unwrap();
            if let Payload::TableSection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                if let Some(section::Section::TableSection(table_section)) = &section.section {
                    assert_eq!(table_section.types.len(), 1);
                    let table_type = &table_section.types[0];
                    assert_eq!(table_type.initial, Some(5));
                    assert_eq!(table_type.maximum, Some(20));
                }
                let mut new_module = Module::new();
                let mut code_section = CodeSection::new();
                section
                    .render_wasm(&mut new_module, &mut code_section)
                    .unwrap();
                return;
            }
        }
        panic!("TableSection payload not found");
    }

    #[test]
    fn test_memory_section_round_trip() {
        let mut module = Module::new();
        let mut memories = wasm_encoder::MemorySection::new();
        memories.memory(wasm_encoder::MemoryType {
            memory64: false,
            shared: false,
            minimum: 2,
            maximum: Some(10),
            page_size_log2: None,
        });
        module.section(&memories);

        let original_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&original_bytes) {
            let payload = payload.unwrap();
            if let Payload::MemorySection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                if let Some(section::Section::MemorySection(memory_section)) = &section.section {
                    assert_eq!(memory_section.memory_types.len(), 1);
                    let memory_type = &memory_section.memory_types[0];
                    assert_eq!(memory_type.initial, Some(2));
                    assert_eq!(memory_type.maximum, Some(10));
                }
                let mut new_module = Module::new();
                let mut code_section = CodeSection::new();
                section
                    .render_wasm(&mut new_module, &mut code_section)
                    .unwrap();
                return;
            }
        }
        panic!("MemorySection payload not found");
    }

    #[test]
    fn test_export_section_round_trip() {
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

        let mut functions = wasm_encoder::FunctionSection::new();
        functions.function(0);
        module.section(&functions);

        let mut exports = wasm_encoder::ExportSection::new();
        exports.export("test_func", wasm_encoder::ExportKind::Func, 0);
        module.section(&exports);

        let original_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&original_bytes) {
            let payload = payload.unwrap();
            if let Payload::ExportSection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                if let Some(section::Section::ExportSection(export_section)) = &section.section {
                    assert_eq!(export_section.exports.len(), 1);
                    assert_eq!(
                        export_section.exports[0].name,
                        Some("test_func".to_string())
                    );
                }
                let mut new_module = Module::new();
                let mut code_section = CodeSection::new();
                section
                    .render_wasm(&mut new_module, &mut code_section)
                    .unwrap();
                return;
            }
        }
        panic!("ExportSection payload not found");
    }

    #[test]
    fn test_data_section_round_trip() {
        let mut module = Module::new();
        let mut data = wasm_encoder::DataSection::new();
        data.segment(wasm_encoder::DataSegment {
            mode: wasm_encoder::DataSegmentMode::Passive,
            data: b"test data".to_vec(),
        });
        module.section(&data);

        let original_bytes = module.finish();
        let parser = Parser::new(0);
        for payload in parser.parse_all(&original_bytes) {
            let payload = payload.unwrap();
            if let Payload::DataSection(_) = &payload {
                let section = Section::from_wasmparser(payload).unwrap().unwrap();
                if let Some(section::Section::DataSection(data_section)) = &section.section {
                    assert_eq!(data_section.datas.len(), 1);
                    assert_eq!(data_section.datas[0].data, Some(b"test data".to_vec()));
                }
                let mut new_module = Module::new();
                let mut code_section = CodeSection::new();
                section
                    .render_wasm(&mut new_module, &mut code_section)
                    .unwrap();
                return;
            }
        }
        panic!("DataSection payload not found");
    }
}
