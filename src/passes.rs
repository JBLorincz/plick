use std::{collections::HashMap, hash::Hash, process};

use crate::{
    ast::{Command, Statement},
    codegen::{codegen::{CodeGenable, Compiler}, utils},
    lexer::{Token, TokenManager},
    parser::{self, parse_opening},
    types::Type, error::errors::CodegenError,
};

pub struct PassResult {
    statements: Vec<Statement>,
    function_return_types: HashMap<String, Type>,
}

///Utilizes the Lexer's TokenManager to create the AST in the form of PassResult
pub fn perform_parse_pass(token_manager: &mut TokenManager) -> Result<PassResult, Vec<CodegenError>> {
    let result = parse_opening(token_manager);


    let mut found_top_level_end = false;
    let mut statements: Vec<Statement> = vec![];
    let mut function_return_types: HashMap<String, Type> = HashMap::new();
    let mut parsing_errors: Vec<CodegenError> = vec![];

        if let Err(err_msg) = result {
            let msg = format!("{}", err_msg);
            parsing_errors.push(CodegenError { message: msg });
        }



    while let Some(ref token) = token_manager.current_token {
        if let Token::END = token {
            found_top_level_end = true;
            break;
        }

        let parser_result = parser::parse_statement(token_manager);

        if let Err(ref err_msg) = parser_result {
            let msg = format!("{}", err_msg);
            parsing_errors.push(CodegenError { message: msg });
        }
        else
        {
            let parser_result = parser_result.unwrap();
// if the statement is a function declaration,
        // then we store its return type.
        if let Command::FunctionDec(ref func) = parser_result.command {
            function_return_types.insert(func.prototype.fn_name.clone(), func.return_type);
            //todo!("Handle function dec storing! {:?}", func);
        }

        statements.push(parser_result);

        }
    }

    if !found_top_level_end {
        let message = "Did not find an end to the program!".to_string();
        let err = CodegenError {message};
        parsing_errors.push(err);
    }
    let output = PassResult {
        statements,
        function_return_types,
    };
    if parsing_errors.len() > 0
    {
        Err(parsing_errors)
    }
    else
    {
        Ok(output)
    }
}

impl PassResult {
    ///Does all the type checking and type handling
    pub fn perform_type_pass(mut self) -> Result<PassResult, String> {
        let mut annotated_statements: Vec<Statement> = vec![];
        for i in &self.statements {
            let mut statement_clone = i.clone();

            //statement_clone.

            annotated_statements.push(statement_clone);
        }

        self.statements = annotated_statements;
        Ok(self)
    }

    ///Actually does the AST to LLVM IR conversion
    pub unsafe fn code_generation_pass(self, compiler: &mut Compiler) -> Result<(), String> {
        for i in self.statements {
            i.codegen(compiler);
        }
        
        if let None = compiler.builder.get_insert_block().unwrap().get_terminator()
        {
        let _build_return_result = compiler
            .builder
            .build_return(None)
            .map_err(|_err| "Error in code generation pass")?;
        }


        compiler.verify_no_placeholder_blocks_exist();
        let num_of_errors = compiler.error_module.get_number_of_errors();
         
        if num_of_errors > 0
        {
            let message = format!("Halting compilation due to {} errors!", num_of_errors);
            return Err(message);
        }

        Ok(())
    }
}
