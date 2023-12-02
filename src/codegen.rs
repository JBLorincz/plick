pub mod ast_implementations;
pub mod named_value_store;
pub mod utils;
pub mod prelude;
pub mod named_value;
pub mod codegen {

    use std::collections::HashMap;
    use std::error::Error;
    use std::vec;

    use crate::ast;
    use crate::ast::Command;
    use crate::ast::Expr;
    use crate::ast::Statement;
    use crate::codegen::named_value::NamedValue;
    use crate::codegen::utils;
    use crate::codegen::utils::branch_only_if_no_terminator;
    use crate::codegen::utils::print_float_value;
    use crate::debugger::DebugController;
    use crate::error::errors::CodegenError;
    use crate::error::errors::ErrorModule;
    use crate::error::get_error;
    use crate::lexer;
    use crate::types::character;
    use crate::types::character::CharValue;
    use crate::types::fixed_decimal;
    use crate::types::fixed_decimal::FixedValue;
    use crate::types::infer_pli_type_via_name;
    use crate::types::Type;
    use crate::types::TypeModule;
    use crate::types::traits::Puttable;
    use inkwell::basic_block::BasicBlock;
    use inkwell::builder::Builder;
    use inkwell::context::Context;
    use inkwell::debug_info::AsDIScope;
    use inkwell::debug_info::DILexicalBlock;
    use inkwell::debug_info::DILocation;
    use inkwell::debug_info::DISubprogram;
    use inkwell::module::Module;
    use inkwell::types::AnyTypeEnum;
    use inkwell::types::BasicMetadataTypeEnum;
    use inkwell::types::FunctionType;
    use inkwell::values::ArrayValue;
    use inkwell::values::BasicMetadataValueEnum;
    use inkwell::values::BasicValueEnum;
    use inkwell::values::CallSiteValue;
    use inkwell::values::InstructionValue;
    use inkwell::values::IntValue;
    use inkwell::values::StructValue;
    use inkwell::values::{
        AnyValue, AnyValueEnum, BasicValue, FloatValue, FunctionValue, PointerValue,
    };
    use inkwell::AddressSpace;
    use inkwell::{builder, context, module};
    use std::cell::RefCell;

    use super::named_value_store::NamedValueHashmapStore;
    use super::named_value_store::NamedValueStore;
    use super::utils::get_current_function;

    ///The object that drives compilation.
    #[derive(Debug)]
    pub struct Compiler<'a, 'ctx> {
        pub context: &'ctx context::Context,
        pub builder: &'a builder::Builder<'ctx>,
        pub module: &'a module::Module<'ctx>,
        pub type_module: TypeModule<'ctx>,
        pub error_module: ErrorModule,
        pub function_properties: RefCell<FunctionProperties<'ctx>>,
        pub debug_controller: Option<&'a DebugController<'ctx>>,

        pub named_values: NamedValueHashmapStore<'ctx>,
    }

    #[derive(Debug,Clone)]
    pub struct FunctionProperties<'ctx>
    {
        labeled_blocks: HashMap<String, BasicBlock<'ctx>>,
        future_jump_blocks: HashMap<String, BasicBlock<'ctx>>,
    }
    
    impl<'ctx> FunctionProperties<'ctx>
    {
        pub fn new() -> Self
        {
            let labeled_blocks = HashMap::new();
            let future_jump_blocks = HashMap::new();
            FunctionProperties 
            {
                labeled_blocks,
                future_jump_blocks
            }
        }

