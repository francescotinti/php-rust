/// Non-fatal diagnostics raised by operators/conversions. The message is the
/// bare text; the evaluator prepends severity and appends " in <file> on line
/// <n>" (display format: main/main.c:1493).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diag {
    Warning(String),
    Deprecated(String),
    Notice(String),
}

/// Fatal (throwable) errors raised by operators. Uncaught display format:
/// Zend/zend_exceptions.c:756.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhpError {
    /// The base `Error` class (e.g. "Call to undefined function f()").
    Error(String),
    TypeError(String),
    DivisionByZeroError(&'static str),
    ArithmeticError(&'static str),
}

impl PhpError {
    pub fn class_name(&self) -> &'static str {
        match self {
            PhpError::Error(_) => "Error",
            PhpError::TypeError(_) => "TypeError",
            PhpError::DivisionByZeroError(_) => "DivisionByZeroError",
            PhpError::ArithmeticError(_) => "ArithmeticError",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            PhpError::Error(m) => m,
            PhpError::TypeError(m) => m,
            PhpError::DivisionByZeroError(m) => m,
            PhpError::ArithmeticError(m) => m,
        }
    }
}

pub type Diags = Vec<Diag>;
