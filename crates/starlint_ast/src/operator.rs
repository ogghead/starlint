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
    #[allow(clippy::wildcard_imports)]
    use super::*;

    // ── BinaryOperator::as_str ──────────────────────────────────────────

    #[test]
    fn binary_operator_as_str_all_variants() {
        assert_eq!(BinaryOperator::Equality.as_str(), "==", "Equality => ==");
        assert_eq!(
            BinaryOperator::Inequality.as_str(),
            "!=",
            "Inequality => !="
        );
        assert_eq!(
            BinaryOperator::StrictEquality.as_str(),
            "===",
            "StrictEquality => ==="
        );
        assert_eq!(
            BinaryOperator::StrictInequality.as_str(),
            "!==",
            "StrictInequality => !=="
        );
        assert_eq!(BinaryOperator::LessThan.as_str(), "<", "LessThan => <");
        assert_eq!(
            BinaryOperator::LessEqualThan.as_str(),
            "<=",
            "LessEqualThan => <="
        );
        assert_eq!(
            BinaryOperator::GreaterThan.as_str(),
            ">",
            "GreaterThan => >"
        );
        assert_eq!(
            BinaryOperator::GreaterEqualThan.as_str(),
            ">=",
            "GreaterEqualThan => >="
        );
        assert_eq!(BinaryOperator::Addition.as_str(), "+", "Addition => +");
        assert_eq!(
            BinaryOperator::Subtraction.as_str(),
            "-",
            "Subtraction => -"
        );
        assert_eq!(
            BinaryOperator::Multiplication.as_str(),
            "*",
            "Multiplication => *"
        );
        assert_eq!(BinaryOperator::Division.as_str(), "/", "Division => /");
        assert_eq!(BinaryOperator::Remainder.as_str(), "%", "Remainder => %");
        assert_eq!(
            BinaryOperator::Exponential.as_str(),
            "**",
            "Exponential => **"
        );
        assert_eq!(BinaryOperator::ShiftLeft.as_str(), "<<", "ShiftLeft => <<");
        assert_eq!(
            BinaryOperator::ShiftRight.as_str(),
            ">>",
            "ShiftRight => >>"
        );
        assert_eq!(
            BinaryOperator::ShiftRightZeroFill.as_str(),
            ">>>",
            "ShiftRightZeroFill => >>>"
        );
        assert_eq!(BinaryOperator::BitwiseOR.as_str(), "|", "BitwiseOR => |");
        assert_eq!(BinaryOperator::BitwiseXOR.as_str(), "^", "BitwiseXOR => ^");
        assert_eq!(BinaryOperator::BitwiseAnd.as_str(), "&", "BitwiseAnd => &");
        assert_eq!(BinaryOperator::In.as_str(), "in", "In => in");
        assert_eq!(
            BinaryOperator::Instanceof.as_str(),
            "instanceof",
            "Instanceof => instanceof"
        );
    }

    // ── BinaryOperator::is_equality ─────────────────────────────────────

    #[test]
    fn binary_operator_is_equality_true_cases() {
        assert!(
            BinaryOperator::Equality.is_equality(),
            "== should be equality"
        );
        assert!(
            BinaryOperator::Inequality.is_equality(),
            "!= should be equality"
        );
        assert!(
            BinaryOperator::StrictEquality.is_equality(),
            "=== should be equality"
        );
        assert!(
            BinaryOperator::StrictInequality.is_equality(),
            "!== should be equality"
        );
    }

    #[test]
    fn binary_operator_is_equality_false_cases() {
        let non_equality = [
            BinaryOperator::LessThan,
            BinaryOperator::LessEqualThan,
            BinaryOperator::GreaterThan,
            BinaryOperator::GreaterEqualThan,
            BinaryOperator::Addition,
            BinaryOperator::Subtraction,
            BinaryOperator::Multiplication,
            BinaryOperator::Division,
            BinaryOperator::Remainder,
            BinaryOperator::Exponential,
            BinaryOperator::ShiftLeft,
            BinaryOperator::ShiftRight,
            BinaryOperator::ShiftRightZeroFill,
            BinaryOperator::BitwiseOR,
            BinaryOperator::BitwiseXOR,
            BinaryOperator::BitwiseAnd,
            BinaryOperator::In,
            BinaryOperator::Instanceof,
        ];
        for op in non_equality {
            assert!(!op.is_equality(), "{} should not be equality", op.as_str());
        }
    }

    // ── BinaryOperator::is_compare ──────────────────────────────────────

    #[test]
    fn binary_operator_is_compare_true_cases() {
        assert!(BinaryOperator::LessThan.is_compare(), "< should be compare");
        assert!(
            BinaryOperator::LessEqualThan.is_compare(),
            "<= should be compare"
        );
        assert!(
            BinaryOperator::GreaterThan.is_compare(),
            "> should be compare"
        );
        assert!(
            BinaryOperator::GreaterEqualThan.is_compare(),
            ">= should be compare"
        );
    }

    #[test]
    fn binary_operator_is_compare_false_cases() {
        let non_compare = [
            BinaryOperator::Equality,
            BinaryOperator::Inequality,
            BinaryOperator::StrictEquality,
            BinaryOperator::StrictInequality,
            BinaryOperator::Addition,
            BinaryOperator::Subtraction,
            BinaryOperator::Multiplication,
            BinaryOperator::Division,
            BinaryOperator::Remainder,
            BinaryOperator::Exponential,
            BinaryOperator::ShiftLeft,
            BinaryOperator::ShiftRight,
            BinaryOperator::ShiftRightZeroFill,
            BinaryOperator::BitwiseOR,
            BinaryOperator::BitwiseXOR,
            BinaryOperator::BitwiseAnd,
            BinaryOperator::In,
            BinaryOperator::Instanceof,
        ];
        for op in non_compare {
            assert!(!op.is_compare(), "{} should not be compare", op.as_str());
        }
    }

    // ── LogicalOperator::as_str ─────────────────────────────────────────

    #[test]
    fn logical_operator_as_str_all_variants() {
        assert_eq!(LogicalOperator::Or.as_str(), "||", "Or => ||");
        assert_eq!(LogicalOperator::And.as_str(), "&&", "And => &&");
        assert_eq!(LogicalOperator::Coalesce.as_str(), "??", "Coalesce => ??");
    }

    // ── UnaryOperator::as_str ───────────────────────────────────────────

    #[test]
    fn unary_operator_as_str_all_variants() {
        assert_eq!(UnaryOperator::UnaryPlus.as_str(), "+", "UnaryPlus => +");
        assert_eq!(
            UnaryOperator::UnaryNegation.as_str(),
            "-",
            "UnaryNegation => -"
        );
        assert_eq!(UnaryOperator::LogicalNot.as_str(), "!", "LogicalNot => !");
        assert_eq!(UnaryOperator::BitwiseNot.as_str(), "~", "BitwiseNot => ~");
        assert_eq!(UnaryOperator::Typeof.as_str(), "typeof", "Typeof => typeof");
        assert_eq!(UnaryOperator::Void.as_str(), "void", "Void => void");
        assert_eq!(UnaryOperator::Delete.as_str(), "delete", "Delete => delete");
    }

    // ── UnaryOperator::is_keyword ───────────────────────────────────────

    #[test]
    fn unary_operator_is_keyword_true_cases() {
        assert!(
            UnaryOperator::Typeof.is_keyword(),
            "typeof should be keyword"
        );
        assert!(UnaryOperator::Void.is_keyword(), "void should be keyword");
        assert!(
            UnaryOperator::Delete.is_keyword(),
            "delete should be keyword"
        );
    }

    #[test]
    fn unary_operator_is_keyword_false_cases() {
        assert!(
            !UnaryOperator::UnaryPlus.is_keyword(),
            "+ should not be keyword"
        );
        assert!(
            !UnaryOperator::UnaryNegation.is_keyword(),
            "- should not be keyword"
        );
        assert!(
            !UnaryOperator::LogicalNot.is_keyword(),
            "! should not be keyword"
        );
        assert!(
            !UnaryOperator::BitwiseNot.is_keyword(),
            "~ should not be keyword"
        );
    }

    // ── UpdateOperator::as_str ──────────────────────────────────────────

    #[test]
    fn update_operator_as_str_all_variants() {
        assert_eq!(UpdateOperator::Increment.as_str(), "++", "Increment => ++");
        assert_eq!(UpdateOperator::Decrement.as_str(), "--", "Decrement => --");
    }

    // ── AssignmentOperator::as_str ──────────────────────────────────────

    #[test]
    fn assignment_operator_as_str_all_variants() {
        assert_eq!(AssignmentOperator::Assign.as_str(), "=", "Assign => =");
        assert_eq!(
            AssignmentOperator::Addition.as_str(),
            "+=",
            "Addition => +="
        );
        assert_eq!(
            AssignmentOperator::Subtraction.as_str(),
            "-=",
            "Subtraction => -="
        );
        assert_eq!(
            AssignmentOperator::Multiplication.as_str(),
            "*=",
            "Multiplication => *="
        );
        assert_eq!(
            AssignmentOperator::Division.as_str(),
            "/=",
            "Division => /="
        );
        assert_eq!(
            AssignmentOperator::Remainder.as_str(),
            "%=",
            "Remainder => %="
        );
        assert_eq!(
            AssignmentOperator::Exponential.as_str(),
            "**=",
            "Exponential => **="
        );
        assert_eq!(
            AssignmentOperator::ShiftLeft.as_str(),
            "<<=",
            "ShiftLeft => <<="
        );
        assert_eq!(
            AssignmentOperator::ShiftRight.as_str(),
            ">>=",
            "ShiftRight => >>="
        );
        assert_eq!(
            AssignmentOperator::ShiftRightZeroFill.as_str(),
            ">>>=",
            "ShiftRightZeroFill => >>>="
        );
        assert_eq!(
            AssignmentOperator::BitwiseOR.as_str(),
            "|=",
            "BitwiseOR => |="
        );
        assert_eq!(
            AssignmentOperator::BitwiseXOR.as_str(),
            "^=",
            "BitwiseXOR => ^="
        );
        assert_eq!(
            AssignmentOperator::BitwiseAnd.as_str(),
            "&=",
            "BitwiseAnd => &="
        );
        assert_eq!(
            AssignmentOperator::LogicalOr.as_str(),
            "||=",
            "LogicalOr => ||="
        );
        assert_eq!(
            AssignmentOperator::LogicalAnd.as_str(),
            "&&=",
            "LogicalAnd => &&="
        );
        assert_eq!(
            AssignmentOperator::LogicalNullish.as_str(),
            "??=",
            "LogicalNullish => ??="
        );
    }

    // ── AssignmentOperator::is_assign ───────────────────────────────────

    #[test]
    fn assignment_operator_is_assign_true_case() {
        assert!(
            AssignmentOperator::Assign.is_assign(),
            "= should be plain assign"
        );
    }

    #[test]
    fn assignment_operator_is_assign_false_cases() {
        let compound = [
            AssignmentOperator::Addition,
            AssignmentOperator::Subtraction,
            AssignmentOperator::Multiplication,
            AssignmentOperator::Division,
            AssignmentOperator::Remainder,
            AssignmentOperator::Exponential,
            AssignmentOperator::ShiftLeft,
            AssignmentOperator::ShiftRight,
            AssignmentOperator::ShiftRightZeroFill,
            AssignmentOperator::BitwiseOR,
            AssignmentOperator::BitwiseXOR,
            AssignmentOperator::BitwiseAnd,
            AssignmentOperator::LogicalOr,
            AssignmentOperator::LogicalAnd,
            AssignmentOperator::LogicalNullish,
        ];
        for op in compound {
            assert!(
                !op.is_assign(),
                "{} should not be plain assign",
                op.as_str()
            );
        }
    }

    // ── VariableDeclarationKind::as_str ─────────────────────────────────

    #[test]
    fn variable_declaration_kind_as_str_all_variants() {
        assert_eq!(VariableDeclarationKind::Var.as_str(), "var", "Var => var");
        assert_eq!(VariableDeclarationKind::Let.as_str(), "let", "Let => let");
        assert_eq!(
            VariableDeclarationKind::Const.as_str(),
            "const",
            "Const => const"
        );
        assert_eq!(
            VariableDeclarationKind::Using.as_str(),
            "using",
            "Using => using"
        );
        assert_eq!(
            VariableDeclarationKind::AwaitUsing.as_str(),
            "await using",
            "AwaitUsing => await using"
        );
    }

    // ── PropertyKind ────────────────────────────────────────────────────

    #[test]
    fn property_kind_equality() {
        assert_eq!(PropertyKind::Init, PropertyKind::Init, "Init == Init");
        assert_eq!(PropertyKind::Get, PropertyKind::Get, "Get == Get");
        assert_eq!(PropertyKind::Set, PropertyKind::Set, "Set == Set");
        assert_ne!(PropertyKind::Init, PropertyKind::Get, "Init != Get");
        assert_ne!(PropertyKind::Init, PropertyKind::Set, "Init != Set");
        assert_ne!(PropertyKind::Get, PropertyKind::Set, "Get != Set");
    }

    #[test]
    fn property_kind_debug() {
        assert_eq!(
            format!("{:?}", PropertyKind::Init),
            "Init",
            "PropertyKind::Init debug repr"
        );
        assert_eq!(
            format!("{:?}", PropertyKind::Get),
            "Get",
            "PropertyKind::Get debug repr"
        );
        assert_eq!(
            format!("{:?}", PropertyKind::Set),
            "Set",
            "PropertyKind::Set debug repr"
        );
    }

    // ── MethodDefinitionKind ────────────────────────────────────────────

    #[test]
    fn method_definition_kind_equality() {
        assert_eq!(
            MethodDefinitionKind::Method,
            MethodDefinitionKind::Method,
            "Method == Method"
        );
        assert_eq!(
            MethodDefinitionKind::Constructor,
            MethodDefinitionKind::Constructor,
            "Constructor == Constructor"
        );
        assert_eq!(
            MethodDefinitionKind::Get,
            MethodDefinitionKind::Get,
            "Get == Get"
        );
        assert_eq!(
            MethodDefinitionKind::Set,
            MethodDefinitionKind::Set,
            "Set == Set"
        );
        assert_ne!(
            MethodDefinitionKind::Method,
            MethodDefinitionKind::Constructor,
            "Method != Constructor"
        );
        assert_ne!(
            MethodDefinitionKind::Method,
            MethodDefinitionKind::Get,
            "Method != Get"
        );
        assert_ne!(
            MethodDefinitionKind::Method,
            MethodDefinitionKind::Set,
            "Method != Set"
        );
        assert_ne!(
            MethodDefinitionKind::Constructor,
            MethodDefinitionKind::Get,
            "Constructor != Get"
        );
        assert_ne!(
            MethodDefinitionKind::Constructor,
            MethodDefinitionKind::Set,
            "Constructor != Set"
        );
        assert_ne!(
            MethodDefinitionKind::Get,
            MethodDefinitionKind::Set,
            "Get != Set"
        );
    }

    #[test]
    fn method_definition_kind_debug() {
        assert_eq!(
            format!("{:?}", MethodDefinitionKind::Method),
            "Method",
            "MethodDefinitionKind::Method debug repr"
        );
        assert_eq!(
            format!("{:?}", MethodDefinitionKind::Constructor),
            "Constructor",
            "MethodDefinitionKind::Constructor debug repr"
        );
        assert_eq!(
            format!("{:?}", MethodDefinitionKind::Get),
            "Get",
            "MethodDefinitionKind::Get debug repr"
        );
        assert_eq!(
            format!("{:?}", MethodDefinitionKind::Set),
            "Set",
            "MethodDefinitionKind::Set debug repr"
        );
    }
}
