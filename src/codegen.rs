pub mod codegen {

use std::collections::HashMap;
use std::hash::Hash;
use std::vec;

use crate::lexer;
use crate::parser;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::{builder, context, module};
use inkwell::types::AnyType;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::FloatType;
use inkwell::types::FunctionType;
use inkwell::values::{AnyValue, AnyValueEnum, BasicValue, FloatValue, FunctionValue, PointerValue };

    ///The object that drives compilation.
    pub struct Compiler<'a, 'ctx>
    {
        pub context: &'ctx context::Context,
        pub builder: &'a builder::Builder<'ctx>,       
        pub module: &'a module::Module<'ctx>,


        pub named_values: HashMap<String,PointerValue<'ctx>>,
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
                parser::Expr::Binary{operator, left, right}  => Box::new(compiler.generate_binary_expression_code( parser::Expr::Binary {operator, left, right})),
                parser::Expr::NumVal { value } => Box::new(compiler.generate_float_code(value as f64)),
                _ => compiler.generate_variable_code(&String::from("ey")),
            }
        }
    }

    impl<'a, 'ctx> Compiler<'a, 'ctx>
    {
         pub fn new(c: &'ctx Context, b: &'a Builder<'ctx>, m: &'a Module<'ctx>) -> Compiler<'a, 'ctx>
        {

            let named_values: HashMap<String,PointerValue<'ctx>> = HashMap::new();
            Compiler { context: c, builder: b, module: m, named_values }
        }
        unsafe fn generate_float_code(&self,value: f64) -> FloatValue<'ctx>
        {
            self.context.f64_type().const_float(value)
        }
        unsafe fn generate_variable_code(&self,variable_name: &str) -> Box<dyn AnyValue<'ctx> + 'ctx>
        {
            let result: Option<&PointerValue> = self.named_values.get(variable_name);
            let result_float: FloatValue = self.builder.build_load(*result.unwrap(),variable_name).unwrap().into_float_value();
            return Box::new(result_float);
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

        unsafe fn generate_binary_expression_code(&self, binary_expr: parser::Expr) -> FloatValue<'ctx>
        {
            if let parser::Expr::Binary { operator, left, right } = binary_expr
            {
                let lhs_codegen  = left.codegen(self);
                let rhs_codegen = right.codegen(self);
               
                let lhs_float = lhs_codegen.as_any_value_enum().into_float_value();
                let rhs_float = rhs_codegen.as_any_value_enum().into_float_value();
                
                if true
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
                    dbg!(lhs_float);
                    dbg!(rhs_float);
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

        fn create_entry_block_alloca(&self, name: &str, funct: &FunctionValue) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = funct.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name).unwrap()
    }
        pub unsafe fn generate_function_code(&mut self, func: parser::Function) -> FunctionValue<'ctx>
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
            let function = self.generate_function_prototype_code(func_name,proto_args);

            //TODO: Check if function body is empty
            //if so, return function here. 
            


            //create a new scope block for the function
            let new_func_block: BasicBlock = self.context.append_basic_block(function, "entry");

            //position the builder's cursor inside that block
            self.builder.position_at_end(new_func_block);

            //fill up the NamedValues array 
            for (i,arg) in function.get_param_iter().enumerate()
            {
                let alloca = self.create_entry_block_alloca(&func.proto.args[i], &function);
                self.builder.build_store(alloca, arg).unwrap();
                self.named_values.insert(func.proto.args[i].clone(),alloca);
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

            function
        }
    }

    

} 

mod tests {
    use std::collections::HashMap;

    use inkwell::{values::PointerValue, context::Context, builder::Builder, module::Module};

    use crate::{parser::{Expr, Function, Prototype}, codegen::codegen::{CodeGenable, Compiler}, lexer::Token};
    use inkwell::context;
    use super::*;

    fn get_test_compiler<'a, 'ctx>(c: &'ctx Context, m: &'a Module<'ctx>, b: &'a Builder<'ctx>) -> Compiler<'a, 'ctx>
    {
        let context = c;
        let module = m;
        let builder = b;
        let named_values: HashMap<String,PointerValue> = HashMap::new();
        let compiler = Compiler {
           context,
           module,
           builder,
           named_values
        };
        compiler
    }
    #[test]
    fn test_constant_codegen()
    {
        let c = Context::create();
        let m = c.create_module("repl");
        let b = c.create_builder();
        let compiler = get_test_compiler(&c, &m, &b);
        
        let consta = Expr::NumVal { value: 3 };

        unsafe {
        let result = consta.codegen(&compiler);
           println!("{}",result.print_to_string()); 
        dbg!("{}", result);
        }
    }

    #[test]
    fn test_binary_codegen()
    {
        
    let c = Context::create();
    let m = c.create_module("repl");
    let b = c.create_builder();
    let mut compiler = get_test_compiler(&c, &m, &b);
        
        let binop = Expr::Binary { operator: Token::MINUS, left: Box::new(Expr::Variable { name: String::from("APPLE") }) , right: Box::new(Expr::NumVal { value: 5 }) };
        let my_proto = Prototype {fn_name: String::from("myFuncName"),args: vec![String::from("APPLE")]};
        let my_func = Function {proto: my_proto, body: binop};

        unsafe {
            
            let result = compiler.generate_function_code(my_func);
           println!("{}",result); 
            dbg!("{}", result);
        }
    }


}
