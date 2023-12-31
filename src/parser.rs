use std::error::Error;

use crate::ast;
use crate::ast::*;
use crate::error;
use crate::error::errors::ParseError;
use crate::types::Type;
use crate::{
    codegen::codegen::CodeGenable,
    error::get_error,
    lexer::{self, Token},
    types::infer_pli_type_via_name,
};

use log::debug;
use log::trace;

///Helper function used to advance the token manager,
///ensuring the token that was 'eaten' was the expected
///token. Otherwise, returns an error.
pub fn parse_token(
    token_manager: &mut lexer::TokenManager,
    token_to_check_for: Token,
) -> Result<(), ParseError> {
    trace!("Calling parse_token looking for {:?}", &token_to_check_for);
    let current_tok_copy = token_manager.current_token.clone();
    if let None = current_tok_copy {
        let message = get_error(&["2"]);
        return Err(ParseError { message });
    }

    let current_tok_copy = current_tok_copy.unwrap();

    if std::mem::discriminant(&token_to_check_for) == std::mem::discriminant(&current_tok_copy) {
        trace!("Found it!");
        token_manager.next_token();
        Ok(())
    } else {
        let message = get_error(&[
            "1",
            &token_to_check_for.to_string(),
            &current_tok_copy.to_string(),
            format!("{}", token_manager.get_source_location()).as_str(),
        ]);
        Err(ParseError { message })
    }
}

pub fn parse_constant_numeric<'a>(
    token_manager: &'a mut lexer::TokenManager,
) -> Result<Expr, ParseError> {
    log::debug!("before trying to find minus!");
    let minus_result = parse_token(token_manager, Token::MINUS);
    log::debug!("after trying to find minus!");

    let mut is_negative = 1.0;
    match minus_result {
        Ok(thing) => {
            is_negative = -1.0;
        }
        _ => (),
    };

    if let Some(Token::NumVal(value)) = token_manager.current_token {
        log::debug!("before numval next");
        token_manager.next_token();
        log::debug!("after numval next");

        Ok(Expr::NumVal {
            value: value * is_negative,
            _type: Type::FixedDecimal,
        })
    } else {
        Err(ParseError {
            message: "Failed to parse numeric!".to_owned(),
        })
    }
}

pub fn parse_if<'a>(token_manager: &'a mut lexer::TokenManager) -> Result<If, ParseError> {
    //current token is IF

    let _if = parse_token(token_manager, Token::IF)?;
    let conditional = parse_expression(token_manager)?;
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

    Ok(If {
        conditional,
        then_statements,
        else_statements,
    })
}

pub fn parse_declare(token_manager: &mut lexer::TokenManager) -> Result<Declare, ParseError> {
    log::info!("Parsing the declare!");
    token_manager.next_token();
    let new_variable_name;
    let mut variable_type = Type::FixedDecimal;

    if let Some(Token::Identifier(ref name)) = token_manager.current_token {
        new_variable_name = name.clone();
    } else {
        let source_loc = token_manager.get_source_location().to_string();
        let message = get_error(&["1", "an identifier", "a non-identifier", &source_loc]);
        return Err(ParseError { message });
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
        Some(Token::CHARACTER) => {
            parse_token(token_manager, Token::CHARACTER)?;

            parse_token(token_manager, Token::OPEN_PAREN)?;
            let numval = parse_constant_numeric(token_manager)?;
            parse_token(token_manager, Token::CLOSED_PAREN)?;

            let string_size = match numval {
                Expr::NumVal { value, _type: _ } => value,
                other => panic!("Expected numval, received {:#?}", other),
            };

            if string_size <= 0.0 {
                panic!("character can't have a size below zero!");
            }
            variable_type = Type::Char(string_size as u32);
        }
        Some(Token::SEMICOLON) => variable_type = variable_type,
        ref other => {
            let message = format!("Could not parse declare statement {:#?}", other);
            return Err(ParseError { message });
        }
    };

    log::info!("Finish parsing declare");
    Ok(Declare {
        var_name: new_variable_name,
        attribute: Some(variable_type),
    })
}
//current token is the semicolon AFTER do
pub fn parse_do_block(
    token_manager: &mut lexer::TokenManager,
) -> Result<Vec<Statement>, ParseError> {
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
pub fn parse_identifier<'a>(
    token_manager: &'a mut lexer::TokenManager,
) -> Result<Expr, ParseError> {
    trace!("Parsing Identifier");
    let identifier_string: String;
    if let Some(Token::Identifier(ref val)) = token_manager.current_token {
        identifier_string = val.clone();
        trace!("The identifier string is: {}", identifier_string);
    } else {
        log::error!("Found token as {:#?}", token_manager.current_token);
        let message = "failed to parse identifier!".to_string();
        return Err(ParseError { message });
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
                let parsed_arg: Expr = parse_expression(token_manager)?;

                args_list.push(parsed_arg);

                expecting_comma = true;
                trace!("turned expecting comma on!");
            } else {
                panic!("{:?}", token_manager.current_token);
            }
        }
        return Ok(Expr::Call {
            fn_name: identifier_string,
            args: args_list,
            _type: Type::TBD,
        });
    } else {
        return Ok(Expr::Variable {
            name: identifier_string,
            _type: Type::FixedDecimal,
        });
    }
}

