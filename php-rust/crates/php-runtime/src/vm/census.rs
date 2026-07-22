//! Op-census instrumentation (WP-33 T0): dynamic op/bigram/type-pair
//! frequency counters that drive the specializing-interpreter arc.
//! OFF by default; enabled with `PHPR_OP_CENSUS=1`. Counting only — no
//! observable behavior change; the dump goes to STDERR so stdout parity
//! is inert even if accidentally enabled during a gate.
//!
//! The counters live in a thread-local (not on `Vm`) so the record hook
//! can read the operand stack without a split-borrow fight.

use std::cell::RefCell;

use crate::bytecode::Op;
use crate::hir::BinOp;
use php_types::Zval;

pub const N_OPS: usize = 176;

pub const OP_NAMES: [&str; N_OPS] = [
    "PushConst", "Pop", "Dup", "LoadSlot", "LoadVar", "PushUndef", "StoreSlot", "Swap",
    "LoadGlobal", "StoreGlobal", "IncDecGlobal", "LoadSuperglobal", "StoreSuperglobal", "IncDecSuperglobal", "FetchDimList", "LoadGlobals",
    "GlobalsDynAssign", "FillDefault", "CoerceParam", "CheckArity", "IncDecSlot", "BindRef", "StaticGuard", "StaticStore",
    "StaticAlias", "PushRef", "MakeRef", "PushArgPlace", "BindRefTo", "BindRefToChecked", "DerefTop", "MakeClosure",
    "MakeFcc", "CallValue", "CallNsFallback", "CallValueArgs", "CallNsFallbackArgs", "Throw", "Rethrow", "CatchMatch",
    "EndFinally", "ParkReturn", "ParkJump", "Binary", "Unary", "Cast", "Jump", "JumpIfFalse",
    "JumpIfTrue", "CmpJmp", "JumpIfNotNull", "JumpIfNull", "Echo", "Print", "Stringify", "ArrayInit",
    "ArrayPush", "ArrayInsert", "ArrayAppendSpread", "CallArgs", "FetchDim", "CoalesceFetchDim", "AssignPath", "AssignOpPath",
    "IncDecPath", "IssetPath", "EmptyPath", "UnsetPath", "Call", "DeclareFn", "DeclareClass", "DeclareTrait",
    "DeclareDeferred", "NewAnonDeferred", "CallBuiltin", "CallBuiltinSpread", "CallHostBuiltin", "CallHostBuiltinRef", "CallHostBuiltinOut", "CallHostBuiltinScanf",
    "CallArrayMultisort", "ConstFetch", "DefineConst", "CallBuiltinRef", "CallBuiltinRefSpread", "CallBuiltinRefCell", "Ret", "Yield",
    "YieldFrom", "IterInit", "IterNext", "IterInitRef", "IterNextRef", "IterPop", "Alloc", "This",
    "Clone", "Eval", "Include", "PropGet", "PropSet", "PropOpSet", "PropIncDec", "PropIsset",
    "PropIssetFetchGate", "PropIssetDyn", "LoadVarDyn", "StoreVarDyn", "BindGlobalDyn", "ClassConstDynamic", "PropGetSilent", "PropGetDynamic",
    "PropGetDynamicSilent", "MatchError", "PropUnset", "MethodCall", "MethodCallArgs", "MethodCallDynamic", "MethodCallDynamicArgs", "MethodCallNamed",
    "CallNamed", "CallSpread", "InvokeMethod", "InstanceOf", "InstanceOfStatic", "InstanceOfDynamic", "InstanceOfBuiltin", "StaticCall",
    "HookCall", "ClosureStatic", "StaticCallArgs", "StaticCallDynamic", "StaticCallDynamicArgs", "StaticCallDynamicMethod", "StaticCallTargetDynamicMethod", "StaticPropGetDynName",
    "StaticPropSetDynName", "StaticCallDynamicMethodArgs", "StaticCallTargetDynamicMethodArgs", "ClassConst", "ClassConstDyn", "ClassConstFromValue", "EnumCase", "ClassNameStatic",
    "ClassNameScope", "AllocStatic", "AllocDynamic", "InvokeCtor", "InvokeCtorArgs", "InitProps", "StampThrowable", "StaticPropGet",
    "StaticPropSet", "StaticPropRef", "StaticPropOpSet", "StaticPropIncDec", "StaticPropGetDynamic", "StaticPropSetDynamic", "StaticPropOpSetDynamic", "StaticPropIncDecDynamic",
    "FieldAssign", "FieldAssignOp", "FieldIncDec", "FieldIsset", "FieldEmpty", "FieldUnset", "Fatal", "EmitNotice",
    "Exit", "SuppressBegin", "SuppressEnd", "Sweep", "ThisPropGet", "CmpJmpConst", "ConcatN", "Nop",
];

