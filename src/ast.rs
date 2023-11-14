use std::string;

use crate::lexer;
use crate::types;
use crate::types::resolve_types;
use crate::types::BaseAttributes;
use crate::types::Type;
///Holds all definition for AST nodes

#[derive(Debug, Clone)]
pub enum Expr {
    Assignment {
        variable_name: String,
        value: Box<Expr>,
    },
    Binary {
        operator: lexer::Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Call {
        fn_name: String,
        args: Vec<Expr>,
        _type: types::Type,
    },
    NumVal {
        value: i32,
        _type: types::Type,
    },
    Char {
        value: String,
    },
    Variable {
        _type: types::Type,
        name: String,
    },
}

impl Expr {
    pub fn new_numval(value: i32) -> Expr {
        Expr::NumVal {
            value,
            _type: Type::FixedDecimal,
        }
    }
    pub fn get_type(&self) -> types::Type {
        match self {
            Expr::Variable {
                ref _type,
                ref name,
            } => return *_type,
            Expr::NumVal {
                ref _type,
                ref value,
            } => return *_type,
            Expr::Call {
                ref _type,
                ref args,
                ref fn_name,
            } => return *_type,
            Expr::Binary {
                ref operator,
                ref left,
                ref right,
            } => resolve_types(&left.get_type(), &right.get_type()).unwrap(),
            Expr::Assignment {
                ref variable_name,
                ref value,
            } => Type::Void,
            Expr::Char { value } => Type::Char(value.len() as u32),
        }
    }
}

///Represents a function prototype
#[derive(Debug, Clone)]
pub struct Prototype {
    pub fn_name: String,
    pub args: Vec<String>, // the names of the arguments - used inside of the function itself.
    pub source_loc: SourceLocation,
}
///Represents a function prototype
#[derive(Debug, Clone)]
pub struct PrototypeArgument {
    pub name: String,
    pub _type: Type,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line_number: u32,
    pub column_number: u32,
}

impl Default for SourceLocation {
    fn default() -> Self {
        SourceLocation {
            line_number: 0,
            column_number: 0,
        }
    }
}

///Represents a user-deined function.
#[derive(Debug, Clone)]
pub struct Function {
    pub prototype: Prototype,
    pub body_statements: Vec<Statement>,
    pub return_value: Option<Expr>,
    pub return_type: Type,
}

///Represents a "full-line" of execution, terminated by a semicolon.
#[derive(Debug, Clone)]
pub struct Statement {
    pub label: Option<String>, //The label attached to this statement
    pub command: Command,
}

///A "command" is the first keyword in a PL/1 statement, denoting
///what the entire statement's purpose is.
#[derive(Debug, Clone)]
pub enum Command {
    Empty, //represents a statement that is just a semicolon by itself.
    END,
    PUT(Put),
    IF(If),
    Declare(Declare),
    Assignment(Assignment),
    FunctionDec(Function),
    EXPR(Expr), //"EXPR" is not a command in pl/1 this just represents a expression statement.
    RETURN(Expr), // specifies the return value of a function
}

impl Command {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

//TODO: Think about making DO its own command
#[derive(Debug, Clone)]
pub struct If {
    ///The actual expression we are evaluating to be TRUE or FALSE
    pub conditional: Expr,
    pub then_statements: Vec<Statement>,
    pub else_statements: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub var_name: String,
    pub value: Expr,
}
#[derive(Debug, Clone)]
pub struct Declare {
    pub var_name: String,
    pub attribute: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Put {
    pub message_to_print: Expr,
}
