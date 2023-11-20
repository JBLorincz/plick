use inkwell::values::{AnyValue, ArrayValue, FloatValue, StructValue};

use crate::{codegen::codegen::{CodeGenable, Compiler}, ast, lexer, types::{character, traits::get_mathable_type}};



impl<'a, 'ctx> CodeGenable<'a, 'ctx> for ast::Expr {
        unsafe fn codegen(
            mut self,
            compiler: &'a Compiler<'a, 'ctx>,
        ) -> Box<dyn AnyValue<'ctx> + 'ctx> {
            match self {
                ast::Expr::Variable { name, _type } => {
                    compiler.generate_variable_code(&name).unwrap()
                }
                ast::Expr::Binary {
                    operator,
                    left,
                    right,
                } => {
                    let bin_res = compiler.generate_binary_expression_code(ast::Expr::Binary {
                        operator,
                        left,
                        right,
                    });
                    let binary_value = bin_res.unwrap();
                    binary_value
                }
                ast::Expr::NumVal { value, _type } => {

                    Box::new(compiler.gen_const_fixed_decimal(value as f64))
                }
                ast::Expr::Char { value } => {

                    let character_value =
                        character::generate_character_code(compiler.context, &value);
                    let arr_value: ArrayValue = character_value.into();
                    Box::new(arr_value)
                }
                ast::Expr::Call {
                    ref fn_name,
                    ref mut args,
                    _type,
                } => {
                    let function_call_result = compiler.generate_function_call_code(fn_name, args);
                    function_call_result.unwrap()
                }
                other => {
                    todo!("Implement codegen ability for {:#?}", other);
                }
            }
        }
    }




impl<'a, 'ctx> Compiler<'a,'ctx> 
{
            pub unsafe fn generate_binary_expression_code(
            &self,
            binary_expr: ast::Expr,
        ) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String> {
            if let ast::Expr::Binary {
                operator,
                left,
                right,
            } = binary_expr
            {
                let lhstype = left.get_type();
                let rhstype = right.get_type();

                let lhs_codegen = left.codegen(self);
                let rhs_codegen = right.codegen(self);
                dbg!(&lhs_codegen);
                dbg!(&rhs_codegen);
                let lhs_float: FloatValue<'ctx>;
                let rhs_float: FloatValue<'ctx>;
                //new mathable code
                
                let lhs_mathable = get_mathable_type(lhs_codegen, lhstype)?;
                lhs_float = lhs_mathable.convert_to_float(self);

                let rhs_mathable = get_mathable_type(rhs_codegen, rhstype)?;
                rhs_float = rhs_mathable.convert_to_float(self);

                if true {
                    //TODO: Make this function return anyvalue and a fixed decimal
                    let compile_result: Result<Box<dyn AnyValue<'ctx> + 'ctx>, String> =
                        match operator {
                            lexer::Token::PLUS => {
                                let var = self
                                    .builder
                                    .build_float_add(lhs_float, rhs_float, "tmpadd")
                                    .unwrap();
                                let fix: StructValue<'ctx> = self.float_value_to_fixed_decimal(var).into();
                                Ok(Box::new(fix))
                            }
                            lexer::Token::MINUS => {
                                let floatval =
                                    self.builder.build_float_sub(lhs_float, rhs_float, "tmpsub").unwrap();
                                let fix: StructValue<'ctx> = self.float_value_to_fixed_decimal(floatval).into();
                                Ok(Box::new(fix))
                            }
                            lexer::Token::MULTIPLY => {
                                let var =
                                    self.builder.build_float_mul(lhs_float, rhs_float, "tmpmul")
                                    .unwrap();
                                let fix: StructValue<'ctx> = self.float_value_to_fixed_decimal(var).into();
                                Ok(Box::new(fix))
                            }
                            lexer::Token::DIVIDE => {
                                let var =
                                    self.builder.build_float_div(lhs_float, rhs_float, "tmpdiv")
                                    .unwrap();
                                let fix: StructValue<'ctx> = self.float_value_to_fixed_decimal(var).into();
                                Ok(Box::new(fix))
                            }
                            lexer::Token::LESS_THAN => {
                                let val = self
                                    .builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OLT,
                                        lhs_float,
                                        rhs_float,
                                        "tmplt",
                                    )
                                    .map_err(|builder_error| {
                                        format!(
                                            "Unable to create less than situation: {}",
                                            builder_error
                                        )
                                    })?;

                                let cmp_as_float = self
                                    .builder
                                    .build_unsigned_int_to_float(
                                        val,
                                        self.context.f64_type(),
                                        "tmpbool",
                                    )
                                    .map_err(|e| {
                                        format!("Unable to convert unsigned int to float: {}", e)
                                    })?;

                                Ok(Box::new(cmp_as_float))
                            }
                            lexer::Token::GREATER_THAN => {
                                let val = self
                                    .builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::OGT,
                                        lhs_float,
                                        rhs_float,
                                        "tmpgt",
                                    )
                                    .map_err(|builder_error| {
                                        format!(
                                            "Unable to create greater than situation: {}",
                                            builder_error
                                        )
                                    })?;

                                let cmp_as_float = self
                                    .builder
                                    .build_unsigned_int_to_float(
                                        val,
                                        self.context.f64_type(),
                                        "tmpbool",
                                    )
                                    .map_err(|e| {
                                        format!("Unable to convert unsigned int to float: {}", e)
                                    })?;
                                Ok(Box::new(cmp_as_float))
                            }
                            _ => {
                                return Err(format!(
                                    "Binary operator had unexpected operator! {:?}",
                                    operator
                                ))
                            }
                        };

                    return compile_result.map_err(|builder_error| {
                        format!(
                            "There was an error building the binary expression: {}",
                            builder_error
                        )
                    });
                } else {
                    Err("Cannot generate binary expression IR without float values!".to_string())
                }
            } else {
                Err("Fed non binary expression to generate binary expression code!".to_string())
            }
        }
}
