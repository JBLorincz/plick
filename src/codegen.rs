mod codegen {

use std::collections::HashMap;
use std::vec;

use crate::lexer;
use crate::parser;
use inkwell::builder;
use inkwell::context;
use inkwell::module;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::FloatType;
use inkwell::types::FunctionType;
use inkwell::values::AnyValue;
use inkwell::values::AnyValueEnum;
use inkwell::values::FloatValue;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;

    ///The object that drives compilation.
    pub struct Compiler<'a>
    {
        builder: builder::Builder<'a>,
        context: context::Context,
        module: module::Module<'a>,

        named_values: HashMap<String,PointerValue<'a>>,
    }


    ///A trait which all provides an interface to compile a syntax element
    pub trait CodeGenable
    {

        unsafe fn codegen<'a>(self, compiler: &'a Compiler<'a>) -> Box<dyn AnyValue + 'a>;
    }


    impl CodeGenable for parser::Expr
    {

        unsafe fn codegen<'a>(self, compiler: &'a Compiler<'a>) -> Box<dyn AnyValue + 'a>
        {
            match self {
                parser::Expr::Variable { name } => compiler.generate_variable_code(&name),
                parser::Expr::NumVal { value } => Box::new(compiler.generate_float_code(value as f64)),
                _ => compiler.generate_variable_code(&String::from("ey")),
            }
        }
    }

    impl<'a> Compiler<'a>
    {
        unsafe fn generate_float_code(&'a self,value: f64) -> FloatValue<'a>
        {
            self.context.f64_type().const_float(value)
        }
        unsafe fn generate_variable_code(&'a self,variable_name: &String) -> Box<dyn AnyValue +'a>
        {
            let result: Option<&PointerValue> = self.named_values.get(variable_name);
            if let Some(&val) = result 
            {
                let myvar: Box<dyn AnyValue> = Box::new(val);
                return myvar;
            }

            Box::new(self.generate_float_code(3.0))
        }

        unsafe fn generate_binary_expression_code(&'a self, binary_expr: parser::Expr) -> Box<dyn AnyValue +'a>
        {
            if let parser::Expr::Binary { operator, left, right } = binary_expr
            {
                let lhs_codegen  = left.codegen(self);
                let rhs_codegen = right.codegen(self);
               
                let lhs_float = lhs_codegen.as_any_value_enum();
                let rhs_float = rhs_codegen.as_any_value_enum();
                
                if let (AnyValueEnum::FloatValue( lhs_float ), AnyValueEnum::FloatValue(rhs_float)) = (lhs_float,rhs_float)
                {
 
                let compile_result = match operator {
                    lexer::Token::PLUS => self.builder.build_float_add(lhs_float, rhs_float, "tmpadd"),
                    lexer::Token::MINUS => self.builder.build_float_sub(lhs_float, rhs_float, "tmpsub"),
                    lexer::Token::MULTIPLY => self.builder.build_float_mul(lhs_float, rhs_float, "tmpmul"),
                    lexer::Token::DIVIDE => self.builder.build_float_div(lhs_float,rhs_float,"tmpdiv"),
                    _ => panic!("Binary operator had unexpected op!"),
                };

                return Box::new(compile_result.unwrap());                   
                }
                else
                {
                    panic!("binary expression code did not have float values!");
                }

            }
            else
            {
                panic!("Fed non binary expression to generate binary expression code!");
            }
        }

        unsafe fn generate_function_prototype_code(&'a self, proto: parser::Prototype) -> FunctionValue
        {
            let ret_type = self.context.f64_type();
        

            let args_types = std::iter::repeat(ret_type)
            .take(proto.args.len())
            .map(|f| f.into())
            .collect::<Vec<BasicMetadataTypeEnum>>();
            
            
            let args_types = args_types.as_slice();

            let fn_type = self.context.f64_type().fn_type(args_types, false);

            self.module.add_function(&proto.fn_name, fn_type, None)
        }
    }

    

} 

mod tests {

}
