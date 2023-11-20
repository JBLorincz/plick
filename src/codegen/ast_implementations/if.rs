use std::error::Error;

use inkwell::values::FloatValue;

use crate::{ast, codegen::{codegen::CodeGenable, utils::get_current_function}, types::{Type, fixed_decimal::FixedValue}, error::get_error};

impl<'a,'ctx> CodeGenable<'a,'ctx> for ast::If
{
    unsafe fn codegen(self, compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>)
                -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> 
                {

                    let codegen_result = self.codegen_with_error_info(compiler);

                    if let Err(_inner_info) = codegen_result
                    {
                        let msg = "Error generating an ast::If statement!";
                        log::error!("{}",&msg);
                        panic!("{}",msg);
                    }

                    codegen_result.unwrap()
                }
}

impl<'a,'ctx> ast::If
{
  unsafe fn codegen_with_error_info(self, compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>)
                -> Result<Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx>, Box<dyn Error>> {
            
            let conditional_type = self.conditional.get_type();

            log::trace!("Conditional in if statement: {:#?}",&self.conditional);

            let conditional_code = self.conditional.codegen(compiler);

            let conditional_as_float: FloatValue;

            match conditional_type {
                Type::FixedDecimal => {
                    let fixed_value =
                        FixedValue::from(conditional_code.as_any_value_enum().into_struct_value());
                    conditional_as_float = compiler.fixed_decimal_to_float(&fixed_value);
                }
                Type::Char(_size) => {
                    panic!("Can't support type Char in if conditional!");
                }
                Type::TBD => {
                    todo!("Can't support type TBD in if conditional!");
                }
                Type::Float => {
                    todo!("Can't support type Float in if conditional!");
                }
                Type::Void => {
                    todo!("Can't support type Void in if conditional!");
                }
            };
            dbg!(&conditional_as_float);
            let comparison = compiler
                .builder
                .build_float_compare(
                    inkwell::FloatPredicate::ONE,
                    conditional_as_float,
                    compiler.generate_float_code(0.0),
                    "ifcond",
                )
                .unwrap();

            //now we build the THEN block
            let current_func = get_current_function(compiler);

            let mut then_block = compiler.context.append_basic_block(current_func, "then");
            let mut else_block = compiler.context.append_basic_block(current_func, "else");
            let if_cont_block = compiler.context.append_basic_block(current_func, "ifcont");

            compiler.builder
                .build_conditional_branch(comparison, then_block, else_block)
                .map_err(|err| get_error(&["8", &err.to_string()]))?;

            compiler.builder.position_at_end(then_block);
            for statement in self.then_statements {
                statement.codegen(compiler);
            }
            //now we add a statement to jump to the if_cont block
            compiler.builder.build_unconditional_branch(if_cont_block)?;
            then_block = compiler.builder.get_insert_block().unwrap();
            //handle else here

            compiler.builder.position_at_end(else_block);
            if let Some(else_statements) = self.else_statements {
                for statement in else_statements {
                    statement.codegen(compiler);
                }
            }
            //now we add a statement to jump to the if_cont block
            compiler.builder.build_unconditional_branch(if_cont_block)?;
            else_block = compiler.builder.get_insert_block().unwrap();

            //handle merge block
            compiler.builder.position_at_end(if_cont_block);
            let return_value = compiler.generate_float_code(-999.0);
            Ok(Box::new(return_value))

    }

}