pub fn op_index(op: &Op) -> usize {
    match op {
        Op::PushConst(..) => 0,
        Op::Pop => 1,
        Op::Dup => 2,
        Op::LoadSlot(..) => 3,
        Op::LoadVar { .. } => 4,
        Op::PushUndef => 5,
        Op::StoreSlot(..) => 6,
        Op::Swap => 7,
        Op::LoadGlobal(..) => 8,
        Op::StoreGlobal(..) => 9,
        Op::IncDecGlobal { .. } => 10,
        Op::LoadSuperglobal(..) => 11,
        Op::StoreSuperglobal(..) => 12,
        Op::IncDecSuperglobal { .. } => 13,
        Op::FetchDimList => 14,
        Op::LoadGlobals => 15,
        Op::GlobalsDynAssign => 16,
        Op::FillDefault { .. } => 17,
        Op::CoerceParam { .. } => 18,
        Op::CheckArity { .. } => 19,
        Op::IncDecSlot { .. } => 20,
        Op::BindRef { .. } => 21,
        Op::StaticGuard { .. } => 22,
        Op::StaticStore { .. } => 23,
        Op::StaticAlias { .. } => 24,
        Op::PushRef(..) => 25,
        Op::MakeRef { .. } => 26,
        Op::PushArgPlace { .. } => 27,
        Op::BindRefTo { .. } => 28,
        Op::BindRefToChecked { .. } => 29,
        Op::DerefTop => 30,
        Op::MakeClosure { .. } => 31,
        Op::MakeFcc { .. } => 32,
        Op::CallValue { .. } => 33,
        Op::CallNsFallback { .. } => 34,
        Op::CallValueArgs => 35,
        Op::CallNsFallbackArgs { .. } => 36,
        Op::Throw => 37,
        Op::Rethrow => 38,
        Op::CatchMatch { .. } => 39,
        Op::EndFinally { .. } => 40,
        Op::ParkReturn => 41,
        Op::ParkJump(..) => 42,
        Op::Binary(..) => 43,
        Op::Unary(..) => 44,
        Op::Cast(..) => 45,
        Op::Jump(..) => 46,
        Op::JumpIfFalse(..) => 47,
        Op::JumpIfTrue(..) => 48,
        Op::CmpJmp { .. } => 49,
        Op::JumpIfNotNull(..) => 50,
        Op::JumpIfNull(..) => 51,
        Op::Echo => 52,
        Op::Print => 53,
        Op::Stringify => 54,
        Op::ArrayInit => 55,
        Op::ArrayPush => 56,
        Op::ArrayInsert => 57,
        Op::ArrayAppendSpread => 58,
        Op::CallArgs { .. } => 59,
        Op::FetchDim => 60,
        Op::CoalesceFetchDim => 61,
        Op::AssignPath { .. } => 62,
        Op::AssignOpPath { .. } => 63,
        Op::IncDecPath { .. } => 64,
        Op::IssetPath { .. } => 65,
        Op::EmptyPath { .. } => 66,
        Op::UnsetPath { .. } => 67,
        Op::Call { .. } => 68,
        Op::DeclareFn { .. } => 69,
        Op::DeclareClass { .. } => 70,
        Op::DeclareTrait { .. } => 71,
        Op::DeclareDeferred { .. } => 72,
        Op::NewAnonDeferred { .. } => 73,
        Op::CallBuiltin { .. } => 74,
        Op::CallBuiltinSpread { .. } => 75,
        Op::CallHostBuiltin { .. } => 76,
        Op::CallHostBuiltinRef { .. } => 77,
        Op::CallHostBuiltinOut { .. } => 78,
        Op::CallHostBuiltinScanf { .. } => 79,
        Op::CallArrayMultisort { .. } => 80,
        Op::ConstFetch { .. } => 81,
        Op::DefineConst { .. } => 82,
        Op::CallBuiltinRef { .. } => 83,
        Op::CallBuiltinRefSpread { .. } => 84,
        Op::CallBuiltinRefCell { .. } => 85,
        Op::Ret => 86,
        Op::Yield { .. } => 87,
        Op::YieldFrom => 88,
        Op::IterInit => 89,
        Op::IterNext { .. } => 90,
        Op::IterInitRef(..) => 91,
        Op::IterNextRef { .. } => 92,
        Op::IterPop => 93,
        Op::Alloc { .. } => 94,
        Op::This => 95,
        Op::Clone => 96,
        Op::Eval => 97,
        Op::Include { .. } => 98,
        Op::PropGet { .. } => 99,
        Op::PropSet { .. } => 100,
        Op::PropOpSet { .. } => 101,
        Op::PropIncDec { .. } => 102,
        Op::PropIsset { .. } => 103,
        Op::PropIssetFetchGate { .. } => 104,
        Op::PropIssetDyn => 105,
        Op::LoadVarDyn => 106,
        Op::StoreVarDyn => 107,
        Op::BindGlobalDyn => 108,
        Op::ClassConstDynamic => 109,
        Op::PropGetSilent { .. } => 110,
        Op::PropGetDynamic => 111,
        Op::PropGetDynamicSilent => 112,
        Op::MatchError(..) => 113,
        Op::PropUnset { .. } => 114,
        Op::MethodCall { .. } => 115,
        Op::MethodCallArgs { .. } => 116,
        Op::MethodCallDynamic { .. } => 117,
        Op::MethodCallDynamicArgs => 118,
        Op::MethodCallNamed { .. } => 119,
        Op::CallNamed { .. } => 120,
        Op::CallSpread { .. } => 121,
        Op::InvokeMethod { .. } => 122,
        Op::InstanceOf { .. } => 123,
        Op::InstanceOfStatic => 124,
        Op::InstanceOfDynamic => 125,
        Op::InstanceOfBuiltin(..) => 126,
        Op::StaticCall { .. } => 127,
        Op::HookCall { .. } => 128,
        Op::ClosureStatic { .. } => 129,
        Op::StaticCallArgs { .. } => 130,
        Op::StaticCallDynamic { .. } => 131,
        Op::StaticCallDynamicArgs { .. } => 132,
        Op::StaticCallDynamicMethod { .. } => 133,
        Op::StaticCallTargetDynamicMethod { .. } => 134,
        Op::StaticPropGetDynName => 135,
        Op::StaticPropSetDynName => 136,
        Op::StaticCallDynamicMethodArgs => 137,
        Op::StaticCallTargetDynamicMethodArgs { .. } => 138,
        Op::ClassConst { .. } => 139,
        Op::ClassConstDyn { .. } => 140,
        Op::ClassConstFromValue { .. } => 141,
        Op::EnumCase { .. } => 142,
        Op::ClassNameStatic => 143,
        Op::ClassNameScope { .. } => 144,
        Op::AllocStatic => 145,
        Op::AllocDynamic => 146,
        Op::InvokeCtor { .. } => 147,
        Op::InvokeCtorArgs => 148,
        Op::InitProps => 149,
        Op::StampThrowable => 150,
        Op::StaticPropGet { .. } => 151,
        Op::StaticPropSet { .. } => 152,
        Op::StaticPropRef { .. } => 153,
        Op::StaticPropOpSet { .. } => 154,
        Op::StaticPropIncDec { .. } => 155,
        Op::StaticPropGetDynamic { .. } => 156,
        Op::StaticPropSetDynamic { .. } => 157,
        Op::StaticPropOpSetDynamic { .. } => 158,
        Op::StaticPropIncDecDynamic { .. } => 159,
        Op::FieldAssign { .. } => 160,
        Op::FieldAssignOp { .. } => 161,
        Op::FieldIncDec { .. } => 162,
        Op::FieldIsset { .. } => 163,
        Op::FieldEmpty { .. } => 164,
        Op::FieldUnset { .. } => 165,
        Op::Fatal(..) => 166,
        Op::EmitNotice(..) => 167,
        Op::Exit { .. } => 168,
        Op::SuppressBegin => 169,
        Op::SuppressEnd => 170,
        Op::Sweep { .. } => 171,
        Op::ThisPropGet { .. } => 172,
        // CmpJmpConst keeps its inline operand off the stack, so it is
        // counted (ops/bigram) but NOT folded into the Binary type-pair
        // matrix — the stack peek would misattribute the pair.
        Op::CmpJmpConst { .. } => 173,
        Op::ConcatN(..) => 174,
        Op::Nop => 175,
    }
}

