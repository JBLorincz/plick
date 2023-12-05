use std::error::Error;

use crate::codegen::{named_value_store::NamedValueStore, utils::build_pow};
use crate::{
    ast,
    codegen::{
        codegen::{CodeGenable, Compiler},
        utils::print_float_value,
    },
    lexer,
    types::{
        character, fixed_decimal::FixedValue, resolve_types, traits::get_mathable_type,
        traits::MathableFactory, Type,
    },
};
use inkwell::{
    values::{AnyValue, ArrayValue, BasicMetadataValueEnum, FloatValue, StructValue},
    FloatPredicate,
};

impl<'a, 'ctx> CodeGenable<'a, 'ctx> for ast::Expr {
    unsafe fn codegen(
        mut self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Box<dyn AnyValue<'ctx> + 'ctx> {
        match self {
            ast::Expr::Variable { name, _type } => {
                let variable_result = compiler.generate_variable_code(&name);
                if let Err(_msg) = variable_result {
                    compiler.error_module.store_msg_from_number(&["12", &name]);
                    compiler.ret_zero()
                } else {
                    variable_result.unwrap()
                }
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
            ast::Expr::Assignment {
                variable_name,
                value,
            } => {
                let variable_type = compiler.named_values.try_get(&variable_name).unwrap()._type;
                let comparison_operation = ast::Expr::Binary {
                    operator: lexer::Token::EQ,
                    left: Box::new(ast::Expr::Variable {
                        _type: variable_type,
                        name: variable_name,
                    }),
                    right: value,
                };

                comparison_operation.codegen(compiler)
            }
            ast::Expr::NumVal { value, _type } => {
                Box::new(compiler.gen_const_fixed_decimal(value as f64))
            }
            ast::Expr::Char { value } => {
                let character_value = character::generate_character_code(compiler.context, &value);
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

impl<'a, 'ctx> Compiler<'a, 'ctx> {
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
            let lhstype = left.get_type(self);
            let rhstype = right.get_type(self);

            let lhs_codegen = left.codegen(self);
            let rhs_codegen = right.codegen(self);

            log::trace!(
                "Binary expression codegens: {:?}, {:?}",
                &lhs_codegen,
                &rhs_codegen
            );

            let lhs_float: FloatValue<'ctx>;
            let rhs_float: FloatValue<'ctx>;
            //new mathable code

            let lhs_mathable = get_mathable_type(lhs_codegen, lhstype)?;
            lhs_float = lhs_mathable.convert_to_float(self);

            let rhs_mathable = get_mathable_type(rhs_codegen, rhstype)?;
            rhs_float = rhs_mathable.convert_to_float(self);

            let output_type = resolve_types(&lhstype, &rhstype).unwrap();

            if true {
                //TODO: Make this function return anyvalue and a fixed decimal
                let compile_result =
                    self.generate_binary_math_code(lhs_float, rhs_float, operator, output_type);

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

    unsafe fn generate_binary_math_code(
        &self,
        lhs_float: FloatValue<'ctx>,
        rhs_float: FloatValue<'ctx>,
        operator: lexer::Token,
        output_type: Type,
    ) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String> {
        let binary_coder =
            BinaryMathCodeEmitter::new(lhs_float, rhs_float, operator.clone(), output_type);
        binary_coder.gen_into_type(self)
    }
}

struct BinaryMathCodeEmitter<'ctx> {
    lhs_float: FloatValue<'ctx>,
    rhs_float: FloatValue<'ctx>,
    operator: lexer::Token,
    output_type: Type,
}

impl<'ctx> BinaryMathCodeEmitter<'ctx> {
    fn new(
        lhs_float: FloatValue<'ctx>,
        rhs_float: FloatValue<'ctx>,
        operator: lexer::Token,
        output_type: Type,
    ) -> Self {
        BinaryMathCodeEmitter {
            lhs_float,
            rhs_float,
            operator,
            output_type,
        }
    }

    pub unsafe fn gen_into_type<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String> {
        let math_float_result: Result<FloatValue<'_>, String> = match self.operator {
            lexer::Token::PLUS => Ok(self.gen_add(compiler).unwrap()),
            lexer::Token::MINUS => Ok(self.gen_min(compiler).unwrap()),
            lexer::Token::MULTIPLY => Ok(self.gen_mul(compiler).unwrap()),
            lexer::Token::EXPONENT => Ok(self.gen_exp(compiler).unwrap()),
            lexer::Token::DIVIDE => Ok(self.gen_div(compiler).unwrap()),
            lexer::Token::LESS_THAN => Ok(self.gen_lt(compiler).unwrap()),
            lexer::Token::GREATER_THAN => Ok(self.gen_gt(compiler).unwrap()),
            lexer::Token::EQ => Ok(self.gen_eq(compiler).unwrap()),
            lexer::Token::AND => Ok(self.gen_and(compiler).unwrap()),
            _ => {
                return Err(format!(
                    "Binary operator had unexpected operator! {:?}",
                    self.operator
                ))
            }
        };

        let x = math_float_result.unwrap();

        let result = match self.output_type {
            Type::FixedDecimal => {
                let fixed_value = FixedValue::create_mathable(&x, compiler);
                let fd_as_struct: StructValue<'ctx> = fixed_value.value;
                return Ok(Box::new(fd_as_struct));
            }
            other => {
                panic!("Can't convert math output into type {}", other);
            }
        };
    }

    unsafe fn gen_add<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let var = compiler
            .builder
            .build_float_add(self.lhs_float, self.rhs_float, "tmpadd")
            .unwrap();

        //let fix: StructValue<'ctx> = compiler.float_value_to_fixed_decimal(var).into();

        //Ok(Box::new(fix))
        Ok(var)
    }
    unsafe fn gen_min<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let floatval = compiler
            .builder
            .build_float_sub(self.lhs_float, self.rhs_float, "tmpsub")
            .unwrap();

        Ok(floatval)
    }
    unsafe fn gen_mul<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let var = compiler
            .builder
            .build_float_mul(self.lhs_float, self.rhs_float, "tmpmul")
            .unwrap();
        //let fix: StructValue<'ctx> = compiler.float_value_to_fixed_decimal(var).into();
        //Ok(Box::new(fix))
        Ok(var)
    }
    unsafe fn gen_div<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let var = compiler
            .builder
            .build_float_div(self.lhs_float, self.rhs_float, "tmpdiv")
            .unwrap();
        //let fix: StructValue<'ctx> = compiler.float_value_to_fixed_decimal(var).into();
        //Ok(Box::new(fix))
        Ok(var)
    }
    unsafe fn gen_exp<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        Ok(build_pow(compiler, self.lhs_float, self.rhs_float))
    }
    unsafe fn gen_lt<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let pred = FloatPredicate::OLT;
        self.gen_cmp_operation(compiler, pred, "tmplt")
    }

    unsafe fn gen_gt<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let pred = FloatPredicate::OGT;
        self.gen_cmp_operation(compiler, pred, "tmpgt")
    }

    unsafe fn gen_eq<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let pred = FloatPredicate::OEQ;
        self.gen_cmp_operation(compiler, pred, "tmpeq")
    }
    unsafe fn gen_and<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let zero_intval = compiler.context.f64_type().const_zero();
        let left_true_or_false = compiler
            .builder
            .build_float_compare(FloatPredicate::ONE, self.lhs_float, zero_intval, "left_and")
            .map_err(|builder_error| {
                format!("Unable to create greater than situation: {}", builder_error)
            })?;

        let right_true_or_false = compiler
            .builder
            .build_float_compare(
                FloatPredicate::ONE,
                self.rhs_float,
                zero_intval,
                "right_and",
            )
            .map_err(|builder_error| {
                format!("Unable to create greater than situation: {}", builder_error)
            })?;

        let cmp_for_and = compiler
            .builder
            .build_and(left_true_or_false, right_true_or_false, "and")
            .unwrap();

        let cmp_as_float = compiler
            .builder
            .build_unsigned_int_to_float(cmp_for_and, compiler.context.f64_type(), "tmpbool")
            .map_err(|e| format!("Unable to convert unsigned int to float: {}", e))?;
        Ok(cmp_as_float)
    }

    unsafe fn gen_cmp_operation<'a>(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
        predicate: FloatPredicate,
        name: &str,
    ) -> Result<FloatValue<'ctx>, Box<dyn Error>> {
        let val = compiler
            .builder
            .build_float_compare(predicate, self.lhs_float, self.rhs_float, name)
            .map_err(|builder_error| {
                format!("Unable to create greater than situation: {}", builder_error)
            })?;

        let cmp_as_float = compiler
            .builder
            .build_unsigned_int_to_float(val, compiler.context.f64_type(), "tmpbool")
            .map_err(|e| format!("Unable to convert unsigned int to float: {}", e))?;
        Ok(cmp_as_float)
    }
}
