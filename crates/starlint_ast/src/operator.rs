//! Operator enums mirroring the ECMAScript specification.
//!
//! These match oxc's operator types 1:1 in variant names and discriminant
//! values, but are defined independently so `starlint_ast` has no oxc
//! dependency.

use serde::{Deserialize, Serialize};

/// Binary expression operators (excludes logical — see [`LogicalOperator`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
    /// `==`
    Equality,
    /// `!=`
    Inequality,
    /// `===`
    StrictEquality,
    /// `!==`
    StrictInequality,
    /// `<`
    LessThan,
    /// `<=`
    LessEqualThan,
    /// `>`
    GreaterThan,
    /// `>=`
    GreaterEqualThan,
    /// `+`
    Addition,
    /// `-`
    Subtraction,
    /// `*`
    Multiplication,
    /// `/`
    Division,
    /// `%`
    Remainder,
    /// `**`
    Exponential,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
    /// `>>>`
    ShiftRightZeroFill,
    /// `|`
    BitwiseOR,
    /// `^`
    BitwiseXOR,
    /// `&`
    BitwiseAnd,
    /// `in`
    In,
    /// `instanceof`
    Instanceof,
}

impl BinaryOperator {
    /// String representation as it appears in source code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Equality => "==",
            Self::Inequality => "!=",
            Self::StrictEquality => "===",
            Self::StrictInequality => "!==",
            Self::LessThan => "<",
            Self::LessEqualThan => "<=",
            Self::GreaterThan => ">",
            Self::GreaterEqualThan => ">=",
            Self::Addition => "+",
            Self::Subtraction => "-",
            Self::Multiplication => "*",
            Self::Division => "/",
            Self::Remainder => "%",
            Self::Exponential => "**",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
            Self::ShiftRightZeroFill => ">>>",
            Self::BitwiseOR => "|",
            Self::BitwiseXOR => "^",
            Self::BitwiseAnd => "&",
            Self::In => "in",
            Self::Instanceof => "instanceof",
        }
    }

    /// Whether this is an equality operator (`==`, `!=`, `===`, `!==`).
    #[must_use]
    pub const fn is_equality(self) -> bool {
        matches!(
            self,
            Self::Equality | Self::Inequality | Self::StrictEquality | Self::StrictInequality
        )
    }

    /// Whether this is a comparison operator (`<`, `<=`, `>`, `>=`).
    #[must_use]
    pub const fn is_compare(self) -> bool {
        matches!(
            self,
            Self::LessThan | Self::LessEqualThan | Self::GreaterThan | Self::GreaterEqualThan
        )
    }
}

/// Logical binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalOperator {
    /// `||`
    Or,
    /// `&&`
    And,
    /// `??`
    Coalesce,
}

impl LogicalOperator {
    /// String representation as it appears in source code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Or => "||",
            Self::And => "&&",
            Self::Coalesce => "??",
        }
    }
}

/// Unary operators (excludes update — see [`UpdateOperator`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// `+`
    UnaryPlus,
    /// `-`
    UnaryNegation,
    /// `!`
    LogicalNot,
    /// `~`
    BitwiseNot,
    /// `typeof`
    Typeof,
    /// `void`
    Void,
    /// `delete`
    Delete,
}

impl UnaryOperator {
    /// String representation as it appears in source code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnaryPlus => "+",
            Self::UnaryNegation => "-",
            Self::LogicalNot => "!",
            Self::BitwiseNot => "~",
            Self::Typeof => "typeof",
            Self::Void => "void",
            Self::Delete => "delete",
        }
    }

    /// Whether this is a keyword operator (`typeof`, `void`, `delete`).
    #[must_use]
    pub const fn is_keyword(self) -> bool {
        matches!(self, Self::Typeof | Self::Void | Self::Delete)
    }
}

/// Update operators (`++`, `--`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpdateOperator {
    /// `++`
    Increment,
    /// `--`
    Decrement,
}

impl UpdateOperator {
    /// String representation as it appears in source code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Increment => "++",
            Self::Decrement => "--",
        }
    }
}

/// Assignment operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssignmentOperator {
    /// `=`
    Assign,
    /// `+=`
    Addition,
    /// `-=`
    Subtraction,
    /// `*=`
    Multiplication,
    /// `/=`
    Division,
    /// `%=`
    Remainder,
    /// `**=`
    Exponential,
    /// `<<=`
    ShiftLeft,
    /// `>>=`
    ShiftRight,
    /// `>>>=`
    ShiftRightZeroFill,
    /// `|=`
    BitwiseOR,
    /// `^=`
    BitwiseXOR,
    /// `&=`
    BitwiseAnd,
    /// `||=`
    LogicalOr,
    /// `&&=`
    LogicalAnd,
    /// `??=`
    LogicalNullish,
}

impl AssignmentOperator {
    /// String representation as it appears in source code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Addition => "+=",
            Self::Subtraction => "-=",
            Self::Multiplication => "*=",
            Self::Division => "/=",
            Self::Remainder => "%=",
            Self::Exponential => "**=",
            Self::ShiftLeft => "<<=",
            Self::ShiftRight => ">>=",
            Self::ShiftRightZeroFill => ">>>=",
            Self::BitwiseOR => "|=",
            Self::BitwiseXOR => "^=",
            Self::BitwiseAnd => "&=",
            Self::LogicalOr => "||=",
            Self::LogicalAnd => "&&=",
            Self::LogicalNullish => "??=",
        }
    }

    /// Whether this is a plain assignment (`=`).
    #[must_use]
    pub const fn is_assign(self) -> bool {
        matches!(self, Self::Assign)
    }
}

/// Variable declaration kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VariableDeclarationKind {
    /// `var`
    Var,
    /// `let`
    Let,
    /// `const`
    Const,
    /// `using`
    Using,
    /// `await using`
    AwaitUsing,
}

impl VariableDeclarationKind {
    /// String representation as it appears in source code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Var => "var",
            Self::Let => "let",
            Self::Const => "const",
            Self::Using => "using",
            Self::AwaitUsing => "await using",
        }
    }
}

/// Property kind in object literals and classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropertyKind {
    /// Normal property or field.
    Init,
    /// Getter (`get x() { ... }`).
    Get,
    /// Setter (`set x(v) { ... }`).
    Set,
}

/// Method definition kind in class bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MethodDefinitionKind {
    /// Regular method.
    Method,
    /// Constructor.
    Constructor,
    /// Getter.
    Get,
    /// Setter.
    Set,
}

#[cfg(test)]
mod tests {
    use super::{BinaryOperator, UnaryOperator};

    #[test]
    fn binary_operator_as_str() {
        assert_eq!(
            BinaryOperator::StrictEquality.as_str(),
            "===",
            "strict equality"
        );
        assert_eq!(
            BinaryOperator::Instanceof.as_str(),
            "instanceof",
            "instanceof"
        );
    }

    #[test]
    fn unary_operator_keyword() {
        assert!(
            UnaryOperator::Typeof.is_keyword(),
            "typeof should be keyword"
        );
        assert!(
            !UnaryOperator::LogicalNot.is_keyword(),
            "! should not be keyword"
        );
    }
}