/// Zval tag buckets for the type-pair matrices.
pub const TAG_NAMES: [&str; 10] =
    ["Null", "Undef", "Bool", "Long", "Double", "Str", "Array", "ObjLike", "Ref", "ArgPlace"];

pub fn tag_index(v: &Zval) -> usize {
    match v {
        Zval::Null => 0,
        Zval::Undef => 1,
        Zval::Bool(_) => 2,
        Zval::Long(_) => 3,
        Zval::Double(_) => 4,
        Zval::Str(_) => 5,
        Zval::Array(_) => 6,
        Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_) | Zval::Resource(_)
        | Zval::WeakHandle(_) => 7,
        Zval::Ref(_) => 8,
        Zval::ArgPlace(_) => 9,
    }
}

pub const N_BINOPS: usize = 21;

pub const BINOP_NAMES: [&str; N_BINOPS] = [
    "Add", "Sub", "Mul", "Div", "Mod", "Pow", "Concat", "BitAnd", "BitOr", "BitXor", "Shl",
    "Shr", "Eq", "NotEq", "Identical", "NotIdentical", "Lt", "Le", "Gt", "Ge", "Spaceship",
];

pub fn binop_index(b: BinOp) -> usize {
    match b {
        BinOp::Add => 0,
        BinOp::Sub => 1,
        BinOp::Mul => 2,
        BinOp::Div => 3,
        BinOp::Mod => 4,
        BinOp::Pow => 5,
        BinOp::Concat => 6,
        BinOp::BitAnd => 7,
        BinOp::BitOr => 8,
        BinOp::BitXor => 9,
        BinOp::Shl => 10,
        BinOp::Shr => 11,
        BinOp::Eq => 12,
        BinOp::NotEq => 13,
        BinOp::Identical => 14,
        BinOp::NotIdentical => 15,
        BinOp::Lt => 16,
        BinOp::Le => 17,
        BinOp::Gt => 18,
        BinOp::Ge => 19,
        BinOp::Spaceship => 20,
    }
}

