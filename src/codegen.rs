pub mod codegen {

use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::vec;

use crate::lexer;
use crate::parser;
use crate::parser::Expr;
use crate::parser::Function;
use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::PointerType;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::BasicValueEnum;
use inkwell::values::CallSiteValue;
use inkwell::values::InstructionValue;
use inkwell::{builder, context, module};
use inkwell::types::AnyType;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::types::FloatType;
use inkwell::types::FunctionType;
use std::cell::RefCell;
use inkwell::values::{AnyValue, AnyValueEnum, BasicValue, FloatValue, FunctionValue, PointerValue };

    ///The object that drives compilation.
    #[derive(Debug)]
    pub struct Compiler<'a, 'ctx>
    {
        pub context: &'ctx context::Context,
        pub builder: &'a builder::Builder<'ctx>,       
        pub module: &'a module::Module<'ctx>,


        pub named_values: HashMap<String,PointerValue<'ctx>>,
        pub arg_stores: RefCell<Vec<Vec<BasicMetadataValueEnum<'ctx>>>>,
    }


    ///A trait which all provides an interface to compile a syntax element
    pub trait CodeGenable<'a,'ctx>
    {

        unsafe fn codegen(self, compiler: &'a Compiler<'a, 'ctx>) -> Box<dyn AnyValue <'ctx> + 'ctx>;
    }


    impl<'a, 'ctx> CodeGenable<'a, 'ctx> for parser::Expr
    {
       

        unsafe fn codegen(mut self, compiler: &'a Compiler<'a, 'ctx>) -> Box<dyn AnyValue <'ctx> +'ctx>
        {
            match self {
                parser::Expr::Variable { name } => compiler.generate_variable_code(&name).unwrap(),
                parser::Expr::Binary{operator, left, right}  => {
                
                    let bin_res = compiler.generate_binary_expression_code( parser::Expr::Binary {operator, left, right});
                    let binary_value = bin_res.unwrap();
                    Box::new(binary_value)
                },
                parser::Expr::NumVal { value } => 
                {
                    Box::new(compiler.generate_float_code(value as f64))
                },
                parser::Expr::Call { ref fn_name, ref mut args } => {
                     let function_call_result = compiler.generate_function_call_code( fn_name, args );
                    function_call_result.unwrap()
                },
                _ => {
                    compiler.generate_variable_code(&String::from("ey")).unwrap()
                },
            }
        }
    }

    impl<'a, 'ctx> Compiler<'a, 'ctx>
    {
         pub fn new(c: &'ctx Context, b: &'a Builder<'ctx>, m: &'a Module<'ctx>) -> Compiler<'a, 'ctx>
        {

            let named_values: HashMap<String,PointerValue<'ctx>> = HashMap::new();
            let arg_stores: RefCell<Vec<Vec<BasicMetadataValueEnum>>> = RefCell::new(vec![]); 
            Compiler { context: c, builder: b, module: m, named_values, arg_stores }
        }
        unsafe fn generate_function_call_code(&self,fn_name: &String,args: &mut Vec<parser::Expr>) 
            -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String>
        {
            let get_func_result:Option<FunctionValue<'ctx>> = self.module.get_function(&fn_name);
            if let None = get_func_result
            {
                return Err(format!("Could not find a function named {}",fn_name.to_string()));
            }
            let func_to_call: FunctionValue<'ctx> = get_func_result.unwrap();


            //handle argument checks here
            if args.len() != func_to_call.get_params().len()
            {
                return Err(format!("argument mismatch trying to create a call to function {}", fn_name));
            }

            let mut codegen_args: Vec<BasicMetadataValueEnum> = vec![];
            
            
             while args.len() > 0
            {
                                let current_arg = args.remove(0);
                                let v: Box<dyn AnyValue<'ctx>> = current_arg.codegen(self);
                                let bve: BasicValueEnum =  match v.as_any_value_enum()
                                {
                                    AnyValueEnum::ArrayValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::IntValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::FloatValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::PointerValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::StructValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::VectorValue(v) => v.as_basic_value_enum(),
                                    _ => return Err("one of the arguments was not a basic value".to_string()),
                                };
                                codegen_args.push(bve.into());

            }                                    
                let call_result = self.builder.build_call
                    (func_to_call, self.arg_stores.borrow().last().unwrap_or(&codegen_args), func_to_call.get_name().to_str().unwrap());

            match call_result {
                Ok(var) => {
                    if let Some(result_value) = var.try_as_basic_value().left()
                    {
                        Ok(Box::new(result_value.into_float_value()))
                    }
                    else
                    {
                        Ok(Box::new(var.try_as_basic_value().right().unwrap()))
                    }
                },
                Err(e) => Err(format!("Error trying to build a call to function {}", fn_name))
            }
        }


        pub unsafe fn generate_hello_world_print(&'a self,) -> CallSiteValue<'ctx>
        {
            
            let glob_string_ptr = self.builder.build_global_string_ptr("Hello World from PL/1!\n", "hello_world_str");
            
            let myptr = glob_string_ptr.unwrap().as_pointer_value();

            let res = self.builder.build_call(self.module.get_function("printf").unwrap(), &[BasicMetadataValueEnum::from(myptr)], "teffy");
            return res.unwrap();
        }

        unsafe fn generate_float_code(&'a self, value: f64) -> FloatValue<'ctx>
        {
            self.context.f64_type().const_float(value)
        }
        unsafe fn generate_variable_code(&self,variable_name: &str) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, &'static str>
        {
            let result: Option<&PointerValue> = self.named_values.get(variable_name);
            let result_float: FloatValue = self
                .builder
                .build_load(*result.ok_or("Could not find {} in the scope")?,variable_name)
                .map_err(|err| "error building a variable code")?
                .into_float_value();

            return Ok(Box::new(result_float));
        }

        unsafe fn generate_binary_expression_code(&self, binary_expr: parser::Expr) -> Result<FloatValue<'ctx>, String>
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
                    lexer::Token::LESS_THAN => {
                            let val = self.builder
                            .build_float_compare(inkwell::FloatPredicate::OLT, lhs_float,rhs_float, "tmplt")
                            .map_err(
                                |builder_error| 
                                format!("Unable to create less than situation: {}",
                                        builder_error)
                                )?
                            ;
                            
                            let cmp_as_float = self
                                .builder
                                .build_unsigned_int_to_float(val, self.context.f64_type(), "tmpbool")
                                .map_err(|e| format!("Unable to convert unsigned int to float: {}", e))?;
                           Ok(cmp_as_float) 
                    },
                     lexer::Token::GREATER_THAN => {
                            let val = self.builder
                            .build_float_compare(inkwell::FloatPredicate::OGT, lhs_float,rhs_float, "tmpgt")
                            .map_err(
                                |builder_error| format!("Unable to create greater than situation: {}", builder_error))?
                            ;
                            
                            let cmp_as_float = self
                                .builder
                                .build_unsigned_int_to_float(val, self.context.f64_type(), "tmpbool")
                                .map_err(|e| format!("Unable to convert unsigned int to float: {}", e))?;
                           Ok(cmp_as_float) 
                    },
                    _ => return Err(format!("Binary operator had unexpected operator! {:?}", operator)),
                };

                return compile_result.map_err(|builder_error| "There was an error building the binary expression.".to_string());                   
                }
                else
                {
                    Err("Cannot generate binary expression IR without float values!".to_string())
                }

            }
            else
            {
                Err("Fed non binary expression to generate binary expression code!".to_string())
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

        pub unsafe fn generate_function_code(&mut self, func: parser::Function) -> Result<FunctionValue<'ctx>, String>
        {
            
            //see if the function has already been defined
            if let Some(_) = self.module.get_function(&func.prototype.fn_name)
            {                               //if a func already exists
               return Err(format!("function named {} already exists!",func.prototype.fn_name));
            }
            
            //clear the named values, which stores all the recognized identifiers
            self.named_values.clear();
    
            //generate the IR for the function prototype
            let func_name = func.prototype.fn_name.clone();
            let proto_args = func.prototype.args.clone();
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
                let alloca = self.create_entry_block_alloca(&func.prototype.args[i], &function);
                self.builder.build_store(alloca, arg).map_err(|builder_err| format!("Was unable to build_store for {:?}",arg).to_string())?;
                self.named_values.insert(func.prototype.args[i].clone(),alloca);
            }

            let function_code = func.body.codegen(self);
            let func_code_enum = function_code.as_any_value_enum();

            if let AnyValueEnum::FloatValue(a)  = func_code_enum {
                let output = self.builder.build_return(Some(&a as &dyn BasicValue));
            }
            else 
            {
                return Err("Function return type was not float value!".to_string());
            }

            Ok(function)
        }


    pub fn initalize_main_function(&self)
    {
            let args: Vec<BasicMetadataTypeEnum> = vec![];
            let main_function_type = self.context.void_type().fn_type(&args, false);
            let main_func = self.module.add_function("main", main_function_type, None);
            //create a new scope block for the function
            let new_func_block = self.context.append_basic_block(main_func, "entry");

            //position the builder's cursor inside that block
            self.builder.position_at_end(new_func_block);



    }


    }
} 

mod tests {
    use std::collections::HashMap;

    use inkwell::{values::{PointerValue, BasicMetadataValueEnum}, context::Context, builder::Builder, module::Module, types::BasicMetadataTypeEnum};

    use crate::{parser::{Expr, Function, Prototype}, codegen::codegen::{CodeGenable, Compiler}, lexer::Token};
    use std::cell::RefCell;
    fn get_test_compiler<'a, 'ctx>(c: &'ctx Context, m: &'a Module<'ctx>, b: &'a Builder<'ctx>) -> Compiler<'a, 'ctx>
    {
        let context = c;
        let module = m;
        let builder = b;
        let named_values: HashMap<String,PointerValue> = HashMap::new();
        let arg_stores: RefCell<Vec<Vec<BasicMetadataValueEnum>>> = RefCell::new(vec![]);
        let compiler = Compiler {
           context,
           module,
           builder,
           named_values,
           arg_stores
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
    fn test_comparisons()
    {
        let c = Context::create();
        let m = c.create_module("repl");
        let b = c.create_builder();
        let compiler = get_test_compiler(&c, &m, &b);
        
        //create a MAIN function here
        compiler.initalize_main_function();
        //finish creating a main function

        let left = Box::new(Expr::NumVal { value: 3 });
        
        let right = Box::new(Expr::NumVal { value: 5});

        let my_binary = Expr::Binary { operator: Token::LESS_THAN, left, right };
        unsafe {
        let result = my_binary.codegen(&compiler);
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
        let my_func = Function {prototype: my_proto, body: binop};

        unsafe {
            
            let result = compiler.generate_function_code(my_func);
        }
    }


}
