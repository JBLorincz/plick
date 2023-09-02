
pub enum Expr
{
    Binary {
        operator: char,
        left: Box<Expr>,
        right: Box<Expr>
    }
}
