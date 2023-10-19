use std::collections::HashMap;
///The error module contains all of the error types.

#[macro_use]
mod errors;

pub fn get_error(error_code: &[&str]) -> String
{
    if error_code.len() == 0
    {
        return "Unknown Error".to_string();
    }
    let split_op = error_code.split_first();
    if let None = split_op
    {
        return "Unknown Error".to_string();
    }

    let arguments = split_op.unwrap().1;


    for error in DIAGNOSTICS.iter()
    {
        if error.0.ends_with(error_code[0])
        {
            let mut inner_message = error.1.to_string();
            for (i,arg) in arguments.iter().enumerate()
            {
                inner_message = inner_message.replace(format!("[{}]",i).as_str(), arg);
            }


            let msg = format!("Error {}: {}",error.0,inner_message);
            return msg;
        }
    }


    return "Unknown Error".to_string();
}



create_errors!{
    E001: "Expected '[0]', recieved '[1]'",
    E002: "End of file was reached unexpectedly",
    E003: "Can't declare label '[0]' after label '[1]'",
    E004: "Can't invoke command '[0]' after command'[1]'",
    E005: "Can't create an expression combining type '[0]' with type '[1]'",
    E006: "Duplicate return statements",
    E007: "Functions in PL/1 cannot return void! Use a subroutine instead!",
}
