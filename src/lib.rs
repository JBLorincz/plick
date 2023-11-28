#![allow(unused_imports, dead_code, unreachable_code)]
use cli::Arguments;
use codegen::codegen::{CodeGenable, Compiler};
use inkwell::builder::Builder;
use inkwell::context::{self, Context};
use inkwell::memory_buffer;
use inkwell::types::FloatType;
use inkwell::{
    memory_buffer::MemoryBuffer,
    module::{self, Module},
    passes::PassManager,
    targets::TargetMachine,
    types::{BasicMetadataTypeEnum, FunctionType, PointerType},
    AddressSpace,
};
use lexer::{Token, TokenManager};
use log::error;
use parser::{parse_expression, parse_function, parse_opening};
use passes::perform_parse_pass;
use std::path::Path;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
use std::{
    env,
    fs::{self},
    process,
};

use codegen::prelude;
use crate::debugger::{setup_module_for_debugging, DebugController};

pub mod ast;
mod codegen;
pub mod cli;
mod debugger;
mod error;
pub mod lexer;
pub mod parser;
mod passes;
pub mod types;

fn drive_compilation<'a, 'ctx>(
    token_manager: &mut TokenManager,
    compiler: &mut Compiler<'a, 'ctx>,
) -> Result<(), String> {
    compiler.initalize_main_function();

    prelude::add_extern_functions(compiler);

    unsafe {
        perform_parse_pass(token_manager)?
            .perform_type_pass()?
            .code_generation_pass(compiler)?;
    }

    Ok(())
}
pub fn initialize_logger() {
    env_logger::init();
}
pub fn compile_input(input: &str, config: Config) {
    execute_compilation_actions(input, &config, &mut |token_manager, compiler, target_machine| {
        let compilation_result = drive_compilation(token_manager, compiler);

        if let Err(err_msg) = compilation_result {
            panic!("{}", err_msg);
        }

        if let Some(dbg) = compiler.debug_controller {
            dbg.builder.finalize();
        }

        //comment for finalize says call before verification
        if config.print_ir {
            println!("{}", compiler.module.print_to_string());
        }

      
        if config.write_ir_to_file
        {
            output_module_as_ir_to_file(&compiler, target_machine,&config);
        }

        verify_module(&compiler);
        if config.write_ir_to_file
        {
        }
        else if config.dry_run {
            output_module_to_memory_buffer(&compiler, target_machine);
        } else {
            output_module_to_file(compiler, &config, target_machine);
        }
    });
}

pub fn execute_compilation_actions<F: FnMut(&mut TokenManager, &mut Compiler, &TargetMachine)>(
    input: &str,
    config: &Config,
    closure: &mut F,
) {
    let filename = config.filename.clone();

    let target_machine = build_default_target_machine(&config);
    //create compiler dependencies
    let c = context::Context::create();
    let b = c.create_builder();
    let m = c.create_module("globalMod");

    let mut optional_debugger: Option<&DebugController<'_>> = None;
    let debugger: DebugController;

    if config.debug_mode {
        debugger = setup_module_for_debugging(&m, &config);
        optional_debugger = Some(&debugger);
    }

    let mut compiler = codegen::codegen::Compiler::new(&c, &b, &m, optional_debugger);

    let mut token_manager = lexer::TokenManager::new(input);

    if let Some(dbg) = optional_debugger {
        token_manager.attach_debugger(dbg);
    }
    closure(&mut token_manager, &mut compiler, &target_machine);
}

fn output_module_to_memory_buffer(compiler: &Compiler, target_machine: &TargetMachine)
    -> MemoryBuffer
{
    let write_to_memory_result =
        target_machine.write_to_memory_buffer(&compiler.module, inkwell::targets::FileType::Object);
    match write_to_memory_result {
        Ok(memory_buffer) => memory_buffer,
        Err(err_message) => {
            error!("memory write failed:");
            error!("{}", err_message);
            panic!("test!");
            process::exit(1);
        }
    }
}
fn output_module_as_ir_to_file(compiler: &Compiler, target_machine: &TargetMachine,config: &Config)
{
    let write_to_memory_result =
        target_machine.write_to_memory_buffer(&compiler.module, inkwell::targets::FileType::Object);

    let x = match write_to_memory_result {
        Ok(memory_buffer) => (),
        Err(err_message) => {
            error!("memory write failed:");
            error!("{}", err_message);
            panic!("test!");
            process::exit(1);
        }
    };
        let file_name =  Path::new(&config.filename);
        compiler.module.print_to_file(file_name);
}
fn verify_module(compiler: &Compiler) {
    let module_verification_result = compiler.module.verify();

    match module_verification_result {
        Ok(()) => println!("Module verified successfully!"),
        Err(err_message) => {
            error!("Module verification failed:");
            error!("{}", err_message);
            panic!("Failed Compilation!");
            process::exit(1);
        }
    }
}

fn output_module_to_file(compiler: &Compiler, config: &Config, target_machine: &TargetMachine) 
{
    let write_to_file_result = target_machine.write_to_file(
        &compiler.module,
        inkwell::targets::FileType::Object,
        Path::new(&config.filename),
    );
    match write_to_file_result {
        Ok(()) => println!("Written to file successfully!"),
        Err(err_message) => {
            println!("file write failed:");
            println!("{}", err_message);
            process::exit(1);
        }
    }
}

fn build_default_target_machine(config: &Config) -> TargetMachine {
    let filename = config.filename.clone();
    let default_triple = TargetMachine::get_default_triple();
    println!(
        "Building for {}",
        &default_triple.as_str().to_string_lossy()
    );
    let init_config = inkwell::targets::InitializationConfig {
        asm_parser: true,
        asm_printer: true,
        base: true,
        disassembler: false,
        info: true,
        machine_code: true,
    };

    inkwell::targets::Target::initialize_all(&init_config);

    let my_target = inkwell::targets::Target::from_triple(&default_triple).unwrap();

    let optimization_level = match config.optimize {
        true => inkwell::OptimizationLevel::Default,
        false => inkwell::OptimizationLevel::None,
    };

    let target_machine = my_target
        .create_target_machine(
            &default_triple,
            "generic",
            "",
            optimization_level,
            inkwell::targets::RelocMode::PIC,
            inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    target_machine
}

pub struct Config {
    pub filename: String,
    pub optimize: bool,
    pub debug_mode: bool,
    pub print_ir: bool,
    pub write_ir_to_file: bool,
    pub dry_run: bool, //if true, won't save the compiled output to the disk - enable during testing
}

impl Default for Config {
    fn default() -> Config {
        Config {
            filename: String::from("a.o"),
            optimize: true,
            debug_mode: true,
            print_ir: false,
            write_ir_to_file: false,
            dry_run: false,
        }
    }
}

impl From<Arguments> for Config {
    fn from(value: Arguments) -> Self {
        let default = Config::default();
        let filename = get_output_filename(&value);
        Config 
        { 
            filename,
            write_ir_to_file: value.save_as_ir,
            ..default
        }
    }
}



fn get_output_filename(arguments: &Arguments) -> String
{
    let path = Path::new(&arguments.path_to_file);
    let file_stem = path.file_stem().unwrap();
    let result = file_stem.to_str().unwrap().to_string();
    result+&get_output_extension(&arguments)
}

fn get_output_extension(arguments: &Arguments) -> String
{
    if arguments.save_as_ir
    {
        return ".ll".to_string();
    }

    ".o".to_string()
}