pub struct OpCensus {
    ops: Box<[u64; N_OPS]>,
    /// bigram[prev * N_OPS + cur] — the fusion oracle.
    bigram: Box<[u64]>,
    /// binary[binop * 100 + lhs_tag * 10 + rhs_tag] for Binary AND CmpJmp.
    binary: Box<[u64; N_BINOPS * 100]>,
    /// fetchdim[base_tag * 10 + key_tag] for FetchDim + CoalesceFetchDim.
    fetchdim: [u64; 100],
    /// incdec[slot_tag] for IncDecSlot.
    incdec: [u64; 10],
    last: usize,
}

impl OpCensus {
    fn new() -> Self {
        OpCensus {
            ops: vec![0u64; N_OPS].into_boxed_slice().try_into().unwrap(),
            bigram: vec![0u64; N_OPS * N_OPS].into_boxed_slice(),
            binary: vec![0u64; N_BINOPS * 100].into_boxed_slice().try_into().unwrap(),
            fetchdim: [0; 100],
            incdec: [0; 10],
            last: N_OPS - 1, // Nop: harmless first-bigram seed
        }
    }

    /// Record one dispatched op. `stack` is the current frame's operand
    /// stack (peeked defensively), `slots` its locals.
    pub fn record(&mut self, op: &Op, stack: &[Zval], slots: &[Zval]) {
        let i = op_index(op);
        self.ops[i] += 1;
        self.bigram[self.last * N_OPS + i] += 1;
        self.last = i;
        match op {
            Op::Binary(b) | Op::CmpJmp { op: b, .. } => {
                if stack.len() >= 2 {
                    let l = tag_index(&stack[stack.len() - 2]);
                    let r = tag_index(&stack[stack.len() - 1]);
                    self.binary[binop_index(*b) * 100 + l * 10 + r] += 1;
                }
            }
            Op::FetchDim | Op::CoalesceFetchDim => {
                if stack.len() >= 2 {
                    let base = tag_index(&stack[stack.len() - 2]);
                    let key = tag_index(&stack[stack.len() - 1]);
                    self.fetchdim[base * 10 + key] += 1;
                }
            }
            Op::IncDecSlot { slot, .. } => {
                if let Some(v) = slots.get(*slot as usize) {
                    self.incdec[tag_index(v)] += 1;
                }
            }
            _ => {}
        }
    }

