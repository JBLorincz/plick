
mod named_value_store;
pub mod codegen {

use std::collections::HashMap;
use std::vec;

use crate::ast;
use crate::debugger::DebugController;
use crate::error::get_error;
use crate::lexer;
use crate::ast::Command;
use crate::ast::Statement;
use crate::types::Type;
use crate::types::TypeModule;
use crate::types::fixed_decimal;
use crate::types::fixed_decimal::FixedValue;
use crate::types::infer_pli_type_via_name;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::debug_info::AsDIScope;
use inkwell::debug_info::DILexicalBlock;
use inkwell::debug_info::DILocation;
use inkwell::debug_info::DISubprogram;
use inkwell::module::Module;
use inkwell::types::AnyTypeEnum;
use inkwell::types::FunctionType;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::BasicValueEnum;
use inkwell::values::CallSiteValue;
use inkwell::values::StructValue;
use inkwell::{builder, context, module};
use inkwell::types::BasicMetadataTypeEnum;
use std::cell::RefCell;
use inkwell::values::{AnyValue, AnyValueEnum, BasicValue, FloatValue, FunctionValue, PointerValue };

use super::named_value_store::NamedValueHashmapStore;
use super::named_value_store::NamedValueStore;

    ///The object that drives compilation.
    #[derive(Debug)]
    pub struct Compiler<'a, 'ctx>
    {
        pub context: &'ctx context::Context,
        pub builder: &'a builder::Builder<'ctx>,       
        pub module: &'a module::Module<'ctx>,
        pub type_module: TypeModule<'ctx>,
        pub debug_controller: Option<&'a DebugController<'ctx>>,

        pub named_values: NamedValueHashmapStore<'ctx>,
    }

    #[derive(Debug,Clone)]
    pub struct NamedValue<'ctx>
    {
        pub name: String,
        pub _type: Type,
        pub value: PointerValue<'ctx>
    }
    
    impl<'ctx> NamedValue<'ctx>
    {
        pub fn new(name: String, _type: Type, value: PointerValue<'ctx>) -> NamedValue<'ctx>
        {
            NamedValue { name, _type, value }
        }
    }

    ///A trait which all provides an interface to compile a syntax element
    pub trait CodeGenable<'a,'ctx>
    {

        unsafe fn codegen(self, compiler: &'a Compiler<'a, 'ctx>) -> Box<dyn AnyValue <'ctx> + 'ctx>;
    }


    impl<'a, 'ctx> CodeGenable<'a, 'ctx> for ast::Expr
    {
       

        unsafe fn codegen(mut self, compiler: &'a Compiler<'a, 'ctx>) -> Box<dyn AnyValue <'ctx> +'ctx>
        {
            match self {
                ast::Expr::Variable { name, _type } => compiler.generate_variable_code(&name).unwrap(),
               ast::Expr::Binary{operator, left, right}  => {
                
                    let bin_res = compiler.generate_binary_expression_code( ast::Expr::Binary {operator, left, right});
                    let binary_value = bin_res.unwrap();
                    binary_value
                },
                ast::Expr::NumVal { value, _type } => 
                {
                    //Box::new(compiler.generate_float_code(value as f64))
                     Box::new(compiler.gen_const_fixed_decimal(value as f64))
                },
                ast::Expr::Call { ref fn_name, ref mut args, _type } => {
                     let function_call_result = compiler.generate_function_call_code( fn_name, args );
                    function_call_result.unwrap()
                },
                _ => {
                    panic!("Hit exhaustive match on codegen expressions!");
                },
            }
        }
    }
    impl<'a, 'ctx> CodeGenable<'a,'ctx> for Statement
    {
        unsafe fn codegen(self, compiler: &'a Compiler<'a, 'ctx>) -> Box<dyn AnyValue <'ctx> +'ctx>
        {
            //DON'T USE EXHAUSTIVE MATCHING, WE WANT IT TO NOT COMPILE
            //IF NEW COMMANDS ARE ADDED.
            match self.command 
            {
            Command::Declare(_dec) => todo!("Implement codegen for declare statement"),
            Command::PUT => Box::new(compiler.generate_hello_world_print()),
            Command::EXPR(expr) => expr.codegen(compiler),
            Command::IF(if_statement) => Box::new(compiler.generate_if_statement_code(if_statement)),
            Command::END => panic!("found END"),
            Command::RETURN(_expr) => panic!("found RETURN!"),
            Command::Empty => panic!("found EMPTY"),
            Command::Assignment(assn) => {
                Box::new(compiler.generate_assignment_code(assn).as_any_value_enum())
            },
            Command::FunctionDec(func) =>{

                let current_function = compiler.builder.get_insert_block().unwrap();
                let llvm_created_function = Box::new(compiler.generate_function_code(func).unwrap());
                compiler.builder.position_at_end(current_function);
                llvm_created_function
            }
            
            }
        }
    }


