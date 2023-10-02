use std::cell::RefCell;

use super::Config;

use crate::codegen::codegen::Compiler;
use inkwell::{debug_info::{self, DICompileUnit, DebugInfoBuilder, DILexicalBlock, DIScope}, module::Module};


///Generates debug info for the PLI files

#[derive(Debug)]
pub struct DebugController<'ctx>
{
    pub builder: DebugInfoBuilder<'ctx>,
    pub compile_unit: DICompileUnit<'ctx>,
    pub lexical_blocks: RefCell<Vec<DIScope<'ctx>>>,

    pub line_number: RefCell<u32>,
    pub column_number: RefCell<u32>,

    pub filename: String,
    pub directory: String,
    pub optimized: bool,
}

pub fn setup_module_for_debugging<'a ,'ctx>(m: &'a Module<'ctx>, config: &Config) -> DebugController<'ctx>
{
    let (dibuilder, compile_unit) = m.create_debug_info_builder(
        true,
        inkwell::debug_info::DWARFSourceLanguage::C,
        &config.filename,
        ".",
        "PL/1 Frontend",
        config.optimize,
        "",
        0,
        "split_name",
        inkwell::debug_info::DWARFEmissionKind::Full,
        0,
        false,
        false,
        "sysroot",
        "sdk");

    dibuilder.create_basic_type("double", 64, 0 , 0);

    DebugController { 
        builder: dibuilder,
        lexical_blocks: RefCell::new(vec![]),
        compile_unit, 
        line_number: RefCell::new(1),
        column_number: RefCell::new(0),
        filename: config.filename.clone(),
        directory: ".".to_string(),
        optimized: config.optimize,
    }

}
