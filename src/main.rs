use std::{env, fs, process};

use clap::Parser;
use plick::{compile_input, initialize_logger, Config};

fn main() {
    initialize_logger();

    let cli_arguments = parse_cli_arguments();
    log::info!("CLI arguments: {:#?}", &cli_arguments);

    let file_to_compile_as_string = read_file_to_string(&cli_arguments.path_to_file);

    let config = Config::from(cli_arguments);
    let results = compile_input(&file_to_compile_as_string, config);

    if !results.was_successful {
        println!("Compilation failed!");
        process::exit(1);
    }
}

fn parse_cli_arguments() -> plick::cli::Arguments {
    let arguments = plick::cli::Arguments::parse();
    arguments
}

fn read_file_to_string(path_to_file: &str) -> String {
    match fs::read_to_string(path_to_file) {
        Ok(file_text) => file_text,
        Err(err) => {
            println!("fatal error: {}", err);
            process::exit(1);
        }
    }
}
