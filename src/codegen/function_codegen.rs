use inkwell::{debug_info::{DISubprogram, AsDIScope}, values::{FunctionValue, PointerValue, BasicValue}, basic_block::BasicBlock};

use crate::{ast, codegen::named_value_store::NamedValueStore, types::Type, error::get_error};

use super::codegen::Compiler;
use super::codegen::*;



impl<'a, 'ctx> Compiler<'a, 'ctx>
    {

        pub unsafe fn generate_function_code(&self, func: ast::Function) -> Result<FunctionValue<'ctx>, String>
        {
            
            //1. see if the function has already been defined
            self.handle_if_function_has_already_been_defined(&func)?;
            //2. clear the named values, which stores all the recognized identifiers
            self.named_values.clear();
    
            let func_name = func.prototype.fn_name.clone();
            let proto_args = func.prototype.args.clone();
            let current_subprogram = self.try_attach_debug_info(&func);
            
            //3. get a list of the arguments with their names and types
            let args: Vec<(String, Type)> = self.get_function_argument_array(proto_args);

            let function = self.generate_function_prototype_code(func_name.clone(),args.clone(), func.return_type);

            self.check_if_function_body_is_empty();

            //create a new scope block for the function
            let new_func_block: BasicBlock = self.context.append_basic_block(function, "entry");

            //position the builder's cursor inside that block
            self.builder.position_at_end(new_func_block);

            self.fill_named_values_array(&function, &func, &args)?;

            self.generate_body_statements_in_function(&func);

            self.remove_debug_lexical_block_if_debug(current_subprogram, &function); 

            self.check_if_function_has_a_return_value(&function,&func)?;
           
            self.build_return_value(&func)?;        

            self.verify_function(function, &func)?; 

            Ok(function)
        }

    fn handle_if_function_has_already_been_defined(&self, func: &ast::Function) -> Result<(), String>
    {
           if let Some(_) = self.module.get_function(&func.prototype.fn_name)
            {                               
               return Err(format!("function named {} already exists!",func.prototype.fn_name));
            }
           Ok(())
    }
    fn get_function_argument_array(&self,proto_args: Vec<String>) -> Vec<(String,Type)>
    {
            proto_args
                .clone()
                .into_iter()
                .map(|name| (name, Type::FixedDecimal))
                .collect()
    }
    fn check_if_function_body_is_empty(&self)
    {
        return;
        todo!("Check if function body is empty. If so, have a way to terminate function generation.")
    }
    fn fill_named_values_array(&self, function: &FunctionValue, func: &ast::Function, args: &Vec<(String,Type)>) -> Result<(), String>
    {
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
            Ok(())
    }
    unsafe fn generate_body_statements_in_function(&self, func: &ast::Function)
    {
            for statement in func.body_statements.iter()
            {
                statement.clone().codegen(self);
            }

    }
    fn remove_debug_lexical_block_if_debug(&self, current_subprogram: Option<DISubprogram>, function: &FunctionValue)
    {
       if let Some(dbg) = self.debug_controller
            {
                function.set_subprogram(current_subprogram.unwrap());
               
                let myblock = dbg.lexical_blocks.borrow_mut().pop();
            }

    }
    fn check_if_function_has_a_return_value(&self, function: &FunctionValue, func: &ast::Function) -> Result<(), String>
    {
         match func.return_value
            {
                None => 
                {
                    return Err(get_error(&["7"]));
                    self.builder.build_return(None)
                        .map_err(|builder_func| format!("error building function return with no value: {}",builder_func))?;
                    //return Ok(function);
                }
                Some(_) => { Ok(())},
            }

    }
    unsafe fn build_return_value(&self, func: &ast::Function) -> Result<(), String>
    {
            // Handle return type
            let function_return_type = func.return_type;

            let return_expr = func.return_value.clone().unwrap();
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
        Ok(())
    }
    fn verify_function(&self, function: FunctionValue, func: &ast::Function) -> Result<(), String>
    {
        let failed_verification = !function.verify(true);
                if failed_verification
                {
                   let module_text = self.module.print_to_string();
                   return Err(format!("Function {} failed to verify: {}", func.prototype.fn_name.clone(), module_text));
                }
                Ok(())
    }
    fn try_attach_debug_info(&self, func: &ast::Function) -> Option<DISubprogram>
    {
        //START OF DEBUG STUFF
            //generate the IR for the function prototype
            let func_name = func.prototype.fn_name.clone();
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

                return Some(myfunc);
            }
            None


}
    }









