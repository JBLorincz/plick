use std::error::Error;

use crate::ast;
use crate::ast::*;
use crate::error;
use crate::types::Type;
use crate::{
    codegen::codegen::CodeGenable,
    error::get_error,
    lexer::{self, Token},
    types::infer_pli_type_via_name,
};

use log::trace;

///Helper function used to advance the token manager,
///ensuring the token that was 'eaten' was the expected
///token. Otherwise, returns an error.
pub fn parse_token(
    token_manager: &mut lexer::TokenManager,
    token_to_check_for: Token,
) -> Result<(), String> {
    let current_tok_copy = token_manager.current_token.clone();
    if let None = current_tok_copy {
        let err_msg = get_error(&["2"]);
        return Err(err_msg);
    }

    let current_tok_copy = current_tok_copy.unwrap();

    if std::mem::discriminant(&token_to_check_for) == std::mem::discriminant(&current_tok_copy) {
        token_manager.next_token();
        Ok(())
    } else {
        let err_msg = get_error(&[
            "1",
            &token_to_check_for.to_string(),
            &current_tok_copy.to_string(),
        ]);
        Err(err_msg)
    }
}

pub fn parse_numeric<'a>(token_manager: &'a mut lexer::TokenManager) -> Expr {
    if let Some(Token::NumVal(value)) = token_manager.current_token {
        token_manager.next_token(); //loads the next token into the token manager.
        return Expr::NumVal {
            value,
            _type: Type::FixedDecimal,
        };
    } else {
        panic!("Failed to parse numeric!");
    }
}

///parses an IF clause
///So far, only supports IF *expr* THEN *statement*;
///or IF *expr* THEN DO *statements*;
pub fn parse_if<'a>(token_manager: &'a mut lexer::TokenManager) -> Result<If, String> {
    //current token is IF

    let _if = parse_token(token_manager, Token::IF)?;
    let conditional = parse_expression(token_manager);
    let mut else_statements: Option<Vec<Statement>> = None;
    let mut then_statements: Vec<Statement> = vec![];

    let _then = parse_token(token_manager, Token::THEN)?;

    let possible_do = parse_token(token_manager, Token::DO);

    if let Ok(()) = possible_do {
        then_statements = parse_do_block(token_manager)?;
    } else {
        let statement = parse_statement(token_manager)?;
        then_statements.push(statement);
    }

    let possible_else = parse_token(token_manager, Token::ELSE);

    if let Ok(()) = possible_else {
        //handle else statements here.
        let mut else_vec = vec![];

        let possible_do = parse_token(token_manager, Token::DO);

        if let Ok(()) = possible_do {
            else_statements = Some(parse_do_block(token_manager)?);
        } else {
            let else_statement = parse_statement(token_manager)?;
            else_vec.push(else_statement);
            else_statements = Some(else_vec);
        }
    }

    //parse_token(token_manager, Token::SEMICOLON)?;

    Ok(If {
        conditional,
        then_statements,
        else_statements,
    })
}

pub fn parse_declare(token_manager: &mut lexer::TokenManager) -> Result<Declare, String> {
    parse_token(token_manager, Token::DECLARE)?;
    let new_variable_name;
    let mut variable_type = Type::FixedDecimal;

    if let Some(Token::Identifier(ref name)) = token_manager.current_token {
        new_variable_name = name.clone();
    } else {
        panic!("waht?");
    }
    token_manager.next_token();

    match token_manager.current_token {
        Some(Token::FIXED) => {
            variable_type = Type::FixedDecimal;
            token_manager.next_token();
        }
        Some(Token::FLOAT) => {
            variable_type = Type::Float;
            token_manager.next_token();
        }
        Some(Token::SEMICOLON) => variable_type = variable_type,
        _ => return Err("Could not parse declare statement".to_string()),
    };

    parse_token(token_manager, Token::SEMICOLON)?;

    Ok(Declare {
        var_name: new_variable_name,
        attribute: Some(variable_type),
    })
}
//current token is the semicolon AFTER do
pub fn parse_do_block(token_manager: &mut lexer::TokenManager) -> Result<Vec<Statement>, String> {
    let mut statements: Vec<Statement> = vec![];

    //parse_token(token_manager,Token::DO)?;
    parse_token(token_manager, Token::SEMICOLON)?;
    loop {
        if let Token::END = token_manager.current_token.as_mut().unwrap() {
            parse_token(token_manager, Token::END)?;
            parse_token(token_manager, Token::SEMICOLON)?;
            break;
        }

        let statement = parse_statement(token_manager)?;
        statements.push(statement);
    }

    Ok(statements)
}

