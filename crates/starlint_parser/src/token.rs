//! Token types produced by the lexer.

/// A single token produced by the lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// Byte offset of the token's first character in the source.
    pub start: u32,
    /// Byte offset past the token's last character in the source.
    pub end: u32,
}

impl Token {
    /// Create a new token.
    #[must_use]
    pub const fn new(kind: TokenKind, start: u32, end: u32) -> Self {
        Self { kind, start, end }
    }
}

/// All token kinds the lexer can produce.
///
/// Roughly 90 variants covering JS/TS/JSX keywords, punctuators, literals,
/// identifiers, and synthetic tokens (EOF, errors).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // === Synthetic ===
    /// End of file.
    Eof,

    // === Identifiers & literals ===
    /// An identifier or unrecognized keyword used as an identifier.
    Identifier,
    /// Numeric literal (integer, float, hex, octal, binary, bigint).
    Number,
    /// String literal (single or double quoted).
    String,
    /// Template literal head (`` ` ... ${ ``).
    TemplateHead,
    /// Template literal middle (`` } ... ${ ``).
    TemplateMiddle,
    /// Template literal tail (`` } ... ` ``).
    TemplateTail,
    /// No-substitution template (`` ` ... ` `` with no `${`).
    NoSubstitutionTemplate,
    /// Regular expression literal (`/pattern/flags`).
    RegExp,
    /// JSX text content (between tags, outside expressions).
    JsxText,

    // === Keywords ===
    /// `break`
    Break,
    /// `case`
    Case,
    /// `catch`
    Catch,
    /// `class`
    Class,
    /// `const`
    Const,
    /// `continue`
    Continue,
    /// `debugger`
    Debugger,
    /// `default`
    Default,
    /// `delete`
    Delete,
    /// `do`
    Do,
    /// `else`
    Else,
    /// `export`
    Export,
    /// `extends`
    Extends,
    /// `false`
    False,
    /// `finally`
    Finally,
    /// `for`
    For,
    /// `function`
    Function,
    /// `if`
    If,
    /// `import`
    Import,
    /// `in`
    In,
    /// `instanceof`
    Instanceof,
    /// `let`
    Let,
    /// `new`
    New,
    /// `null`
    Null,
    /// `of`
    Of,
    /// `return`
    Return,
    /// `super`
    Super,
    /// `switch`
    Switch,
    /// `this`
    This,
    /// `throw`
    Throw,
    /// `true`
    True,
    /// `try`
    Try,
    /// `typeof`
    Typeof,
    /// `var`
    Var,
    /// `void`
    Void,
    /// `while`
    While,
    /// `with`
    With,
    /// `yield`
    Yield,

    // === Strict mode / contextual keywords ===
    /// `async`
    Async,
    /// `await`
    Await,
    /// `enum`
    Enum,
    /// `from`
    From,
    /// `get`
    Get,
    /// `set`
    Set,
    /// `static`
    Static,
    /// `as`
    As,
    /// `implements`
    Implements,
    /// `interface`
    Interface,
    /// `package`
    Package,
    /// `private`
    Private,
    /// `protected`
    Protected,
    /// `public`
    Public,

    // === TypeScript keywords ===
    /// `type`
    Type,
    /// `namespace`
    Namespace,
    /// `module`
    Module,
    /// `declare`
    Declare,
    /// `abstract`
    Abstract,
    /// `readonly`
    Readonly,
    /// `override`
    Override,
    /// `keyof`
    Keyof,
    /// `unique`
    Unique,
    /// `infer`
    Infer,
    /// `is`
    Is,
    /// `asserts`
    Asserts,
    /// `any`
    Any,
    /// `unknown`
    Unknown,
    /// `never`
    Never,
    /// `using`
    Using,
    /// `satisfies`
    Satisfies,

    // === Punctuators ===
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `.`
    Dot,
    /// `...`
    DotDotDot,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `<`
    LAngle,
    /// `>`
    RAngle,
    /// `<=`
    LessEq,
    /// `>=`
    GreaterEq,
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    /// `===`
    EqEqEq,
    /// `!==`
    NotEqEq,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `**`
    StarStar,
    /// `++`
    PlusPlus,
    /// `--`
    MinusMinus,
    /// `<<`
    LessLess,
    /// `>>`
    GreaterGreater,
    /// `>>>`
    GreaterGreaterGreater,
    /// `&`
    Amp,
    /// `|`
    Pipe,
    /// `^`
    Caret,
    /// `~`
    Tilde,
    /// `!`
    Bang,
    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,
    /// `??`
    QuestionQuestion,
    /// `?`
    Question,
    /// `?.`
    QuestionDot,
    /// `:`
    Colon,
    /// `=>`
    Arrow,
    /// `@`
    At,
    /// `#`
    Hash,

    // === Assignment operators ===
    /// `=`
    Eq,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    StarEq,
    /// `/=`
    SlashEq,
    /// `%=`
    PercentEq,
    /// `**=`
    StarStarEq,
    /// `<<=`
    LessLessEq,
    /// `>>=`
    GreaterGreaterEq,
    /// `>>>=`
    GreaterGreaterGreaterEq,
    /// `&=`
    AmpEq,
    /// `|=`
    PipeEq,
    /// `^=`
    CaretEq,
    /// `||=`
    PipePipeEq,
    /// `&&=`
    AmpAmpEq,
    /// `??=`
    QuestionQuestionEq,
}

