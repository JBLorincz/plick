use std::{env, fs::{self}, process};
use codegen::codegen::{Compiler, CodeGenable};
use lexer::{Token, TokenManager};
use parser::{parse_expression, parse_function, parse_opening };
use inkwell::{targets::TargetMachine, types::{BasicMetadataTypeEnum, PointerType, FunctionType}, AddressSpace, module};
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
    compiler.initalize_main_function();

        //Below is introducing "builtin functions" the compiler needs to accomplish things like IO

        let printf_arg_type: PointerType<'ctx> = compiler.context.i8_type().ptr_type(AddressSpace::default());
            let printf_type: FunctionType<'ctx> = compiler.context.i32_type().fn_type(&[BasicMetadataTypeEnum::from(printf_arg_type)], true);
            let printf_func = compiler.module.add_function("printf", printf_type, Some(module::Linkage::DLLImport));

    let mut current_label_string: Option<String> = None;
      while let Some(ref token) = token_manager.current_token
      {
          if let Token::END = token
          {
              found_top_level_end = true;
              compiler.builder.build_return(None);
              break;
          }
          let parser_result = parser::parse_statement(token_manager);
          
          if let Err(err_msg) = parser_result
          {
              let msg = format!("Finished parsing: {}", err_msg);
              println!("{}",msg);
              break;
          }
          let parser_result = parser_result.unwrap();

          unsafe {
            parser_result.codegen(compiler);
        }
      }
//    while let Some(ref token) = token_manager.current_token
//    {
//        match token 
//        {
//            Token::SEMICOLON  => {
//                token_manager.next_token();
//                current_label_string = None;
//            },
//            Token::LABEL(label_string) => {
//                
//                if let Some(_) = current_label_string
//                {
//                    panic!("Can't declare two labels in a row!");
//                }
//
//                current_label_string = Some(label_string.to_string()); //store the fact something
//                token_manager.next_token();                                                     //is labelled
//            },
//            Token::PUT => {
//                unsafe {
//                compiler.generate_hello_world_print();
//                }
//                token_manager.next_token();
//            }
//           Token::PROCEDURE => {
//               let fn_name: String;
//               match current_label_string {
//                   Some(ref val) => fn_name = val.clone(),
//                   None => panic!("Could not find the label associated with a function definition!")
//               }
//               unsafe {
//                   let parsed_function = parse_function(token_manager,fn_name).unwrap();
//                   compiler.generate_function_code(parsed_function);
//                   compiler.builder.position_at_end(compiler.module.get_first_function().unwrap().get_first_basic_block().unwrap());
//
//                   current_label_string = None; //stopgap for transition to statement-driven logic
//               }
//           }, 
//            Token::END => {
//                found_top_level_end = true;
//                compiler.builder.build_return(None);
//                break; 
//            },
//            _ => {
//                unsafe {
//                parse_expression(token_manager).codegen(&mut compiler);
//                }
//            },
//            
//        }
//         
//        
//    }
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
        println!("{}",m.print_to_string());
        match module_verification_result
        {
            Ok(()) => println!("Module verified successfully!"),
            Err(err_message) => {
                println!("Module verification failed:");
                println!("{}",err_message);
                panic!("WHAAA");
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
                panic!("WHOO");
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
                    LOL: PROCEDURE ();  999-444;
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
     #[should_panic(expected = "two labels")]
    fn test_double_label_panic() -> ()
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: LOL: PROCEDURE (A);  A-4;
                END;
                BOL: PROCEDURE(); 4-7; END;
                LOL(6);
                LOL(8);
                BOL();
                BOL();
                LOL(2);
                END;";
        
        let mut conf = Config::default();
        conf.filename = "failfile.o".to_string();
        compile_input(input,conf);
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
    fn drive_hello_world(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; 2 + 4; END;";
    //let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
    //    PROCEDURE ();  2+2; END; TESTFUNC(); END;";
        let conf = Config::default();
        compile_input(input,conf);

        
    }
}
