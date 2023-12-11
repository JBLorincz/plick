use common::test_normal_compile;
use std::error::Error;
pub mod common;
pub mod error_tests;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";
mod full_compile_tests {
    use crate::common::run_new_test;

    use super::*;
    #[test]
    fn function_test() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);







                LOL: PROCEDURE ();  RETURN 999-444; END;







                BOL: PROCEDURE(); PUT LIST('BOL'); RETURN 0; END;
                LOL();
                PUT LIST('Second');
                LOL();
                BOL();
                BOL();
                LOL();
                PUT LIST('Third');
                PUT LIST('Fourth');
                END;";

        let _output = run_new_test(input)?;
        Ok(())
    }
    #[test]
    fn return_test() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE ();  RETURN 999-444;
                END;
                BOL: PROCEDURE(); PUT LIST(' BOL '); RETURN 0; END;
                LOL();
                PUT LIST('HELLO');
                LOL();
                BOL();
                BOL();
                LOL();
                PUT LIST('HELLO');
                PUT LIST('HELLO');
                END;";

        let output = run_new_test(input)?;
        assert_eq!("HELLO BOL  BOL HELLOHELLO", output.stdout);
        Ok(())
    }
    #[test]
    fn create_and_print_negative() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                VARIABLE = -23;
                PUT LIST(VARIABLE);
                VARIABLE = -VARIABLE;
                PUT LIST(VARIABLE);
                VARIABLE = 0 - VARIABLE;
                PUT LIST(VARIABLE);
                VARIABLE = VARIABLE + 23;
                PUT LIST(VARIABLE);
                VARIABLE =  -VARIABLE;
                PUT LIST(VARIABLE);
                VARIABLE =  -23 + 22;
                PUT LIST(VARIABLE);
                VARIABLE =  22 - 23;
                PUT LIST(VARIABLE);
                VARIABLE =  VARIABLE + 1;
                PUT LIST(VARIABLE);
                VARIABLE =  1 - 1;
                PUT LIST(VARIABLE);
                VARIABLE =  1.0 - -1.0;
                PUT LIST(VARIABLE);
                VARIABLE =  1 + 0.0;
                PUT LIST(VARIABLE);
                VARIABLE =   22 - 22;
                PUT LIST(VARIABLE);
                END;";

        let output = run_new_test(input)?;
        assert_eq!("-(0000000000000023.000000000000000)+(0000000000000023.000000000000000)-(0000000000000023.000000000000000)+(0000000000000000.000000000000000)+(0000000000000000.000000000000000)-(0000000000000001.000000000000000)-(0000000000000001.000000000000000)+(0000000000000000.000000000000000)+(0000000000000000.000000000000000)+(0000000000000002.000000000000000)+(0000000000000001.000000000000000)+(0000000000000000.000000000000000)", output.stdout);
        Ok(())
    }
    #[test]
    fn small_neg() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                VARIABLE = 22 - 21;
                PUT LIST(VARIABLE);
                END;";

        let output = run_new_test(input)?;
        assert_eq!("+(0000000000000001.000000000000000)", output.stdout);
        Ok(())
    }
    #[test]
    fn not_operator_test() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                FLAG = 0;
                FLAG = NOT FLAG;
                PUT LIST(FLAG);
                FLAG = NOT FLAG;
                PUT LIST(FLAG);
                FLAG = NOT(NOT (1+2+3+4));
                PUT LIST(FLAG);
                END;";

        let output = run_new_test(input)?;
        assert_eq!("+(0000000000000001.000000000000000)+(0000000000000000.000000000000000)+(0000000000000001.000000000000000)", output.stdout);
        Ok(())
    }
    #[test]
    fn binary_expression_precision() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                VARIABLE = ( 1 + 1 + 1 + 1 + 1 ) / 5 ;
                PUT LIST(VARIABLE);
                END;";

        let output = run_new_test(input)?;
        assert_eq!("+(0000000000000001.000000000000000)", output.stdout);
        Ok(())
    }
    #[test]
    fn float_decimal_test() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                DECLARE VAR FLOAT;
                VAR = 4E+01;
                VAR = VAR + 1E+01;
                PUT LIST(VAR);
                END;";

        let output = run_new_test(input)?;
        assert_eq!("5.000000", output.stdout);
        Ok(())
    }
    #[test]
    //#[ignore = "very rightmost decimal digit gets set to 1 for some reason, find and fix please"]
    fn decimal_tests() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);

	FLAG = 6.2;
	FLAG = FLAG - 0.7;
	PUT LIST(FLAG);

	BARS = 4.99;
	PUT LIST(BARS);
	BARS = BARS - 0.3;
	PUT LIST(BARS);

