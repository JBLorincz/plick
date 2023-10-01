use crate::codegen::codegen::Compiler;
use inkwell::{debug_info::{self, DICompileUnit, DebugInfoBuilder}, module::Module};


///Generates debug info for the PLI files

#[derive(Debug)]
pub struct DebugController<'ctx>
{
    pub builder: DebugInfoBuilder<'ctx>,
    pub compile_unit: DICompileUnit<'ctx> 
}

pub fn setup_module_for_debugging<'a ,'ctx>(m: &'a Module<'ctx>, filename: &str) -> DebugController<'ctx>
{
    let (dibuilder, compile_unit) = m.create_debug_info_builder(
        true,
        inkwell::debug_info::DWARFSourceLanguage::C,
        filename,
        ".",
        "PL/1 Frontend",
        false,
        "",
        0,
        "split_name",
        inkwell::debug_info::DWARFEmissionKind::Full,
        0,
        false,
        false,
        "sysroot",
        "sdk");

    DebugController { builder: dibuilder, compile_unit  }
}
