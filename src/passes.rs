use std::{collections::HashMap, hash::Hash, process, error::Error};

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
    pub found_errors: Vec<Box<dyn Error>>
}

///Utilizes the Lexer's TokenManager to create the AST in the form of PassResult
pub fn perform_parse_pass(token_manager: &mut TokenManager) -> PassResult {
    let result = parse_opening(token_manager);


    let mut found_top_level_end = false;
    let mut statements: Vec<Statement> = vec![];
    let mut function_return_types: HashMap<String, Type> = HashMap::new();
    let mut found_errors: Vec<Box<dyn Error>> = vec![];

        if let Err(parse_error) = result {
            found_errors.push(Box::new(parse_error));
        }



    while let Some(ref token) = token_manager.current_token {
        log::trace!("Cycling to token: {:#?}", token);
        if let Token::END = token {
            found_top_level_end = true;
            break;
        }

        let parser_result = parser::parse_statement(token_manager);

        if let Err(parse_error) = parser_result {

            found_errors.push(Box::new(parse_error));
            
            while token_manager.current_token != Some(Token::SEMICOLON)
            {

                if let None = token_manager.current_token
                {
                    break;
                }

                token_manager.next_token();
            }

                token_manager.next_token();
        }
        else
        {
            let parser_result = parser_result.unwrap();
            
        // if the statement is a function declaration,
        // then we store its return type.
        if let Command::FunctionDec(ref func) = parser_result.command {
            function_return_types.insert(func.prototype.fn_name.clone(), func.return_type);
        }

        statements.push(parser_result);

        }
    }

    if !found_top_level_end {
        let message = "Did not find an end to the program!".to_string();
        let err = CodegenError {message};
        found_errors.push(Box::new(err));
    }
    let output = PassResult {
        statements,
        function_return_types,
        found_errors
    };

        output
}

impl PassResult {

    ///Reserved function for pre-codegen type inference
    pub fn perform_type_pass(mut self) -> Result<PassResult, String> {
        let mut annotated_statements: Vec<Statement> = vec![];
        for i in &self.statements {

            let statement_clone = i.clone();

            annotated_statements.push(statement_clone);
        }

        self.statements = annotated_statements;
        Ok(self)
    }

    pub unsafe fn code_generation_pass(mut self, compiler: &mut Compiler) -> Result<Self, String> {
        for i in &self.statements {
            i.clone().codegen(compiler);
        }
        
        if let None = compiler.builder.get_insert_block().unwrap().get_terminator()
        {
        let _build_return_result = compiler
            .builder
            .build_return(None)
            .map_err(|_err| "Error in code generation pass")?;
        }


        compiler.verify_no_placeholder_blocks_exist();

        
        let mut mapped_vec: Vec<Box<dyn Error>> = compiler.error_module.get_all_errors()
            .iter()
            .map(|item| Box::new(item.clone()).into())
            .collect();

        self.found_errors.append(&mut mapped_vec);

        Ok(self)
    }


    pub fn get_errors_as_string(&self) -> Vec<String>
    {
        let mut string_vec: Vec<String> = vec![];
        for ref error in &self.found_errors
        {
            string_vec.push(error.to_string());
        }
        string_vec
    }
}