impl TokenKind {
    /// Whether this token can start a statement or expression (used for ASI).
    #[must_use]
    pub const fn is_keyword(self) -> bool {
        matches!(
            self,
            Self::Break
                | Self::Case
                | Self::Catch
                | Self::Class
                | Self::Const
                | Self::Continue
                | Self::Debugger
                | Self::Default
                | Self::Delete
                | Self::Do
                | Self::Else
                | Self::Export
                | Self::Extends
                | Self::False
                | Self::Finally
                | Self::For
                | Self::Function
                | Self::If
                | Self::Import
                | Self::In
                | Self::Instanceof
                | Self::Let
                | Self::New
                | Self::Null
                | Self::Of
                | Self::Return
                | Self::Super
                | Self::Switch
                | Self::This
                | Self::Throw
                | Self::True
                | Self::Try
                | Self::Typeof
                | Self::Var
                | Self::Void
                | Self::While
                | Self::With
                | Self::Yield
                | Self::Async
                | Self::Await
                | Self::Enum
        )
    }

    /// Whether this token is an assignment operator (`=`, `+=`, etc.).
    #[must_use]
    pub const fn is_assignment_operator(self) -> bool {
        matches!(
            self,
            Self::Eq
                | Self::PlusEq
                | Self::MinusEq
                | Self::StarEq
                | Self::SlashEq
                | Self::PercentEq
                | Self::StarStarEq
                | Self::LessLessEq
                | Self::GreaterGreaterEq
                | Self::GreaterGreaterGreaterEq
                | Self::AmpEq
                | Self::PipeEq
                | Self::CaretEq
                | Self::PipePipeEq
                | Self::AmpAmpEq
                | Self::QuestionQuestionEq
        )
    }

    /// Whether this token kind is a literal (string, number, boolean, null, regex, template).
    #[must_use]
    pub const fn is_literal(self) -> bool {
        matches!(
            self,
            Self::String
                | Self::Number
                | Self::True
                | Self::False
                | Self::Null
                | Self::RegExp
                | Self::NoSubstitutionTemplate
                | Self::TemplateHead
        )
    }

    /// Whether this token is `true` or `false`.
    #[must_use]
    pub const fn is_boolean(self) -> bool {
        matches!(self, Self::True | Self::False)
    }
}

