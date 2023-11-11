use std::error::Error;

use plick::{Config, compile_input};

pub fn initialize_test_logger()
{
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
            dry_run: true,
            ..Config::default()
         };


    config
}
