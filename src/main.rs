use std::{env, process, fs};

use plick::{Config, compile_input};

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








