use inkwell::{values::{FloatValue, StructValue, IntValue, PointerValue, BasicValue}, basic_block::BasicBlock};

use crate::{codegen::{codegen::Compiler, utils::{self, get_nth_digit_of_a_float}}, types::fixed_decimal::{FixedDecimalToFloatBuilder, BEFORE_DIGIT_COUNT}};

use super::{FixedValue, create_empty_fixed};

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub unsafe fn fixed_decimal_to_float(&self, fixed_value: &FixedValue<'ctx>) -> FloatValue<'ctx> {
        log::info!("Converting fixed value {:?} into a decimal!", fixed_value);

        let fixed_value_as_struct_value: StructValue<'ctx> = fixed_value.value;

        let mut fd_to_float_converter = FixedDecimalToFloatBuilder::new(self,&fixed_value_as_struct_value);

        fd_to_float_converter.alloca_struct_value();

        let sign_bit_value = fd_to_float_converter.get_sign_bit_value();
        
        let ptr_to_before_array = fd_to_float_converter.get_before_ptr();

        dbg!(ptr_to_before_array);
        let before_arr = fd_to_float_converter.get_before_array();
        dbg!(before_arr);

        let zero_intval = self.context.i8_type().const_zero();
        let mut before_int_values: Vec<IntValue<'ctx>> =
            vec![zero_intval; BEFORE_DIGIT_COUNT as usize];

        for i in 0..BEFORE_DIGIT_COUNT as usize {
            let current_digit_index = self.context.i8_type().const_int(i as u64, false);
        
            let digit_int_val = fd_to_float_converter.load_digit_from_digit_array(current_digit_index, ptr_to_before_array);

            //now we take the array value, build a GEP for the inner array
            before_int_values[i] = digit_int_val;
        }

        let result_floatval = fd_to_float_converter.sum_up_before_digits_into_a_float(before_int_values);

        //TODO: if negative number, multiply by -1.0
        let conditional = sign_bit_value;

        let mut negative_float = result_floatval.get_type().const_float(0.0);
        let mut positive_float = result_floatval.get_type().const_float(0.0);
        let then_block = ||
        {
            let rhs = result_floatval.get_type().const_float(-1.0);
            negative_float = self.builder.build_float_mul(result_floatval, rhs, "make_float_neg").unwrap();
        };
        let else_block = ||
        {
            let rhs = result_floatval.get_type().const_float(1.0);
            positive_float = self.builder.build_float_mul(result_floatval, rhs, "keep_float_positive").unwrap();
        };
        let blocks = self.build_if_else_control_flow
            (conditional,
             then_block,
             else_block
             );

        let phi = self.builder.build_phi(result_floatval.get_type(), "result_of_negation").unwrap();
        phi.add_incoming(&[(&negative_float.as_basic_value_enum(),blocks.then_block)]);
        phi.add_incoming(&[(&positive_float.as_basic_value_enum(),blocks.else_block)]);
        



        phi.as_basic_value().into_float_value()
    }


    pub unsafe fn float_value_to_fixed_decimal(&'a self, float_value: FloatValue<'ctx>) -> FixedValue<'ctx>
    {
        let zero_intval = self.context.i8_type().const_zero();
        let fixed_value: StructValue<'ctx> = create_empty_fixed(self.context, &self.type_module.fixed_type);
        let allocd_fd: PointerValue<'ctx> = self.builder.build_alloca(fixed_value.get_type(), "allocate_fd_for_fixed_conv").unwrap();
        self.builder.build_store(allocd_fd, fixed_value).unwrap();
        self.builder.build_alloca(float_value.get_type(), "allocate_float_for_fixed_conv").unwrap();


        let rhs: FloatValue<'ctx> = float_value.get_type().const_zero();


            let conditional = self
            .builder
            .build_float_compare(
                inkwell::FloatPredicate::OLT,
                float_value,
                rhs,
                "is_float_neg")
            .unwrap();

            //start of sign bit loading
            let current_func = utils::get_current_function(self);
            let then_block = self.context.append_basic_block(current_func, "then_block");
            let else_block = self.context.append_basic_block(current_func, "else_block");
            let cont_block = self.context.append_basic_block(current_func, "cont_block");
                self.
                builder.
                build_conditional_branch(conditional, then_block, else_block)
                .expect("This should be fine!");

            self.builder.position_at_end(then_block);
            
            let sign_ptr = self.builder.build_struct_gep(allocd_fd, 0, "get_sign_bit_ptr").unwrap();
            self.builder.build_store(sign_ptr, self.context.bool_type().const_int(1, false)).unwrap();
            let rhs = float_value.get_type().const_float(-1.0);
            let float_value_neg_to_pos = self.builder.build_float_mul(float_value, rhs, "turn_negative_float_into_positive").unwrap();
            self.builder.build_unconditional_branch(cont_block).unwrap();
                        
            self.builder.position_at_end(else_block);
            let sign_ptr = self.builder.build_struct_gep(allocd_fd, 0, "get_sign_bit_ptr").unwrap();
            self.builder.build_store(sign_ptr, self.context.bool_type().const_int(0, false)).unwrap();
            let rhs = float_value.get_type().const_float(1.0);
            let float_value_pos_to_pos = self.builder.build_float_mul(float_value, rhs, "keep_float_positive").unwrap();
            self.builder.build_unconditional_branch(cont_block).unwrap();

            self.builder.position_at_end(cont_block);
            //end of sign bit loading
            let phi = self
                .builder
                .build_phi(float_value_pos_to_pos.get_type(), "float_after_maybe_neg")
                .unwrap();
                phi.add_incoming(&[(&float_value_neg_to_pos,then_block)]);
                phi.add_incoming(&[(&float_value_pos_to_pos,else_block)]);



            let float_value = phi.as_basic_value().into_float_value();
            //start of calculating fv digits
            let bdcl = self.context.append_basic_block(current_func, "before_digits_calculation_loop");
            self.builder.build_unconditional_branch(bdcl).unwrap();
            self.builder.position_at_end(bdcl);

            //counter 

            let before_array_ptr = self.builder.build_struct_gep(allocd_fd, 1, "get_before_array_ptr").unwrap();
            for i in 0..BEFORE_DIGIT_COUNT
            {
                let index_as_intval = self.context.i8_type().const_int(i as u64, false);
                let digit_to_load_up = get_nth_digit_of_a_float(self, &float_value, index_as_intval);
                let digi_as_i8 = self.builder.build_int_cast(digit_to_load_up, self.context.i8_type(), "turn_i64_into_i8").unwrap();

                let current_digit_ptr = self.builder.build_gep(before_array_ptr,&[zero_intval,index_as_intval],"getdigiforconv")
                    .unwrap();

                self.builder.build_store(current_digit_ptr, digi_as_i8).unwrap();
            }
            

            let after_before_loop = self.context.append_basic_block(current_func, "after_digits_loop");
            self.builder.build_unconditional_branch(after_before_loop).unwrap();
            self.builder.position_at_end(after_before_loop);
            //end of calculating fv digits

        
        let fd_struct: StructValue<'ctx> = self.builder.build_load(allocd_fd, "loading_final_fd").unwrap().into_struct_value(); 
        FixedValue::new(fd_struct)
    }





    fn build_if_else_control_flow<T,Y>(&self, conditional: IntValue<'ctx>, mut then_block_gen: T, mut else_block_gen: Y) -> ThenElseBlocks<'ctx>
        where T: FnMut(), Y: FnMut() 
    {
            let current_func = utils::get_current_function(self);
            let then_block = self.context.append_basic_block(current_func, "then_block");
            let else_block = self.context.append_basic_block(current_func, "else_block");
            let merge_block = self.context.append_basic_block(current_func, "merge_block");


                self.
                builder.
                build_conditional_branch(conditional, then_block, else_block)
                .expect("This should be fine!");


            self.builder.position_at_end(then_block);
            then_block_gen();
            self.builder.build_unconditional_branch(merge_block).unwrap();
                        

            self.builder.position_at_end(else_block);
            else_block_gen();
            self.builder.build_unconditional_branch(merge_block).unwrap();


            self.builder.position_at_end(merge_block);


            let x: ThenElseBlocks<'ctx> = ThenElseBlocks {then_block, else_block, merge_block};
            x

    }
}



struct ThenElseBlocks<'ctx>
{
    then_block: BasicBlock<'ctx>,
    else_block: BasicBlock<'ctx>,
    merge_block: BasicBlock<'ctx>,
}
