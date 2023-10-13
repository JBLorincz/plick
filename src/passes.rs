use crate::{ast::Statement, lexer::{Token, TokenManager}, parser::{parse_opening, self}, codegen::codegen::{Compiler, CodeGenable}};

pub struct PassResult
{
    statements: Vec<Statement>
}



pub fn perform_parse_pass(token_manager: &mut TokenManager) -> Result<PassResult, String>
{
    parse_opening(token_manager)?;

    let mut found_top_level_end = false;
    let mut statements: Vec<Statement> = vec![];
    //compiler.initalize_main_function();

        //Below is introducing "builtin functions" the compiler needs to accomplish things like IO

     //   let printf_arg_type: PointerType<'ctx> = compiler.context.i8_type().ptr_type(AddressSpace::default());
     //       let printf_type: FunctionType<'ctx> = compiler.context.i32_type().fn_type(&[BasicMetadataTypeEnum::from(printf_arg_type)], true);
    

        //    let _printf_func = compiler.module.add_function("printf", printf_type, Some(module::Linkage::DLLImport));

      while let Some(ref token) = token_manager.current_token
      {
          if let Token::END = token
          {
              found_top_level_end = true;
          //    let build_return_result = compiler.builder.build_return(None);
          //    if let Err(err_msg) = build_return_result
          //    {
          //        return Err(err_msg.to_string());
          //    }
              break;
          }
          let parser_result = parser::parse_statement(token_manager);
          
          if let Err(err_msg) = parser_result
          {
              let msg = format!("Finished parsing: {}", err_msg);
              return Err(msg);
          }
          let parser_result = parser_result.unwrap();
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
         
         let output = PassResult {statements};
         Ok(output)
}

impl PassResult
{
    pub fn perform_type_pass(self) -> Result<PassResult, String>
    {
        Ok(self)
    }
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




