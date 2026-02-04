use crate::ast::{Expression, Program, Statement};
use crate::token::{Lexer, Token};

#[derive(PartialEq, PartialOrd, Debug, Copy, Clone)]
enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        // myFunction(X)
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Equal | Token::NotEqual => Precedence::Equals,
        Token::LessThan | Token::GreaterThan => Precedence::LessGreater,
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Asterisk | Token::Slash => Precedence::Product,
        Token::LParen => Precedence::Call,
        _ => Precedence::Lowest,
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
    pub errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            cur_token,
            peek_token,
            errors: vec![],
        }
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = vec![];

        while self.cur_token != Token::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Program { statements }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token {
            Token::Int => {
                // Heuristic: if it's "int fname() {", it's a function.
                // if it's "int x = 5;", it's a let/var decl.
                // Ideally we check the peek token properly.
                // Assuming "int <ident>"
                if let Token::Identifier(_) = self.peek_token {
                    // Peek one more to check if it's '(' (func) or '=' (var) or ';' (var)
                    // But our Lexer doesn't support peeking 2 ahead easily unless we modify it or consume tokens.
                    // Let's consume 'int'.

                    // Actually, let's just cheat a bit and say 'int' starts a Let statement for now,
                    // unless we see '(', then it's a function?
                    // In C, `int main()` vs `int x;`.
                    // A proper parser would handle types. Here, `int` is just a keyword starting a decl.

                    // Let's implement parse_let_statement which might also turn out to be a function if we are at top level?
                    // For simplicity, let's treat `Token::Int` as the start of a `Let` statement for now,
                    // and handle functions separately or detect them here.
                    self.parse_let_statement()
                } else {
                    None
                }
            }
            Token::Return => self.parse_return_statement(),
            Token::LBrace => Some(Statement::Block(self.parse_block_statement())),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        // match `int identifier`
        self.next_token(); // consume 'int' (or type keyword)

        let name = match &self.cur_token {
            Token::Identifier(n) => n.clone(),
            _ => return None,
        };

        if self.peek_token == Token::LParen {
            // It's a function definition! `int main() { ... }`
            return self.parse_function_statement(name);
        }

        if !self.expect_peek(Token::Assign) {
            // maybe `int x;`? Support uninitialized?
            // For now, require assignment.
            return None;
        }

        self.next_token(); // consume '='

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Some(Statement::Let { name, value })
    }

    fn parse_function_statement(&mut self, name: String) -> Option<Statement> {
        // cur_token is Identifier(name). peek is LParen.
        self.next_token(); // consume Identifier. Now cur is LParen.

        let params = self.parse_function_parameters()?;

        if !self.expect_peek(Token::LBrace) {
            return None;
        }

        let body = Statement::Block(self.parse_block_statement());

        Some(Statement::Function {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<String>> {
        let mut identifiers = vec![];

        if self.peek_token == Token::RParen {
            self.next_token();
            return Some(identifiers);
        }

        self.next_token();

        // Expect "int x" not just "x"
        // Simplification: assume parameters are just typed `int x`
        // We need to consume Type then Identifier.

        // Loop for params: type ident, type ident
        loop {
            match self.cur_token {
                Token::Int => {
                    self.next_token(); // consume type
                    match &self.cur_token {
                        Token::Identifier(ident) => identifiers.push(ident.clone()),
                        _ => return None,
                    }
                }
                _ => return None,
            }

            if self.peek_token == Token::Comma {
                self.next_token();
                self.next_token();
            } else {
                break;
            }
        }

        if !self.expect_peek(Token::RParen) {
            return None;
        }

        Some(identifiers)
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token(); // consume 'return'

        let return_value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Some(Statement::Return(return_value))
    }

    fn parse_block_statement(&mut self) -> Vec<Statement> {
        // cur_token is LBrace
        self.next_token();

        let mut statements = vec![];

        while self.cur_token != Token::RBrace && self.cur_token != Token::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        statements
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek(Token::LParen) {
            return None;
        }

        self.next_token(); // consume LParen
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(Token::RParen) {
            return None;
        }

        if !self.expect_peek(Token::LBrace) {
            return None;
        }

        let consequence = Box::new(Statement::Block(self.parse_block_statement()));
        let mut alternative = None;

        if self.peek_token == Token::Else {
            self.next_token();

            if !self.expect_peek(Token::LBrace) {
                return None;
            }
            alternative = Some(Box::new(Statement::Block(self.parse_block_statement())));
        }

        Some(Statement::If {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_while_statement(&mut self) -> Option<Statement> {
        if !self.expect_peek(Token::LParen) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(Token::RParen) {
            return None;
        }
        if !self.expect_peek(Token::LBrace) {
            return None;
        }

        let body = Box::new(Statement::Block(self.parse_block_statement()));

        Some(Statement::While { condition, body })
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Some(Statement::Expression(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left = match &self.cur_token {
            Token::Identifier(i) => Expression::Identifier(i.clone()),
            Token::Integer(i) => Expression::Integer(*i),
            Token::String(s) => Expression::String(s.clone()),
            Token::Minus => {
                let op = self.cur_token.clone();
                self.next_token();
                let right = self.parse_expression(Precedence::Prefix)?;
                Expression::Prefix {
                    operator: op,
                    right: Box::new(right),
                }
            }
            Token::LParen => {
                self.next_token();
                let expr = self.parse_expression(Precedence::Lowest)?;
                if !self.expect_peek(Token::RParen) {
                    return None;
                }
                expr
            }
            _ => return None,
        };

        while self.peek_token != Token::Semicolon && precedence < token_precedence(&self.peek_token)
        {
            match self.peek_token {
                Token::LParen => {
                    self.next_token();
                    left = self.parse_call_expression(left)?;
                }
                Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk
                | Token::Equal
                | Token::NotEqual
                | Token::LessThan
                | Token::GreaterThan => {
                    self.next_token();
                    let op = self.cur_token.clone();

                    self.next_token(); // Advance to start of right expression

                    let right = self.parse_expression(token_precedence(&op))?;
                    left = Expression::Infix {
                        left: Box::new(left),
                        operator: op,
                        right: Box::new(right),
                    };
                }
                _ => return Some(left),
            }
        }

        Some(left)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        // cur_token is LParen
        let mut args = vec![];

        if self.peek_token == Token::RParen {
            self.next_token();
            return Some(Expression::Call {
                function: Box::new(function),
                arguments: args,
            });
        }

        self.next_token();
        args.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token == Token::Comma {
            self.next_token();
            self.next_token();
            args.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(Token::RParen) {
            return None;
        }

        Some(Expression::Call {
            function: Box::new(function),
            arguments: args,
        })
    }

    fn expect_peek(&mut self, expected: Token) -> bool {
        if self.peek_token == expected {
            self.next_token();
            true
        } else {
            // Here we could add an error "Expected X got Y"
            self.errors.push(format!(
                "Expected {:?}, got {:?}",
                expected, self.peek_token
            ));
            false
        }
    }
}
