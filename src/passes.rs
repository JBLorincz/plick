use std::{collections::HashMap, hash::Hash};

use crate::{ast::{Statement, Command}, lexer::{Token, TokenManager}, parser::{parse_opening, self}, codegen::codegen::{Compiler, CodeGenable}, types::Type};

pub struct PassResult
{
    statements: Vec<Statement>,
    function_return_types: HashMap<String, Type>
}


///Utilizes the Lexer's TokenManager to create the AST in the form of PassResult
pub fn perform_parse_pass(token_manager: &mut TokenManager) -> Result<PassResult, String>
{
    parse_opening(token_manager)?;

    let mut found_top_level_end = false;
    let mut statements: Vec<Statement> = vec![];
    let mut function_return_types: HashMap<String,Type> = HashMap::new();

      while let Some(ref token) = token_manager.current_token
      {
          if let Token::END = token
          {
              found_top_level_end = true;
              break;
          }

          let parser_result = parser::parse_statement(token_manager);
          
          if let Err(err_msg) = parser_result
          {
              let msg = format!("Finished parsing: {}", err_msg);
              return Err(msg);
          }
          let parser_result = parser_result.unwrap();

          // if the statement is a function declaration,
          // then we store its return type.
          if let Command::FunctionDec(ref func) = parser_result.command
          {
              function_return_types.insert(func.prototype.fn_name.clone(), func.return_type);
              //todo!("Handle function dec storing! {:?}", func);
          }

          statements.push(parser_result);
        //  unsafe {
        //      dbg!(&parser_result);
        //    parser_result.codegen(compiler);
        //    println!("Genned above stuff.");
        //    println!("New token is: {:?}", token_manager.current_token);
        //}
      }

         if !found_top_level_end
         {
             return Err("Did not find an end to the program!".to_string());
         }
         let output = PassResult {statements, function_return_types};
         Ok(output)
}

impl PassResult
{
    ///Does all the type checking and type handling
    pub fn perform_type_pass(mut self) -> Result<PassResult, String>
    {
        let mut annotated_statements: Vec<Statement> = vec![];
        for i in &self.statements
        {
            let mut statement_clone = i.clone();

            //statement_clone.

            annotated_statements.push(statement_clone);
        }
        
        self.statements = annotated_statements;
        Ok(self)
    }

    ///Actually does the AST to LLVM IR conversion
    pub unsafe fn code_generation_pass(self, compiler: &mut Compiler) -> Result<(), String>
    {
        for i in self.statements
        {
            i.codegen(compiler);
        }

              let _build_return_result = compiler.builder.build_return(None)
                  .map_err(|_err| "Error in code generation pass")?;

              Ok(())
    }
}