        pub fn reset(&mut self, other: &FunctionProperties<'ctx>)
        {
            if !self.future_jump_blocks.is_empty()
            {
                panic!("Trying to leave a function without having all blocks defined!");
            }

            self.labeled_blocks.clear();
            
            for kvp in other.labeled_blocks.iter()
            {
                self.labeled_blocks.insert(kvp.0.clone(), kvp.1.clone());
            }
            for kvp in other.future_jump_blocks.iter()
            {
                self.future_jump_blocks.insert(kvp.0.clone(), kvp.1.clone());
            }
        }
        pub fn get_labeled_block(&self, name: &str) -> Option<BasicBlock<'ctx>>
        {
            self.labeled_blocks.get(name).map(|bb| bb.clone())
        }
        pub fn store_labeled_block(&mut self, name: &str, block: BasicBlock<'ctx>) -> Result<(), Box<dyn Error>>
        {
            let result = self.labeled_blocks.insert(name.to_string(), block);
            match result
            {
                Some(val) => Err(Box::new(CodegenError{message: format!("Used the label {} more than once!",name)})),
                None => Ok(())
            }


        }
        pub fn get_future_labeled_block(&self, name: &str) -> Option<BasicBlock<'ctx>>
        {
            self.future_jump_blocks.get(name).map(|bb| bb.clone())
        }
        pub fn store_placeholder_block(&mut self,name: &str,location_of_placeholder: BasicBlock<'ctx>)
        {
            self.future_jump_blocks.insert(name.to_string(), location_of_placeholder);
        }
        pub fn remove_label(&mut self, name: &str)
        {
            self.future_jump_blocks.remove(name);
        }
    }
    



    ///A trait which all provides an interface to compile a syntax element
    pub trait CodeGenable<'a, 'ctx> {
        unsafe fn codegen(self, compiler: &'a Compiler<'a, 'ctx>)
            -> Box<dyn AnyValue<'ctx> + 'ctx>;
    }

    
    /// This function runs for each line of the program, compiling it
    ///DON'T USE EXHAUSTIVE MATCHING, WE WANT IT TO NOT COMPILE
    ///IF NEW COMMANDS ARE ADDED.
    impl<'a, 'ctx> CodeGenable<'a, 'ctx> for Statement {
        unsafe fn codegen(
            self,
            compiler: &'a Compiler<'a, 'ctx>,
        ) -> Box<dyn AnyValue<'ctx> + 'ctx> {

            match self.label
            {
                Some(label_name) => compiler.codegen_label(&label_name).unwrap(),
                None => ()
            };

            match self.command {
                Command::Declare(dec) => {
                    dec.codegen(compiler)
                }
                Command::PUT(put) => {
                    put.codegen(compiler)
                }
                Command::GET(get) => {
                    get.codegen(compiler)
                }
                Command::GO(go) => {
                    go.codegen(compiler)
                }
                Command::EXPR(expr) => {
                    expr.codegen(compiler)
                }
                Command::IF(if_statement) => {
                    if_statement.codegen(compiler)
                }
                Command::Assignment(assn) => {
                    assn.codegen(compiler)
                }
                Command::FunctionDec(func) => {
                    func.codegen(compiler)
                }

                Command::END => panic!("found END"),
                Command::RETURN(_expr) => todo!("Handle top level return"),
                Command::Empty => panic!("found EMPTY"),
            }
        }
    }

    impl<'a, 'ctx> Compiler<'a, 'ctx> {
        pub fn new(
            c: &'ctx Context,
            b: &'a Builder<'ctx>,
            m: &'a Module<'ctx>,
            d: Option<&'a DebugController<'ctx>>,
        ) -> Compiler<'a, 'ctx> {

            let named_values: NamedValueHashmapStore = NamedValueHashmapStore::new();
            let error_module: ErrorModule = ErrorModule::new();
            let function_properties  = RefCell::new(FunctionProperties::new()); 
            Compiler {
                context: c,
                builder: b,
                module: m,
                named_values,
                error_module,
                function_properties,
                debug_controller: d,
                type_module: TypeModule::new(&c),
            }
        }



        ///NOTE: does not assign anything to variables
        unsafe fn create_empty_variable_and_return_ptr(&self, name: &str, _type: &Type) -> PointerValue<'ctx>
        {
            dbg!(&_type);

            let function = &get_current_function(self);

            let variable_ptr = self.create_entry_block_alloca(name, function, _type);


            dbg!(&variable_ptr);

            self.named_values
                .insert(NamedValue::new(name.to_string(), _type.clone(), variable_ptr));

            variable_ptr
        }

        pub unsafe fn allocate_variable(&self, assignment: &ast::Assignment) -> PointerValue<'ctx> {

            let current_function = get_current_function(self);
            let _type = assignment.value.get_type(self);

            self.create_entry_block_alloca(&assignment.var_name, &current_function, &_type)
        }

        pub unsafe fn assign_variable(
            &self,
            assignment: ast::Assignment,
            new_variable: PointerValue<'ctx>,
        ) -> BasicValueEnum<'ctx> {
            let _type = assignment.value.get_type(self);
            let value_to_store = assignment.value.codegen(self);

            let initial_value: BasicValueEnum<'ctx> =
                self.convert_anyvalue_to_basicvalue(value_to_store);
            let _store_result = self.builder.build_store(new_variable, initial_value);

            initial_value
        }


        pub unsafe fn create_or_load_variable(&self, variable_name: &str, _type: &Type) -> PointerValue<'ctx>
        {
            let variable_in_map = self.named_values.try_get(variable_name);

            match variable_in_map {

                Some(named_value) => {
                    if named_value._type != *_type
                    {
                        panic!("{} vs {}",named_value._type, _type)
                    }
                    return named_value.pointer;
                }

                None => {
                    self.create_empty_variable_and_return_ptr(variable_name, _type)
                }
            }
        }


        

        pub fn get_format_string_for_type(_type: &Type) -> String
        {
            match _type
            {
                Type::FixedDecimal => "%lf".to_string(),
                Type::Float => "%lf".to_string(),
                Type::Char(string_length) => " \'%[^\']\'".to_string(),
                Type::Void => panic!("Can't get format string for type Void!"),
                Type::TBD => panic!("Can't get format string for type TBD!"),
            }
        }

        pub unsafe fn convert_anyvalue_to_basicvalue(
            &self,
            value: Box<dyn AnyValue<'ctx> + 'ctx>,
        ) -> BasicValueEnum<'ctx> {
            let bve: BasicValueEnum = match value.as_any_value_enum() {
                AnyValueEnum::ArrayValue(v) => v.as_basic_value_enum(),
                AnyValueEnum::IntValue(v) => v.as_basic_value_enum(),
                AnyValueEnum::FloatValue(v) => v.as_basic_value_enum(),
                AnyValueEnum::PointerValue(v) => v.as_basic_value_enum(),
                AnyValueEnum::StructValue(v) => v.as_basic_value_enum(),
                AnyValueEnum::VectorValue(v) => v.as_basic_value_enum(),
                other => panic!("Could not build store of type {}", other),
            };
            bve
        }


        pub unsafe fn get_function(&self, name: &str) -> Result<FunctionValue<'ctx>, Box<dyn Error>>
        {
            self.module.get_function(name).ok_or(Box::new(CodegenError{ message: "Function named ___ not found!".to_string()}))
        }

        pub unsafe fn generate_function_call_code(
            &self,
            fn_name: &String,
            args: &mut Vec<ast::Expr>,
        ) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String> {
            //let get_func_result: Result<FunctionValue<'ctx>> = self.get_function(&fn_name);
            //if let None = get_func_result {
            //    return Err(format!(
            //        "Could not find a function named {}",
            //        fn_name.to_string()
            //    ));
            //}

            //let function_to_call: FunctionValue<'ctx> = get_func_result.unwrap();
            let function_to_call: FunctionValue<'ctx> = self.get_function(&fn_name).map_err(|_err| "lol".to_string())?;

            //handle argument checks here
            if args.len() != function_to_call.get_params().len() {
                return Err(format!(
                    "argument mismatch trying to create a call to function {}",
                    fn_name
                ));
            }

            let mut codegen_args: Vec<BasicMetadataValueEnum> = vec![];

            //TODO: perform typechecking on arguments here

            while args.len() > 0 {
                let current_arg = args.remove(0);
                let v: Box<dyn AnyValue<'ctx>> = current_arg.codegen(self);
                let bve: BasicValueEnum = match v.as_any_value_enum() {
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

            let call_return_value = self
                .builder
                .build_call(
                    function_to_call,
                    &codegen_args,
                    function_to_call.get_name().to_str().unwrap(),
                )
                .map_err(|err| {
                    format!(
                        "Error trying to build a call to function {}: {}",
                        fn_name, err
                    )
                })?;

            let returned_value = call_return_value.try_as_basic_value();

            if let Some(result_value) = returned_value.left() {
                Ok(Box::new(result_value))
            } else {
                Ok(Box::new(returned_value.right().unwrap()))
            }
        }

pub unsafe fn print_puttable(&'a self, item: &impl Puttable<'a,'ctx>) -> CallSiteValue<'ctx> {
                let string_ptr = item.get_pointer_to_printable_string(self);

                let res = self.builder.build_call(
                    self.get_function("printf").unwrap(),
                    &[BasicMetadataValueEnum::from(string_ptr)],
                    "printf",
                ).unwrap();

                res

        }

        pub unsafe fn print_const_string(&'a self, const_string: &str) -> CallSiteValue<'ctx> {
            let glob_string_ptr = self
                .builder
                .build_global_string_ptr(const_string, "my_const");

            let myptr = glob_string_ptr.unwrap().as_pointer_value();

            let res = self.builder.build_call(
                self.get_function("printf").unwrap(),
                &[BasicMetadataValueEnum::from(myptr)],
                "print_const_string",
            );
            return res.unwrap();
        }

        pub unsafe fn generate_float_code(&'a self, value: f64) -> FloatValue<'ctx> {
            self.context.f64_type().const_float(value)
        }
        pub unsafe fn generate_variable_code(
            &'a self,
            variable_name: &str,
        ) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String> {
            log::info!("Generating variable code for variable named {}",variable_name);
            let named_value: NamedValue<'ctx> = self.named_values.try_get(variable_name).unwrap();

            let variable_type = named_value._type;
            dbg!(format!("Type is: {}", variable_type));
            let var_ptr: PointerValue<'ctx> = named_value.pointer;
            let result_value: BasicValueEnum<'ctx> = self
                .builder
                .build_load(var_ptr,variable_name)
                .map_err(|err| format!("error building a variable code: {}", err))?;

            match variable_type {
                Type::FixedDecimal => {
                    let fixed_decimal_struct = result_value.into_struct_value();
                    return Ok(Box::new(fixed_decimal_struct));
                }
                Type::Char(_size) => {
                    let character_array = result_value.into_array_value();
                    return Ok(Box::new(character_array));
                }

                Type::TBD => {
                    panic!("Tried to retrieve a variable of type TBD!")
                }
                Type::Float => {
                    panic!("Implement type Float")
                }
                Type::Void => {
                    panic!("Tried to retrieve a variable of type Void!")
                }
            }
        }

        pub unsafe fn generate_function_prototype_code(
            self: &'a Self,
            fn_name: String,
            fn_arguments: Vec<ast::PrototypeArgument>,
            return_type: Type,
        ) -> FunctionValue<'ctx> {
            let llvm_return_type: AnyTypeEnum<'ctx> =
                self.convert_plick_type_to_llvm_any_type(return_type);
            let is_variable_num_of_args = false;

            let args_types: Vec<Type> = fn_arguments
                .clone()
                .into_iter()
                .map(|arg| arg._type)
                .collect();

            let args_types: Vec<BasicMetadataTypeEnum> = args_types
                .into_iter()
                .map(|ty| self.convert_plick_type_to_llvm_basic_type(ty).into())
                .collect();

            let args_types = args_types.as_slice();

            //create the function prototype type info

            let fn_type: FunctionType<'ctx> = match llvm_return_type {
                AnyTypeEnum::VoidType(ty) => ty.fn_type(args_types, is_variable_num_of_args),
                AnyTypeEnum::ArrayType(_ty) => {
                    todo!("Not implemeneted returning arraytype!")
                }
                AnyTypeEnum::FloatType(_ty) => {
                    todo!("Implement functions returning FloatType")
                }
                AnyTypeEnum::FunctionType(_ty) => {
                    todo!("Implement functions returning FunctionType")
                }
                AnyTypeEnum::IntType(_ty) => {
                    todo!("Implement functions returning IntType")
                }
                AnyTypeEnum::PointerType(_ty) => {
                    todo!("Implement functions returning PointerType")
                }
                AnyTypeEnum::StructType(ty) => ty.fn_type(args_types, is_variable_num_of_args),
                AnyTypeEnum::VectorType(_ty) => {
                    todo!("Implement functions returning VectorType")
                }
            };

            // create a new function prototype
            let llvm_function_value = self.module.add_function(&fn_name, fn_type, None);

            //name the arguments in the IR
            for (i, param) in llvm_function_value.get_param_iter().enumerate() {
                param.set_name(fn_arguments[i].name.as_str());
            }

            llvm_function_value
        }

        pub fn create_entry_block_alloca(
            &self,
            variable_name: &str,
            function: &FunctionValue,
            variable_type: &Type,
        ) -> PointerValue<'ctx> {
            
            log::info!("Generating declare code!");


            let builder = self.context.create_builder();
            let llvm_type_of_alloca =
                self.convert_plick_type_to_llvm_basic_type(variable_type.clone());
            let entry = function.get_first_basic_block().unwrap();

            match entry.get_first_instruction() {
                Some(first_instr) => builder.position_before(&first_instr),
                None => builder.position_at_end(entry),
            }

            builder
                .build_alloca(llvm_type_of_alloca, variable_name)
                .unwrap()
        }

        ///Generates a function DEFINITION, including the body

        ///creates the main func and returns its value
        pub fn initalize_main_function(&self) -> FunctionValue<'ctx> {
            let args: Vec<BasicMetadataTypeEnum> = vec![];
            let main_function_type = self.context.void_type().fn_type(&args, false);
            let main_func = self.module.add_function("main", main_function_type, None);
            //create a new scope block for the function
            let new_func_block = self.context.append_basic_block(main_func, "entry");

            //position the builder's cursor inside that block
            self.builder.position_at_end(new_func_block);

            main_func
        }



        unsafe fn codegen_label(&self, label_name: &str) -> Result<(), Box<dyn Error>>
        {
            log::debug!("Generating a block named label {}",label_name);
            let old_block = self.builder.get_insert_block().unwrap();

            let label_block = self.context.insert_basic_block_after(old_block, label_name);

            let x = branch_only_if_no_terminator(self,label_block);

            self.function_properties.borrow_mut().store_labeled_block(label_name, label_block.clone())?;

            let myblo = self.function_properties.borrow().future_jump_blocks.clone();
            let substitute_block = myblo.get(label_name).clone();
            //now go back and see if any future jump calls need to be rehydrated
            match substitute_block
            {
                Some(blok) => {
                    log::warn!("BLOK HAS BEEN ENTERED!");
                    dbg!(blok);
                    dbg!(label_block);
                    blok.replace_all_uses_with(&label_block); blok.remove_from_function().unwrap(); 

                },
                None => ()
            };
            self.function_properties.borrow_mut().remove_label(label_name);
            //end rehydration


            self.builder.position_at_end(label_block);
            
            Ok(())
        }
    }
}

mod tests {
    use crate::error::errors::ErrorModule;
    use crate::types::{infer_pli_type_via_name, Type};
    use crate::{ast::SourceLocation, types::TypeModule};
    use inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        types::BasicMetadataTypeEnum,
        values::{BasicMetadataValueEnum, PointerValue},
    };
    use std::collections::HashMap;

