#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
}
impl Program {
    pub fn new(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Expression(Expression),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Block(Vec<Statement>),
    Return(Expression),
    FunctionDefinition(FunctionDefinition),
    InitDeclaration(InitDeclaration),
}

#[derive(Debug, PartialEq, Eq)]
pub struct IfStatement {
    pub condition: Expression,
    pub consequence: Box<Statement>,
    pub alternative: Option<Box<Statement>>,
}
impl IfStatement {
    pub fn new(
        condition: Expression,
        consequence: Statement,
        alternative: Option<Statement>,
    ) -> Self {
        Self {
            condition,
            consequence: Box::new(consequence),
            alternative: alternative.map(Box::new),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
}
impl WhileStatement {
    pub fn new(condition: Expression, body: Statement) -> Self {
        Self {
            condition,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ForStatement {
    pub init: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub post: Option<Box<Statement>>,
    pub body: Box<Statement>,
}
impl ForStatement {
    pub fn new(
        init: Option<Statement>,
        condition: Option<Expression>,
        post: Option<Statement>,
        body: Statement,
    ) -> Self {
        Self {
            init: init.map(Box::new),
            condition,
            post: post.map(Box::new),
            body: Box::new(body),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FunctionDefinition {
    pub name: String,
    pub arguments: Vec<Expression>, // Expression::LocalVariable
    pub body: Vec<Statement>,
}
impl FunctionDefinition {
    pub fn new(name: String, arguments: Vec<Expression>, body: Vec<Statement>) -> Self {
        Self {
            name,
            arguments,
            body,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InitDeclaration {
    pub name: String,
    pub offset: usize,
    pub type_: Type,
    pub init: Option<Expression>,
}
impl InitDeclaration {
    pub fn new(name: String, offset: usize, type_: Type, init: Option<Expression>) -> Self {
        Self {
            name,
            offset,
            type_,
            init,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Primitive(TypeEnum),
    Array { type_: Box<Type>, size: i32 },
    Pointer(Box<Type>),
}
impl Type {
    pub fn size(&self) -> usize {
        match self {
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeEnum {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    LocalVariable {
        name: String,
        offset: usize,
        type_: Type,
    },
    Integer(i32),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Call(CallExpression),
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryExpression {
    pub lhs: Box<Expression>,
    pub op: BinaryOperator,
    pub rhs: Box<Expression>,
}
impl BinaryExpression {
    pub fn new(lhs: Expression, op: BinaryOperator, rhs: Expression) -> Self {
        Self {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Assignment,
    Plus,
    Minus,
    Slash,
    Asterisk,
    Lt,
    LtEq,
    Eq,
    NotEq,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryExpression {
    pub expr: Box<Expression>,
    pub op: UnaryOperator,
    // prefix: bool,
}
impl UnaryExpression {
    pub fn new(expr: Expression, op: UnaryOperator) -> Self {
        Self {
            expr: Box::new(expr),
            op,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnaryOperator {
    Minus,
    Dereference,
    Reference,
    // Bang,
    // Increment,
    // Decrement,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CallExpression {
    pub callee_name: String,
    pub arguments: Vec<Expression>,
}
impl CallExpression {
    pub fn new(callee_name: String, arguments: Vec<Expression>) -> Self {
        Self {
            callee_name,
            arguments,
        }
    }
}