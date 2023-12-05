use std::error::Error;

use log::debug;

use crate::{
    ast,
    codegen::{
        codegen::CodeGenable,
        utils::{self, get_current_function},
    },
};

impl<'a, 'ctx> CodeGenable<'a, 'ctx> for ast::Go {
    unsafe fn codegen(
        self,
        compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>,
    ) -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> {
        self.codegen_with_error_info(compiler).unwrap()
    }
}

impl<'a, 'ctx> ast::Go {
    unsafe fn codegen_with_error_info(
        self,
        compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>,
    ) -> Result<Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx>, Box<dyn Error>> {
        let does_labeled_block_exist = compiler
            .function_properties
            .borrow()
            .get_labeled_block(&self.label_to_go_to);

        let nul = compiler.context.i64_type().const_zero();
        if let Some(labeled_block) = does_labeled_block_exist.clone() {
            utils::branch_only_if_no_terminator(compiler, labeled_block);
            return Ok(Box::new(nul));
        } else {
            let x = compiler
                .function_properties
                .borrow_mut()
                .get_future_labeled_block(&self.label_to_go_to);

            if let Some(block) = x {
                debug!(
                    "Does function have terminator {:?}",
                    compiler
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                );

                utils::branch_only_if_no_terminator(compiler, block);

                return Ok(Box::new(nul));
            } else {
                debug!(
                    "Does function have terminator {:?}",
                    compiler
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                );

                let mut placeholder_block = compiler
                    .context
                    .append_basic_block(get_current_function(compiler), "PLACEHOLDER");

                compiler
                    .function_properties
                    .borrow_mut()
                    .store_placeholder_block(&self.label_to_go_to, placeholder_block);

                utils::branch_only_if_no_terminator(compiler, placeholder_block);
                return Ok(Box::new(nul));
            }
        }
    }
}
