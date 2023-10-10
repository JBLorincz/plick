use crate::lexer;
use crate::types;
use crate::types::Type;
///Holds all definition for AST nodes


#[derive(Debug, Clone)]
pub enum Expr
{
    Assignment {
        variable_name: String,
        value: Box<Expr>
    },
    Binary {
        operator: lexer::Token,
        left: Box<Expr>,
        right: Box<Expr>
    },

    Call {
        fn_name: String,
        args: Vec<Expr>
    },
    NumVal {
        value: i32,
        _type: types::Type,
    },
    Variable {
        _type: types::Type,
        name: String 
    }

        
}

impl Expr
{
    pub fn new_numval(value: i32) -> Expr
    {
        Expr::NumVal { value, _type: Type::FixedDecimal }
    }
}

///Represents a function prototype
#[derive(Debug,Clone)]
pub struct Prototype {

        pub fn_name: String,
        pub args: Vec<String>, // the names of the arguments - used inside of the function itself.
        pub source_loc: SourceLocation
}


#[derive(Debug,Clone)]
pub struct SourceLocation
{
    pub line_number: u32,
    pub column_number: u32,
}

impl Default for SourceLocation
{
    fn default() -> Self {
        SourceLocation { line_number: 0, column_number: 0 }
    }
}

///Represents a user-deined function.
#[derive(Debug,Clone)]
pub struct Function {
    pub prototype: Prototype,
    pub body_statements: Vec<Statement>,
    pub return_value: Option<Expr>,
}

///Represents a "full-line" of execution, terminated by a semicolon.
#[derive(Debug,Clone)]
pub struct Statement {
    pub label: Option<String>, //The label attached to this statement
    pub command: Command, 
}

///A "command" is the first keyword in a PL/1 statement, denoting
///what the entire statement's purpose is.
#[derive(Debug,Clone)]
pub enum Command {
    Empty, //represents a statement that is just a semicolon by itself.
    END,
    PUT,
    IF(If),
    Assignment(Assignment),
    FunctionDec(Function), 
    EXPR(Expr),   //"EXPR" is not a command in pl/1 this just represents a expression statement.
    RETURN(Expr), // specifies the return value of a function
}

impl Command
{
    pub fn to_string(&self) -> String
    {
        format!("{:?}",self)
    }
}

//TODO: Think about making DO its own command
#[derive(Debug,Clone)]
pub struct If
{
    pub conditional: Expr,
    pub then_statements: Vec<Statement>,
    pub else_statements: Option<Vec<Statement>>
}


#[derive(Debug,Clone)]
pub struct Assignment
{
    pub var_name: String,
    pub value: Expr
}
