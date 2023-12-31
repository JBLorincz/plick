use crate::{ast::SourceLocation, debugger::DebugController};
use log::{debug, error, info, log_enabled, trace, Level};

pub fn get_token_list(compilable_file: &str) -> Vec<Token> {
    let char_iter = compilable_file.chars();
    let token_iter = TokenIterator::new(char_iter);

    let result = token_iter.collect();
    trace!("token list loaded: {:?}", result);
    result
}

///Thus function returns the next token from the token iterator.
pub fn get_next_token(compilable_file: String) -> Option<Token> {
    let char_iter = compilable_file.chars();
    let mut token_iter = TokenIterator::new(char_iter);
    token_iter.next()
}

pub struct TokenManager<'a, 'b> {
    pub current_token: Option<Token>,
    token_iter: TokenIterator<'a, 'b>,
}

impl<'a, 'b> TokenManager<'a, 'b> {
    pub fn new(token_string: &str) -> TokenManager {
        let chars_over = TokenIterator::new(token_string.chars());

        let mut result = TokenManager {
            current_token: None,
            token_iter: chars_over,
        };

        result.next_token();

        result
    }

    pub fn attach_debugger(&mut self, dbg: &'a DebugController<'b>) {
        self.token_iter.dbg_info = Some(dbg);
    }

    ///Thus function returns the next token from the token iterator.
    pub fn next_token(&mut self) -> &Option<Token> {
        self.current_token = self.token_iter.next();
        trace!("Next Token: {:?}", &self.current_token);

        &self.current_token
    }

    pub fn get_line_and_column_numbers(&self) -> (u32, u32) {
        (self.token_iter.line_number, self.token_iter.column_number)
    }

    pub fn get_source_location(&self) -> SourceLocation {
        SourceLocation {
            line_number: self.token_iter.line_number,
            column_number: self.token_iter.column_number,
        }
    }
}

struct TokenIterator<'a, 'b> {
    char_iter: std::str::Chars<'a>,
    next_char: Option<char>,
    pub dbg_info: Option<&'a DebugController<'b>>,
    pub line_number: u32,
    pub column_number: u32,
}
impl<'a, 'b> TokenIterator<'a, 'b> {
    fn new(char_iter: std::str::Chars<'_>) -> TokenIterator {
        TokenIterator {
            char_iter,
            next_char: Some(' '), //this is a space character. Don't touch.
            line_number: 1,
            column_number: 0,
            dbg_info: None,
        }
    }
    fn get_next_char(&mut self) -> Option<char> {
        self.next_char = self.char_iter.next();
        if let Some('\n') = self.next_char {
            self.line_number += 1;
            self.column_number = 0;
        } else {
            self.column_number += 1;
        }

        if let Some(ref dbg) = self.dbg_info {
            *dbg.line_number.borrow_mut() = self.line_number;
            *dbg.column_number.borrow_mut() = self.column_number;
        }

        self.next_char
    }
    fn is_character_special(ch: char) -> bool {
        let special_chars = vec!['/', '(', ')', '\'', '+', '-', '*', ',', '=', ';', '<', '>'];

        if special_chars.contains(&ch) {
            true
        } else {
            false
        }
    }
    //next_char is '
    fn process_string(&mut self, mut current_word_buffer: String) -> String {
        self.get_next_char(); //skip over the first tick
        while let Some(char) = self.next_char {
            match char {
                '\'' => break,
                ch => current_word_buffer.push(ch),
            }
            self.get_next_char();
        }
        self.get_next_char();
        current_word_buffer
    }

    //current char is *
    fn process_comment(&mut self) -> Option<()> {
        let mut found_second_star = false;
        loop {
            let current_char = self.get_next_char()?;
            if current_char == '*' {
                found_second_star = true;
            } else if current_char == '/' && found_second_star {
                self.get_next_char();
                break;
            }
        }
        Some(())
    }

