use crate::token::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },
    Return(Expression),
    Expression(Expression),
    Block(Vec<Statement>),
    If {
        condition: Expression,
        consequence: Box<Statement>,         // Should be a Block
        alternative: Option<Box<Statement>>, // Should be a Block
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Statement>, // Should be a Block
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),
    Integer(i64),
    String(String),
    #[allow(dead_code)]
    Boolean(bool), // For true/false usually, strictly speaking lexer didn't have bool literals yet, maybe will add later
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Call {
        function: Box<Expression>, // Identifier
        arguments: Vec<Expression>,
    },
}