    use crate::{
        ast::{Expr, Function, Prototype},
        codegen::codegen::{CodeGenable, Compiler},
        lexer::Token,
    };
    use std::cell::RefCell;

    use super::codegen::FunctionProperties;
    use super::named_value_store::{NamedValueHashmapStore, NamedValueStore};
    fn get_test_compiler<'a, 'ctx>(
        c: &'ctx Context,
        m: &'a Module<'ctx>,
        b: &'a Builder<'ctx>,
    ) -> Compiler<'a, 'ctx> {
        let context = c;
        let module = m;
        let builder = b;
        let named_values = NamedValueHashmapStore::new();
        let debug_controller = None;
        let error_module = ErrorModule::new();
        let function_properties = RefCell::new(FunctionProperties::new());
        let compiler = Compiler {
            context,
            module,
            builder,
            error_module,
            named_values,
            function_properties,
            debug_controller,
            type_module: TypeModule::new(&context),
        };
        compiler
    }
    #[test]
    fn test_constant_codegen() {
        let c = Context::create();
        let m = c.create_module("repl");
        let b = c.create_builder();
        let compiler = get_test_compiler(&c, &m, &b);

        let consta = Expr::new_numval(3.0);

        unsafe {
            let result = consta.codegen(&compiler);
            println!("{}", result.print_to_string());
            dbg!("{}", result);
        }
    }

