use std::{env, process, fs};

use plick::{Config, compile_input};

fn main() {
    
    let args = handle_arguments();

    //now we have the path as compilable_file_path
    let input: String;
        match fs::read_to_string(args.path_to_file)
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


pub struct Arguments
{
    working_directory: String,
    path_to_file: String 
}

fn handle_arguments() -> Arguments
{
    let mut args_iter = env::args();
   
    let working_directory = args_iter.next().unwrap();

    let compilable_file_path_option = args_iter.next();
    let path_to_file; 
    match compilable_file_path_option{
    Some(path) => {
        println!("The path you gave was: {}", path);
        path_to_file = path;
    },
    None => {
        println!("fatal error: no input files");
        println!("compilation terminated.");
        process::exit(1);
    }
    };

    Arguments { working_directory, path_to_file }
}
