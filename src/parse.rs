use crate::{
    ast::{
        BinaryExpression, BinaryOperator, CallExpression, Expression, ForStatement,
        FunctionDefinition, IfStatement, InitDeclaration, Program, Statement, Type, TypeEnum,
        UnaryExpression, UnaryOperator, WhileStatement,
    },
    lex::{Lexer, Token},
};

// entry
pub(crate) fn parse(input: String) -> Result<Program, String> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse()
}

struct LVar {
    name: String,
    offset: usize,
    type_: Type,
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
enum Precedence {
    Lowest,
    Assignment,
    Equals,
    LessGreater,
    Sum,
    Product,
}

struct Parser {
    lexer: Lexer,
    current_token: Token,
    peeked_token: Token,
    locals: Vec<LVar>,
}

impl Parser {
    fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next();
        let peeked_token = lexer.next();
        Self {
            lexer,
            current_token,
            peeked_token,
            locals: Vec::new(),
        }
    }

    fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while self.current_token != Token::Eof {
            statements.push(self.parse_statement()?);
            self.next_token();
        }
        Ok(Program::new(statements))
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token {
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Return => self.parse_return_statement(),
            Token::LBrace => self.parse_block_statement(),
            Token::Void
            | Token::Char
            | Token::Short
            | Token::Int
            | Token::Long
            | Token::Float
            | Token::Double => {
                let ty = self.parse_type()?;
                match self.current_token.clone() {
                    Token::Identifier(name) => match self.peeked_token {
                        Token::Assignment | Token::SemiColon => {
                            self.next_token();
                            self.parse_variable_declaration(ty, name)
                        }
                        Token::LParen => {
                            self.next_token();
                            self.parse_function_declaration(name)
                        }
                        _ => Err(format!(
                            "expected token '=' or '(' but got {:?}",
                            self.current_token
                        )),
                    },
                    _ => Err(format!(
                        "expected identifier but got {:?}",
                        self.current_token
                    )),
                }
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.next_token(); // skip 'if'

        if self.current_token == Token::LParen {
            self.next_token();
        } else {
            return Err(format!(
                "expected token '(' but got {:?}",
                self.current_token
            ));
        }

        let condition = self.parse_expression(Precedence::Lowest)?;

        if self.peeked_token == Token::RParen {
            self.next_token(); // skip current
            self.next_token(); // skip ')'
        } else {
            return Err(format!(
                "expected token ')' but got {:?}",
                self.peeked_token
            ));
        }

        let consequence = self.parse_statement()?;

        let alternative = if self.peeked_token == Token::Else {
            self.next_token(); // skip current
            self.next_token(); // skip 'else'
            Some(self.parse_statement()?)
        } else {
            None
        };

        Ok(Statement::If(IfStatement::new(
            condition,
            consequence,
            alternative,
        )))
    }

    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        self.next_token(); // skip 'while'

        if self.current_token == Token::LParen {
            self.next_token();
        } else {
            return Err(format!(
                "expected token '(' but got {:?}",
                self.current_token
            ));
        }

        let condition = self.parse_expression(Precedence::Lowest)?;

        if self.peeked_token == Token::RParen {
            self.next_token(); // skip current
            self.next_token(); // skip ')'
        } else {
            return Err(format!(
                "expected token ')' but got {:?}",
                self.peeked_token
            ));
        }

        let body = self.parse_statement()?;
        Ok(Statement::While(WhileStatement::new(condition, body)))
    }

    fn parse_for_statement(&mut self) -> Result<Statement, String> {
        self.next_token(); // skip 'for'

        if self.current_token == Token::LParen {
            self.next_token();
        } else {
            return Err(format!(
                "expected token '(' but got {:?}",
                self.current_token
            ));
        }

        let init = if self.current_token == Token::SemiColon {
            None
        } else {
            Some(self.parse_expression_statement()?)
        };
        self.next_token(); // skip ';'

        let condition = if self.current_token == Token::SemiColon {
            None
        } else {
            let expr = self.parse_expression(Precedence::Lowest)?;
            self.next_token();
            if self.current_token == Token::SemiColon {
                self.next_token();
                Some(expr)
            } else {
                return Err(format!(
                    "expected token ';' but got {:?}",
                    self.current_token
                ));
            }
        };

        let step = if self.current_token == Token::RParen {
            None
        } else {
            let expr = self.parse_statement()?;
            if self.current_token == Token::RParen {
                self.next_token();
                Some(expr)
            } else {
                return Err(format!(
                    "expected token ')' but got {:?}",
                    self.current_token
                ));
            }
        };

        let body = self.parse_statement()?;

        Ok(Statement::For(ForStatement::new(
            init, condition, step, body,
        )))
    }