    impl<'a, 'ctx> Compiler<'a, 'ctx>
    {
         pub fn new(c: &'ctx Context, b: &'a Builder<'ctx>, m: &'a Module<'ctx>, d: Option<&'a DebugController<'ctx>>) -> Compiler<'a, 'ctx>
        {

            //let named_values: RefCell<HashMap<String,NamedValue<'ctx>>> = RefCell::new(HashMap::new());
            let named_values: NamedValueHashmapStore = NamedValueHashmapStore::new();
            Compiler { 
                context: c, 
                builder: b, 
                module: m, 
                named_values, 
                debug_controller: d, 
                type_module: TypeModule::new(&c) }
        }

        unsafe fn generate_assignment_code(&self, assignment: ast::Assignment) -> Box<dyn BasicValue<'ctx> +'ctx> 
        {
            let variable_in_map = self.named_values.try_get(&assignment.var_name);
            let _type = assignment.value.get_type();

            match variable_in_map {
                Some(_pointer_value) => {
                    let value_to_store = assignment.value.codegen(self);

                    let initial_value: BasicValueEnum<'ctx> = self.convert_anyvalue_to_basicvalue(value_to_store);
                    let _store_result = self.builder.build_store(_pointer_value.value, initial_value);
                    return Box::new(initial_value);
                }
                None => { //VARIABLE CREATION HERE
                            self.create_variable(assignment) 
                        }
            }
        }
        unsafe fn create_variable(&self, assignment: ast::Assignment) -> Box<dyn BasicValue<'ctx> +'ctx>
        {

                            let _type = assignment.value.get_type();
                            let name = assignment.var_name.clone();
                            let variable_ptr = self.allocate_variable(&assignment);
                            let value_of_variable =self.assign_variable(assignment,variable_ptr);
                            self.named_values.insert(NamedValue::new(name, _type, variable_ptr ));

                            Box::new(value_of_variable)

        }
        unsafe fn allocate_variable(&self, assignment: &ast::Assignment) -> PointerValue<'ctx>
        {
                    let current_function = self.builder.get_insert_block().unwrap().get_parent().unwrap();
                    //let inferred_type = infer_pli_type_via_name(&assignment.var_name);
                    let _type = assignment.value.get_type();
                    self.create_entry_block_alloca(&assignment.var_name, &current_function, &_type)

                   
        }
        unsafe fn assign_variable(&self, assignment: ast::Assignment, new_variable: PointerValue<'ctx>)
            -> BasicValueEnum<'ctx>
        {

                    let _type = assignment.value.get_type();
                    let value_to_store = assignment.value.codegen(self);

                    let initial_value: BasicValueEnum<'ctx> = self.convert_anyvalue_to_basicvalue(value_to_store);
                    let _store_result = self.builder.build_store(new_variable, initial_value);
                    

                    initial_value

        }

        unsafe fn generate_if_statement_code(&self, if_statement: ast::If) -> FloatValue<'ctx>
        {
            let conditional_type = if_statement.conditional.get_type();
            dbg!(&if_statement.conditional);
            let conditional_code = if_statement.conditional.codegen(self);

            let conditional_as_float: FloatValue;

            match conditional_type
            {
                Type::FixedDecimal =>
                {
                    let fv: FixedValue = FixedValue::from(conditional_code.as_any_value_enum().into_struct_value());
                    conditional_as_float = self.fixed_decimal_to_float(fv);
                },
                Type::TBD => {todo!("Can't support type TBD in if conditional!");},
                Type::Float => {todo!("Can't support type Float in if conditional!");},
                Type::Void => {todo!("Can't support type Void in if conditional!");},
            };

//            if let AnyValueEnum::FloatValue(val) = conditional_code.as_any_value_enum()
//            {
//                conditional_as_float = val;
//            }
//            else
//            {
//                panic!("Not a float value!"); 
//            }

            let comparison = self
                .builder
                .build_float_compare(inkwell::FloatPredicate::ONE, conditional_as_float, self.generate_float_code(0.0), "ifcond")
                .unwrap();

            //now we build the THEN block
            let current_func = self.builder.get_insert_block().unwrap().get_parent().unwrap();
            let mut then_block = self.context.append_basic_block(current_func, "then");
            let mut else_block = self.context.append_basic_block(current_func, "else");
            let if_cont_block = self.context.append_basic_block(current_func, "ifcont");

            self.builder.build_conditional_branch(comparison, then_block, else_block);

            self.builder.position_at_end(then_block);
            for statement in if_statement.then_statements
            {
                statement.codegen(self);
            }
            //now we add a statement to jump to the if_cont block
            self.builder.build_unconditional_branch(if_cont_block);
            then_block = self.builder.get_insert_block().unwrap(); 
            //handle else here
            
             self.builder.position_at_end(else_block);
            if let Some(else_statements) = if_statement.else_statements
            {
                for statement in else_statements
                {
                    statement.codegen(self);
                }
            }
            //now we add a statement to jump to the if_cont block
            self.builder.build_unconditional_branch(if_cont_block);
            else_block = self.builder.get_insert_block().unwrap(); 

            //handle merge block
            self.builder.position_at_end(if_cont_block);

            self.generate_float_code(-999.0)
        }

        
        unsafe fn convert_anyvalue_to_basicvalue(&self, value: Box<dyn AnyValue<'ctx> +'ctx>) -> BasicValueEnum<'ctx>
        {
              let bve: BasicValueEnum =  match value.as_any_value_enum()
                                {
                                    AnyValueEnum::ArrayValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::IntValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::FloatValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::PointerValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::StructValue(v) => v.as_basic_value_enum(),
                                    AnyValueEnum::VectorValue(v) => v.as_basic_value_enum(),
                                    other => panic!("Could not build store of type {}",other)
                                };
              bve
        }

        unsafe fn generate_function_call_code(&self,fn_name: &String,args: &mut Vec<ast::Expr>) 
            -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String>
        {
            let get_func_result:Option<FunctionValue<'ctx>> = self.module.get_function(&fn_name);
            if let None = get_func_result
            {
                return Err(format!("Could not find a function named {}",fn_name.to_string()));
            }


            let function_to_call: FunctionValue<'ctx> = get_func_result.unwrap();

            //handle argument checks here
            if args.len() != function_to_call.get_params().len()
            {
                return Err(format!("argument mismatch trying to create a call to function {}", fn_name));
            }

            let mut codegen_args: Vec<BasicMetadataValueEnum> = vec![];
            
            //TODO: perform typechecking on arguments here
            
            
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


                let call_return_value = self.builder.build_call(
                    function_to_call,
                    &codegen_args,
                    function_to_call.get_name().to_str().unwrap()
                    )
                    .map_err(|err| format!("Error trying to build a call to function {}: {}", fn_name, err))
                    ?;
            
                    let returned_value = call_return_value.try_as_basic_value();

                    if let Some(result_value) = returned_value.left()
                    {
                        Ok(Box::new(result_value))
                    }
                    else
                    {
                        Ok(Box::new(returned_value.right().unwrap()))
                    }
        }

        pub unsafe fn generate_hello_world_print(&'a self) -> CallSiteValue<'ctx>
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
        unsafe fn generate_variable_code(&'a self,variable_name: &str) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String>
        {
            let named_value: NamedValue<'ctx> = self.named_values.try_get(variable_name).unwrap();

            let variable_type = named_value._type; 
            dbg!(format!("Type is: {}",variable_type));
            let var_ptr: PointerValue<'ctx> = named_value.value;
            let result_value: BasicValueEnum<'ctx> = self
                .builder
                .build_load(var_ptr,variable_name)
                .map_err(|err| format!("error building a variable code: {}", err))?
                //.into_float_value()
                ;

            match variable_type
            {
                Type::FixedDecimal =>
                {
                    let struct_value = result_value.into_struct_value();
                    return Ok(Box::new(struct_value));
                },
                Type::TBD => {panic!("Tried to retrieve a variable of type TBD!")},
                Type::Float => {panic!("Implement type Float")},
                Type::Void => {panic!("Tried to retrieve a variable of type Void!")},
            }


            //return Ok(Box::new(result_value));
        }

        unsafe fn generate_binary_expression_code(&self, binary_expr: ast::Expr) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>, String>
        {
            if let ast::Expr::Binary { operator, left, right } = binary_expr
            {
                let lhstype = left.get_type();
                let rhstype = right.get_type();
                
                let lhs_codegen = left.codegen(self);
                let rhs_codegen = right.codegen(self);
                dbg!(&lhs_codegen); 
                dbg!(&rhs_codegen); 
                let lhs_float: FloatValue<'ctx>;
                let rhs_float: FloatValue<'ctx>;

                match lhstype
                {
                    Type::FixedDecimal => {
                        let lhs_struct = lhs_codegen.as_any_value_enum().into_struct_value();
                        let fixed_dec = fixed_decimal::FixedValue::new(lhs_struct);

                        lhs_float = self.fixed_decimal_to_float(fixed_dec);
                    },
                    other_type => todo!("Implement type conversion to float for {:?}",other_type)
                };


                match rhstype
                {
                    Type::FixedDecimal => {
                        let rhs_struct = rhs_codegen.as_any_value_enum().into_struct_value();
                        let fixed_dec = fixed_decimal::FixedValue::new(rhs_struct);

                        rhs_float = self.fixed_decimal_to_float(fixed_dec);
                    },
                    other_type => todo!("Implement type conversion to llvm floatvalue for {:?}",other_type)
                };
                //panic!("{:?}, {:?}", lhstype, rhstype);

                //let lhs_float = lhs_codegen.as_any_value_enum().into_float_value();
                //let rhs_float = rhs_codegen.as_any_value_enum().into_float_value();
                
                if true
                {
                
                    //TODO: Make this function return anyvalue and a fixed decimal
                let compile_result: Result<Box<dyn AnyValue<'ctx> + 'ctx>, String> = match operator {
                    lexer::Token::PLUS => {
                        let var = self.builder.build_float_add(lhs_float, rhs_float, "tmpadd").unwrap();
                        let fix = self.gen_const_fixed_decimal(0.0);
                        Ok(Box::new(fix))
                    },
                    lexer::Token::MINUS => {
                        let floatval = self.builder.build_float_sub(lhs_float, rhs_float, "tmpsub");
                        let fix = self.gen_const_fixed_decimal(0.0);
                        Ok(Box::new(fix))
                    },
                    lexer::Token::MULTIPLY => {
                        let var = self.builder.build_float_mul(lhs_float, rhs_float, "tmpmul");
                        let fix = self.gen_const_fixed_decimal(0.0);
                        Ok(Box::new(fix))
                    },
                    lexer::Token::DIVIDE =>
                    {
                        let var = self.builder.build_float_div(lhs_float,rhs_float,"tmpdiv");
                        let fix = self.gen_const_fixed_decimal(0.0);
                        Ok(Box::new(fix))
                    },
                    lexer::Token::LESS_THAN => {
                            let val = self.builder
                            .build_float_compare(inkwell::FloatPredicate::OLT, lhs_float,rhs_float, "tmplt")
                            .map_err(
                                |builder_error| 
                                format!("Unable to create less than situation: {}",
                                        builder_error)
                                )?;
                            
                            let cmp_as_float = self
                                .builder
                                .build_unsigned_int_to_float(val, self.context.f64_type(), "tmpbool")
                                .map_err(|e| format!("Unable to convert unsigned int to float: {}", e))?;

                           Ok(Box::new(cmp_as_float)) 
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
                           Ok(Box::new(cmp_as_float)) 
                    },
                    _ => return Err(format!("Binary operator had unexpected operator! {:?}", operator)),
                };

                return compile_result
                    .map_err(|builder_error| format!("There was an error building the binary expression: {}", builder_error));                   
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

        unsafe fn generate_function_prototype_code(self: &'a Self, fn_name: String, fn_arguments: Vec<(String, Type)>, return_type: Type) -> FunctionValue<'ctx>
        {
            let llvm_return_type: AnyTypeEnum<'ctx> = self.convert_plick_type_to_llvm_any_type(return_type);
            let is_variable_num_of_args = false; 

           let args_types: Vec<Type> = fn_arguments.clone().into_iter().map(|arg| arg.1).collect();

           let args_types: Vec<BasicMetadataTypeEnum> = args_types
               .into_iter()
               .map(|ty| self.convert_plick_type_to_llvm_basic_type(ty).into())
               .collect();
            
            
            let args_types = args_types.as_slice();

            //create the function prototype type info

            let fn_type: FunctionType<'ctx> = match llvm_return_type 
            {
                AnyTypeEnum::VoidType(ty) => {ty.fn_type(args_types, is_variable_num_of_args)},
                AnyTypeEnum::ArrayType(_ty) => {todo!("Not implemeneted returning arraytype!")},
                AnyTypeEnum::FloatType(_ty) => {todo!("Implement functions returning FloatType")},
                AnyTypeEnum::FunctionType(_ty) => {todo!("Implement functions returning FunctionType")},
                AnyTypeEnum::IntType(_ty) => {todo!("Implement functions returning IntType")},
                AnyTypeEnum::PointerType(_ty) => {todo!("Implement functions returning PointerType")},
                AnyTypeEnum::StructType(ty) => {ty.fn_type(args_types, is_variable_num_of_args)},
                AnyTypeEnum::VectorType(_ty) => {todo!("Implement functions returning VectorType")},
            };

             
            // create a new function prototype
            let llvm_function_value = self.module.add_function(&fn_name, fn_type, None);

            //name the arguments in the IR
            for (i,param) in llvm_function_value.get_param_iter().enumerate()
            {
               param.set_name(fn_arguments[i].0.as_str());
            }

            llvm_function_value
        }

        fn create_entry_block_alloca(&self, argument_name: &str, function: &FunctionValue, argument_type: &Type ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let llvm_type_of_alloca = self.convert_plick_type_to_llvm_basic_type(argument_type.clone());
        let entry = function.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(llvm_type_of_alloca, argument_name).unwrap()
    }

        ///Generates a function DEFINITION, including the body
        pub unsafe fn generate_function_code(&self, func: ast::Function) -> Result<FunctionValue<'ctx>, String>
        {
            
            //see if the function has already been defined
            if let Some(_) = self.module.get_function(&func.prototype.fn_name)
            {                               
               return Err(format!("function named {} already exists!",func.prototype.fn_name));
            }
            
            //clear the named values, which stores all the recognized identifiers
            self.named_values.clear();
    
            //START OF DEBUG STUFF
            //generate the IR for the function prototype
            let func_name = func.prototype.fn_name.clone();
            let proto_args = func.prototype.args.clone();
            let line_no = func.prototype.source_loc.line_number;
            let column_no = func.prototype.source_loc.column_number;
            let mut current_subprogram: Option<DISubprogram> = None;

            if let Some(dbg) = self.debug_controller
            {
                let name = func_name.as_str();
                let linkage_name = None;
                let scope_line = line_no;
                let is_definition = true;
                let is_local_to_unit = true;
                let flags = 0; 
                let is_optimized = dbg.optimized;

                let scope = dbg.builder.create_file(&dbg.filename, &dbg.directory);
                
                //TODO: Fill out parameter and return stuff here.
                let ditype = dbg.builder.create_subroutine_type(scope,None,&[],0);

                let myfunc = dbg.builder.create_function(
                        scope.as_debug_info_scope(),
                        &name,
                        linkage_name,
                        scope,
                        line_no,
                        ditype, 
                        is_local_to_unit,
                        is_definition,
                        scope_line,
                        flags,
                        is_optimized);

                dbg.lexical_blocks.borrow_mut().push(myfunc.as_debug_info_scope());

                let current_loc = dbg.builder.create_debug_location(self.context, line_no, column_no, myfunc.as_debug_info_scope(), None);
                dbg!(current_loc);

                self.builder.set_current_debug_location(current_loc);
               
                current_subprogram = Some(myfunc);
                dbg.builder.finalize();
            }
            //END OF DEBUG STUFF

            let args: Vec<(String, Type)> = proto_args
                .clone()
                .into_iter()
                .map(|name| (name, Type::FixedDecimal))
                .collect();

            let function = self.generate_function_prototype_code(func_name.clone(),args.clone(), func.return_type);
            //TODO: Check if function body is empty
            //if so, return function here. 

            //create a new scope block for the function
            let new_func_block: BasicBlock = self.context.append_basic_block(function, "entry");

            //position the builder's cursor inside that block
            self.builder.position_at_end(new_func_block);

            //fill up the NamedValues array 
            for (i,arg) in function.get_param_iter().enumerate()
            {
                let alloca: PointerValue<'ctx> = self.create_entry_block_alloca(&args[i].0, &function,&args[i].1);
                self
                    .builder
                    .build_store(alloca, arg)
                    .map_err(|builder_err| format!("Was unable to build_store for {:?}: {}",arg,builder_err).to_string())?;
                
                let name = func.prototype.args[i].clone();
                self
                    .named_values
                    .insert(NamedValue { name, _type: Type::FixedDecimal, value: alloca });
            }

            for statement in func.body_statements.iter()
            {
                statement.clone().codegen(self);
            }

            if let Some(dbg) = self.debug_controller
            {
                function.set_subprogram(current_subprogram.unwrap());
               
                let myblock = dbg.lexical_blocks.borrow_mut().pop();
            }
            else
            {
           
            }
            match func.return_value
            {
                None => 
                {
                    return Err(get_error(&["7"]));
                    self.builder.build_return(None)
                        .map_err(|builder_func| format!("error building function return with no value: {}",builder_func))?;
                    return Ok(function);
                }
                Some(_) => {},
            }
            let function_return_type = func.return_type;
            let function_return_value = func.return_value.unwrap().codegen(self);
            
            let return_value_as_enum = function_return_value.as_any_value_enum();
            
            match function_return_type
            {
                Type::FixedDecimal =>
                {
                    let struct_value = return_value_as_enum.into_struct_value();
                    self.builder.build_return(Some(&struct_value as &dyn BasicValue))
                    .map_err(|err| err.to_string())?;
                },
                Type::Float => {
                    todo!("Implement functions that return Float!");
                },
                Type::TBD => {
                    todo!("Implement functions that return TBD!");
                },
                Type::Void => {
                    todo!("Implement functions that return Void!");
                }
            };
        

//            if let AnyValueEnum::FloatValue(a)  = return_value_as_enum {
//                let _output = self.builder.build_return(Some(&a as &dyn BasicValue));
//            }
//            else 
//            {
//                return Err("Function return type was not float value!".to_string());
//            }



             let failed_verification = !function.verify(true);
                if failed_verification
                {
                   self.module.print_to_stderr();
                   panic!("Function {} failed to verify:", func.prototype.fn_name.clone());
                }
            Ok(function)
        }


    ///creates the main func and returns its value
    pub fn initalize_main_function(&self) -> FunctionValue<'ctx>
    {
            let args: Vec<BasicMetadataTypeEnum> = vec![];
            let main_function_type = self.context.void_type().fn_type(&args, false);
            let main_func = self.module.add_function("main", main_function_type, None);
            //create a new scope block for the function
            let new_func_block = self.context.append_basic_block(main_func, "entry");

            //position the builder's cursor inside that block
            self.builder.position_at_end(new_func_block);

            main_func

    }


    }
} 

mod tests {
    use std::collections::HashMap;
    use crate::{ast::SourceLocation, types::TypeModule};
    use crate::types::{Type, infer_pli_type_via_name};
    use inkwell::{values::{PointerValue, BasicMetadataValueEnum}, context::Context, builder::Builder, module::Module, types::BasicMetadataTypeEnum};

    use crate::{ast::{Expr, Function, Prototype}, codegen::codegen::{CodeGenable, Compiler}, lexer::Token};
    use std::cell::RefCell;

    use super::codegen::NamedValue;
    use super::named_value_store::{NamedValueHashmapStore, NamedValueStore};
    fn get_test_compiler<'a, 'ctx>(c: &'ctx Context, m: &'a Module<'ctx>, b: &'a Builder<'ctx>) -> Compiler<'a, 'ctx>
    {
        let context = c;
        let module = m;
        let builder = b;
        let named_values  = NamedValueHashmapStore::new();
        let debug_controller = None;
        let compiler = Compiler {
           context,
           module,
           builder,
           named_values,
           debug_controller,
           type_module: TypeModule::new(&context)
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
        
        let consta = Expr::new_numval(3);

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

        let left = Box::new(Expr::new_numval(3));
        
        let right = Box::new(Expr::new_numval(5));

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
    let compiler = get_test_compiler(&c, &m, &b);
        
        let binop = Expr::Binary { 
            operator: Token::MINUS,
            left: Box::new(Expr::Variable { name: String::from("APPLE"),
            _type: Type::FixedDecimal }) , 
            right: Box::new(Expr::new_numval(5))
        };
        let source_loc: SourceLocation = SourceLocation::default(); 
        let my_proto = Prototype {fn_name: String::from("myFuncName"),args: vec![String::from("APPLE")], source_loc};
        let my_func = Function {prototype: my_proto, body_statements: vec![], return_value: Some(binop), return_type: infer_pli_type_via_name("myFuncName")};

        unsafe {
            
            let _result = compiler.generate_function_code(my_func);
        }
    }


}