    fn process_mult(&mut self) -> Option<Token> {
        let current_char = self.get_next_char()?;

        if current_char == '*' {
            self.get_next_char()?;
            return Some(Token::EXPONENT);
        }
        Some(Token::MULTIPLY)
    }
}
impl Iterator for TokenIterator<'_, '_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.next_char {
            return None;
        }
        let mut current_word_buffer = String::new();
        while let Some(current_character) = self.next_char {
            if current_word_buffer.is_empty() && current_character.is_whitespace() {
                self.get_next_char();
                continue;
            }
            let is_special = TokenIterator::is_character_special(current_character);

            if is_special && !current_word_buffer.is_empty() {
                return convert_string_to_token(&current_word_buffer);
            } else if is_special && current_character == '\'' {
                current_word_buffer = self.process_string(current_word_buffer);
                return Some(Token::STRING(current_word_buffer));
            } else if is_special && current_character == '/' {
                trace!("/ character found in lexing");

                let next_lext_char = self.get_next_char();
                let ch = next_lext_char?;

                if ch == '*' {
                    self.process_comment()?;
                    continue;
                } else {
                    return Some(Token::DIVIDE);
                }
            } else if is_special && current_character == '*' {
                return Some(self.process_mult().unwrap());
            }

            //we have skipped over all the whitespace and are now building are buffer.
            if !current_character.is_whitespace() && !is_special {
                current_word_buffer.push(current_character);
            } else if is_special && current_word_buffer.len() == 0 {
                current_word_buffer.push(current_character);
                self.get_next_char();
                break;
            } else if is_special {
                break; //we dont need to get next char because itll be handled next iteration
            } else
            //if the current character is whitespace
            {
                self.get_next_char();
                break;
            }

            if current_character == ':' {
                self.get_next_char();
                current_word_buffer.pop();
                return Some(Token::LABEL(current_word_buffer));
            }

            self.get_next_char();
        }

        convert_string_to_token(&current_word_buffer)
    }
}

pub fn convert_string_to_token(input: &str) -> Option<Token> {
    if let Ok(number) = input.parse() {
        return Some(Token::NumVal(number));
    }
    if input.len() == 0 {
        return None;
    }
    Some(match input.to_uppercase().as_str() {
        "PROCEDURE" | "PROC" => Token::PROCEDURE,
        ";" => Token::SEMICOLON,
        "," => Token::COMMA,
        "**" => Token::EXPONENT,
        "*" => Token::MULTIPLY,
        "(" => Token::OPEN_PAREN,
        ")" => Token::CLOSED_PAREN,
        "<" => Token::LESS_THAN,
        ">" => Token::GREATER_THAN,
        //"," => Token::COMMA,
        "/" => Token::DIVIDE,
        "+" => Token::PLUS,
        "-" => Token::MINUS,
        "IF" => Token::IF,
        "ELSE" => Token::ELSE,
        "THEN" => Token::THEN,
        "DO" => Token::DO,
        "FIXED" => Token::FIXED,
        "FLOAT" => Token::FLOAT,
        "=" => Token::EQ,
        "PUT" => Token::PUT,
        "GET" => Token::GET,
        "RETURN" | "RET" => Token::RETURN,
        "DATA" => Token::DATA,
        "END" => Token::END,
        "WHILE" => Token::WHILE,
        "LIST" => Token::LIST,
        "SKIP" => Token::SKIP,
        "GO" => Token::GO,
        "DECLARE" | "DCL" => Token::DECLARE,
        "CHARACTER" | "CHAR" => Token::CHARACTER,
        "OPTIONS" => Token::OPTIONS,
        "AND" | "&" => Token::AND,
        "NOT" => Token::NOT,
        _ => Token::Identifier(input.to_owned()),
    })
}

#[derive(Debug, PartialEq, Clone)]
#[allow(non_camel_case_types)]
pub enum Token {
    //EOF,
    IF,
    THEN,
    ELSE,
    RETURN,
    OPEN_PAREN,
    CLOSED_PAREN,
    PROCEDURE, // the procedure or proc token
    DECLARE,
    DO,
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE,
    EXPONENT,
    WHILE,
    END,
    PUT,
    LESS_THAN,
    GREATER_THAN,
    LABEL(String),
    SKIP,
    STRING(String),
    EQ,
    SEMICOLON,
    FIXED,
    FLOAT,
    COMMA,
    DATA,
    GET,
    GO,
    OPTIONS,
    AND,
    NOT,
    LIST,
    CHARACTER,
    NumVal(f64),        // integer
    Identifier(String), //an identifier / variable name
}

impl Token {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

mod tests {
    use std::fs;

    use crate::initialize_test_logger;