END;";

        let output = run_new_test(input)?;

        assert_eq!("+(0000000000000005.500000000000000)+(0000000000000004.990000000000000)+(0000000000000004.690000000000000)", output.stdout);

        Ok(())
    }
    #[test]
    fn test_func_with_param() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE (A);  RETURN A-4;
                END;
                BOL: PROCEDURE(); 4-7; PUT LIST('HELLO'); RETURN 0; END;
                LOL(6);
                LOL(8);
                BOL();
                BOL();
                LOL(2);
                END;";

        let output = run_new_test(input)?;
        assert_eq!("HELLOHELLO", output.stdout);
        Ok(())
    }
    #[test]
    fn test_no_space_between_multiply_and_divide() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                VARIABLE = 5*5;
                VARIABLE = 5/5;
                PUT LIST('RAN');
                END;";

        let output = run_new_test(input)?;
        assert_eq!("RAN", output.stdout);
        Ok(())
    }
    #[test]
    fn exponent_operator() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                A = 9 ** 2;
                IF A = 81 THEN PUT LIST('GOOD');
                END;";

        let output = run_new_test(input)?;
        assert_eq!("GOOD".to_owned(), output.stdout);
        Ok(())
    }
    #[test]
    fn goto_loop_thrice() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                FLAG = 10;
                LOOP: Put LIST('Hello!');        
                FLAG = FLAG - 1;
                IF FLAG < 8 THEN GO FIN;
                GO LOOP;

                FIN: 
                PUT LIST('End!');
                END;
        ";

        let output = run_new_test(input)?;
        assert_eq!("Hello!Hello!Hello!End!", output.stdout);
        Ok(())
    }

    #[test]
    fn putting_numbers_and_strings_test() -> Result<(), Box<dyn Error>> {
        let input = "
    HELLO:   PROCEDURE OPTIONS (MAIN);

	PUT LIST(3,0);
	DECLARE VALUE FIXED;
	VALUE = 3;
	PUT LIST(VALUE);
	VALUE = 4;
	PUT LIST(VALUE);
	VALUE = 5;
	PUT LIST(VALUE);
	PUT LIST('FINAL VALUE');
	DECLARE MYLOL CHARACTER(50);
	MYLOL = 'Testy';
	PUT LIST(MYLOL);
end;
            ";

        let output = run_new_test(input)?;
        assert_eq!("+(0000000000000003.000000000000000)+(0000000000000000.000000000000000)+(0000000000000003.000000000000000)+(0000000000000004.000000000000000)+(0000000000000005.000000000000000)FINAL VALUETesty", output.stdout);
        Ok(())
    }

    #[test]
    fn if_statement_false() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN PUT LIST('INLINE IF IS TRUE\n'); END;";

        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())
    }

    #[test]
    fn if_statement_equality_op() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                FLAG = 4;
                IF FLAG = 4 THEN PUT LIST('INLINE IF IS TRUE\n'); END;";

        let output = run_new_test(input)?;
        assert_eq!("INLINE IF IS TRUE".to_owned() + LINE_ENDING, output.stdout);
        Ok(())
    }
    #[test]
    fn and_logic() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                A = 2 AND 1;
                IF A = 1 THEN PUT LIST('GOOD');
                A = 0 AND 0;
                IF A = 1 THEN PUT LIST('BAD!');
                A = 0 AND 1;
                IF A = 1 THEN PUT LIST('BAD!');
                A = 1 AND 0;
                IF A = 1 THEN PUT LIST('BAD!');
                A = 1 AND 1;
                IF A = 1 THEN PUT LIST('GOOD');
                A = 1 AND 1;
                IF A AND 1 THEN PUT LIST('GOOD');
                END;";

        let output = run_new_test(input)?;
        assert_eq!("GOODGOODGOOD".to_owned(), output.stdout);
        Ok(())
    }
    #[test]
    fn if_statement_equality_op_var() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                A = 3;
                B = 2;
                C = 3;
                A = B = C;
                IF A = 0 THEN PUT LIST('INLINE IF IS TRUE\n'); END;";

        let output = run_new_test(input)?;
        assert_eq!("INLINE IF IS TRUE".to_owned() + LINE_ENDING, output.stdout);
        Ok(())
    }

    #[test]
    fn if_statement_equality_op_var_false() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                A = 3;
                B = 2;
                C = 2;
                A = B = C;
                PUT LIST(A); END;";

        let output = run_new_test(input)?;
        assert_eq!(
            "+(0000000000000001.000000000000000)".to_owned(),
            output.stdout
        );
        Ok(())
    }

    #[test]
    fn if_statement_dynamic_false() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                FLAG = 1;
                FLAG = FLAG - 1;
                IF FLAG THEN PUT LIST('INLINE IF IS TRUE\n'); ELSE PUT LIST('FALSE\n'); END;";

        let output = run_new_test(input)?;
        assert_eq!("FALSE".to_owned() + LINE_ENDING, output.stdout);
        Ok(())
    }
    #[test]
    fn if_statement_dynamic_true() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                FLAG = 0;
                FLAG = FLAG + 1;
                IF FLAG THEN PUT LIST('TRUE\n'); ELSE PUT LIST('FALSE\n'); END;";

        let output = run_new_test(input)?;
        assert_eq!("TRUE".to_owned() + LINE_ENDING, output.stdout);
        Ok(())
    }
    #[test]
    fn if_statement_true() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 1234567 THEN PUT LIST('INLINE IF IS TRUE\n'); END;";

        let output = run_new_test(input)?;
        assert_eq!("INLINE IF IS TRUE".to_owned() + LINE_ENDING, output.stdout);
        Ok(())
    }

    #[test]
    fn if_else_statement_false() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN DO; PUT LIST('HELLO', 'HELLO', 'HELLO\n'); END; ELSE DO; PUT LIST('HELLO', 'HELLO', 'HELLO', 'HELLO\n'); END; END;";

        let output = run_new_test(input)?;
        assert_eq!(
            "HELLOHELLOHELLOHELLO".to_owned() + LINE_ENDING,
            output.stdout
        );
        Ok(())
    }
    #[test]
    fn if_else_statement_true() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 1 THEN DO; PUT LIST ('HELLO', 'HELLO', 'HELLO\n'); END; ELSE DO; PUT LIST('HELLO', 'HELLO', 'HELLO', 'HELLO\n'); END; END;";

        let output = run_new_test(input)?;
        assert_eq!("HELLOHELLOHELLO".to_owned() + LINE_ENDING, output.stdout);
        Ok(())
    }

    #[test]
    fn mutation_test() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        FLAG = 1; FLAG = 0; IF FLAG THEN PUT LIST('FOOBIE\n'); END;";

        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())
    }

    #[test]
    fn drive_hello_world() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; 2 + 4; END;";

        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())
    }
    #[test]
    fn string_test() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        X = 'HELLO'; END;";

        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())
    }

    #[test]
    fn first_string_print_test() -> Result<(), Box<dyn Error>> {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        PUT LIST('BEEP'); END;";

        let output = run_new_test(input)?;
        assert_eq!("BEEP", output.stdout);
        Ok(())
    }
}

