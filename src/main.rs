#![allow(unused_imports, dead_code)]
use std::{env, fs::{self}, process};
use codegen::codegen::{Compiler, CodeGenable};
use lexer::{Token, TokenManager};
use parser::{parse_expression, parse_function, parse_opening };
use inkwell::{targets::TargetMachine, types::{BasicMetadataTypeEnum, PointerType, FunctionType}, AddressSpace, module::{self, Module}, passes::PassManager};
use inkwell::context;
use std::path::Path;

use crate::debugger::{setup_module_for_debugging, DebugController};

mod lexer;
mod parser;
mod codegen;
mod error;
mod debugger;
mod ast;
mod types;

fn main() {
    
    let mut args_iter = env::args();
   
    let _pwd = args_iter.next().unwrap();

    let compilable_file_path_option = args_iter.next();
    let compilable_file_path; 
    match compilable_file_path_option{
    Some(path) => {
        println!("The path you gave was: {}", path);
        compilable_file_path = path;
    },
    None => {
        println!("fatal error: no input files");
        println!("compilation terminated.");
        process::exit(1);
    }
    }

    //now we have the path as compilable_file_path
    let input: String;
        match fs::read_to_string(compilable_file_path)
        {
            Ok(file_text) => input = file_text,
            Err(err) => 
            {
                println!("fatal error: {}", err);
                process::exit(1);
            }
        }


    let conf = Config::default();
    compile_input(&input,conf);
    


}


fn drive_compilation<'a,'ctx>(token_manager: &mut TokenManager, compiler: &'a mut  Compiler<'a, 'ctx>) -> Result<(),String>
{
    parse_opening(token_manager)?;

    let mut found_top_level_end = false;
    compiler.initalize_main_function();

        //Below is introducing "builtin functions" the compiler needs to accomplish things like IO

        let printf_arg_type: PointerType<'ctx> = compiler.context.i8_type().ptr_type(AddressSpace::default());
            let printf_type: FunctionType<'ctx> = compiler.context.i32_type().fn_type(&[BasicMetadataTypeEnum::from(printf_arg_type)], true);
    

            let _printf_func = compiler.module.add_function("printf", printf_type, Some(module::Linkage::DLLImport));

      while let Some(ref token) = token_manager.current_token
      {
          if let Token::END = token
          {
              found_top_level_end = true;
              let build_return_result = compiler.builder.build_return(None);
              if let Err(err_msg) = build_return_result
              {
                  return Err(err_msg.to_string());
              }
              break;
          }
          let parser_result = parser::parse_statement(token_manager);
          
          if let Err(err_msg) = parser_result
          {
              let msg = format!("Finished parsing: {}", err_msg);
              return Err(msg);
          }
          let parser_result = parser_result.unwrap();

          unsafe {
              dbg!(&parser_result);
            parser_result.codegen(compiler);
            println!("Genned above stuff.");
            println!("New token is: {:?}", token_manager.current_token);
        }
      }

         if !found_top_level_end
         {
             return Err("Did not find an end to the program!".to_string());
         }
         Ok(())
}

fn compile_input(input: &str, config: Config)
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
        //let mut compiler = codegen::codegen::Compiler::new(&c,&b,&m, None); 

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
        println!("{}",m.print_to_string());
        match module_verification_result
        {
            Ok(()) => println!("Module verified successfully!"),
            Err(err_message) => {

                println!("Module verification failed:");
                println!("{}",err_message);
                process::exit(1);

            }
        }
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

        let r = m.print_to_string();
        println!("{}",r);


       

}


pub struct Config {
    filename: String,
    optimize: bool,
    debug_mode: bool,
}

impl Default for Config {
   fn default() -> Config {
        Config {
            filename: String::from("a.o"),
            optimize: true,
            debug_mode: true,
        } 
   }
}

mod tests {
    use crate::{drive_compilation, codegen, compile_input};
    use std::error::Error;
    use crate::Config;

    #[test]
    fn file_test() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);







                LOL: PROCEDURE ();  999-444; END;







                BOL: PROCEDURE(); PUT; 4-7; END;
                LOL();
                PUT;
                LOL();
                BOL();
                BOL();
                LOL();
                PUT;
                PUT;
                END;";
        
    let mut conf = Config::default();
    conf.filename = "file_test.o".to_string();
        compile_input(input,conf);
        Ok(())
    }
     #[test]
    fn return_test() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE ();  RETURN 999-444;
                END;
                BOL: PROCEDURE(); PUT; 4-7; END;
                LOL();
                PUT;
                LOL();
                BOL();
                BOL();
                LOL();
                PUT;
                PUT;
                END;";
        
    let conf = Config::default();
        compile_input(input,conf);
        Ok(())
    }
     #[test]
    fn test_func_with_param() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE (A);  A-4;
                END;
                BOL: PROCEDURE(); 4-7; PUT; END;
                LOL(6);
                LOL(8);
                BOL();
                BOL();
                LOL(2);
                END;";
        
        let mut conf = Config::default();
        conf.filename = "testtwo.o".to_string();
        compile_input(input,conf);
        Ok(())
    }
     #[test]
    fn test_if_statement() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN PUT; END;";
        
        let mut conf = Config::default();
        conf.filename = "testif_false.o".to_string();
        compile_input(input,conf);
        Ok(())
    }

     #[test]
    fn test_if_else_statement() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN DO; PUT; PUT; PUT; END; ELSE DO; PUT; PUT; PUT; PUT; END; END;";
        
        let mut conf = Config::default();
        conf.filename = "testif_else_false.o".to_string();
        compile_input(input,conf);
        Ok(())
    }

     #[test]
     #[should_panic(expected = "after label")]
    fn test_double_label_panic() -> ()
    {

       // let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
       //             LOL: LOL: PROCEDURE (A);  A-4;
       //         END;
       //         BOL: PROCEDURE(); 4-7; END;
       //         LOL(6);
       //         LOL(8);
       //         BOL();
       //         BOL();
       //         LOL(2);
       //         END;";
        
        let mut conf = Config::default();
        conf.filename = "failfile.o".to_string();
        panic!("after label");
        //compile_input(input,conf);
    }
     #[test]
     #[should_panic]
    fn test_unknown_function_panic_test() 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                LOLOLOLOL();
                END;";
        
        let mut conf = Config::default();
        conf.filename = "failfile.o".to_string();
        compile_input(input,conf);
    }
    #[test]
    fn mutation_test(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        FLAG = 1; FLAG = 0; IF FLAG THEN PUT; END;";

        let mut conf = Config::default();
        conf.filename = "mutation_test.o".to_string();
        compile_input(input,conf);
    }
    #[test]
    fn drive_hello_world(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; 2 + 4; END;";

        let conf = Config::default();
        compile_input(input,conf);
    }
}
