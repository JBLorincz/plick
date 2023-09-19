use std::{env, fs::{self}};

use lexer::Token;
use parser::{parse_expression, parse_function, parse_opening};

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


fn drive_compilation(compilable_file: &str)
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
                parse_function(&mut token_manager);
           }, 
            _ => {parse_expression(&mut token_manager);},
        }
    }
}



mod tests {
    use crate::drive_compilation;

    
    #[test]
    fn drive_hello_world(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; A + 4";

        drive_compilation(input);

        
    }
}