    use super::Token::*;
    use super::*;

    #[test]
    fn basic_parse() {
        let input = "FLAG = 0;";

        let output: Vec<Token> = vec![
            Token::Identifier("FLAG".to_string()),
            Token::EQ,
            Token::NumVal(0.0),
            Token::SEMICOLON,
        ];

        assert_eq!(output, get_token_list(input));
    }
    #[test]
    fn lex_touching_multiply() {
        initialize_test_logger();
        let input = "5*5;";
        let token_list: Vec<Token> = vec![
            Token::NumVal(5.0),
            Token::MULTIPLY,
            Token::NumVal(5.0),
            Token::SEMICOLON,
        ];
        assert_eq!(get_token_list(input), token_list);
    }
    #[test]
    fn hello_world_parse() {
        let input = fs::read_to_string("./test_pli_files/hello_world.pli").unwrap();
        let output: Vec<Token> = vec![
            LABEL(String::from("HELLO")),
            PROCEDURE,
            OPTIONS,
            OPEN_PAREN,
            Identifier(String::from("MAIN")),
            CLOSED_PAREN,
            SEMICOLON,
            Identifier(String::from("FLAG")),
            EQ,
            NumVal(0.0),
            SEMICOLON,
            LABEL(String::from("LOOP")),
            DO,
            WHILE,
            OPEN_PAREN,
            Identifier(String::from("FLAG")),
            EQ,
            NumVal(0.0),
            CLOSED_PAREN,
            SEMICOLON,
            PUT,
            SKIP,
            DATA,
            OPEN_PAREN,
            STRING(String::from("HELLO WORLD!")),
            CLOSED_PAREN,
            SEMICOLON,
            END,
            Identifier(String::from("LOOP")),
            SEMICOLON,
            END,
            Identifier(String::from("HELLO")),
            SEMICOLON,
        ];

        assert_eq!(output, get_token_list(&input));
    }
    #[test]
    fn hello_world_with_commentparse() {
        let input = format!(
            "{}{}",
            "/*This is a comment!*/",
            fs::read_to_string("./test_pli_files/hello_world.pli").unwrap()
        );
        let output: Vec<Token> = vec![
            LABEL(String::from("HELLO")),
            PROCEDURE,
            OPTIONS,
            OPEN_PAREN,
            Identifier(String::from("MAIN")),
            CLOSED_PAREN,
            SEMICOLON,
            Identifier(String::from("FLAG")),
            EQ,
            NumVal(0.0),
            SEMICOLON,
            LABEL(String::from("LOOP")),
            DO,
            WHILE,
            OPEN_PAREN,
            Identifier(String::from("FLAG")),
            EQ,
            NumVal(0.0),
            CLOSED_PAREN,
            SEMICOLON,
            PUT,
            SKIP,
            DATA,
            OPEN_PAREN,
            STRING(String::from("HELLO WORLD!")),
            CLOSED_PAREN,
            SEMICOLON,
            END,
            Identifier(String::from("LOOP")),
            SEMICOLON,
            END,
            Identifier(String::from("HELLO")),
            SEMICOLON,
        ];

        assert_eq!(output, get_token_list(&input));
    }

    #[test]
    fn eq_with_statement() {
        let input = "/*A program to assign a variable*/A = 4;";
        let output = vec![Identifier(String::from("A")), EQ, NumVal(4.0), SEMICOLON];

        assert_eq!(get_token_list(input), output);
    }

    #[test]
    fn test_lexing_function_call() {
        let input = "MIN(2,3);";
        let output = vec![
            Identifier(String::from("MIN")),
            OPEN_PAREN,
            NumVal(2.0),
            COMMA,
            NumVal(3.0),
            CLOSED_PAREN,
            SEMICOLON,
        ];

        assert_eq!(get_token_list(input), output);
    }

    #[test]
    fn test_binary_operator_lexing() {
        let input = "+ / / + + - *  ";
        let output = vec![PLUS, DIVIDE, DIVIDE, PLUS, PLUS, MINUS, MULTIPLY];

        assert_eq!(get_token_list(input), output);
    }
    #[test]
    fn test_touching_binary_lexing() {
        let input = "2+2";
        let output = vec![NumVal(2.0), PLUS, NumVal(2.0)];

        assert_eq!(get_token_list(input), output);
    }
}