//parses identifiers like variable names but also function calls
pub fn parse_identifier<'a>(token_manager: &'a mut lexer::TokenManager) -> Expr {
    trace!("Parsing Identifier");
    let identifier_string: String;
    if let Some(Token::Identifier(ref val)) = token_manager.current_token {
        identifier_string = val.clone();
        trace!("The identifier string is: {}", identifier_string);
    } else {
        panic!("failed to parse identifier!");
    }
    let mut args_list: Vec<Expr> = vec![];
    token_manager.next_token(); // prime next token
    if let Some(Token::OPEN_PAREN) = token_manager.current_token
    // if this is a function call
    {
        trace!("Found an open parenthesis first!");
        //function call here.
        //now we loop through each expression in the arguments
        let mut expecting_comma: bool = false; // expecting comma does not affect breaking
        trace!("Turning expecting comma off!");
        token_manager.next_token();
        loop {
            trace!("Looping!");
            if let Some(Token::CLOSED_PAREN) = token_manager.current_token {
                trace!("found a closed parenthesis!");
                token_manager.next_token(); // eat the next token, ready for next use
                break;
            } else if let Some(Token::COMMA) = token_manager.current_token {
                if expecting_comma {
                    trace!("Found comma at right place, continuing!");
                    expecting_comma = false;
                    token_manager.next_token(); // eat the token
                } else {
                    panic!("Expected an expression, found a comma!!");
                }
            } else if let Some(ref token) = token_manager.current_token {
                //parse as expression
                trace!("Found a token called {:#?}", *token);
                let parsed_arg: Expr = parse_expression(token_manager);

                args_list.push(parsed_arg);

                expecting_comma = true;
                trace!("turned expecting comma on!");
            } else {
                panic!("{:?}", token_manager.current_token);
            }
        }
        return Expr::Call {
            fn_name: identifier_string,
            args: args_list,
            _type: Type::TBD,
        };
    } else {
        return Expr::Variable {
            name: identifier_string,
            _type: Type::FixedDecimal,
        };
    }
}

///the current token is a '(' / Token::OPEN_PAREN
pub fn parse_parenthesis_expression(token_manager: &mut lexer::TokenManager) -> Expr {
    token_manager.next_token();
    let result: Expr = parse_expression(token_manager);

    if token_manager.current_token != Some(Token::CLOSED_PAREN) {
        panic!("Missing closed parenthesis on parenthesis expression!");
    }

    return result;
}

pub fn parse_expression<'a>(token_manager: &'a mut lexer::TokenManager) -> Expr {
    let left_handed_side = parse_primary_expression(token_manager);
    if let Expr::Variable { name, _type } = left_handed_side.clone() {
        if let Some(Token::EQ) = token_manager.current_token {
            token_manager.next_token(); // eat the equal token
            let expression_value = parse_expression(token_manager);
            //return Expr::Assignment(name, expression_value);
            return Expr::Assignment {
                variable_name: name,
                value: Box::new(expression_value),
            };
        }
    } else if let Expr::Char { value } = left_handed_side.clone() {
        token_manager.next_token(); //eat the string token
        return left_handed_side;
    }
    return match token_manager.current_token {
        Some(Token::PLUS)
        | Some(Token::MINUS)
        | Some(Token::DIVIDE)
        | Some(Token::MULTIPLY)
        | Some(Token::GREATER_THAN)
        | Some(Token::LESS_THAN) => {
            let token_precedence =
                get_binary_operator_precedence(token_manager.current_token.as_ref().unwrap());

            build_recursive_binary_tree(token_manager, left_handed_side, token_precedence)
        }
        _ => left_handed_side,
    };
}

