    use std::error::Error;

    use plick::{compile_input, Config};


    #[test]
    fn file_test() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);







                LOL: PROCEDURE ();  RETURN 999-444; END;







                BOL: PROCEDURE(); PUT; RETURN 0; END;
                LOL();
                PUT;
                LOL();
                BOL();
                BOL();
                LOL();
                PUT;
                PUT;
                END;";
        
    let mut conf = Config::default();
    conf.filename = "file_test.o".to_string();
        compile_input(input,conf);
        Ok(())
    }
     #[test]
    fn return_test() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE ();  RETURN 999-444;
                END;
                BOL: PROCEDURE(); PUT; RETURN 0; END;
                LOL();
                PUT;
                LOL();
                BOL();
                BOL();
                LOL();
                PUT;
                PUT;
                END;";
        
    let conf = Config::default();
        compile_input(input,conf);
        Ok(())
    }
     #[test]
    fn test_func_with_param() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                    LOL: PROCEDURE (A);  RETURN A-4;
                END;
                BOL: PROCEDURE(); 4-7; PUT; RETURN 0; END;
                LOL(6);
                LOL(8);
                BOL();
                BOL();
                LOL(2);
                END;";
        
        let mut conf = Config::default();
        conf.filename = "testtwo.o".to_string();
        compile_input(input,conf);
        Ok(())
    }
     #[test]
    fn test_if_statement() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN PUT; END;";
        
        let mut conf = Config::default();
        conf.filename = "testif_false.o".to_string();
        compile_input(input,conf);
        Ok(())
    }

     #[test]
    fn test_if_else_statement() -> Result<(), Box<dyn Error>> 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                IF 0 THEN DO; PUT; PUT; PUT; END; ELSE DO; PUT; PUT; PUT; PUT; END; END;";
        
        let mut conf = Config::default();
        conf.filename = "testif_else_false.o".to_string();
        compile_input(input,conf);
        Ok(())
    }

     #[test]
     #[should_panic(expected = "after label")]
    fn test_double_label_panic() -> ()
    {

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
        
        let mut conf = Config::default();
        conf.filename = "failfile.o".to_string();
        compile_input(input,conf);
    }
     #[test]
     #[should_panic]
    fn test_unknown_function_panic_test() 
    {

        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
                LOLOLOLOL();
                END;";
        
        let mut conf = Config::default();
        conf.filename = "failfile.o".to_string();
        compile_input(input,conf);
    }
    #[test]
    fn mutation_test(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        FLAG = 1; FLAG = 0; IF FLAG THEN PUT; END;";

        let mut conf = Config::default();
        conf.filename = "mutation_test.o".to_string();
        compile_input(input,conf);
    }
    #[test]
    fn drive_hello_world(){
        let input = "HELLO:   PROCEDURE OPTIONS (MAIN);
        2 + 2 + 4 / 6; 2 + 4; END;";

        let conf = Config::default();
        compile_input(input,conf);
    }



mod lexer_and_parser_integration_tests
{
    use plick::parser;
    use plick::lexer;
    use plick::ast;
    fn test_binaries()
    {
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
