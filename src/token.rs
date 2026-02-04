#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Int,
    Return,
    If,
    Else,
    While,

    // Identifiers and Literals
    Identifier(String),
    Integer(i64),
    String(String),

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Assign,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,

    // End of File
    EOF,

    // Invalid
    Illegal(String),
}

pub struct Lexer<'a> {
    #[allow(dead_code)]
    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.chars.next() {
            Some(c) => match c {
                '=' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        Token::Equal
                    } else {
                        Token::Assign
                    }
                }
                '"' => {
                    let mut str_val = String::new();
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c == '"' {
                            break;
                        }
                        let c = self.chars.next().unwrap();
                        if c == '\\' {
                            if let Some(&next_next) = self.chars.peek() {
                                match next_next {
                                    'n' => {
                                        self.chars.next();
                                        str_val.push('\n');
                                    }
                                    'r' => {
                                        self.chars.next();
                                        str_val.push('\r');
                                    }
                                    't' => {
                                        self.chars.next();
                                        str_val.push('\t');
                                    }
                                    '"' => {
                                        self.chars.next();
                                        str_val.push('"');
                                    }
                                    '\\' => {
                                        self.chars.next();
                                        str_val.push('\\');
                                    }
                                    _ => str_val.push('\\'), // Keep backslash if unknown escape
                                }
                            } else {
                                str_val.push('\\');
                            }
                        } else {
                            str_val.push(c);
                        }
                    }
                    if let Some(&'"') = self.chars.peek() {
                        self.chars.next(); // Consume closing quote
                        Token::String(str_val)
                    } else {
                        Token::Illegal("Unterminated string".to_string())
                    }
                }
                '!' => {
                    if let Some(&'=') = self.chars.peek() {
                        self.chars.next();
                        Token::NotEqual
                    } else {
                        Token::Illegal(c.to_string()) // For now we don't support just '!'
                    }
                }
                '+' => Token::Plus,
                '-' => Token::Minus,
                '*' => Token::Asterisk,
                '/' => {
                    if let Some(&'/') = self.chars.peek() {
                        // It's a comment! Skip until newline
                        while let Some(&c) = self.chars.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.chars.next();
                        }
                        self.next_token() // Recursively call next_token to get the actual next token
                    } else {
                        Token::Slash
                    }
                }
                '<' => Token::LessThan,
                '>' => Token::GreaterThan,
                '(' => Token::LParen,
                ')' => Token::RParen,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                ';' => Token::Semicolon,
                ',' => Token::Comma,
                _ if c.is_ascii_digit() => {
                    let mut num_str = c.to_string();
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c.is_ascii_digit() {
                            num_str.push(self.chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    Token::Integer(num_str.parse().unwrap())
                }
                _ if c.is_ascii_alphabetic() || c == '_' => {
                    let mut ident = c.to_string();
                    while let Some(&next_c) = self.chars.peek() {
                        if next_c.is_ascii_alphanumeric() || next_c == '_' {
                            ident.push(self.chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    match ident.as_str() {
                        "int" => Token::Int,
                        "return" => Token::Return,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "while" => Token::While,
                        _ => Token::Identifier(ident),
                    }
                }
                _ => Token::Illegal(c.to_string()),
            },
            None => Token::EOF,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token() {
        let input = "int x = 5; if (x > 10) { return x; }";
        let mut lexer = Lexer::new(input);

        let tests = vec![
            Token::Int,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Integer(5),
            Token::Semicolon,
            Token::If,
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::GreaterThan,
            Token::Integer(10),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::Identifier("x".to_string()),
            Token::Semicolon,
            Token::RBrace,
            Token::EOF,
        ];

        for expected in tests {
            let tok = lexer.next_token();
            assert_eq!(tok, expected);
        }
    }
}
