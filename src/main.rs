use std::{env, process, fs};

use plick::{Config, compile_input, initialize_logger};

fn main() {
    
    initialize_logger();
    let cli_arguments = parse_cli_arguments();

    let file_to_compile_as_string = read_file_to_string(&cli_arguments.path_to_file);

    let config = Config::default();
    compile_input(&file_to_compile_as_string,config);
}


pub struct Arguments
{
    working_directory: String,
    path_to_file: String 
}

fn parse_cli_arguments() -> Arguments
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


fn read_file_to_string(path_to_file: &str) -> String
{
    match fs::read_to_string(path_to_file)
        {
            Ok(file_text) => file_text,
            Err(err) => 
            {
                println!("fatal error: {}", err);
                process::exit(1);
            }
        }
}
