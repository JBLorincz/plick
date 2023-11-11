#![allow(unused_imports, dead_code)]
use std::{env, fs::{self}, process};
use codegen::codegen::{Compiler, CodeGenable};
use lexer::{Token, TokenManager};
use parser::{parse_expression, parse_function, parse_opening };
use inkwell::{targets::TargetMachine, types::{BasicMetadataTypeEnum, PointerType, FunctionType}, AddressSpace, module::{self, Module}, passes::PassManager};
use inkwell::context;
use passes::perform_parse_pass;
use std::path::Path;

use crate::debugger::{setup_module_for_debugging, DebugController};

pub mod lexer;
pub mod parser;
mod codegen;
mod error;
mod debugger;
pub mod ast;
mod types;
mod passes;



fn drive_compilation<'a,'ctx>(token_manager: &mut TokenManager, compiler: &'a mut  Compiler<'a, 'ctx>) -> Result<(),String>
{
    compiler.initalize_main_function();

        //Below is introducing "builtin functions" the compiler needs to accomplish things like IO

        let printf_arg_type: PointerType<'ctx> = compiler
            .context
            .i8_type()
            .ptr_type(AddressSpace::default());
        
        let printf_type: FunctionType<'ctx> = compiler
            .context
            .i32_type()
            .fn_type(&[BasicMetadataTypeEnum::from(printf_arg_type)], true);
    

            let _printf_func = compiler.module.add_function("printf", printf_type, Some(module::Linkage::DLLImport));
            
            unsafe
            {
                perform_parse_pass(token_manager)?.perform_type_pass()?.code_generation_pass(compiler)?;
            }


            Ok(())
}

pub fn compile_input(input: &str, config: Config)
{
         let filename = config.filename.clone();
        let default_triple = TargetMachine::get_default_triple();
        println!("Building for {}", &default_triple.as_str().to_string_lossy());
        let init_config = inkwell::targets::InitializationConfig
        {
            asm_parser: true,
            asm_printer: true,
            base: true,
            disassembler: false,
            info: true,
            machine_code: true
        };


        inkwell::targets::Target::initialize_all(&init_config);

        let my_target = inkwell::targets::Target::from_triple(&default_triple).unwrap();

        
        let optimization_level = match config.optimize
        {
            true => inkwell::OptimizationLevel::Default,
            false => inkwell::OptimizationLevel::None,
        };

    let target_machine = my_target.create_target_machine(&default_triple, "generic", "",
    optimization_level, inkwell::targets::RelocMode::PIC, inkwell::targets::CodeModel::Default).unwrap();

        //create compiler dependencies
        let c = context::Context::create(); 
        let b = c.create_builder();
        let m = c.create_module("globalMod");

        let mut optional_debugger: Option<&DebugController<'_>> = None;
        let debugger: DebugController; 
        
        if config.debug_mode
        {
            debugger = setup_module_for_debugging(&m, &config);
            optional_debugger = Some(&debugger);
        }

        let mut compiler = codegen::codegen::Compiler::new(&c,&b,&m, optional_debugger); 

        let mut token_manager = lexer::TokenManager::new(input);
       
        if let Some(dbg) = optional_debugger
        {
            token_manager.attach_debugger(dbg);
        }
        
        let compilation_result = drive_compilation(&mut token_manager,&mut compiler);

        if let Err(err_msg) = compilation_result
        {
            panic!("{}",err_msg);
        }
        

         if let Some(dbg) = optional_debugger
        {
            dbg.builder.finalize();
        }

        //comment for finalize says call before verification

        let module_verification_result = m.verify();

        if config.print_ir
        {
            println!("{}",m.print_to_string());
        }

        match module_verification_result
        {
            Ok(()) => println!("Module verified successfully!"),
            Err(err_message) => {

                println!("Module verification failed:");
                println!("{}",err_message);
                panic!("Failed Compilation!");
                process::exit(1);

            }
        }


        if config.dry_run
        {

            let write_to_memory_result = 
                target_machine.write_to_memory_buffer(&m, inkwell::targets::FileType::Object);
            match write_to_memory_result
            {
            Ok(_memoryBuffer) => println!("Written to memory buffer successfully!"),
            Err(err_message) => {

                println!("memory write failed:");
                println!("{}",err_message);
                panic!("test!");
                process::exit(1);

            }
        }
        }
        else
        {
        
        let write_to_file_result = target_machine.write_to_file(&m, inkwell::targets::FileType::Object, Path::new(&filename));
        match write_to_file_result
        {
            Ok(()) => println!("Written to file successfully!"),
            Err(err_message) => {

                println!("file write failed:");
                println!("{}",err_message);
                process::exit(1);

            }
        }

        }
      
}

pub struct Config {
    pub filename: String,
    pub optimize: bool,
    pub debug_mode: bool,
    pub print_ir: bool,
    pub dry_run: bool //if true, won't save the compiled output to the disk - enable during testing
}

impl Default for Config {
   fn default() -> Config {
        Config {
            filename: String::from("a.o"),
            optimize: true,
            debug_mode: true,
            print_ir: true,
            dry_run: false
        } 
   }
}
