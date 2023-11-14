use std::{error::Error, env, mem, process::{Command, Output}, time::UNIX_EPOCH};

use plick::{Config, compile_input, compile_input_to_memory};
use env_logger::Env;
use uuid::Uuid;
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
pub fn test_memory_compile_and_run(input: &str) -> Result<Output, Box<dyn Error>>
{

    let mut conf = generate_test_config();

        let mystr: String = Uuid::new_v4().into();
        let path_to_object_file = "TEST_".to_string() + &mystr + ".o";
        let path_to_exe = "EXE_".to_string() + &mystr + ".exe";
        conf.filename = path_to_object_file.clone();
        
        compile_input(input,conf);
        

        let path_to_exe = "./".to_string()+&path_to_exe;

        let test_file = TestFile::new(&path_to_exe, &path_to_object_file);

        test_file.link_file()?;
        let output = test_file.run_file()?;
        dbg!(&output);
        test_file.cleanup();

             Ok(output) 
}



struct TestFile
{
    path_to_exe: String,
    path_to_object_file: String
}
impl TestFile
{
pub fn new(exe: &str, obj: &str) -> Self
{
    TestFile { path_to_exe: exe.to_string(), path_to_object_file: obj.to_string() }
}
fn link_file(&self) -> Result<(), Box<dyn Error>>
{
       Command::new("cc")
        .arg(&self.path_to_object_file)
        .arg("-o")
        .arg(&self.path_to_exe)
        .spawn()
        .expect("cc command failed to start")
        .wait()?;

       Ok(())
}

fn run_file(&self) -> Result<Output, Box<dyn Error>>
{
        dbg!(&self.path_to_exe);
       let program_output = Command::new(&self.path_to_exe)
           .output()
           .expect("Failed to run the test command!")
           ;


       Ok(program_output)

}

fn cleanup(&self)
{
       Command::new("rm")
            .arg(&self.path_to_exe)
            .arg(&self.path_to_object_file)
           .spawn()
           .expect("Failed to run the test command!")
           .wait()
           .expect("Trouble running file!");
}

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