pub fn build_recursive_binary_tree(
    token_manager: &mut lexer::TokenManager,
    lhs: Expr,
    precendence: i32,
) -> Expr {
    //LHS has to be a binary node.
    let operator_token: Token = token_manager.current_token.as_ref().unwrap().clone();
    token_manager.next_token();
    //if the current precedence is GREATER than the lhs precendence,
    if let Expr::Binary {
        operator,
        left,
        right,
    } = lhs
    {
        if precendence > get_binary_operator_precedence(&operator) {
            // meaning we have to make the RHS side of the LHS the LHS of our
            // new RHS
            let new_rhs = parse_expression(token_manager);

            //take 2 + 3 * 5
            //normally it will look like a tree of
            //           /   3
            //    bin - >  - +
            //           \   2
            //
            //    but we will make it look like
            //
            //           /   (3 * 5)
            //    bin - >  - +
            //           \   2
            let inner_binary = Expr::Binary {
                operator: operator_token,
                left: right,
                right: Box::new(new_rhs),
            };
            return Expr::Binary {
                operator,
                left,
                right: Box::new(inner_binary),
            };
        } else {
            let rhs_expression = parse_expression(token_manager);
            Expr::Binary {
                operator: operator_token,
                left: Box::new(Expr::Binary {
                    operator,
                    left,
                    right,
                }),
                right: Box::new(rhs_expression),
            }
        }
    } else {
        let right = parse_expression(token_manager);
        return Expr::Binary {
            operator: operator_token,
            left: Box::new(lhs),
            right: Box::new(right),
        };
    }
}
pub fn parse_primary_expression(token_manager: &mut lexer::TokenManager) -> Expr {
    match token_manager.current_token.as_ref().unwrap() {
        Token::OPEN_PAREN => parse_parenthesis_expression(token_manager),
        Token::Identifier(_) => parse_identifier(token_manager),
        Token::NumVal(_) => parse_numeric(token_manager),
        Token::STRING(value) => Expr::Char {
            value: value.clone(),
        },
        other => panic!("Can't parse top level token type {:?}", other),
    }
}

pub fn get_binary_operator_precedence(token: &lexer::Token) -> i32 {
    match token {
        Token::PLUS => 20,
        Token::MINUS => 20,
        Token::MULTIPLY => 40,
        Token::DIVIDE => 40,
        Token::LESS_THAN => 10,
        Token::GREATER_THAN => 10,
        _ => -1,
    }
}

//The token is currently PROCEDURE
//CALC: PROCEDURE(A,B,C); // we are just parsing this part.
//      RETURN(A+B+C);
//      END;
pub fn parse_function_prototype(
    token_manager: &mut lexer::TokenManager,
    label_name: String,
) -> Result<Prototype, String> {
    parse_token(token_manager, Token::PROCEDURE)?;

    let source_loc = token_manager.get_source_location();

    //token should now be open paren

    parse_token(token_manager, Token::OPEN_PAREN)?;
    let mut expecting_comma = false;
    let mut args_list: Vec<String> = vec![];
    loop {
        if let Some(Token::CLOSED_PAREN) = token_manager.current_token {
            parse_token(token_manager, Token::CLOSED_PAREN)?;

            parse_token(token_manager, Token::SEMICOLON)?;
            break;
        } else if let Some(Token::COMMA) = token_manager.current_token {
            if expecting_comma {
                expecting_comma = false;
                parse_token(token_manager, Token::COMMA)?;
            } else {
                return Err(format!(
                    "Expected an expression, found a comma at {}:{}",
                    source_loc.line_number, source_loc.column_number
                )
                .to_string());
            }
        } else if let Some(ref token) = token_manager.current_token {
            //parse as expression
            trace!("Found a token called {:#?}", *token);
            let parsed_arg: Expr = parse_expression(token_manager);

            let arg_name: String;
            if let Expr::Variable { name, _type } = parsed_arg {
                arg_name = name.clone();
            } else {
                return Err(format!("Expected variable in function prototype, found _").to_string());
            }

            args_list.push(arg_name);

            expecting_comma = true;
            trace!("turned expecting comma on!");
        } else {
            return Err(format!("{:?}", token_manager.current_token).to_string());
        }
    }

    Ok(Prototype {
        fn_name: label_name,
        args: args_list,
        source_loc,
    })
}