    fn render(&self) -> String {
        use std::fmt::Write as _;
        let mut o = String::with_capacity(8192);
        let total: u64 = self.ops.iter().sum();
        let _ = writeln!(o, "== PHPR_OP_CENSUS: {total} ops dispatched ==");
        let mut idx: Vec<usize> = (0..N_OPS).collect();
        idx.sort_by_key(|&i| std::cmp::Reverse(self.ops[i]));
        let _ = writeln!(o, "-- op counts (top 40) --");
        for &i in idx.iter().take(40) {
            if self.ops[i] == 0 {
                break;
            }
            let _ = writeln!(
                o,
                "{:>12}  {:5.2}%  {}",
                self.ops[i],
                self.ops[i] as f64 * 100.0 / total as f64,
                OP_NAMES[i]
            );
        }
        let _ = writeln!(o, "-- bigrams (top 40) --");
        let mut bi: Vec<(u64, usize)> = self
            .bigram
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(i, &c)| (c, i))
            .collect();
        bi.sort_unstable_by(|a, b| b.cmp(a));
        for &(c, i) in bi.iter().take(40) {
            let _ = writeln!(o, "{:>12}  {} -> {}", c, OP_NAMES[i / N_OPS], OP_NAMES[i % N_OPS]);
        }
        let _ = writeln!(o, "-- Binary/CmpJmp type pairs (top 40) --");
        let mut pairs: Vec<(u64, usize)> = self
            .binary
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(i, &c)| (c, i))
            .collect();
        pairs.sort_unstable_by(|a, b| b.cmp(a));
        for &(c, i) in pairs.iter().take(40) {
            let _ = writeln!(
                o,
                "{:>12}  {} ({}, {})",
                c,
                BINOP_NAMES[i / 100],
                TAG_NAMES[(i / 10) % 10],
                TAG_NAMES[i % 10]
            );
        }
        let _ = writeln!(o, "-- FetchDim base x key --");
        for (i, &c) in self.fetchdim.iter().enumerate() {
            if c > 0 {
                let _ = writeln!(o, "{:>12}  base={} key={}", c, TAG_NAMES[i / 10], TAG_NAMES[i % 10]);
            }
        }
        let _ = writeln!(o, "-- IncDecSlot slot tags --");
        for (i, &c) in self.incdec.iter().enumerate() {
            if c > 0 {
                let _ = writeln!(o, "{:>12}  {}", c, TAG_NAMES[i]);
            }
        }
        o
    }
}

thread_local! {
    static CENSUS: RefCell<Option<Box<OpCensus>>> = const { RefCell::new(None) };
}

/// Arm the census for this run if `PHPR_OP_CENSUS` is set; returns whether
/// the per-tick hook should record (hoisted into a `Vm` bool). Without the
/// `op-census` feature this is a constant `false` and the run_loop hook is
/// compiled out entirely (a dead per-tick branch costs ~3% on op-dense
/// workloads — measured 5-pair interleaved A/B, WP-33 C1).
#[cfg(not(feature = "op-census"))]
#[inline(always)]
pub fn census_arm() -> bool {
    false
}

#[cfg(feature = "op-census")]
pub fn census_arm() -> bool {
    if std::env::var_os("PHPR_OP_CENSUS").is_none() {
        return false;
    }
    CENSUS.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_none() {
            *c = Some(Box::new(OpCensus::new()));
        }
    });
    true
}

/// Record one op (census-on path only).
#[cfg(feature = "op-census")]
#[cold]
pub fn census_record(op: &Op, stack: &[Zval], slots: &[Zval]) {
    CENSUS.with(|c| {
        if let Some(census) = c.borrow_mut().as_deref_mut() {
            census.record(op, stack, slots);
        }
    });
}

