use crate::{
    ast::{BinaryExpression, BinaryOperator, Expression, UnaryExpression, UnaryOperator},
    lex::{Lexer, Token},
};

pub(crate) fn parse(input: String) -> Expression {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse().unwrap()
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
enum Precedence {
    Lowest,
    Sum,
    Product,
    // Unary,
}

struct Parser {
    lexer: Lexer,
    current_token: Token,
    peeked_token: Token,
}

impl Parser {
    fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next();
        let peeked_token = lexer.next();
        Self {
            lexer,
            current_token,
            peeked_token,
        }
    }

    fn parse(&mut self) -> Result<Expression, String> {
        self.parse_expression(Precedence::Lowest)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, String> {
        let mut expr = match self.current_token {
            Token::Integer(n) => Expression::Integer(n),
            Token::LParen => self.parse_grouped_expression()?,
            Token::Minus => self.parse_unary_expression()?,
            _ => return Err(format!("Invalid token: {:?}", self.current_token)),
        };

        while self.peeked_token != Token::Eof && precedence < self.peek_precedence() {
            expr = match self.peeked_token {
                Token::Plus | Token::Minus | Token::Asterisk | Token::Slash => {
                    self.next_token();
                    self.parse_binary_expression(expr)?
                }
                _ => panic!(""), // TODO:
            }
        }

        Ok(expr)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, String> {
        match self.current_token {
            Token::Minus => {
                self.next_token();
                let expr = self.parse_expression(Precedence::Product)?;
                Ok(Expression::Unary(UnaryExpression::new(
                    expr,
                    UnaryOperator::Minus,
                )))
            }
            _ => unreachable!(),
        }
    }

    fn parse_binary_expression(&mut self, left: Expression) -> Result<Expression, String> {
        let op = match self.current_token {
            Token::Plus => BinaryOperator::Plus,
            Token::Minus => BinaryOperator::Minus,
            Token::Asterisk => BinaryOperator::Asterisk,
            Token::Slash => BinaryOperator::Slash,
            _ => {
                return Err(format!(
                    "Expected binary operator, but got {:?}",
                    self.current_token
                ))
            }
        };
        let precedence = self.get_precedence(self.current_token);
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Ok(Expression::Binary(BinaryExpression::new(left, op, right)))
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression, String> {
        self.next_token();
        let expr = self.parse_expression(Precedence::Lowest)?;
        if self.peeked_token != Token::RParen {
            return Err(format!("Expected ')', but got {:?}", self.peeked_token));
        }
        self.next_token();
        Ok(expr)
    }

    fn peek_precedence(&self) -> Precedence {
        self.get_precedence(self.peeked_token)
    }

    fn get_precedence(&self, token: Token) -> Precedence {
        match token {
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Slash | Token::Asterisk => Precedence::Product,
            _ => Precedence::Lowest,
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.peeked_token;
        self.peeked_token = self.lexer.next();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let cases = vec![
            (String::from("5"), Expression::Integer(5)),
            (String::from("10"), Expression::Integer(10)),
            (
                String::from("-10"),
                Expression::Unary(UnaryExpression::new(
                    Expression::Integer(10),
                    UnaryOperator::Minus,
                )),
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(parse(input), expected);
        }
    }

    #[test]
    fn test_binary_expression() {
        let case = vec![
            (
                String::from("5 + 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Integer(5),
                    BinaryOperator::Plus,
                    Expression::Integer(5),
                )),
            ),
            (
                String::from("5 - 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Integer(5),
                    BinaryOperator::Minus,
                    Expression::Integer(5),
                )),
            ),
            (
                String::from("5 * 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Integer(5),
                    BinaryOperator::Asterisk,
                    Expression::Integer(5),
                )),
            ),
            (
                String::from("5 / 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Integer(5),
                    BinaryOperator::Slash,
                    Expression::Integer(5),
                )),
            ),
            // include unary
            (
                String::from("-5 + 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Unary(UnaryExpression::new(
                        Expression::Integer(5),
                        UnaryOperator::Minus,
                    )),
                    BinaryOperator::Plus,
                    Expression::Integer(5),
                )),
            ),
            (
                String::from("5 + -5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Integer(5),
                    BinaryOperator::Plus,
                    Expression::Unary(UnaryExpression::new(
                        Expression::Integer(5),
                        UnaryOperator::Minus,
                    )),
                )),
            ),
        ];
    }

    #[test]
    fn test_binary_expression_with_precedence() {
        let cases = vec![
            (
                String::from("5 + 5 * 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Integer(5),
                    BinaryOperator::Plus,
                    Expression::Binary(BinaryExpression::new(
                        Expression::Integer(5),
                        BinaryOperator::Asterisk,
                        Expression::Integer(5),
                    )),
                )),
            ),
            (
                String::from("1 * 2 + 3 * 4"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Binary(BinaryExpression::new(
                        Expression::Integer(1),
                        BinaryOperator::Asterisk,
                        Expression::Integer(2),
                    )),
                    BinaryOperator::Plus,
                    Expression::Binary(BinaryExpression::new(
                        Expression::Integer(3),
                        BinaryOperator::Asterisk,
                        Expression::Integer(4),
                    )),
                )),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let expr = parser.parse().unwrap();
            assert_eq!(expr, expected);
        }
    }

    #[test]
    fn test_binary_expression_with_paren() {
        let cases = vec![
            (
                String::from("(5 + 5) * 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Binary(BinaryExpression::new(
                        Expression::Integer(5),
                        BinaryOperator::Plus,
                        Expression::Integer(5),
                    )),
                    BinaryOperator::Asterisk,
                    Expression::Integer(5),
                )),
            ),
            (
                String::from("1 * (2 + 3) * 4"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Binary(BinaryExpression::new(
                        Expression::Integer(1),
                        BinaryOperator::Asterisk,
                        Expression::Binary(BinaryExpression::new(
                            Expression::Integer(2),
                            BinaryOperator::Plus,
                            Expression::Integer(3),
                        )),
                    )),
                    BinaryOperator::Asterisk,
                    Expression::Integer(4),
                )),
            ),
            (
                String::from("1 * (2 * (3 + 4)) * 5"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Binary(BinaryExpression::new(
                        Expression::Integer(1),
                        BinaryOperator::Asterisk,
                        Expression::Binary(BinaryExpression::new(
                            Expression::Integer(2),
                            BinaryOperator::Asterisk,
                            Expression::Binary(BinaryExpression::new(
                                Expression::Integer(3),
                                BinaryOperator::Plus,
                                Expression::Integer(4),
                            )),
                        )),
                    )),
                    BinaryOperator::Asterisk,
                    Expression::Integer(5),
                )),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let expr = parser.parse().unwrap();
            assert_eq!(expr, expected);
        }
    }
}