pub fn parse_arguments_in_parens(token_manager: &mut lexer::TokenManager) -> Result<Vec<Expr>, String>

{
    parse_token(token_manager, Token::OPEN_PAREN)?;
    let mut expecting_comma = false;
    let mut args_list: Vec<Expr> = vec![];
    loop {

        let source_loc = token_manager.get_source_location();
        if let Some(Token::CLOSED_PAREN) = token_manager.current_token {
            parse_token(token_manager, Token::CLOSED_PAREN)?;

            //parse_token(token_manager, Token::SEMICOLON)?;
            break;
        } else if let Some(Token::COMMA) = token_manager.current_token {
            if expecting_comma {
                expecting_comma = false;
                parse_token(token_manager, Token::COMMA)?;
            } else {
                return Err(format!(
                    "Expected an expression, found a comma at {}",
                    source_loc
                )
                .to_string());
            }
        } else if let Some(ref token) = token_manager.current_token {
            //
            //START OF CALLBACK
            trace!("Found a token called {:#?}", *token);
            let parsed_arg: Expr = parse_expression(token_manager);

            //let arg_name: String;
            //if let Expr::Variable { name, _type } = parsed_arg {
            //    arg_name = name.clone();
            //} else {
            //    return Err(format!("Expected variable in function prototype, found _").to_string());
            //}

            args_list.push(parsed_arg);
            //END OF CALLBACK SHIT

            expecting_comma = true;
            trace!("turned expecting comma on!");
        } else {
            return Err(format!("{:?}", token_manager.current_token).to_string());
        }
    }
    Ok(args_list)
}



pub fn parse_put(token_manager: &mut lexer::TokenManager) -> Result<Put, String> {
    parse_token(token_manager, Token::PUT)?;
    let expr_to_print = parse_expression(token_manager);

    Ok(Put {
        message_to_print: expr_to_print,
    })
}

pub fn parse_function(
    token_manager: &mut lexer::TokenManager,
    label_name: String,
) -> Result<Function, String> {
    let proto = parse_function_prototype(token_manager, label_name)?;
    let fn_name_copy = proto.fn_name.clone();
    let mut body_statements: Vec<Statement> = vec![];
    let mut return_value: Option<Expr> = None;

    loop {
        let current_statement = parse_statement(token_manager)?;
        body_statements.push(current_statement);

        if let Command::RETURN(ref expr) = body_statements.last().unwrap().command {
            //handle double return statements error in a function
            if let Some(_expr) = return_value {
                return Err(get_error(&["6"]));
            }

            return_value = Some(expr.clone());

            //lets remove the return from body_statements as well
            body_statements.pop();
        } else if let Command::END = body_statements.last().unwrap().command {
            body_statements.pop(); //remove this from the body_statements.
            break;
        }
    }

    trace!("Exiting the function parsing!");

    parse_token(token_manager, Token::SEMICOLON)?;

    Ok(Function {
        prototype: proto,
        body_statements,
        return_value,
        return_type: infer_pli_type_via_name(&fn_name_copy),
    })
}

