use std::{env, fs::{self}, rc::Rc, cell::RefCell};

use codegen::codegen::{Compiler, CodeGenable};
use lexer::Token;
use parser::{parse_expression, parse_function, parse_opening, Function, Prototype};

mod lexer;
mod parser;
mod codegen;
//Steps:
//1. File access - read the files
//2. Character manipulator - read chars, maybe strip comments here.
//3. Scanner - parse into tokens
// 2 and 3 comprise the LEXER
//4. Parser - parse tokens into an Abstract Syntax Tree (AST)
// Below is back-end (handled by LLVM)
//5. Optimizer - produces a Reduced AST
//6. Code Generator - produces Raw object code
//7. Peep hole optimizer - produces optimized object code
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
        return;
    }
    }

    //now we have the path as compilable_file_path
    let compilable_file = fs::read_to_string(compilable_file_path).unwrap(); 
    // handle error stuff here.

    //BIG TODO: Handle all the linking or whatever, making one big giant compilable file.

    
    //println!("{}", compilable_file);

    



}


fn drive_compilation<'a,'ctx>(compilable_file: &str,  mut compiler: Compiler<'a, 'ctx>) -> Compiler<'a,'ctx>
{
    let mut token_manager = lexer::TokenManager::new(compilable_file);
    parse_opening(&mut token_manager);


    while let Some(ref token) = token_manager.current_token
    {
        match token 
        {
            Token::SEMICOLON | Token::LABEL(_)  => {
                token_manager.next_token();
            },
           Token::PROCEDURE => {
               unsafe {
                   compiler.generate_function_code(parse_function(&mut token_manager));
               }
           }, 
            _ => {
                unsafe {
                parse_expression(&mut token_manager).codegen(&compiler);
                }
            },
        }
    }
    
    return compiler;
}



mod tests {
    use std::{fs, io::Write, path::Path};

    use inkwell::{targets::TargetMachine, passes::PassManager, context};

    use crate::{drive_compilation, codegen};
    use std::error::Error;

    #[test]
    fn file_test() -> Result<(), Box<dyn Error>> 
    {
        let filename = "a.o";
        dbg!("heyo");
        let default_triple = TargetMachine::get_default_triple();
        dbg!("{}", &default_triple);
        let init_config = inkwell::targets::InitializationConfig
        {
            asm_parser: true,
            asm_printer: true,
            base: true,
            disassembler: false,
            info: true,
            machine_code: true
        };

        dbg!("writing all!");
        let my_trip = default_triple.as_str();
       let mut res = fs::File::create(filename)?;
       res.write_all(my_trip.to_bytes());
       

        inkwell::targets::Target::initialize_all(&init_config);
        let my_target = inkwell::targets::Target::from_triple(&default_triple)?;
        dbg!("{}", &default_triple);
    let target_machine = my_target.create_target_machine(&default_triple, "generic", "",
    inkwell::OptimizationLevel::None, inkwell::targets::RelocMode::Default, inkwell::targets::CodeModel::Default).unwrap();


        let c = context::Context::create(); 
        let b = c.create_builder();
        let m = c.create_module("globalMod");
        
        let mut compiler = codegen::codegen::Compiler::new(&c,&b,&m); 
        
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        PROCEDURE (); 2 + 2; END;";

        drive_compilation(input,compiler);
        target_machine.write_to_file(&m, inkwell::targets::FileType::Assembly, Path::new(filename));

       Ok(())
    }
    
    #[test]
    fn drive_hello_world(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; A + 4";

        //drive_compilation(input);

        
    }

















}