    #[test]
    #[ignore = "need to add line that adds extern funcs"]
    fn test_comparisons() {
        let c = Context::create();
        let m = c.create_module("repl");
        let b = c.create_builder();
        let compiler = get_test_compiler(&c, &m, &b);

        //create a MAIN function here
        compiler.initalize_main_function();
        //finish creating a main function

        let left = Box::new(Expr::new_numval(3.0));

        let right = Box::new(Expr::new_numval(5.0));

        let my_binary = Expr::Binary {
            operator: Token::LESS_THAN,
            left,
            right,
        };
        unsafe {
            let result = my_binary.codegen(&compiler);
            println!("{}", result.print_to_string());
            dbg!("{}", result);
        }
    }

    #[test]
    #[ignore = "need to add line that adds extern funcs"]
    fn test_binary_codegen() {
        let c = Context::create();
        let m = c.create_module("repl");
        let b = c.create_builder();
        let compiler = get_test_compiler(&c, &m, &b);

        let binop = Expr::Binary {
            operator: Token::MINUS,
            left: Box::new(Expr::Variable {
                name: String::from("APPLE"),
                _type: Type::FixedDecimal,
            }),
            right: Box::new(Expr::new_numval(5.0)),
        };
        let source_loc: SourceLocation = SourceLocation::default();
        let my_proto = Prototype {
            fn_name: String::from("myFuncName"),
            args: vec![String::from("APPLE")],
            source_loc,
        };
        let my_func = Function {
            prototype: my_proto,
            body_statements: vec![],
            return_value: Some(binop),
            return_type: infer_pli_type_via_name("myFuncName"),
        };

        unsafe {
            let _result = compiler.generate_function_code(my_func);
        }
    }
}