    fn parse_block_statement(&mut self) -> Result<Statement, String> {
        self.next_token(); // skip '{'
        let mut statements = Vec::new();
        while self.current_token != Token::RBrace {
            statements.push(self.parse_statement()?);
            self.next_token();
        }
        Ok(Statement::Block(statements))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        self.next_token(); // skip 'return'
        let expr = self.parse_expression(Precedence::Lowest)?;

        if self.peeked_token == Token::SemiColon {
            self.next_token();
        } else {
            return Err(format!(
                "expected token ';' but got {:?}",
                self.peeked_token
            ));
        }

        Ok(Statement::Return(expr))
    }

    fn parse_variable_declaration(
        &mut self,
        type_: Type,
        name: String,
    ) -> Result<Statement, String> {
        let offset = match self.new_local_var(type_.clone(), name.clone())? {
            // TODO: size
            Expression::LocalVariable { offset, .. } => offset,
            _ => unreachable!(),
        };

        let init_expr = match self.current_token {
            Token::SemiColon => None,
            Token::Assignment => {
                self.next_token();
                Some(self.parse_expression(Precedence::Lowest)?)
            }
            _ => {
                return Err(format!(
                    "expected token ';' but got {:?}",
                    self.current_token
                ))
            }
        };
        self.next_token(); // skip ';'

        Ok(Statement::InitDeclaration(InitDeclaration::new(
            name, offset, type_, // TODO: other types
            init_expr,
        )))
    }

