#![allow(unused_imports, dead_code)]
use codegen::codegen::{CodeGenable, Compiler};
use inkwell::builder::Builder;
use inkwell::context::{self, Context};
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

use crate::debugger::{setup_module_for_debugging, DebugController};

pub mod ast;
mod codegen;
mod debugger;
mod error;
pub mod lexer;
pub mod parser;
mod passes;
mod types;

fn drive_compilation<'a, 'ctx>(
    token_manager: &mut TokenManager,
    compiler: &mut Compiler<'a, 'ctx>,
) -> Result<(), String> {
    compiler.initalize_main_function();

    //Below is introducing "builtin functions" the compiler needs to accomplish things like IO

    let printf_arg_type: PointerType<'ctx> =
        compiler.context.i8_type().ptr_type(AddressSpace::default());

    let printf_type: FunctionType<'ctx> = compiler
        .context
        .i32_type()
        .fn_type(&[BasicMetadataTypeEnum::from(printf_arg_type)], true);

    let _printf_func =
        compiler
            .module
            .add_function("printf", printf_type, Some(module::Linkage::DLLImport));

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
    //let filename = config.filename.clone();

    //let target_machine = build_default_target_machine(&config);
    ////create compiler dependencies
    //let c = context::Context::create();
    //let b = c.create_builder();
    //let m = c.create_module("globalMod");

    //let mut optional_debugger: Option<&DebugController<'_>> = None;
    //let debugger: DebugController;

    //if config.debug_mode {
    //    debugger = setup_module_for_debugging(&m, &config);
    //    optional_debugger = Some(&debugger);
    //}

    //let mut compiler = codegen::codegen::Compiler::new(&c, &b, &m, optional_debugger);

    //let mut token_manager = lexer::TokenManager::new(input);

    //if let Some(dbg) = optional_debugger {
    //    token_manager.attach_debugger(dbg);
    //}

    execute_compilation_actions(input, &config, |token_manager, compiler, target_machine| {
    let compilation_result = drive_compilation(token_manager, compiler);

    if let Err(err_msg) = compilation_result {
        panic!("{}", err_msg);
    }

    if let Some(dbg) = compiler.debug_controller {
        dbg.builder.finalize();
    }

    //comment for finalize says call before verification

    let module_verification_result = compiler.module.verify();

    if config.print_ir {
        println!("{}", compiler.module.print_to_string());
    }

    match module_verification_result {
        Ok(()) => println!("Module verified successfully!"),
        Err(err_message) => {
            error!("Module verification failed:");
            error!("{}", err_message);
            panic!("Failed Compilation!");
            process::exit(1);
        }
    }

    if config.dry_run {
        let write_to_memory_result =
            target_machine.write_to_memory_buffer(&compiler.module, inkwell::targets::FileType::Object);
        match write_to_memory_result {
            Ok(_memoryBuffer) => println!("Written to memory buffer successfully!"),
            Err(err_message) => {
                error!("memory write failed:");
                error!("{}", err_message);
                panic!("test!");
                process::exit(1);
            }
        }
    } else {
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

    });
    
}

pub fn compile_input_to_memory(input: &str, config: &Config) -> Result<MemoryBuffer, String> {
    Err(String::from("NotImplemented!"))
}


pub fn execute_compilation_actions<F: Fn(&mut TokenManager, &mut Compiler, &TargetMachine)>(input: &str, config: &Config, closure: F)
{
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
    pub dry_run: bool, //if true, won't save the compiled output to the disk - enable during testing
}

impl Default for Config {
    fn default() -> Config {
        Config {
            filename: String::from("a.o"),
            optimize: true,
            debug_mode: true,
            print_ir: true,
            dry_run: false,
        }
    }
}

