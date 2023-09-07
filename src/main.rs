use std::{env, fs::{self}};

mod lexer;
mod parser;
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
    let mut token_list: Vec<lexer::Token> = lexer::get_token_list(compilable_file);
    while let Some(token) = token_list.pop()
    {

    }
}