mod should_fails {
    use crate::common::run_new_test;

    use super::common::initialize_test_logger;
    use super::*;

    #[test]
    #[should_panic(expected = "support type Char")]
    fn string_conditional_panic() -> () {
        initialize_test_logger();
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE (A);  RETURN A-4;
                END;
                LOL(2);
                IF 'HELLO' THEN LOL;
                END;";

        test_normal_compile(input).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_unknown_function_panic_test() {
        initialize_test_logger();
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                LOLOLOLOL();
                END;";

        test_normal_compile(input).unwrap();
    }
}

mod lexer_and_parser_integration_tests {
    use std::error::Error;

    use super::common::initialize_test_logger;
    use plick::ast;
    use plick::ast::Expr;
    use plick::lexer;
    use plick::parser;
    #[test]
    fn test_binaries() -> Result<(), Box<dyn Error>> {
        initialize_test_logger();
        let mut token_manager = lexer::TokenManager::new("2 + 2");
        let result = parser::parse_expression(&mut token_manager)?;

        if let ast::Expr::Binary {
            operator,
            left,
            right,
        } = result
        {
            assert_eq!(lexer::Token::PLUS, operator);

            let left_expr: ast::Expr = *left;

            if let ast::Expr::NumVal { value, _type } = left_expr {
                assert_eq!(value, 2.0);
            } else {
                panic!("not numval");
            }

            let right_expr: ast::Expr = *right;
            if let ast::Expr::NumVal { value, _type } = right_expr {
                assert_eq!(value, 2.0);
            } else {
                panic!("not numval");
            }
        } else {
            panic!("Expression was not a binary, was a {:?}", result);
        }

        //2. nested binaries
        //
        let mut token_manager = lexer::TokenManager::new("2 + 3 * 5");
        let result = parser::parse_expression(&mut token_manager)?;

        if let ast::Expr::Binary {
            operator,
            left,
            right,
        } = result
        {
            // this is the 2 in 2 + 3 * 5
            assert_eq!(lexer::Token::PLUS, operator);

            let left_expr: ast::Expr = *left;

            if let ast::Expr::NumVal { value, _type } = left_expr {
                assert_eq!(value, 2.0);
            } else {
                panic!("not numval");
            }

            let right_expr: ast::Expr = *right; // this is the 3 * 5 in 2 + 3 * 5
            if let ast::Expr::Binary {
                operator,
                left,
                right,
            } = right_expr
            {
                if let ast::Expr::NumVal { value, _type } = *left {
                    assert_eq!(3.0, value);
                } else {
                    panic!("not a numval!")
                }

                if let lexer::Token::MULTIPLY = operator {
                } else {
                    panic!("not a multiply!")
                }

                if let ast::Expr::NumVal { value, _type } = *right {
                    assert_eq!(5.0, value);

                    Ok(())
                } else {
                    panic!("not a numval!")
                }
            } else {
                panic!("not numval");
            }
        } else {
            panic!("Expression was not a binary, was a {:?}", result);
        }
    }

    #[test]
    fn parse_const_negative_number() -> Result<(), Box<dyn Error>> {
        let input = "-234";
        let mut tok_man = lexer::TokenManager::new(input);

        let result = parser::parse_constant_numeric(&mut tok_man)?;
        let resulting_num;
        match result {
            Expr::NumVal { value, _type } => {
                resulting_num = value;
            }
            other => {
                panic!("Expected numval, found {:#?}", other);
            }
        };

        assert_eq!(resulting_num, -234.0);
        Ok(())
    }

    #[test]
    fn parse_decimal_number() -> Result<(), Box<dyn Error>> {
        let input = "86.231";
        let mut tok_man = lexer::TokenManager::new(input);

        let result = parser::parse_constant_numeric(&mut tok_man)?;
        let resulting_num;
        match result {
            Expr::NumVal { value, _type } => {
                resulting_num = value;
            }
            other => {
                panic!("Expected numval, found {:#?}", other);
            }
        };

        assert_eq!(resulting_num, 86.231);

        Ok(())
    }
}
