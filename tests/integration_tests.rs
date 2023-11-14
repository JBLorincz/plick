    use std::error::Error;
    
    use common::test_normal_compile;
    use log::{debug, error, warn, trace, log_enabled, info, Level};

    mod common;

mod full_compile_tests
{
    use crate::common::{initialize_test_logger, run_new_test};

    use super::*;
   #[test]
    fn file_test() -> Result<(), Box<dyn Error>> 
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);







                LOL: PROCEDURE ();  RETURN 999-444; END;







                BOL: PROCEDURE(); PUT 'BOL'; RETURN 0; END;
                LOL();
                PUT 'Second';
                LOL();
                BOL();
                BOL();
                LOL();
                PUT 'Third';
                PUT 'Fourth';
                END;";

            let output = run_new_test(input)?;
            Ok(())


      }
     #[test]
    fn return_test() -> Result<(), Box<dyn Error>> 
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE ();  RETURN 999-444;
                END;
                BOL: PROCEDURE(); PUT ' BOL '; RETURN 0; END;
                LOL();
                PUT 'HELLO';
                LOL();
                BOL();
                BOL();
                LOL();
                PUT 'HELLO';
                PUT 'HELLO';
                END;";
        


            let output = run_new_test(input)?;
            assert_eq!("HELLO BOL  BOL HELLOHELLO", output.stdout);
            Ok(())
    }
     #[test]
    fn test_func_with_param() -> Result<(), Box<dyn Error>> 
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE (A);  RETURN A-4;
                END;
                BOL: PROCEDURE(); 4-7; PUT 'HELLO'; RETURN 0; END;
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
    fn test_if_statement() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN PUT 'INLINE IF IS TRUE\n'; END;";
        
        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())

    }

     #[test]
    fn test_if_else_statement() -> Result<(), Box<dyn Error>> 
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN DO; PUT 'HELLO'; PUT 'HELLO'; PUT 'HELLO\n'; END; ELSE DO; PUT 'HELLO'; PUT 'HELLO'; PUT 'HELLO'; PUT 'HELLO\n'; END; END;";

        let output = run_new_test(input)?;
        assert_eq!("HELLOHELLOHELLOHELLO\n", output.stdout);
        Ok(())

    }

    #[test]
    fn mutation_test() -> Result<(), Box<dyn Error>>
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        FLAG = 1; FLAG = 0; IF FLAG THEN PUT 'FOOBIE\n'; END;";
        
        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())


    }
    #[test]
    fn drive_hello_world() -> Result<(), Box<dyn Error>>
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; 2 + 4; END;";
        
        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())

    }
    #[test]
    fn string_test() -> Result<(), Box<dyn Error>>
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        X = 'HELLO'; END;";

        let output = run_new_test(input)?;
        assert_eq!("", output.stdout);
        Ok(())


    }

    #[test]
    fn first_string_print_test() -> Result<(), Box<dyn Error>>
    {
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        PUT 'BEEP'; END;";

        let output = run_new_test(input)?;
        assert_eq!("BEEP", output.stdout);
        Ok(())
    }


}

mod should_fails
{
    use super::*;
    use super::common::initialize_test_logger;
     #[test]
     #[should_panic(expected = "after label")]
    fn test_double_label_panic() -> ()
    {

        initialize_test_logger();
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: LOL: PROCEDURE (A);  A-4;
                END;
                BOL: PROCEDURE(); 4-7; END;
                LOL(6);
                LOL(8);
                BOL();
                BOL();
                LOL(2);
                END;";
        

        test_normal_compile(input);
    }

    #[test]
    #[should_panic(expected="support type Char")]
    fn string_conditional_panic() -> ()
    {

        initialize_test_logger();
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE (A);  RETURN A-4;
                END;
                LOL(2);
                IF 'HELLO' THEN LOL;
                END;";
        
        test_normal_compile(input);
    }
     #[test]
     #[should_panic]
    fn test_unknown_function_panic_test() 
    {

        initialize_test_logger();
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                LOLOLOLOL();
                END;";
        
        test_normal_compile(input);
    }


}

mod lexer_and_parser_integration_tests
{
    use plick::parser;
    use plick::lexer;
    use plick::ast;
    use super::common::initialize_test_logger;
    fn test_binaries()
    {
        initialize_test_logger();
        let mut token_manager = lexer::TokenManager::new("2 + 2");
        let result = parser::parse_expression(&mut token_manager);

        if let ast::Expr::Binary { operator, left, right } = result
        {
            assert_eq!(lexer::Token::PLUS, operator);

            let left_expr: ast::Expr = *left;

            if let ast::Expr::NumVal { value, _type } = left_expr{
                assert_eq!(value, 2);
            }
            else
            {
                panic!("not numval");
            }

            let right_expr: ast::Expr = *right;
            if let ast::Expr::NumVal { value, _type } = right_expr{
                assert_eq!(value, 2);
            }
            else
            {
                panic!("not numval");
            }
        }
        else
        {
            panic!("Expression was not a binary, was a {:?}", result);
        }

        //2. nested binaries
        //
        let mut token_manager = lexer::TokenManager::new("2 + 3 * 5");
        let result = parser::parse_expression(&mut token_manager);

        if let ast::Expr::Binary { operator, left, right } = result
        { // this is the 2 in 2 + 3 * 5
            assert_eq!(lexer::Token::PLUS, operator);

            let left_expr: ast::Expr = *left;

            if let ast::Expr::NumVal { value, _type } = left_expr{
                assert_eq!(value, 2);
            }
            else
            {
                panic!("not numval");
            }

            let right_expr: ast::Expr = *right; // this is the 3 * 5 in 2 + 3 * 5
            if let ast::Expr::Binary { operator, left, right } = right_expr{
                
                if let ast::Expr::NumVal { value, _type } = *left{
                    assert_eq!(3, value);
                }   
                else { panic!("not a numval!")}
                
                if let lexer::Token::MULTIPLY  = operator{
                }   
                else { panic!("not a multiply!")}
                
                if let ast::Expr::NumVal { value, _type } = *right{
                    assert_eq!(5, value);
                }   
                else { panic!("not a numval!")}
            }
            else
            {
                panic!("not numval");
            }
        }
        else
        {
            panic!("Expression was not a binary, was a {:?}", result);
        }
    }
}