///Parses a PL/1 statement
pub fn parse_statement(token_manager: &mut lexer::TokenManager) -> Result<Statement, String> {
    let mut command: Command = Command::Empty;
    let mut label: Option<String> = None;
    while let Some(ref token) = token_manager.current_token {
        dbg!(&token);
        match token {
            Token::SEMICOLON => {
                token_manager.next_token(); //eat the semicolon
                break; //statement is now over since semicolon has been found.
            }
            Token::LABEL(label_string) => {
                if let Some(other_label) = label {
                    return Err(get_error(&["3", &label_string, &other_label]));
                }
                match command {
                    Command::Empty => (),
                    other_command => {
                        return Err(get_error(&["4", "LABEL", &other_command.to_string()]));
                    }
                }

                label = Some(label_string.to_string()); //store the fact something
                token_manager.next_token(); //is labelled
            }
            Token::PUT => {
                match command {
                    Command::Empty => command = Command::PUT(parse_put(token_manager)?),
                    other_command => {
                        return Err(get_error(&["4", "PUT", &other_command.to_string()]));
                    }
                }
                dbg!(&token_manager.current_token);
                //panic!("Put just ran!");
                parse_token(token_manager, Token::SEMICOLON)?;
                break;
                //token_manager.next_token();
            }
            Token::PROCEDURE => {
                let fn_name: String;
                match label {
                    Some(ref val) => fn_name = val.clone(),
                    None => {
                        panic!("Could not find the label associated with a function definition!")
                    }
                }

                let result = parse_function(token_manager, fn_name.clone())?;
                return Ok(Statement {
                    label: Some(fn_name),
                    command: Command::FunctionDec(result),
                });
            }
            Token::END => {
                match command {
                    Command::Empty => command = Command::END,
                    other_command => {
                        return Err(get_error(&["4", "END", &other_command.to_string()]));
                    }
                }
                token_manager.next_token();
                break;
            }
            Token::DECLARE => {
                let declare_statement = parse_declare(token_manager)?;
                match command {
                    Command::Empty => command = Command::Declare(declare_statement),
                    other_command => {
                        return Err(get_error(&["4", "DECLARE", &other_command.to_string()]));
                    }
                }
                token_manager.next_token();
                break;
            }
            Token::IF => {
                let if_statement = parse_if(token_manager).unwrap();
                match command {
                    Command::Empty => command = Command::IF(if_statement),
                    other_command => {
                        return Err(get_error(&["4", "IF", &other_command.to_string()]));
                    }
                }
                //token_manager.next_token();
                break;
            }
            Token::RETURN => {
                token_manager.next_token();
                let token_after_return = &token_manager.current_token.clone().unwrap().clone();
                if let Token::SEMICOLON = token_after_return {
                    match command {
                        Command::Empty => command = Command::RETURN(Expr::new_numval(-1)),
                        other_command => {
                            return Err(get_error(&["4", "RETURN", &other_command.to_string()]));
                        }
                    }
                    token_manager.next_token();
                    return Ok(Statement { label, command });
                }
                match command {
                    Command::Empty => command = Command::RETURN(parse_expression(token_manager)),
                    other_command => {
                        return Err(get_error(&["4", "RETURN", &other_command.to_string()]));
                    }
                }
                token_manager.next_token();
                break;
            }

            _ => {
                let expr = parse_expression(token_manager);
                dbg!(&expr);
                let new_command;
                if let Expr::Assignment {
                    variable_name,
                    value,
                } = expr
                {
                    let assn = Assignment {
                        var_name: variable_name,
                        value: *value,
                    };
                    new_command = Command::Assignment(assn);
                } else {
                    new_command = Command::EXPR(expr)
                }
                dbg!(&command);
                match command {
                    Command::Empty => command = new_command,
                    other_command => {
                        return Err(get_error(&["4", "expression", &other_command.to_string()]));
                    }
                }
            }
        }
    } // end while loop

    Ok(Statement { label, command })
}

///parses the beginning of a PL/1 Program.
///They look like this:
/// ANY_LABEL_HERE : PROCDURE OPTIONS (MAIN);
pub fn parse_opening(token_manager: &mut lexer::TokenManager) -> Result<(), String> {
    if let Some(Token::LABEL(_)) = token_manager.current_token {
        token_manager.next_token();
    } else {
        panic!("Program not beginning with a label!");
    }
    if let Some(Token::PROCEDURE) = token_manager.current_token {
        token_manager.next_token();
    } else {
        panic!("Program missing main proc");
    }
    if let Some(Token::OPTIONS) = token_manager.current_token {
        token_manager.next_token();
    } else {
        panic!("Program missing OPTIONS attribute on main procedure!");
    }
    if let Some(Token::OPEN_PAREN) = token_manager.current_token {
        token_manager.next_token();
    } else {
        panic!("Program missing OPEN PAREN on main procedure!");
    }
    if let Some(Token::Identifier(ref var)) = token_manager.current_token {
        if var == "MAIN" {
            token_manager.next_token();
        } else {
            panic!("Option in main procedure is not MAIN");
        }
    } else {
        panic!("Program missing MAIN OPTION on main procedure!");
    }
    if let Some(Token::CLOSED_PAREN) = token_manager.current_token {
        token_manager.next_token();
    } else {
        panic!("Program missing CLOSED PAREN on main procedure!");
    }
    parse_token(token_manager, Token::SEMICOLON)?;

    Ok(())
}