///the current token is a '(' / Token::OPEN_PAREN
pub fn parse_parenthesis_expression(
    token_manager: &mut lexer::TokenManager,
) -> Result<Expr, ParseError> {
    log::trace!("Calling parse_paren_expression");
    parse_token(token_manager, Token::OPEN_PAREN)?;

    let result: Expr = parse_expression(token_manager)?;

    parse_token(token_manager, Token::CLOSED_PAREN)?;

    log::trace!("Exiting parse_paren_expression");
    return Ok(result);
}

pub fn parse_expression<'a>(
    token_manager: &'a mut lexer::TokenManager,
) -> Result<Expr, ParseError> {
    log::trace!("starting a parse_expression() ");

    let left_handed_side = parse_primary_expression(token_manager)?;
    let lhs_expr = left_handed_side.expression.clone();
    let was_lhs_in_paren = left_handed_side.in_parenthesis;
    log::trace!("left side: {:#?}", lhs_expr);

    if let Expr::Variable { ref name, _type } = lhs_expr {
        if let Some(Token::EQ) = token_manager.current_token {
            log::trace!("EQUAL expression");
            parse_token(token_manager, Token::EQ).expect("always true"); // eat the equal token
            let expression_value = parse_expression(token_manager);
            //return Expr::Assignment(name, expression_value);
            return Ok(Expr::Assignment {
                variable_name: name.clone(),
                value: Box::new(expression_value.unwrap()),
            });
        }
    } else if let Expr::Char { value } = lhs_expr.clone() {
        log::trace!("parsing a CHAR expression");
        token_manager.next_token(); //eat the string token
        return Ok(lhs_expr);
    }
    match token_manager.current_token {
        Some(Token::PLUS)
        | Some(Token::MINUS)
        | Some(Token::DIVIDE)
        | Some(Token::AND)
        | Some(Token::EXPONENT)
        | Some(Token::MULTIPLY)
        | Some(Token::GREATER_THAN)
        | Some(Token::LESS_THAN) => {
            let token_precedence =
                get_binary_operator_precedence(token_manager.current_token.as_ref().unwrap());

            log::trace!(
                "building recursive binary tree!: LHS = {:#?} CURRENT_TOKEN: {:#?}",
                lhs_expr,
                token_manager.current_token
            );
            build_recursive_binary_tree(token_manager, lhs_expr, token_precedence, was_lhs_in_paren)
        }
        _ => Ok(lhs_expr),
    }
}

pub fn build_recursive_binary_tree(
    token_manager: &mut lexer::TokenManager,
    lhs: Expr,
    precendence: i32,
    is_lhs_in_paren: bool,
) -> Result<Expr, ParseError> {
    //LHS has to be a binary node.
    let operator_token: Token = token_manager.current_token.as_ref().unwrap().clone();
    trace!("Operator in this: {:#?}", operator_token);
    let tok = token_manager.next_token();
    trace!("Skipped {:#?}", tok);
    //if the current precedence is GREATER than the lhs precendence,
    if let Expr::Binary {
        operator,
        left,
        right,
    } = lhs
    {
        if precendence > get_binary_operator_precedence(&operator) && !is_lhs_in_paren {
            // meaning we have to make the RHS side of the LHS the LHS of our
            // new RHS
            let new_rhs = parse_expression(token_manager)?;

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
            return Ok(Expr::Binary {
                operator,
                left,
                right: Box::new(inner_binary),
            });
        } else {
            let rhs_expression = parse_expression(token_manager)?;
            Ok(Expr::Binary {
                operator: operator_token,
                left: Box::new(Expr::Binary {
                    operator,
                    left,
                    right,
                }),
                right: Box::new(rhs_expression),
            })
        }
    } else {
        let right = parse_expression(token_manager)?;
        return Ok(Expr::Binary {
            operator: operator_token,
            left: Box::new(lhs),
            right: Box::new(right),
        });
    }
}