    fn parse_function_declaration(&mut self, name: String) -> Result<Statement, String> {
        let mut params = Vec::new();
        while self.peeked_token != Token::RParen {
            self.next_token();
            let type_ = self.parse_type()?;

            let name = match self.current_token.clone() {
                Token::Identifier(name) => name,
                _ => {
                    return Err(format!(
                        "expected identifier but got {:?}",
                        self.peeked_token
                    ))
                }
            };
            if self.peeked_token == Token::Comma {
                self.next_token();
            }

            params.push((type_, name));
        }

        if self.peeked_token == Token::RParen {
            self.next_token();
            self.next_token(); // skip ')'
        } else {
            return Err(format!(
                "expected token ')' but got {:?}",
                self.peeked_token
            ));
        }

        let params = params
            .iter()
            .map(|(t, name)| self.new_local_var(t.clone(), name.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let body = match self.parse_block_statement()? {
            Statement::Block(body) => body,
            _ => unreachable!(),
        };

        Ok(Statement::FunctionDefinition(FunctionDefinition::new(
            name, params, body,
        )))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expression(Precedence::Lowest)?;

        if self.peeked_token == Token::SemiColon || self.peeked_token == Token::RParen {
            self.next_token();
        } else {
            return Err(format!(
                "expected token ';' or ')' but got {:?}",
                self.peeked_token
            ));
        }

        Ok(Statement::Expression(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, String> {
        let mut expr = match self.current_token.clone() {
            Token::Integer(n) => Expression::Integer(n),
            Token::LParen => self.parse_grouped_expression()?,
            Token::Minus => self.parse_unary_expression()?,
            Token::Identifier(name) => match self.peeked_token {
                Token::LParen => {
                    self.next_token(); // skip identifier
                    self.parse_call_expression(name)?
                }
                _ => self.parse_identifier_expression(name)?,
            },
            _ => return Err(format!("Invalid token: {:?}", self.current_token)),
        };

        while self.peeked_token != Token::Eof && precedence < self.peek_precedence() {
            expr = match self.peeked_token {
                Token::Assignment
                | Token::Plus
                | Token::Minus
                | Token::Asterisk
                | Token::Slash
                | Token::Eq
                | Token::NotEq
                | Token::Lt
                | Token::Gt
                | Token::LtEq
                | Token::GtEq => {
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
        let (op, swap) = match self.current_token {
            Token::Assignment => (BinaryOperator::Assignment, false),
            Token::Plus => (BinaryOperator::Plus, false),
            Token::Minus => (BinaryOperator::Minus, false),
            Token::Asterisk => (BinaryOperator::Asterisk, false),
            Token::Slash => (BinaryOperator::Slash, false),
            Token::Lt => (BinaryOperator::Lt, false),
            Token::Gt => (BinaryOperator::Lt, true),
            Token::LtEq => (BinaryOperator::LtEq, false),
            Token::GtEq => (BinaryOperator::LtEq, true),
            Token::Eq => (BinaryOperator::Eq, false),
            Token::NotEq => (BinaryOperator::NotEq, false),
            _ => {
                return Err(format!(
                    "Expected binary operator, but got {:?}",
                    self.current_token
                ))
            }
        };
        let precedence = self.get_precedence(self.current_token.clone());
        self.next_token();
        let right = self.parse_expression(precedence)?;

        // when swap is true, swap left and right
        if swap {
            Ok(Expression::Binary(BinaryExpression::new(right, op, left)))
        } else {
            Ok(Expression::Binary(BinaryExpression::new(left, op, right)))
        }
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

    fn parse_identifier_expression(&mut self, name: String) -> Result<Expression, String> {
        let offset = self.find_local_var(&name);
        match offset {
            Some(LVar { offset, type_, .. }) => Ok(Expression::LocalVariable {
                name,
                offset: *offset,
                type_: type_.clone(),
            }),
            None => Err(format!("Undefined variable: {}", name)),
        }
    }

    fn parse_call_expression(&mut self, callee_name: String) -> Result<Expression, String> {
        let mut arguments = vec![];

        while self.peeked_token != Token::RParen {
            self.next_token();
            let expr = self.parse_expression(Precedence::Lowest)?;
            arguments.push(expr);
            if self.peeked_token == Token::Comma {
                self.next_token();
            }
        }

        self.next_token(); // skip ')'

        Ok(Expression::Call(CallExpression::new(
            callee_name,
            arguments,
        )))
    }

    fn new_local_var(&mut self, type_: Type, name: String) -> Result<Expression, String> {
        let alloca = self.sizeof(&type_);
        let offset = self.locals.last().map(|l| l.offset).unwrap_or(0) + alloca;
        let v = LVar {
            name: name.clone(),
            offset,
            type_: type_.clone(),
        };
        self.locals.push(v);
        Ok(Expression::LocalVariable {
            name,
            offset,
            type_,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let base = match self.current_token {
            Token::Void => Type::Primitive(TypeEnum::Void),
            Token::Char => Type::Primitive(TypeEnum::Char),
            Token::Short => Type::Primitive(TypeEnum::Short),
            Token::Int => Type::Primitive(TypeEnum::Int),
            Token::Long => Type::Primitive(TypeEnum::Long),
            Token::Float => Type::Primitive(TypeEnum::Float),
            Token::Double => Type::Primitive(TypeEnum::Double),
            _ => return Err(format!("Expected type, but got {:?}", self.current_token)),
        };
        self.next_token();

        let mut t = base;
        // TODO: array
        while self.current_token == Token::Asterisk {
            t = Type::Pointer(Box::new(t));
            self.next_token();
        }
        Ok(t)
    }

    fn sizeof(&self, t: &Type) -> usize {
        match t {
            Type::Primitive(TypeEnum::Void) => 0,
            Type::Primitive(TypeEnum::Char) => 1,
            Type::Primitive(TypeEnum::Short) => 2,
            Type::Primitive(TypeEnum::Int) => 8, // FIXME: clash with 4 now.
            Type::Primitive(TypeEnum::Long) => 8,
            Type::Primitive(TypeEnum::Float) => 4,
            Type::Primitive(TypeEnum::Double) => 8,
            Type::Pointer(_) => 8,
            Type::Array { size, .. } => (size * 8) as usize,
        }
    }

    fn peek_precedence(&self) -> Precedence {
        self.get_precedence(self.peeked_token.clone())
    }

    fn get_precedence(&self, token: Token) -> Precedence {
        match token {
            Token::Assignment => Precedence::Assignment,
            Token::Eq | Token::NotEq => Precedence::Equals,
            Token::Lt | Token::LtEq | Token::Gt | Token::GtEq => Precedence::LessGreater,
            Token::Plus | Token::Minus => Precedence::Sum,
            Token::Slash | Token::Asterisk => Precedence::Product,
            _ => Precedence::Lowest,
        }
    }

    fn find_local_var(&self, name: &str) -> Option<&LVar> {
        self.locals.iter().find(|s| s.name == name)
    }

    fn next_token(&mut self) {
        self.current_token = self.peeked_token.clone();
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
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(
                parser.parse_expression(Precedence::Lowest).unwrap(),
                expected
            );
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

        for (input, expected) in case {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(
                parser.parse_expression(Precedence::Lowest).unwrap(),
                expected
            );
        }
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
            (
                String::from("1 * 2 >= 3 * 4 == 0"),
                Expression::Binary(BinaryExpression::new(
                    Expression::Binary(BinaryExpression::new(
                        Expression::Binary(BinaryExpression::new(
                            Expression::Integer(3),
                            BinaryOperator::Asterisk,
                            Expression::Integer(4),
                        )),
                        BinaryOperator::LtEq,
                        Expression::Binary(BinaryExpression::new(
                            Expression::Integer(1),
                            BinaryOperator::Asterisk,
                            Expression::Integer(2),
                        )),
                    )),
                    BinaryOperator::Eq,
                    Expression::Integer(0),
                )),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(
                parser.parse_expression(Precedence::Lowest).unwrap(),
                expected
            );
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
            assert_eq!(
                parser.parse_expression(Precedence::Lowest).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let cases = vec![
            (
                String::from("return 5;"),
                Statement::Return(Expression::Integer(5)),
            ),
            (
                String::from("return 5 + 5;"),
                Statement::Return(Expression::Binary(BinaryExpression::new(
                    Expression::Integer(5),
                    BinaryOperator::Plus,
                    Expression::Integer(5),
                ))),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse_statement().unwrap(), expected);
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let cases = vec![
            (
                String::from("int a = 0; if (a == 0) return 0; "),
                vec![
                    Statement::InitDeclaration(InitDeclaration::new(
                        String::from("a"),
                        4,
                        Type::Primitive(TypeEnum::Int),
                        Some(Expression::Integer(0)),
                    )),
                    Statement::If(IfStatement::new(
                        Expression::Binary(BinaryExpression::new(
                            Expression::LocalVariable {
                                name: String::from("a"),
                                offset: 4,
                                type_: Type::Primitive(TypeEnum::Int),
                            },
                            BinaryOperator::Eq,
                            Expression::Integer(0),
                        )),
                        Statement::Return(Expression::Integer(0)),
                        None,
                    )),
                ],
            ),
            (
                String::from("int a = 0; if (a == 0) return 0; else return 1;"),
                vec![
                    Statement::InitDeclaration(InitDeclaration::new(
                        String::from("a"),
                        4,
                        Type::Primitive(TypeEnum::Int),
                        Some(Expression::Integer(0)),
                    )),
                    Statement::If(IfStatement::new(
                        Expression::Binary(BinaryExpression::new(
                            Expression::LocalVariable {
                                name: String::from("a"),
                                offset: 4,
                                type_: Type::Primitive(TypeEnum::Int),
                            },
                            BinaryOperator::Eq,
                            Expression::Integer(0),
                        )),
                        Statement::Return(Expression::Integer(0)),
                        Some(Statement::Return(Expression::Integer(1))),
                    )),
                ],
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse().unwrap().statements, expected);
        }
    }

    #[test]
    fn test_parse_while_statement() {
        let cases = vec![(
            String::from("int a = 0; while (a == 0) return 0;"),
            vec![
                Statement::InitDeclaration(InitDeclaration::new(
                    String::from("a"),
                    4,
                    Type::Primitive(TypeEnum::Int),
                    Some(Expression::Integer(0)),
                )),
                Statement::While(WhileStatement::new(
                    Expression::Binary(BinaryExpression::new(
                        Expression::LocalVariable {
                            name: String::from("a"),
                            offset: 4,
                            type_: Type::Primitive(TypeEnum::Int),
                        },
                        BinaryOperator::Eq,
                        Expression::Integer(0),
                    )),
                    Statement::Return(Expression::Integer(0)),
                )),
            ],
        )];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse().unwrap().statements, expected);
        }
    }

    #[test]
    fn test_parse_for_statement() {
        let cases = vec![(
            String::from("int i = 0; for (i = 0; i < 10; i = i + 1) return 0;"),
            vec![
                Statement::InitDeclaration(InitDeclaration::new(
                    String::from("i"),
                    4,
                    Type::Primitive(TypeEnum::Int),
                    Some(Expression::Integer(0)),
                )),
                Statement::For(ForStatement::new(
                    Some(Statement::Expression(Expression::Binary(
                        BinaryExpression::new(
                            Expression::LocalVariable {
                                name: String::from("i"),
                                offset: 4,
                                type_: Type::Primitive(TypeEnum::Int),
                            },
                            BinaryOperator::Assignment,
                            Expression::Integer(0),
                        ),
                    ))),
                    Some(Expression::Binary(BinaryExpression::new(
                        Expression::LocalVariable {
                            name: String::from("i"),
                            offset: 4,
                            type_: Type::Primitive(TypeEnum::Int),
                        },
                        BinaryOperator::Lt,
                        Expression::Integer(10),
                    ))),
                    Some(Statement::Expression(Expression::Binary(
                        BinaryExpression::new(
                            Expression::LocalVariable {
                                name: String::from("i"),
                                offset: 4,
                                type_: Type::Primitive(TypeEnum::Int),
                            },
                            BinaryOperator::Assignment,
                            Expression::Binary(BinaryExpression::new(
                                Expression::LocalVariable {
                                    name: String::from("i"),
                                    offset: 4,
                                    type_: Type::Primitive(TypeEnum::Int),
                                },
                                BinaryOperator::Plus,
                                Expression::Integer(1),
                            )),
                        ),
                    ))),
                    Statement::Return(Expression::Integer(0)),
                )),
            ],
        )];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse().unwrap().statements, expected);
        }
    }

    #[test]
    fn test_parse_block_statement() {
        let cases = vec![
            (String::from("{}"), Statement::Block(vec![])),
            (
                String::from("{ return 0; }"),
                Statement::Block(vec![Statement::Return(Expression::Integer(0))]),
            ),
            (
                String::from("{ int i = 0; i = i + 1; return 0; }"),
                Statement::Block(vec![
                    Statement::InitDeclaration(InitDeclaration::new(
                        String::from("i"),
                        4,
                        Type::Primitive(TypeEnum::Int),
                        Some(Expression::Integer(0)),
                    )),
                    Statement::Expression(Expression::Binary(BinaryExpression::new(
                        Expression::LocalVariable {
                            name: String::from("i"),
                            offset: 4,
                            type_: Type::Primitive(TypeEnum::Int),
                        },
                        BinaryOperator::Assignment,
                        Expression::Binary(BinaryExpression::new(
                            Expression::LocalVariable {
                                name: String::from("i"),
                                offset: 4,
                                type_: Type::Primitive(TypeEnum::Int),
                            },
                            BinaryOperator::Plus,
                            Expression::Integer(1),
                        )),
                    ))),
                    Statement::Return(Expression::Integer(0)),
                ]),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse_statement().unwrap(), expected);
        }
    }

    #[test]
    fn parse_call_expression() {
        let cases = vec![
            (
                String::from("foo();"),
                Expression::Call(CallExpression::new(String::from("foo"), vec![])),
            ),
            (
                String::from("bar(1, 2);"),
                Expression::Call(CallExpression::new(
                    String::from("bar"),
                    vec![Expression::Integer(1), Expression::Integer(2)],
                )),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(
                parser.parse_expression(Precedence::Lowest).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_parse_init_declaration() {
        let cases = vec![
            (
                String::from("int a = 0;"),
                Statement::InitDeclaration(InitDeclaration::new(
                    String::from("a"),
                    4,
                    Type::Primitive(TypeEnum::Int),
                    Some(Expression::Integer(0)),
                )),
            ),
            (
                String::from("int i;"),
                Statement::InitDeclaration(InitDeclaration::new(
                    String::from("i"),
                    4,
                    Type::Primitive(TypeEnum::Int),
                    None,
                )),
            ),
            (
                String::from("int *i;"),
                Statement::InitDeclaration(InitDeclaration::new(
                    String::from("i"),
                    8,
                    Type::Pointer(Box::new(Type::Primitive(TypeEnum::Int))),
                    None,
                )),
            ),
            (
                String::from("int **i;"),
                Statement::InitDeclaration(InitDeclaration::new(
                    String::from("i"),
                    8,
                    Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Primitive(
                        TypeEnum::Int,
                    ))))),
                    None,
                )),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse_statement().unwrap(), expected);
        }
    }

    #[test]
    fn test_parse_function_definition() {
        let cases = vec![
            (
                String::from("int foo() { return 0; }"),
                Statement::FunctionDefinition(FunctionDefinition::new(
                    String::from("foo"),
                    vec![],
                    vec![Statement::Return(Expression::Integer(0))],
                )),
            ),
            (
                String::from("int foo(int a, int b) { return 0; }"),
                Statement::FunctionDefinition(FunctionDefinition::new(
                    String::from("foo"),
                    vec![
                        Expression::LocalVariable {
                            name: String::from("a"),
                            offset: 4,
                            type_: Type::Primitive(TypeEnum::Int),
                        },
                        Expression::LocalVariable {
                            name: String::from("b"),
                            offset: 8,
                            type_: Type::Primitive(TypeEnum::Int),
                        },
                    ],
                    vec![Statement::Return(Expression::Integer(0))],
                )),
            ),
        ];
        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse_statement().unwrap(), expected);
        }
    }

    #[test]
    fn test_parse() {
        let cases = vec![
            (
                String::from("5;1+2*3;"),
                Program::new(vec![
                    Statement::Expression(Expression::Integer(5)),
                    Statement::Expression(Expression::Binary(BinaryExpression::new(
                        Expression::Integer(1),
                        BinaryOperator::Plus,
                        Expression::Binary(BinaryExpression::new(
                            Expression::Integer(2),
                            BinaryOperator::Asterisk,
                            Expression::Integer(3),
                        )),
                    ))),
                ]),
            ),
            (
                String::from("bar(1, 2); return 0;"),
                Program::new(vec![
                    Statement::Expression(Expression::Call(CallExpression::new(
                        String::from("bar"),
                        vec![Expression::Integer(1), Expression::Integer(2)],
                    ))),
                    Statement::Return(Expression::Integer(0)),
                ]),
            ),
            (
                String::from(
                    r#"
                        int foo(int i) {
                            return i;
                        }
                        int main() {
                            int a = foo(10);
                            return 10;
                        }"#,
                ),
                Program::new(vec![
                    Statement::FunctionDefinition(FunctionDefinition::new(
                        String::from("foo"),
                        vec![Expression::LocalVariable {
                            name: String::from("i"),
                            offset: 4,
                            type_: Type::Primitive(TypeEnum::Int),
                        }],
                        vec![Statement::Return(Expression::LocalVariable {
                            name: String::from("i"),
                            offset: 4,
                            type_: Type::Primitive(TypeEnum::Int),
                        })],
                    )),
                    Statement::FunctionDefinition(FunctionDefinition::new(
                        String::from("main"),
                        vec![],
                        vec![
                            Statement::InitDeclaration(InitDeclaration::new(
                                String::from("a"),
                                8,
                                Type::Primitive(TypeEnum::Int),
                                Some(Expression::Call(CallExpression::new(
                                    String::from("foo"),
                                    vec![Expression::Integer(10)],
                                ))),
                            )),
                            Statement::Return(Expression::Integer(10)),
                        ],
                    )),
                ]),
            ),
        ];

        for (input, expected) in cases {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            assert_eq!(parser.parse().unwrap(), expected);
        }
    }
}