// TRAITS ////////////////

pub trait Parseable {
    fn parse_from_tokens(token_manager: &mut lexer::TokenManager) -> Result<Box<Self>, Box<dyn Error>>;
}

impl Parseable for ast::IOList
{
    fn parse_from_tokens(token_manager: &mut lexer::TokenManager) -> Result<Box<Self>, Box<dyn Error>> {
        
        parse_token(token_manager, Token::LIST)?;
        
        let items: Vec<Expr> = parse_arguments_in_parens(token_manager)?;

        //parse_token(token_manager, Token::SEMICOLON)?;


        Ok(Box::new(IOList{ items}))

    }
}




mod tests {

    use crate::lexer::TokenManager;

    use super::*;

    #[test]
    fn construct_binary() {
        let lhs = Expr::new_numval(4);
        let rhs = Expr::new_numval(6);

        let _test = Expr::Binary {
            operator: Token::PLUS,
            left: Box::new(lhs),
            right: Box::new(rhs),
        };

        let lhsvar = Expr::Variable {
            name: String::from("x"),
            _type: Type::FixedDecimal,
        };

        let rhsvar = Expr::Variable {
            name: String::from("y"),
            _type: Type::FixedDecimal,
        };

        let _test = Expr::Binary {
            operator: Token::PLUS,
            left: Box::new(lhsvar),
            right: Box::new(rhsvar),
        };

        let lhsvar = Expr::Variable {
            name: String::from("x"),
            _type: Type::FixedDecimal,
        };
        if let Expr::Variable {
            name,
            _type: Type::FixedDecimal,
        } = lhsvar
        {
            assert_eq!(name, "x");
        } else {
            panic!("panicking here!");
        }
    }

    #[test]
    fn test_parsing_numeric() {
        let mut tok_man = TokenManager::new("4");

        let result: Expr = parse_numeric(&mut tok_man);

        if let Expr::NumVal { value, _type } = result {
            assert_eq!(4, value);
        } else {
            panic!("Result of parse numeric was not a numeric expression!");
        }
    }

    #[test]
    fn parse_list() {
        let mut tok_man = TokenManager::new("LIST(A,B,C)");

        let list: Box<IOList> = IOList::parse_from_tokens(&mut tok_man).unwrap();

        assert_eq!(list.items.len(),3);
        dbg!("{:#?}",&list);
    }

    #[test]
    fn test_parsing_identifier() {
        let mut tok_man = TokenManager::new("MIN(2,3);");
        let result = parse_identifier(&mut tok_man);
        if let Expr::Call {
            fn_name,
            args,
            _type,
        } = result
        {
            assert_eq!(fn_name, "MIN");
            assert_eq!(args.len(), 2);

            if let Expr::NumVal { value, _type } = args[0] {
                assert_eq!(value, 2);
            } else {
                panic!("args[0] was not type numval");
            }
            assert_eq!(Token::SEMICOLON, tok_man.current_token.unwrap());
        } else {
            panic!("Was not a call Expr");
        }
    }

