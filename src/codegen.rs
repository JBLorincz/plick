mod codegen {

use crate::parser;
use inkwell::builder;
use inkwell::context;
use inkwell::module;
use inkwell::values::AnyValue;
use inkwell::values::FloatValue;

    ///The object that drives compilation.
    pub struct Compiler<'a>
    {
        builder: builder::Builder<'a>,
        context: context::Context,
        module: module::Module<'a>,
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
    }

    

} 

mod tests {

}
