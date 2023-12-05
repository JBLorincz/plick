use std::collections::HashMap;
///The error module contains all of the error types.

#[macro_use]
pub mod errors;

pub fn get_error(params: &[&str]) -> String {
    if params.len() == 0 {
        return "Unknown Error".to_string();
    }
    let split_op = params.split_first();
    if let None = split_op {
        return "Unknown Error".to_string();
    }

    let arguments = split_op.unwrap().1;
    let error_code = params[0];
    for error_pair in DIAGNOSTICS.iter() {
        let full_error_code = error_pair.0;
        if full_error_code.ends_with(error_code) {
            let mut error_message = error_pair.1.to_string();
            for (i, arg) in arguments.iter().enumerate() {
                error_message = error_message.replace(format!("[{}]", i).as_str(), arg);
            }

            let msg = format!("Error {}: {}", full_error_code, error_message);
            return msg;
        }
    }

    return "Unknown Error".to_string();
}

//The way to use this: get_error(&["8", "This is dynamic arguments here"]);
create_errors! {
    E001: "Expected '[0]', recieved '[1]' [2]",
    E002: "End of file was reached unexpectedly",
    E003: "Can't declare label '[0]' after label '[1]'",
    E004: "Can't invoke command '[0]' after command'[1]'",
    E005: "Can't create an expression combining type '[0]' with type '[1]'",
    E006: "Duplicate return statements",
    E007: "Functions in PL/1 cannot return void! Use a subroutine instead!",
    E008: "Error building if statement: [0]",
    E009: "Can't assign when type of variable '[0]' does not match type of value '[1]'",
    E010: "Cannot find a block named '[0]'",
    E011: "Cannot define a block more than once: '[0]'",
    E012: "Variable '[0]' is undefined!",

}