/// Dump and clear at end of run. `PHPR_OP_CENSUS=1` prints on STDERR;
/// an absolute-path value appends to that file instead — needed when the
/// workload spawns phpr subprocesses that inherit the env (a stderr dump
/// from a child would pollute output a test harness captures and asserts
/// on, e.g. PHPUnit separate-process tests).
pub fn census_dump() {
    let report = match CENSUS.with(|c| c.borrow_mut().take()) {
        Some(census) => census.render(),
        None => return,
    };
    match std::env::var("PHPR_OP_CENSUS") {
        Ok(path) if path.starts_with('/') => {
            use std::io::Write as _;
            if let Ok(mut f) =
                std::fs::OpenOptions::new().create(true).append(true).open(&path)
            {
                let _ = f.write_all(report.as_bytes());
            }
        }
        _ => eprint!("{report}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_index_maps_sampled_variants_in_range() {
        // A sample across the whole numbering: first, payload-carrying,
        // struct-like, and last variants. Every index must be < N_OPS and
        // agree with its OP_NAMES row.
        let samples: Vec<(Op, &str)> = vec![
            (Op::Pop, "Pop"),
            (Op::Dup, "Dup"),
            (Op::PushUndef, "PushUndef"),
            (Op::Swap, "Swap"),
            (Op::FetchDim, "FetchDim"),
            (Op::CoalesceFetchDim, "CoalesceFetchDim"),
            (Op::Echo, "Echo"),
            (Op::Ret, "Ret"),
            (Op::This, "This"),
            (Op::ArrayInit, "ArrayInit"),
            (Op::DerefTop, "DerefTop"),
            (Op::Nop, "Nop"),
        ];
        for (op, name) in &samples {
            let i = op_index(op);
            assert!(i < N_OPS, "{name} index {i} out of range");
            assert_eq!(OP_NAMES[i], *name, "OP_NAMES row mismatch for {name}");
        }
        // Nop is the last variant by construction.
        assert_eq!(op_index(&Op::Nop), N_OPS - 1);
    }

    #[test]
    fn op_names_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for n in OP_NAMES {
            assert!(seen.insert(n), "duplicate OP_NAMES entry: {n}");
        }
    }

    #[test]
    fn tag_index_buckets() {
        assert_eq!(tag_index(&Zval::Null), 0);
        assert_eq!(TAG_NAMES[tag_index(&Zval::Null)], "Null");
        assert_eq!(tag_index(&Zval::Undef), 1);
        assert_eq!(tag_index(&Zval::Bool(true)), 2);
        assert_eq!(tag_index(&Zval::Long(42)), 3);
        assert_eq!(TAG_NAMES[tag_index(&Zval::Long(42))], "Long");
        assert_eq!(tag_index(&Zval::Double(1.5)), 4);
        let s = Zval::Str(php_types::PhpStr::new(b"x".to_vec()));
        assert_eq!(tag_index(&s), 5);
        assert_eq!(TAG_NAMES[tag_index(&s)], "Str");
        let a = Zval::Array(std::rc::Rc::new(php_types::PhpArray::new()));
        assert_eq!(tag_index(&a), 6);
        assert_eq!(TAG_NAMES.len(), 10);
    }

    #[test]
    fn binop_index_covers_names() {
        assert_eq!(BINOP_NAMES.len(), N_BINOPS);
        assert_eq!(BINOP_NAMES[binop_index(BinOp::Add)], "Add");
        assert_eq!(BINOP_NAMES[binop_index(BinOp::Concat)], "Concat");
        assert!(binop_index(BinOp::Spaceship) < N_BINOPS);
    }

    #[test]
    fn record_counts_ops_and_binary_matrix() {
        let mut c = OpCensus::new();
        let stack = vec![Zval::Long(1), Zval::Long(2)];
        c.record(&Op::Binary(BinOp::Add), &stack, &[]);
        c.record(&Op::Pop, &stack, &[]);
        assert_eq!(c.ops[op_index(&Op::Binary(BinOp::Add))], 1);
        assert_eq!(c.ops[op_index(&Op::Pop)], 1);
        // bigram Binary→Pop registered
        let bi = op_index(&Op::Binary(BinOp::Add)) * N_OPS + op_index(&Op::Pop);
        assert_eq!(c.bigram[bi], 1);
        // binary matrix: Add with Long(l)×Long(r) — lhs is stack[len-2]
        let cell = binop_index(BinOp::Add) * 100 + 3 * 10 + 3;
        assert_eq!(c.binary[cell], 1);
    }
}
