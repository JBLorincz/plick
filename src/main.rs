use std::{env, process, fs};

use clap::Parser;
use plick::{Config, compile_input, initialize_logger};

fn main() {
    
    initialize_logger();
    
    let cli_arguments = parse_cli_arguments();
    dbg!(&cli_arguments);


    let file_to_compile_as_string = read_file_to_string(&cli_arguments.path_to_file);

    let config = Config::from(cli_arguments);
    compile_input(&file_to_compile_as_string,config);
}




fn parse_cli_arguments() -> plick::cli::Arguments
{
    let arguments = plick::cli::Arguments::parse();
    arguments
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
