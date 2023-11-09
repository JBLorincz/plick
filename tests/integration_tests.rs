mod tests {
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
}
