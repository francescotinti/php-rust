/// Non-fatal diagnostics raised by operators/conversions. The message is the
/// bare text; the evaluator prepends severity and appends " in <file> on line
/// <n>" (display format: main/main.c:1493).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diag {
    Warning(String),
    Deprecated(String),
    Notice(String),
}

impl Diag {
    /// The severity label PHP prints before the message (`main/main.c:1480`,
    /// `error_type_to_string`): e.g. `Warning`, `Deprecated`, `Notice`.
    pub fn severity(&self) -> &'static str {
        match self {
            Diag::Warning(_) => "Warning",
            Diag::Deprecated(_) => "Deprecated",
            Diag::Notice(_) => "Notice",
        }
    }

    /// The bare diagnostic text (no severity, no location).
    pub fn message(&self) -> &str {
        match self {
            Diag::Warning(m) | Diag::Deprecated(m) | Diag::Notice(m) => m,
        }
    }
}

/// Fatal (throwable) errors raised by operators, plus user `throw` (step 20).
/// Uncaught display format: Zend/zend_exceptions.c:756.
///
/// `PartialEq`/`Eq` are intentionally not derived: the [`PhpError::Thrown`]
/// payload is a [`Zval`], which is not comparable (it carries `f64`). Nothing in
/// the tree compares `PhpError` values directly.
#[derive(Debug, Clone)]
pub enum PhpError {
    /// The base `Error` class (e.g. "Call to undefined function f()").
    Error(String),
    TypeError(String),
    ValueError(String),
    /// Subclass of TypeError; the class name is still "ArgumentCountError".
    ArgumentCountError(String),
    DivisionByZeroError(&'static str),
    ArithmeticError(&'static str),
    /// A user-`throw`n object unwinding the stack (step 20). Carries the thrown
    /// [`Zval::Object`]; caught by a matching `catch`, otherwise rendered as an
    /// uncaught fatal. The class name / message come from the object itself, so
    /// the `class_name`/`message` accessors return sentinels for this variant.
    Thrown(crate::Zval),
}

impl PhpError {
    /// The throwable class name, for the engine-error variants. [`PhpError::Thrown`]
    /// returns a sentinel — the real class is read from the object at the render
    /// site (`Evaluator::render_fatal`), never through here.
    pub fn class_name(&self) -> &'static str {
        match self {
            PhpError::Error(_) => "Error",
            PhpError::TypeError(_) => "TypeError",
            PhpError::ValueError(_) => "ValueError",
            PhpError::ArgumentCountError(_) => "ArgumentCountError",
            PhpError::DivisionByZeroError(_) => "DivisionByZeroError",
            PhpError::ArithmeticError(_) => "ArithmeticError",
            PhpError::Thrown(_) => "Exception",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            PhpError::Error(m) => m,
            PhpError::TypeError(m) => m,
            PhpError::ValueError(m) => m,
            PhpError::ArgumentCountError(m) => m,
            PhpError::DivisionByZeroError(m) => m,
            PhpError::ArithmeticError(m) => m,
            PhpError::Thrown(_) => "",
        }
    }
}

pub type Diags = Vec<Diag>;