pub fn parse_primary_expression(
    token_manager: &mut lexer::TokenManager,
) -> Result<PrimaryExpressionParseResult, ParseError> {
    let current_token = token_manager.current_token.as_ref().unwrap().clone();
    log::debug!(
        "parsing primary expression! Token found: {:#?}",
        current_token
    );

    let mut was_in_parenthesis = false;
    if is_token_infix_operator(current_token.clone()) {
        token_manager.next_token();
        let result_expr: Expr = Expr::Infix {
            operator: current_token.clone(),
            operand: Box::new(parse_primary_expression(token_manager).unwrap().expression),
        };

        return Ok(PrimaryExpressionParseResult::new(
            result_expr,
            was_in_parenthesis,
        ));
    }

    let result_expr = match token_manager.current_token.as_ref().unwrap() {
        Token::OPEN_PAREN => {
            was_in_parenthesis = true;
            parse_parenthesis_expression(token_manager)?
        }
        Token::Identifier(the_identifier) => {
            log::trace!("running identifier code for {}", the_identifier);
            if the_identifier.ends_with("E") {
                let mut the_identifier = the_identifier.clone();
                the_identifier.pop();

                if let Ok(_float) = the_identifier.parse::<f64>() {
                    log::trace!("PARSING FLOAT CONST {}", the_identifier);
                    parse_float_const(token_manager)?
                } else {
                    log::trace!("NOT PARSING FLOAT CONST {}", the_identifier);
                    parse_identifier(token_manager)?
                }
            } else {
                log::trace!("The current token is: {:#?}", token_manager.current_token);
                parse_identifier(token_manager)?
            }
        }
        Token::NumVal(_) => parse_constant_numeric(token_manager)?,
        Token::NOT => parse_constant_numeric(token_manager)?,
        Token::STRING(value) => Expr::Char {
            value: value.clone(),
        },
        other => {
            return Err(ParseError {
                message: format!("Can't parse top level token type {:?}", other),
            });
        }
    };

    Ok(PrimaryExpressionParseResult::new(
        result_expr,
        was_in_parenthesis,
    ))
}
fn is_token_infix_operator(current_token: Token) -> bool {
    match current_token {
        Token::NOT => true,
        Token::MINUS => true,
        _ => false,
    }
}

pub struct PrimaryExpressionParseResult {
    pub expression: Expr,
    pub in_parenthesis: bool,
}
impl PrimaryExpressionParseResult {
    fn new(expression: Expr, in_parenthesis: bool) -> Self {
        PrimaryExpressionParseResult {
            expression,
            in_parenthesis,
        }
    }
}

pub fn get_binary_operator_precedence(token: &lexer::Token) -> i32 {
    match token {
        Token::PLUS => 20,
        Token::MINUS => 20,
        Token::MULTIPLY => 40,
        Token::EXPONENT => 80,
        Token::AND => 4,
        Token::DIVIDE => 40,
        Token::LESS_THAN => 8,
        Token::GREATER_THAN => 8,
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
) -> Result<Prototype, ParseError> {
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
                let message = format!(
                    "Expected an expression, found a comma at {}:{}",
                    source_loc.line_number, source_loc.column_number
                )
                .to_string();

                return Err(ParseError { message });
            }
        } else if let Some(ref token) = token_manager.current_token {
            //parse as expression
            trace!("Found a token called {:#?}", *token);
            let parsed_arg: Expr = parse_expression(token_manager)?;

            let arg_name: String;
            if let Expr::Variable { name, _type } = parsed_arg {
                arg_name = name.clone();
            } else {
                let message =
                    format!("Expected variable in function prototype, found _").to_string();
                return Err(ParseError { message });
            }

            args_list.push(arg_name);

            expecting_comma = true;
            trace!("turned expecting comma on!");
        } else {
            let message = format!("{:?}", token_manager.current_token).to_string();
            return Err(ParseError { message });
        }
    }

    Ok(Prototype {
        fn_name: label_name,
        args: args_list,
        source_loc,
    })
}

pub fn parse_arguments_in_parens(
    token_manager: &mut lexer::TokenManager,
) -> Result<Vec<Expr>, ParseError> {
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
                let message = format!("Expected an expression, found a comma at {}", source_loc);

                return Err(ParseError { message });
            }
        } else if let Some(ref token) = token_manager.current_token {
            //
            //START OF CALLBACK
            trace!("Found a token called {:#?}", *token);
            let parsed_arg: Expr = parse_expression(token_manager)?;

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
            let message = format!("{:?}", token_manager.current_token).to_string();
            return Err(ParseError { message });
        }
    }
    Ok(args_list)
}

