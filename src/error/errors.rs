use std::{error::Error, fmt::Display};

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