    #[test]
    fn test_parse_parenthesis_expression() {
        let mut tok_man = TokenManager::new("(25665)");

        let result: Expr = parse_parenthesis_expression(&mut tok_man);

        if let Expr::NumVal { value, _type } = result {
            assert_eq!(25665, value);
        } else {
            panic!("NOT A NUMVAL!");
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_paren_bad_syntax() {
        let mut tok_man = TokenManager::new("(2 min(2,3))");

        let _result: Expr = parse_parenthesis_expression(&mut tok_man);
    }

    #[test]
    fn test_parse_primaries() {
        let mut tok_man = TokenManager::new("2; MIN(9,254); FLAG; (4);");

        let result = parse_expression(&mut tok_man);
        tok_man.next_token();
        if let Expr::NumVal { value, _type } = result {
            assert_eq!(value, 2);
        } else {
            panic!("Not a numval 2!");
        }

        let result = parse_expression(&mut tok_man);
        tok_man.next_token();
        if let Expr::Call { fn_name, .. } = result {
            assert_eq!("MIN", fn_name);
        } else {
            panic!("Not a MIN func!");
        }

        let result = parse_expression(&mut tok_man);
        tok_man.next_token();

        if let Expr::Variable {
            name,
            _type: Type::FixedDecimal,
        } = result
        {
            assert_eq!("FLAG", name);
        } else {
            panic!("Not a variable named FLAG!");
        }

        let result = parse_expression(&mut tok_man);
        tok_man.next_token();

        if let Expr::NumVal { value, _type } = result {
            assert_eq!(4, value);
        } else {
            panic!("Not a numval of value 4!");
        }
    }

    #[test]
    fn test_parsing_prototype() -> Result<(), String> {
        let mut token_manager = TokenManager::new("PROCEDURE(A,B,C);");
        let my_var: Prototype = parse_function_prototype(&mut token_manager, String::from("CALC"))?;

        assert_eq!(String::from("CALC"), my_var.fn_name);
        assert_eq!(my_var.args.len(), 3);

        let test_results = vec!["A", "B", "C"];
        let mut index = 0;
        for (_siz, arg) in my_var.args.iter().enumerate() {
            assert_eq!(*arg, String::from(test_results[index]));
            index += 1;
        }

        Ok(())
    }
    #[test]
    #[should_panic(expected = "OPEN_PAREN")]
    fn test_parsing_prototype_panic() {
        let mut token_manager = TokenManager::new("PROCEDURE A,B,C);");
        let my_var: Prototype =
            parse_function_prototype(&mut token_manager, String::from("CALC")).unwrap();

        assert_eq!(String::from("CALC"), my_var.fn_name);
        assert_eq!(my_var.args.len(), 3);

        let test_results = vec!["A", "B", "C"];
        let mut index = 0;
        for (_siz, arg) in my_var.args.iter().enumerate() {
            assert_eq!(*arg, String::from(test_results[index]));
            index += 1;
        }
    }

    #[test]
    fn test_parsing_if() {
        let mut token_manager = TokenManager::new("IF 1 THEN PUT 'One Was Set!'; END;");
        let res = parse_if(&mut token_manager);
        let end = parse_statement(&mut token_manager);
        dbg!(&res);
        if let Err(err_msg) = res {
            panic!("{}", err_msg);
        }

        dbg!(&end);
        if let Err(err_msg) = end {
            panic!("{}", err_msg);
        } else if let Ok(statement) = end {
            if let Command::END = statement.command {
            } else {
                panic!("EXPECTED Command::END, GOT {:?}", statement.command);
            }
        }
    }

    #[test]
    fn test_parsing_prototype_noargs() -> Result<(), String> {
        let mut token_manager = TokenManager::new("PROCEDURE();");
        let my_var: Prototype = parse_function_prototype(&mut token_manager, String::from("CALC"))?;

        assert_eq!(String::from("CALC"), my_var.fn_name);
        assert_eq!(my_var.args.len(), 0);
        Ok(())
    }
    #[test]
    fn test_parsing_function() {
        let mut token_manager = TokenManager::new("PROCEDURE (A,B,C); A + B + C; END;");

        let _my_function = parse_function(&mut token_manager, "TESTFUNC".to_string());
    }
    #[test]
    fn test_parsing_declare() -> Result<(), String> {
        let mut token_manager =
            TokenManager::new("DECLARE x FIXED; PUT 'HELLO'; PUT 'TWO'; PUT 'MESSAGE';");

        let decl = parse_declare(&mut token_manager)?;

        assert_eq!(decl.var_name, "x");
        assert_eq!(decl.attribute, Some(Type::FixedDecimal));

        //make sure declare sets up parsing for next line
        parse_statement(&mut token_manager)?;
        parse_statement(&mut token_manager)?;
        parse_statement(&mut token_manager)?;
        Ok(())
    }
}
