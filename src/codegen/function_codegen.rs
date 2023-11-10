use inkwell::{debug_info::{DISubprogram, AsDIScope}, values::{FunctionValue, PointerValue, BasicValue}, basic_block::BasicBlock};

use crate::{ast, codegen::named_value_store::NamedValueStore, types::Type, error::get_error};

use super::codegen::Compiler;
use super::codegen::*;



impl<'a, 'ctx> Compiler<'a, 'ctx>
    {

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

            let return_expr = func.return_value.unwrap();
            dbg!(&return_expr);
            let function_return_value = return_expr.codegen(self);
            dbg!(&function_return_value); 
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
                   let module_text = self.module.print_to_string();
                   panic!("Function {} failed to verify: {}", func.prototype.fn_name.clone(), module_text);
                }
            Ok(function)
        }
    }