/// Look up a keyword from an identifier string.
///
/// Returns `Some(keyword_kind)` if the string is a keyword, or `None` if it's
/// a plain identifier.
#[must_use]
pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
    // Use a match on length + first char for fast rejection before full compare.
    match s {
        "as" => Some(TokenKind::As),
        "do" => Some(TokenKind::Do),
        "if" => Some(TokenKind::If),
        "in" => Some(TokenKind::In),
        "is" => Some(TokenKind::Is),
        "of" => Some(TokenKind::Of),
        "any" => Some(TokenKind::Any),
        "for" => Some(TokenKind::For),
        "get" => Some(TokenKind::Get),
        "let" => Some(TokenKind::Let),
        "new" => Some(TokenKind::New),
        "set" => Some(TokenKind::Set),
        "try" => Some(TokenKind::Try),
        "var" => Some(TokenKind::Var),
        "case" => Some(TokenKind::Case),
        "else" => Some(TokenKind::Else),
        "enum" => Some(TokenKind::Enum),
        "from" => Some(TokenKind::From),
        "null" => Some(TokenKind::Null),
        "this" => Some(TokenKind::This),
        "true" => Some(TokenKind::True),
        "type" => Some(TokenKind::Type),
        "void" => Some(TokenKind::Void),
        "with" => Some(TokenKind::With),
        "async" => Some(TokenKind::Async),
        "await" => Some(TokenKind::Await),
        "break" => Some(TokenKind::Break),
        "catch" => Some(TokenKind::Catch),
        "class" => Some(TokenKind::Class),
        "const" => Some(TokenKind::Const),
        "false" => Some(TokenKind::False),
        "infer" => Some(TokenKind::Infer),
        "keyof" => Some(TokenKind::Keyof),
        "never" => Some(TokenKind::Never),
        "super" => Some(TokenKind::Super),
        "throw" => Some(TokenKind::Throw),
        "while" => Some(TokenKind::While),
        "using" => Some(TokenKind::Using),
        "yield" => Some(TokenKind::Yield),
        "delete" => Some(TokenKind::Delete),
        "export" => Some(TokenKind::Export),
        "import" => Some(TokenKind::Import),
        "module" => Some(TokenKind::Module),
        "public" => Some(TokenKind::Public),
        "return" => Some(TokenKind::Return),
        "static" => Some(TokenKind::Static),
        "switch" => Some(TokenKind::Switch),
        "typeof" => Some(TokenKind::Typeof),
        "unique" => Some(TokenKind::Unique),
        "assert" | "asserts" => Some(TokenKind::Asserts),
        "default" => Some(TokenKind::Default),
        "declare" => Some(TokenKind::Declare),
        "extends" => Some(TokenKind::Extends),
        "finally" => Some(TokenKind::Finally),
        "package" => Some(TokenKind::Package),
        "private" => Some(TokenKind::Private),
        "unknown" => Some(TokenKind::Unknown),
        "abstract" => Some(TokenKind::Abstract),
        "continue" => Some(TokenKind::Continue),
        "debugger" => Some(TokenKind::Debugger),
        "function" => Some(TokenKind::Function),
        "override" => Some(TokenKind::Override),
        "readonly" => Some(TokenKind::Readonly),
        "interface" => Some(TokenKind::Interface),
        "namespace" => Some(TokenKind::Namespace),
        "protected" => Some(TokenKind::Protected),
        "satisfies" => Some(TokenKind::Satisfies),
        "implements" => Some(TokenKind::Implements),
        "instanceof" => Some(TokenKind::Instanceof),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, keyword_from_str};

    #[test]
    fn keywords_recognized() {
        assert_eq!(keyword_from_str("const"), Some(TokenKind::Const));
        assert_eq!(keyword_from_str("async"), Some(TokenKind::Async));
        assert_eq!(keyword_from_str("instanceof"), Some(TokenKind::Instanceof));
    }

    #[test]
    fn non_keywords() {
        assert_eq!(keyword_from_str("foo"), None);
        assert_eq!(keyword_from_str("myVar"), None);
    }

    #[test]
    fn assignment_operators() {
        assert!(TokenKind::Eq.is_assignment_operator());
        assert!(TokenKind::PlusEq.is_assignment_operator());
        assert!(!TokenKind::Plus.is_assignment_operator());
    }
}
