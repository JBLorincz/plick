use std::{error::Error, env, mem, process::{Command, Output}, time::UNIX_EPOCH, path::Path};

use plick::{Config, compile_input};
use uuid::Uuid;
const RUST_LOG_CONFIG_STRING: &str = "trace";
pub fn initialize_test_logger()
{
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", RUST_LOG_CONFIG_STRING)
    }

    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn test_normal_compile(input: &str) -> Result<(), Box<dyn Error>>
{

    let conf = generate_test_config();
        compile_input(input,conf);
        Ok(())
}


pub fn run_new_test(input: &str) -> Result<RunTestResult, Box<dyn Error>>
{
       initialize_test_logger();

        let output = full_compile_test_and_run(input)?;

            let output_string: String;
            let stderr_string: String;
            let exit_code: i32;
    unsafe
        {
            output_string = String::from_utf8_unchecked(output.stdout);
            stderr_string = String::from_utf8_unchecked(output.stderr);
            exit_code = output.status.code().unwrap_or(0);
        }

        
        
        Ok(RunTestResult::new(output_string,stderr_string,exit_code))

}

pub fn full_compile_test_and_run(input: &str) -> Result<Output, Box<dyn Error>>
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
    if cfg!(target_env="msvc")
    {
    return self.link_file_msvc();
    }
    else
    {
    return self.link_file_gnu();
    }
}

fn link_file_gnu(&self) -> Result<(), Box<dyn Error>>
{
       Command::new("cc")
        .arg(&self.path_to_object_file)
        .arg("-o")
        .arg(&self.path_to_exe)
        .arg("-lm")
        .spawn()
        .expect("cc command failed to start")
        .wait()?;

       Ok(())
}


fn link_file_msvc(&self) -> Result<(), Box<dyn Error>>
{
       Command::new("cl")
        .arg(&self.path_to_object_file)
        .arg("/Fe".to_owned()+&self.path_to_exe)
        .arg("/link")
        .arg("msvcrt.lib")
        .arg("legacy_stdio_definitions.lib")
        .spawn()
        .expect("cc command failed to start")
        .wait()?;

        Ok(())
}



fn run_file(&self) -> Result<Output, Box<dyn Error>>
{

    if cfg!(target_env="msvc")
    {
    return self.run_file_windows();
    }
    else
    {
    return self.run_file_unix();
    }
}

fn run_file_unix(&self) -> Result<Output, Box<dyn Error>>
{
        dbg!(&self.path_to_exe);
       let program_output = Command::new(&self.path_to_exe)
           .output()
           .expect("Failed to run the test command!")
           ;


       Ok(program_output)

}

fn run_file_windows(&self) -> Result<Output, Box<dyn Error>>
{
    let file_name = Path::new(&self.path_to_exe).file_name().unwrap();

       let program_output = Command::new("cmd")
           .arg("/C")
           .arg(file_name)
           .output()
           .expect("Failed to run the test command!")
           ;


       Ok(program_output)
}
fn cleanup(&self)
{

    if cfg!(target_os = "windows")
    {
    return self.cleanup_windows();
    }
    else
    {
    return self.cleanup_unix();
    }
}

fn cleanup_unix(&self)
{
       Command::new("rm")
            .arg(&self.path_to_exe)
            .arg(&self.path_to_object_file)
           .spawn()
           .expect("Failed to run the test command!")
           .wait()
           .expect("Trouble running file!");
}

fn cleanup_windows(&self)
{
       Command::new("cmd")
            .arg("/C")
            .arg("del")
            .arg(&self.path_to_exe)
            .arg(&self.path_to_object_file)
           .spawn()
           .expect("Failed to run the test command!")
           .wait()
           .expect("Trouble running file!");
}

}


pub struct RunTestResult
{
    pub stdout: String,
    pub stderr: String,
    pub error_code: i32,
}

impl RunTestResult
{
    pub fn new(stdout: String, stderr: String, errorcode: i32) -> Self
    {
        RunTestResult { stdout, stderr, error_code: errorcode }
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
