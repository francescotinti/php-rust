//! ext/tokenizer: `token_get_all` / `token_name`, built on mago's lexer.
//!
//! mago (phpr's PHP front-end) already lexes PHP into a `Token { kind, start,
//! value }` stream. We run that lexer and map each `TokenKind` to PHP's tokenizer
//! output: either a single-character string (single-byte operators) or a
//! `[T_id, text, line]` triple. Line numbers are counted from the byte offset
//! (`Position` only stores an offset).
//!
//! String interpolation / heredoc interiors and `yield from` merging tokenize
//! differently in mago than in PHP's re2c scanner — those are a documented
//! phase-2 gap (see PHPR_DIVERGENCES / php-rust-tokenizer-plan).

use std::borrow::Cow;
use std::rc::Rc;

use mago_database::file::File;
use mago_syntax::lexer::Lexer;
use mago_syntax::settings::LexerSettings;
use mago_syntax::token::{DocumentKind, TokenKind};
use mago_syntax_core::input::Input;
use php_types::{convert, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

impl Vm<'_> {
    /// `token_get_all(string $code, int $flags = 0): array`.
    pub(super) fn ho_token_get_all(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let code = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags);
        let mut arr = PhpArray::new();
        for e in token_get_all(code.as_bytes()) {
            match e.id {
                None => {
                    let _ = arr.append(Zval::Str(PhpStr::new(e.text)));
                }
                Some(id) => {
                    let mut inner = PhpArray::new();
                    let _ = inner.append(Zval::Long(id as i64));
                    let _ = inner.append(Zval::Str(PhpStr::new(e.text)));
                    let _ = inner.append(Zval::Long(e.line as i64));
                    let _ = arr.append(Zval::Array(Rc::new(inner)));
                }
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `token_name(int $id): string`.
    pub(super) fn ho_token_name(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags);
        Ok(Zval::Str(PhpStr::from_str(token_name(id))))
    }
}

/// One `token_get_all` element: a single-char token (`id == None`, `text` is the
/// one byte) or a `[id, text, line]` triple.
pub struct Entry {
    pub id: Option<u32>,
    pub text: Vec<u8>,
    pub line: u32,
}

/// How a mago `TokenKind` renders in `token_get_all`.
enum Map {
    /// Emit the raw value as a single-character string (single-byte operators).
    Char,
    /// Emit `[id, text, line]`.
    Id(u32),
}

/// 1-based line of `offset` (PHP token line = line of the token's first byte).
fn line_of(src: &[u8], offset: usize) -> u32 {
    1 + src[..offset.min(src.len())].iter().filter(|&&b| b == b'\n').count() as u32
}

/// Tokenize `src` into PHP `token_get_all` entries.
pub fn token_get_all(src: &[u8]) -> Vec<Entry> {
    let file = File::ephemeral(Cow::Borrowed(b"tokenizer".as_slice()), Cow::Owned(src.to_vec()));
    let input = Input::from_file(&file);
    let mut lexer = Lexer::new(input, LexerSettings::default());
    let mut out = Vec::new();
    while let Some(res) = lexer.advance() {
        let tok = match res {
            Ok(t) => t,
            Err(_) => break,
        };
        let line = line_of(src, tok.start.offset as usize);
        // `namespace\Foo` is T_NAME_RELATIVE, not T_NAME_QUALIFIED.
        let mapped = if matches!(tok.kind, TokenKind::QualifiedIdentifier)
            && tok.value.starts_with(b"namespace\\")
        {
            Map::Id(264)
        } else {
            map_kind(tok.kind)
        };
        match mapped {
            Map::Char => out.push(Entry { id: None, text: tok.value.to_vec(), line }),
            Map::Id(id) => out.push(Entry { id: Some(id), text: tok.value.to_vec(), line }),
        }
    }
    merge_open_tag_whitespace(&mut out);
    classify_ampersands(&mut out);
    out
}

/// PHP 8.1 splits `&` into T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG (409) when the
/// next non-whitespace token is a variable or `...`, else
/// T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG (410). It is never a bare `&` char.
fn classify_ampersands(entries: &mut [Entry]) {
    for i in 0..entries.len() {
        if entries[i].id.is_none() && entries[i].text == b"&" {
            // Peek the next non-whitespace token.
            let mut j = i + 1;
            while j < entries.len() && entries[j].id == Some(397) {
                j += 1;
            }
            let followed = matches!(entries.get(j).and_then(|e| e.id), Some(266) | Some(404));
            entries[i].id = Some(if followed { 409 } else { 410 });
        }
    }
}

/// PHP's `T_OPEN_TAG` swallows the single whitespace byte that follows `<?php`;
/// mago emits it as a separate whitespace token. Move that one byte back into the
/// open tag (dropping the whitespace token if it becomes empty).
fn merge_open_tag_whitespace(entries: &mut Vec<Entry>) {
    let mut i = 0;
    while i + 1 < entries.len() {
        if entries[i].id == Some(394)
            && entries[i].text == b"<?php"
            && entries[i + 1].id == Some(397)
            && !entries[i + 1].text.is_empty()
        {
            let b = entries[i + 1].text.remove(0);
            entries[i].text.push(b);
            if entries[i + 1].text.is_empty() {
                entries.remove(i + 1);
            }
        }
        i += 1;
    }
}

fn map_kind(kind: TokenKind) -> Map {
    use TokenKind as K;
    let id: u32 = match kind {
        // ---- single-byte operators / punctuation → single-char string ----
        K::Semicolon | K::Comma | K::Colon | K::Question | K::At | K::Dot | K::Plus
        | K::Minus | K::Asterisk | K::Slash | K::Percent | K::Equal | K::Ampersand
        | K::Pipe | K::Caret | K::Tilde | K::Bang | K::LessThan | K::GreaterThan
        | K::LeftBrace | K::RightBrace | K::LeftBracket | K::RightBracket
        | K::LeftParenthesis | K::RightParenthesis | K::Dollar | K::Backtick
        | K::DoubleQuote => return Map::Char,

        // ---- identifiers / literals ----
        K::LiteralInteger => 260, // T_LNUMBER
        K::LiteralFloat => 261,   // T_DNUMBER
        K::Identifier
        // `self`/`parent`/`true`/`false`/`null` are T_STRING, not keywords.
        | K::Self_ | K::Parent | K::True | K::False | K::Null => 262, // T_STRING
        K::FullyQualifiedIdentifier => 263, // T_NAME_FULLY_QUALIFIED
        K::QualifiedIdentifier => 265,      // T_NAME_QUALIFIED
        K::Variable => 266,                 // T_VARIABLE
        K::InlineText | K::InlineShebang => 267, // T_INLINE_HTML
        K::StringPart => 268,               // T_ENCAPSED_AND_WHITESPACE
        K::LiteralString | K::PartialLiteralString => 269, // T_CONSTANT_ENCAPSED_STRING

        // ---- keywords ----
        K::Include => 272,
        K::IncludeOnce => 273,
        K::Eval => 274,
        K::Require => 275,
        K::RequireOnce => 276,
        K::Or => 277,   // T_LOGICAL_OR
        K::Xor => 278,  // T_LOGICAL_XOR
        K::And => 279,  // T_LOGICAL_AND
        K::Print => 280,
        K::Yield => 281,
        K::Instanceof => 283,
        K::New => 284,
        K::Clone => 285,
        K::Exit | K::Die => 286, // T_EXIT
        K::If => 287,
        K::ElseIf => 288,
        K::Else => 289,
        K::EndIf => 290,
        K::Echo => 291,
        K::Do => 292,
        K::While => 293,
        K::EndWhile => 294,
        K::For => 295,
        K::EndFor => 296,
        K::Foreach => 297,
        K::EndForeach => 298,
        K::Declare => 299,
        K::EndDeclare => 300,
        K::As => 301,
        K::Switch => 302,
        K::EndSwitch => 303,
        K::Case => 304,
        K::Default => 305,
        K::Match => 306,
        K::Break => 307,
        K::Continue => 308,
        K::Goto => 309,
        K::Function => 310,
        K::Fn => 311,
        K::Const => 312,
        K::Return => 313,
        K::Try => 314,
        K::Catch => 315,
        K::Finally => 316,
        K::Throw => 317,
        K::Use => 318,
        K::Insteadof => 319,
        K::Global => 320,
        K::Static => 321,
        K::Abstract => 322,
        K::Final => 323,
        K::Private => 324,
        K::Protected => 325,
        K::Public => 326,
        K::PrivateSet => 327,
        K::ProtectedSet => 328,
        K::PublicSet => 329,
        K::Readonly => 330,
        K::Var => 331,
        K::Unset => 332,
        K::Isset => 333,
        K::Empty => 334,
        K::HaltCompiler => 335,
        K::Class => 336,
        K::Trait => 337,
        K::Interface => 338,
        K::Enum => 339,
        K::Extends => 340,
        K::Implements => 341,
        K::Namespace => 342,
        K::List => 343,
        K::Array => 344,
        K::Callable => 345,

        // ---- magic constants ----
        K::LineConstant => 346,      // T_LINE
        K::FileConstant => 347,      // T_FILE
        K::DirConstant => 348,       // T_DIR
        K::ClassConstant => 349,     // T_CLASS_C
        K::TraitConstant => 350,     // T_TRAIT_C
        K::MethodConstant => 351,    // T_METHOD_C
        K::FunctionConstant => 352,  // T_FUNC_C
        K::PropertyConstant => 353,  // T_PROPERTY_C
        K::NamespaceConstant => 354, // T_NS_C

        K::HashLeftBracket => 355, // T_ATTRIBUTE  `#[`

        // ---- compound-assignment operators ----
        K::PlusEqual => 356,
        K::MinusEqual => 357,
        K::AsteriskEqual => 358,
        K::SlashEqual => 359,
        K::DotEqual => 360,
        K::PercentEqual => 361,
        K::AmpersandEqual => 362,
        K::PipeEqual => 363,
        K::CaretEqual => 364,
        K::LeftShiftEqual => 365,
        K::RightShiftEqual => 366,
        K::QuestionQuestionEqual => 367,
        K::AsteriskAsteriskEqual => 407, // T_POW_EQUAL

        // ---- comparison / logical / shift ----
        K::PipePipe => 368,             // T_BOOLEAN_OR
        K::AmpersandAmpersand => 369,   // T_BOOLEAN_AND
        K::EqualEqual => 370,           // T_IS_EQUAL
        K::BangEqual | K::LessThanGreaterThan => 371, // T_IS_NOT_EQUAL
        K::EqualEqualEqual => 372,      // T_IS_IDENTICAL
        K::BangEqualEqual => 373,       // T_IS_NOT_IDENTICAL
        K::LessThanEqual => 374,        // T_IS_SMALLER_OR_EQUAL
        K::GreaterThanEqual => 375,     // T_IS_GREATER_OR_EQUAL
        K::LessThanEqualGreaterThan => 376, // T_SPACESHIP
        K::LeftShift => 377,            // T_SL
        K::RightShift => 378,           // T_SR
        K::PlusPlus => 379,             // T_INC
        K::MinusMinus => 380,           // T_DEC

        // ---- casts ----
        K::IntCast | K::IntegerCast => 381,          // T_INT_CAST
        K::FloatCast | K::DoubleCast | K::RealCast => 382, // T_DOUBLE_CAST
        K::StringCast | K::BinaryCast => 383,        // T_STRING_CAST
        K::ArrayCast => 384,
        K::ObjectCast => 385,
        K::BoolCast | K::BooleanCast => 386,
        K::UnsetCast => 387,
        K::VoidCast => 388,

        // ---- object access / arrows ----
        K::MinusGreaterThan => 389,         // T_OBJECT_OPERATOR
        K::QuestionMinusGreaterThan => 390, // T_NULLSAFE_OBJECT_OPERATOR
        K::EqualGreaterThan => 391,         // T_DOUBLE_ARROW

        // ---- comments / whitespace / tags ----
        K::SingleLineComment | K::HashComment | K::MultiLineComment => 392, // T_COMMENT
        K::DocBlockComment => 393, // T_DOC_COMMENT
        K::OpenTag | K::ShortOpenTag => 394, // T_OPEN_TAG
        K::EchoTag => 395,        // T_OPEN_TAG_WITH_ECHO
        K::CloseTag => 396,       // T_CLOSE_TAG
        K::Whitespace => 397,     // T_WHITESPACE

        // ---- heredoc / interpolation ----
        K::DocumentStart(DocumentKind::Heredoc | DocumentKind::Nowdoc) => 398, // T_START_HEREDOC
        K::DocumentEnd => 399,     // T_END_HEREDOC
        K::DollarLeftBrace => 400, // T_DOLLAR_OPEN_CURLY_BRACES

        // ---- misc operators ----
        K::ColonColon => 402,        // T_DOUBLE_COLON
        K::NamespaceSeparator => 403, // T_NS_SEPARATOR
        K::DotDotDot => 404,         // T_ELLIPSIS
        K::QuestionQuestion => 405,  // T_COALESCE
        K::AsteriskAsterisk => 406,  // T_POW
        K::PipeGreaterThan => 408,   // T_PIPE

        // Fallbacks: `from` (outside `yield from`) and anything unmapped read as
        // an identifier — the closest PHP token.
        K::From => 262, // T_STRING
        _ => 262,       // T_STRING
    };
    Map::Id(id)
}

/// `token_name($id)` — the constant name for a token id, or "UNKNOWN".
pub fn token_name(id: i64) -> &'static str {
    match id {
        260 => "T_LNUMBER",
        261 => "T_DNUMBER",
        262 => "T_STRING",
        263 => "T_NAME_FULLY_QUALIFIED",
        264 => "T_NAME_RELATIVE",
        265 => "T_NAME_QUALIFIED",
        266 => "T_VARIABLE",
        267 => "T_INLINE_HTML",
        268 => "T_ENCAPSED_AND_WHITESPACE",
        269 => "T_CONSTANT_ENCAPSED_STRING",
        270 => "T_STRING_VARNAME",
        271 => "T_NUM_STRING",
        272 => "T_INCLUDE",
        273 => "T_INCLUDE_ONCE",
        274 => "T_EVAL",
        275 => "T_REQUIRE",
        276 => "T_REQUIRE_ONCE",
        277 => "T_LOGICAL_OR",
        278 => "T_LOGICAL_XOR",
        279 => "T_LOGICAL_AND",
        280 => "T_PRINT",
        281 => "T_YIELD",
        282 => "T_YIELD_FROM",
        283 => "T_INSTANCEOF",
        284 => "T_NEW",
        285 => "T_CLONE",
        286 => "T_EXIT",
        287 => "T_IF",
        288 => "T_ELSEIF",
        289 => "T_ELSE",
        290 => "T_ENDIF",
        291 => "T_ECHO",
        292 => "T_DO",
        293 => "T_WHILE",
        294 => "T_ENDWHILE",
        295 => "T_FOR",
        296 => "T_ENDFOR",
        297 => "T_FOREACH",
        298 => "T_ENDFOREACH",
        299 => "T_DECLARE",
        300 => "T_ENDDECLARE",
        301 => "T_AS",
        302 => "T_SWITCH",
        303 => "T_ENDSWITCH",
        304 => "T_CASE",
        305 => "T_DEFAULT",
        306 => "T_MATCH",
        307 => "T_BREAK",
        308 => "T_CONTINUE",
        309 => "T_GOTO",
        310 => "T_FUNCTION",
        311 => "T_FN",
        312 => "T_CONST",
        313 => "T_RETURN",
        314 => "T_TRY",
        315 => "T_CATCH",
        316 => "T_FINALLY",
        317 => "T_THROW",
        318 => "T_USE",
        319 => "T_INSTEADOF",
        320 => "T_GLOBAL",
        321 => "T_STATIC",
        322 => "T_ABSTRACT",
        323 => "T_FINAL",
        324 => "T_PRIVATE",
        325 => "T_PROTECTED",
        326 => "T_PUBLIC",
        327 => "T_PRIVATE_SET",
        328 => "T_PROTECTED_SET",
        329 => "T_PUBLIC_SET",
        330 => "T_READONLY",
        331 => "T_VAR",
        332 => "T_UNSET",
        333 => "T_ISSET",
        334 => "T_EMPTY",
        335 => "T_HALT_COMPILER",
        336 => "T_CLASS",
        337 => "T_TRAIT",
        338 => "T_INTERFACE",
        339 => "T_ENUM",
        340 => "T_EXTENDS",
        341 => "T_IMPLEMENTS",
        342 => "T_NAMESPACE",
        343 => "T_LIST",
        344 => "T_ARRAY",
        345 => "T_CALLABLE",
        346 => "T_LINE",
        347 => "T_FILE",
        348 => "T_DIR",
        349 => "T_CLASS_C",
        350 => "T_TRAIT_C",
        351 => "T_METHOD_C",
        352 => "T_FUNC_C",
        353 => "T_PROPERTY_C",
        354 => "T_NS_C",
        355 => "T_ATTRIBUTE",
        356 => "T_PLUS_EQUAL",
        357 => "T_MINUS_EQUAL",
        358 => "T_MUL_EQUAL",
        359 => "T_DIV_EQUAL",
        360 => "T_CONCAT_EQUAL",
        361 => "T_MOD_EQUAL",
        362 => "T_AND_EQUAL",
        363 => "T_OR_EQUAL",
        364 => "T_XOR_EQUAL",
        365 => "T_SL_EQUAL",
        366 => "T_SR_EQUAL",
        367 => "T_COALESCE_EQUAL",
        368 => "T_BOOLEAN_OR",
        369 => "T_BOOLEAN_AND",
        370 => "T_IS_EQUAL",
        371 => "T_IS_NOT_EQUAL",
        372 => "T_IS_IDENTICAL",
        373 => "T_IS_NOT_IDENTICAL",
        374 => "T_IS_SMALLER_OR_EQUAL",
        375 => "T_IS_GREATER_OR_EQUAL",
        376 => "T_SPACESHIP",
        377 => "T_SL",
        378 => "T_SR",
        379 => "T_INC",
        380 => "T_DEC",
        381 => "T_INT_CAST",
        382 => "T_DOUBLE_CAST",
        383 => "T_STRING_CAST",
        384 => "T_ARRAY_CAST",
        385 => "T_OBJECT_CAST",
        386 => "T_BOOL_CAST",
        387 => "T_UNSET_CAST",
        388 => "T_VOID_CAST",
        389 => "T_OBJECT_OPERATOR",
        390 => "T_NULLSAFE_OBJECT_OPERATOR",
        391 => "T_DOUBLE_ARROW",
        392 => "T_COMMENT",
        393 => "T_DOC_COMMENT",
        394 => "T_OPEN_TAG",
        395 => "T_OPEN_TAG_WITH_ECHO",
        396 => "T_CLOSE_TAG",
        397 => "T_WHITESPACE",
        398 => "T_START_HEREDOC",
        399 => "T_END_HEREDOC",
        400 => "T_DOLLAR_OPEN_CURLY_BRACES",
        401 => "T_CURLY_OPEN",
        402 => "T_DOUBLE_COLON",
        403 => "T_NS_SEPARATOR",
        404 => "T_ELLIPSIS",
        405 => "T_COALESCE",
        406 => "T_POW",
        407 => "T_POW_EQUAL",
        408 => "T_PIPE",
        409 => "T_AMPERSAND_FOLLOWED_BY_VAR_OR_VARARG",
        410 => "T_AMPERSAND_NOT_FOLLOWED_BY_VAR_OR_VARARG",
        411 => "T_BAD_CHARACTER",
        // Single-char tokens report their character; everything else UNKNOWN.
        i if (0..256).contains(&i) => "UNKNOWN",
        _ => "UNKNOWN",
    }
}
