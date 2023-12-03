use plick::compile_input;

use crate::common::{generate_error_test_config, initialize_test_logger};

fn run_error_test(input: &str, expected_error: &str)
{

    initialize_test_logger();
    let config = generate_error_test_config();
    let results = compile_input(input, config);

    for error in results.errors
    {
        log::info!("Checking if error {:#?} is matched by {:#?}", error.message, expected_error);
        if error.message.contains(expected_error)
        {
            panic!("Found error {:?}, as was expected by this test!", expected_error);
        }
    }
}


#[test]
#[should_panic]
fn wrong_type_assignment()
{

    let input = 
"HELLO:   PROCEDURE OPTIONS (MAIN);

DCL MYVAR CHARACTER(20);
MYVAR = 3.32;
PUT LIST(MYVAR);

END;";

run_error_test(input, "E009");
}

#[test]
#[should_panic]
fn missing_label()
{

    let input = 
"HELLO:   PROCEDURE OPTIONS (MAIN);

DCL MYVAR CHARACTER(20);
GO MISSING_LABEL;
GO ANOTHER_LABEL;
GO TEST;

END;";

run_error_test(input, "E010");
}

#[test]
#[should_panic]
fn multiple_labels()
{

    let input = 
"HELLO:   PROCEDURE OPTIONS (MAIN);

NONDUPE_LABEL:
PUT LIST('HELLO');
DUPE_LABEL:
PUT LIST('HELLO');
DUPE_LABEL:
PUT LIST('HELLO');
END;";

run_error_test(input, "E011");
}
    #[test]
     #[should_panic]
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
        

        run_error_test(input,"E003");
    }