pub fn parse_put(token_manager: &mut lexer::TokenManager) -> Result<Put, ParseError> {
    parse_token(token_manager, Token::PUT)?;

    let messages_to_print = *IOList::parse_from_tokens(token_manager)?;

    Ok(Put { messages_to_print })
}

pub fn parse_function(
    token_manager: &mut lexer::TokenManager,
    label_name: String,
) -> Result<Function, ParseError> {
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
                let message = get_error(&["6"]);
                return Err(ParseError { message });
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
pub fn parse_statement(token_manager: &mut lexer::TokenManager) -> Result<Statement, ParseError> {
    debug!("Calling parse statement!");
    let mut command: Command = Command::Empty;
    let mut label: Option<String> = None;
    while let Some(ref token) = token_manager.current_token {
        log::debug!("Token at beginning of statement: {:#?}", &token);
        match token {
            Token::SEMICOLON => {
                parse_token(token_manager, Token::SEMICOLON)?;
                break; //statement is now over since semicolon has been found.
            }
            Token::LABEL(label_string) => {
                if let Some(other_label) = label {
                    let message = get_error(&["3", &label_string, &other_label]);
                    return Err(ParseError { message });
                }
                match command {
                    Command::Empty => (),
                    other_command => {
                        let message = get_error(&["4", "LABEL", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }

                label = Some(label_string.to_string()); //store the fact something
                token_manager.next_token(); //is labelled
            }
            Token::PUT => {
                match command {
                    Command::Empty => command = Command::PUT(parse_put(token_manager)?),
                    other_command => {
                        let message = get_error(&["4", "PUT", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }
                parse_token(token_manager, Token::SEMICOLON)?;
                break;
            }
            Token::GO => {
                match command {
                    Command::Empty => {
                        command = Command::GO(*(ast::Go::parse_from_tokens(token_manager).unwrap()))
                    }
                    other_command => {
                        let message = get_error(&["4", "PUT", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }
                parse_token(token_manager, Token::SEMICOLON)?;
                break;
            }
            //            Token::PUT => {
            //                match command {
            //                    Command::Empty => command = Command::PUT(parse_put(token_manager)?),
            //                    other_command => {
            //                        let message = get_error(&["4", "PUT", &other_command.to_string()]);
            //                        return Err(ParseError { message });
            //                    }
            //                }
            //
            //                parse_token(token_manager, Token::SEMICOLON)?;
            //                break;
            //            }
            Token::GET => {
                match command {
                    Command::Empty => {
                        parse_token(token_manager, Token::GET)?;
                        let list_to_get = *IOList::parse_from_tokens(token_manager).unwrap();
                        command = Command::GET(Get { list_to_get })
                    }
                    other_command => {
                        let message = get_error(&["4", "PUT", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }

                parse_token(token_manager, Token::SEMICOLON)?;
                break;
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
                        let message = get_error(&["4", "END", &other_command.to_string()]);
                        return Err(ParseError { message });
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
                        let message = get_error(&["4", "DECLARE", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }
                parse_token(token_manager, Token::SEMICOLON)?;
                break;
            }
            Token::IF => {
                let if_statement = parse_if(token_manager)?;
                match command {
                    Command::Empty => command = Command::IF(if_statement),
                    other_command => {
                        let message = get_error(&["4", "IF", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }

                break;
            }
            Token::RETURN => {
                token_manager.next_token();
                let token_after_return = &token_manager.current_token.clone().unwrap().clone();
                if let Token::SEMICOLON = token_after_return {
                    match command {
                        Command::Empty => command = Command::RETURN(Expr::new_numval(-1.0)),
                        other_command => {
                            let message = get_error(&["4", "RETURN", &other_command.to_string()]);
                            return Err(ParseError { message });
                        }
                    }
                    token_manager.next_token();
                    return Ok(Statement { label, command });
                }
                match command {
                    Command::Empty => command = Command::RETURN(parse_expression(token_manager)?),
                    other_command => {
                        let message = get_error(&["4", "RETURN", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }
                trace!("Eating Token {:#?}", token_manager.current_token);
                token_manager.next_token();
                break;
            }

            _ => {
                let expr = parse_expression(token_manager)?;
                log::debug!("Parsing an expression statement: {:#?}", &expr);
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
                log::debug!("Assigning command {:#?}", &command);
                match command {
                    Command::Empty => command = new_command,
                    other_command => {
                        let message = get_error(&["4", "expression", &other_command.to_string()]);
                        return Err(ParseError { message });
                    }
                }
            }
        }
    } // end while loop

    Ok(Statement { label, command })
}

pub fn parse_float_const(token_manager: &mut lexer::TokenManager) -> Result<Expr, ParseError> {
    let identifier = token_manager.current_token.clone().unwrap();
    let final_float: f64;
    if let Token::Identifier(mystr) = identifier {
        log::trace!("mystr is: {}", mystr);
        let mut mystr = mystr.clone();

        if !mystr.ends_with("E") {
            return Err(ParseError {
                message: "Not a float value!".to_owned(),
            });
        }
        mystr.pop();
        log::trace!("mystr is: {}", mystr);
        let mut float_constant: f64 = mystr.parse().unwrap();

        let sign: Token = token_manager.next_token().clone().unwrap();
        let mut exp_sign = 1;
        match sign {
            Token::MINUS => {
                exp_sign = -1;
            }
            Token::PLUS => {}
            _ => {
                return Err(ParseError {
                    message: "Error parsing float".to_owned(),
                });
            }
        };

        token_manager.next_token();
        let exponent_expr = parse_constant_numeric(token_manager)?;
        if let Expr::NumVal { value, _type } = exponent_expr {
            let exponent = value as i32 * exp_sign;
            final_float = float_constant.powi(exponent);
        } else {
            return Err(ParseError {
                message: "expected identifier".to_owned(),
            });
        }
    } else {
        return Err(ParseError {
            message: "expected identifier".to_owned(),
        });
    }

    Ok(Expr::NumVal {
        value: final_float,
        _type: Type::Float,
    })
}

///parses the beginning of a PL/1 Program.
///They look like this:
/// ANY_LABEL_HERE : PROCDURE OPTIONS (MAIN);
pub fn parse_opening(token_manager: &mut lexer::TokenManager) -> Result<(), ParseError> {
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
    fn parse_from_tokens(token_manager: &mut lexer::TokenManager) -> Result<Box<Self>, ParseError>;
}

impl Parseable for ast::IOList {
    fn parse_from_tokens(token_manager: &mut lexer::TokenManager) -> Result<Box<Self>, ParseError> {
        parse_token(token_manager, Token::LIST)?;

        let items: Vec<Expr> = parse_arguments_in_parens(token_manager)?;

        Ok(Box::new(IOList { items }))
    }
}

impl Parseable for ast::Go {
    fn parse_from_tokens(token_manager: &mut lexer::TokenManager) -> Result<Box<Self>, ParseError> {
        parse_token(token_manager, Token::GO)?;

        //TODO: Implement a "Parse Raw Word To String" function
        let exp = parse_identifier(token_manager)?;
        if let Expr::Variable {
            _type: _,
            name: nam,
        } = exp
        {
            Ok(Box::new(Go {
                label_to_go_to: nam,
            }))
        } else {
            panic!(
                "Expected some string 'identifier' type after GO command, found something else!"
            );
        }
    }
}

mod tests {

    use crate::{initialize_test_logger, lexer::TokenManager};

    use super::*;

    #[test]
    fn construct_binary() {
        let lhs = Expr::new_numval(4.0);
        let rhs = Expr::new_numval(6.0);

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
    fn test_parsing_numeric() -> Result<(), ParseError> {
        let mut tok_man = TokenManager::new("4");

        let result: Expr = parse_constant_numeric(&mut tok_man)?;

        if let Expr::NumVal { value, _type } = result {
            assert_eq!(4.0, value);

            Ok(())
        } else {
            panic!("Result of parse numeric was not a numeric expression!");
        }
    }
    #[test]
    fn test_parsing_float_numeric() -> Result<(), ParseError> {
        initialize_test_logger();
        let mut tok_man = TokenManager::new("4.0E+01");

        let result: Expr = parse_float_const(&mut tok_man)?;

        if let Expr::NumVal { value, _type } = result {
            assert_eq!(4.0, value);
            assert_eq!(Type::Float, _type);

            Ok(())
        } else {
            panic!("Result of parse numeric was not a numeric expression!");
        }
    }

    #[test]
    fn parse_list() {
        let mut tok_man = TokenManager::new("LIST(A,B,C)");

        let list: Box<IOList> = IOList::parse_from_tokens(&mut tok_man).unwrap();

        assert_eq!(list.items.len(), 3);
        dbg!("{:#?}", &list);
    }

    #[test]
    fn parse_get() {
        let mut tok_man = TokenManager::new("GET LIST(A,B,C);");

        let statement = parse_statement(&mut tok_man).unwrap();

        match statement.command {
            Command::GET(list) => {
                dbg!(list);
            }
            other => panic!(
                "Expected GET LIST to parse into a GET command, but received a {:#?}",
                other
            ),
        };
    }

    #[test]
    fn test_parsing_identifier() -> Result<(), ParseError> {
        let mut tok_man = TokenManager::new("MIN(2,3);");
        let result = parse_identifier(&mut tok_man)?;
        if let Expr::Call {
            fn_name,
            args,
            _type,
        } = result
        {
            assert_eq!(fn_name, "MIN");
            assert_eq!(args.len(), 2);

            if let Expr::NumVal { value, _type } = args[0] {
                assert_eq!(value, 2.0);
            } else {
                panic!("args[0] was not type numval");
            }
            assert_eq!(Token::SEMICOLON, tok_man.current_token.unwrap());
            Ok(())
        } else {
            panic!("Was not a call Expr");
        }
    }

    #[test]
    fn test_parse_parenthesis_expression() -> Result<(), ParseError> {
        initialize_test_logger();
        let mut tok_man = TokenManager::new("(25665)");

        let result: Expr = parse_parenthesis_expression(&mut tok_man)?;

        if let Expr::NumVal { value, _type } = result {
            assert_eq!(25665.0, value);
            Ok(())
        } else {
            panic!("NOT A NUMVAL!");
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_paren_bad_syntax() {
        let mut tok_man = TokenManager::new("(2 min(2,3))");

        let _result: Expr = parse_parenthesis_expression(&mut tok_man).unwrap();
    }

    #[test]
    fn test_parse_primaries() -> Result<(), ParseError> {
        let mut tok_man = TokenManager::new("2; MIN(9,254); FLAG; (4);");

        let result = parse_expression(&mut tok_man)?;
        tok_man.next_token();
        if let Expr::NumVal { value, _type } = result {
            assert_eq!(value, 2.0);
        } else {
            panic!("Not a numval 2!");
        }

        let result = parse_expression(&mut tok_man)?;
        tok_man.next_token();
        if let Expr::Call { fn_name, .. } = result {
            assert_eq!("MIN", fn_name);
        } else {
            panic!("Not a MIN func!");
        }

        let result = parse_expression(&mut tok_man)?;
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

        let result = parse_expression(&mut tok_man)?;
        tok_man.next_token();

        if let Expr::NumVal { value, _type } = result {
            assert_eq!(4.0, value);

            Ok(())
        } else {
            panic!("Not a numval of value 4!");
        }
    }

    #[test]
    fn test_parsing_prototype() -> Result<(), ParseError> {
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
        let mut token_manager = TokenManager::new("IF 1 THEN PUT LIST('One Was Set!'); END;");
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
    fn test_parsing_prototype_noargs() -> Result<(), ParseError> {
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
    fn parse_binary_with_parenthesis() {
        initialize_test_logger();

        let mut token_manager = TokenManager::new("VARIABLE = (1+2+3) /4;");

        let _my_function = parse_statement(&mut token_manager);
    }
    #[test]
    fn test_parsing_declare() -> Result<(), ParseError> {
        let mut token_manager =
            TokenManager::new("DECLARE x FIXED; PUT LIST('HELLO', 'TWO', 'MESSAGE');");

        let decl = parse_declare(&mut token_manager)?;

        assert_eq!(decl.var_name, "x");
        assert_eq!(decl.attribute, Some(Type::FixedDecimal));

        //make sure declare sets up parsing for next line
        parse_statement(&mut token_manager)?;
        parse_statement(&mut token_manager)?;
        parse_statement(&mut token_manager)?;
        Ok(())
    }

    #[test]
    fn parsing_paren_and_divide() -> Result<(), ParseError> {
        let mut token_manager = TokenManager::new("( 1 + 1 + 1 + 1 + 1 ) / 5");

        let decl = parse_expression(&mut token_manager)?;

        match decl {
            Expr::Binary {
                operator,
                left,
                right,
            } => match operator {
                Token::DIVIDE => {}
                _ => {
                    panic!("Wrong OP")
                }
            },
            _ => {
                panic!("Not a Binary!");
            }
        };

        Ok(())
    }
}
