mod codegen {

use std::collections::HashMap;
use std::vec;

use crate::lexer;
use crate::parser;
use inkwell::basic_block::BasicBlock;
use inkwell::builder;
use inkwell::context;
use inkwell::module;
use inkwell::types::AnyType;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::FloatType;
use inkwell::types::FunctionType;
use inkwell::values::AnyValue;
use inkwell::values::AnyValueEnum;
use inkwell::values::BasicValue;
use inkwell::values::FloatValue;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;

    ///The object that drives compilation.
    pub struct Compiler<'a, 'ctx>
    {
        pub context: &'ctx context::Context,
        pub builder: &'a builder::Builder<'ctx>,       
        pub module: &'a module::Module<'ctx>,


        named_values: HashMap<String,PointerValue<'ctx>>,
    }


    ///A trait which all provides an interface to compile a syntax element
    pub trait CodeGenable<'a,'ctx>
    {

        unsafe fn codegen(self, compiler: &'a Compiler<'a, 'ctx>) -> Box<dyn AnyValue <'ctx> + 'ctx>;
    }


    impl<'a, 'ctx> CodeGenable<'a, 'ctx> for parser::Expr
    {

        unsafe fn codegen(self, compiler: &'a Compiler<'a, 'ctx>) -> Box<dyn AnyValue <'ctx> +'ctx>
        {
            match self {
                parser::Expr::Variable { name } => compiler.generate_variable_code(&name),
                parser::Expr::NumVal { value } => Box::new(compiler.generate_float_code(value as f64)),
                _ => compiler.generate_variable_code(&String::from("ey")),
            }
        }
    }

    impl<'a, 'ctx> Compiler<'a, 'ctx>
    {
        unsafe fn generate_float_code(&self,value: f64) -> FloatValue<'ctx>
        {
            self.context.f64_type().const_float(value)
        }
        unsafe fn generate_variable_code(&self,variable_name: &str) -> Box<dyn AnyValue<'ctx> + 'ctx>
        {
            let result: Option<&PointerValue> = self.named_values.get(variable_name);
            if let Some(&val) = result 
            {
                let myvar: Box<dyn AnyValue<'ctx>> = Box::new(val);
                return myvar;
            }
            else
            {
                panic!("Could not find variable named {}",variable_name);
            }

        }

        unsafe fn generate_binary_expression_code(&self, binary_expr: parser::Expr) -> FloatValue
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

                return compile_result.unwrap();                   
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

        unsafe fn generate_function_prototype_code(self: &'a Self, fn_name: String, proto_args: Vec<String>) -> FunctionValue<'ctx>
        {
            let ret_type = self.context.f64_type();
        

            let args_types = std::iter::repeat(ret_type) //make iterator that repeats f64_type
            .take(proto_args.len()) //limit it to the length of args iterations
            .map(|f| f.into()) 
            .collect::<Vec<BasicMetadataTypeEnum>>(); //convert the FloatType to BasicMetadataType
                                                      // Enum
            
            
            let args_types = args_types.as_slice(); //convert the vec to slice


            //create the function prototype type info
            let fn_type = self.context.f64_type().fn_type(args_types, false);// create the

            // create a new function prototype
            let func_val = self.module.add_function(&fn_name, fn_type, None);

            //name the arguments in the IR
            for (i,param) in func_val.get_param_iter().enumerate()
            {
               param.into_float_value().set_name(proto_args[i].as_str());
            }

            func_val
        }


        unsafe fn generate_function_code(&'a mut self, func: parser::Function) -> FunctionValue
        {
            
            //see if the function has already been defined
            if let Some(_) = self.module.get_function(&func.proto.fn_name)
            {                               //if a func already exists
               panic!("function named {} already exists!",func.proto.fn_name);
            }
            
            //clear the named values, which stores all the recognized identifiers
            self.named_values.clear();

            //generate the IR for the function prototype
            let func_name = func.proto.fn_name.clone();
            let proto_args = func.proto.args.clone();
            let proto_code = self.generate_function_prototype_code(func_name,proto_args);

            //create a new scope block for the function
            let new_func_block: BasicBlock = self.context.append_basic_block(proto_code, "entry");

            //position the builder's cursor inside that block
            self.builder.position_at_end(new_func_block);

            //fill up the NamedValues array 
            for (i,arg) in proto_code.get_param_iter().enumerate()
            {
                self.named_values.insert(func.proto.args[i].clone(),arg.into_pointer_value());
            }

            let function_code = func.body.codegen(self);
            let func_code_enum = function_code.as_any_value_enum();

            if let AnyValueEnum::FloatValue(a)  = func_code_enum {
                let output = self.builder.build_return(Some(&a as &dyn BasicValue));
            }
            else 
            {
                panic!("Function return type was not float value!");
            }

            proto_code
        }
    }

    

} 

mod tests {

}
