mod codegen {

use std::collections::HashMap;

use crate::parser;
use inkwell::builder;
use inkwell::context;
use inkwell::module;
use inkwell::values::AnyValue;
use inkwell::values::FloatValue;
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

        unsafe fn codegen<'a>(&'a self, compiler: &'a Compiler<'a>) -> Box<dyn AnyValue + 'a>;
    }


    impl CodeGenable for parser::Expr
    {

        unsafe fn codegen<'a>(&'a self, compiler: &'a Compiler<'a>) -> Box<dyn AnyValue + 'a>
        {
            match self {
                parser::Expr::NumVal { value } => Box::new(compiler.generate_float_code(*value as f64)),
                parser::Expr::Variable { name } => compiler.generate_variable_code(name),
                _ => Box::new(compiler.generate_float_code(-1.0)) 
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
                //let rhs_codegen = right.codegen(self);

                lhs_codegen
            }
            else
            {
                panic!("Fed non binary expression to generate binary expression code!");
            }
        }
    }

    

} 

mod tests {

}
