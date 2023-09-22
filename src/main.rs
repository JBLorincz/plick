use std::{env, fs::{self}, process};
use codegen::codegen::{Compiler, CodeGenable};
use lexer::{Token, TokenManager};
use parser::{parse_expression, parse_function, parse_opening };
use inkwell::{targets::TargetMachine, types::BasicMetadataTypeEnum};
use inkwell::context;
use std::path::Path;

mod lexer;
mod parser;
mod codegen;


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


fn drive_compilation<'a,'ctx>(token_manager: &mut TokenManager,  mut compiler: &'a mut  Compiler<'a, 'ctx>)
{
    parse_opening(token_manager);

    let mut found_top_level_end = false;
    let args: Vec<BasicMetadataTypeEnum> = vec![];
    let main_function_type = compiler.context.void_type().fn_type(&args, false);
    let main_func = compiler.module.add_function("main", main_function_type, None);
            //create a new scope block for the function
            let new_func_block = compiler.context.append_basic_block(main_func, "entry");

            //position the builder's cursor inside that block
            compiler.builder.position_at_end(new_func_block);
    while let Some(ref token) = token_manager.current_token
    {
        match token 
        {
            Token::SEMICOLON | Token::LABEL(_)  => {
                token_manager.next_token();
            },
           Token::PROCEDURE => {
               unsafe {
                   compiler.generate_function_code(parse_function(token_manager));
                   compiler.builder.position_at_end(compiler.module.get_first_function().unwrap().get_first_basic_block().unwrap());
               }
           }, 
            Token::END => {
                found_top_level_end = true;
                compiler.builder.build_return(None);
                break; 
            },
            _ => {
                unsafe {
                parse_expression(token_manager).codegen(&mut compiler);
                }
            },
            
        }
         
        
    }
         if !found_top_level_end
         {
             panic!("Did not find an end to the program!");
         }
}

fn compile_input(input: &str, config: Config)
{
         let filename = config.filename;
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

    let target_machine = my_target.create_target_machine(&default_triple, "generic", "",
    inkwell::OptimizationLevel::None, inkwell::targets::RelocMode::Default, inkwell::targets::CodeModel::Default).unwrap();


        let c = context::Context::create(); 
        let b = c.create_builder();
        let m = c.create_module("globalMod");
        
        let mut compiler = codegen::codegen::Compiler::new(&c,&b,&m); 

        let mut token_manager = lexer::TokenManager::new(input);
        drive_compilation(&mut token_manager,&mut compiler);

        let module_verification_result = m.verify();
        match module_verification_result
        {
            Ok(()) => println!("Module verified successfully!"),
            Err(err_message) => {
                println!("Module verification failed:");
                println!("{}",err_message);
                process::exit(1);
            }
        }

        let write_to_file_result = target_machine.write_to_file(&m, inkwell::targets::FileType::Assembly, Path::new(&filename));
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


struct Config {
    filename: String
}

impl Default for Config {
   fn default() -> Config {
        Config {
            filename: String::from("a.o")
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
        PROCEDURE ();  999-444; END; TESTFUNC();  END;";
        
    let conf = Config::default();
        compile_input(input,conf);
        Ok(())
    }
    
    #[test]
    fn drive_hello_world(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; 2 + 4; END;";
    //let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
    //    PROCEDURE ();  2+2; END; TESTFUNC(); END;";
        let conf = Config::default();
        compile_input(input,conf);

        
    }
}
