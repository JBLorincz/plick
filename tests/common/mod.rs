use std::{error::Error, env};

use plick::{Config, compile_input};
use env_logger::Env;
const RUST_LOG_CONFIG_STRING: &str = "trace";
pub fn initialize_test_logger()
{
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", RUST_LOG_CONFIG_STRING)
    }

    let _ = env_logger::builder().is_test(true).try_init();
    //env_logger::init();
}

pub fn test_normal_compile(input: &str) -> Result<(), Box<dyn Error>>
{

    let conf = generate_test_config();
        compile_input(input,conf);
        Ok(())
}


pub fn generate_test_config() -> Config
{
    let config = 
         Config
         {
            dry_run: false,
            ..Config::default()
         };


    config
}
