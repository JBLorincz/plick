use std::{error::Error, fmt::Display, cell::RefCell};

use super::*;

//used to generate all the error structs
macro_rules! create_errors
{
    ($($ecode:ident: $message:expr,)*) => (
        pub static DIAGNOSTICS: &[(&str, &str)] = &[
            $( (stringify!($ecode), $message), )*
        ];
    )

}

#[derive(Debug)]
pub struct ErrorModule
{
    compile_errors: RefCell<Vec<CodegenError>>,
    pub is_error_test: bool,
}
impl ErrorModule
{
    pub fn new(is_error_test: bool) -> Self
    {
        ErrorModule { 
            compile_errors: RefCell::new(vec![]),
            is_error_test
        }
    }

    pub fn store_error_msg(&self, message: &str)
    {
        log::error!("{}", message);

        let message = message.to_owned();

        self.compile_errors.borrow_mut().push(CodegenError{message});
    }
    pub fn store_msg_from_number(&self, params: &[&str])
    {
        let message = get_error(params);
        self.store_error_msg(&message);
    }
    pub fn get_number_of_errors(&self) -> usize
    {
        return self.compile_errors.borrow().len();

    }
    pub fn get_all_errors(&self) -> Vec<CodegenError>
    {
        self.compile_errors.borrow().clone()
    }
}



#[derive(Debug)]
pub struct CodegenError
{
    pub message: String
}

impl Display for CodegenError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CodegenError: {}", self.message)
    }
}

impl Error for CodegenError
{
}

impl Clone for CodegenError
{
    fn clone(&self) -> Self {
        CodegenError { message: self.message.clone() }
    }
}
