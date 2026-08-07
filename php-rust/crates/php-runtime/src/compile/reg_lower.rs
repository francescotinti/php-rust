//! Register-lowering pass — Leva B stage 2 v3 "raw registers"
//! (doc/plans-archive/REGISTER_BYTECODE_PLAN.md §5; post-mortem v1/v2 in
//! sessions/WP_SESSION_44.md; re-armed in S-97.1 under the micro-category
//! judge — the WP-44 verdict was rendered on the diluted WordPress
//! aggregate, see wp97-harness/arith-decomposition.out).
//!
//! v3 design rule (the discriminating experiment after the enum-operand
//! hybrid measured +1.2% consistent on A/B): the run_loop must see ZERO
//! runtime operand dispatch. Every fused shape is its own MONOMORPHIC op
//! with bare u16 indices ([`Op::BinarySS`]/[`Op::BinarySSDst`]/
//! [`Op::BinarySC`]/[`Op::BinarySCDst`]/[`Op::BinaryDst`]/[`Op::CmpJmpSS`]/
//! [`Op::CmpJmpSC`]); the compiler does all the resolution here. Shapes
//! outside this set are NOT rewritten (no stack-lhs source folds, no 1:1
//! CmpJmpConst rename): the stack forms keep their existing monomorphic
//! handlers, so no polymorphism is added anywhere.
//!
//! Fold rules:
//! - sources: `LoadVar` (only when the name const is byte-identical to
//!   `slot_names[slot]`, so the fused handler re-synthesises the exact
//!   "Undefined variable" warning) and `PushConst`; `LoadSlot` (silent,
//!   cold) is never folded.
//! - const is ALWAYS rhs. A written const-lhs folds ONLY for mirrorable
//!   comparisons (Lt↔Gt, Le↔Ge; Eq-family unchanged — compares emit no
//!   coercion diags and no operand-typed errors). Spaceship is not
//!   mirrorable → no fold. **S-97.1 divergence from WP-44 v3**: the
//!   commutative-arith swap (`3 + $x` → `$x + 3`) was DROPPED — an
//!   "Unsupported operand types" TypeError names the operands in ORDER
//!   (`int + array` vs `array + int`), so the swap is observable when the
//!   slot holds a non-numeric at runtime. The corpus never caught it
//!   ("corretto per fortuna del corpus" ≠ "corretto").
//! - dst folds: `Binary, StoreSlot s` and `Binary, Dup, StoreSlot s, Pop`
//!   sink into the `*Dst` forms (net stack/slot/gc_note effect identical —
//!   the only elision is the transient duplicate, which no longer exists
//!   to note).
//!
//! Window guards (plan §3): one source line per window (diagnostic
//! parity), no jump target or exc-region boundary mid-window (the head MAY
//! be a target), folded indices fit u16. Compaction remaps every `Addr`
//! (op stream + exc table); addresses past the original length
//! (`Addr::MAX` jump-threading terminals) are preserved. `max_temps`
//! stays 0.

use crate::bytecode::{Addr, Const, Func, Op};
use crate::hir::{BinOp, UnOp};

/// CONTRATTO DI MODO di `PHPR_REG_LOWER` (S-100 punto 1 — KS-MA-101-1,
/// A-HO-101-2, A-HE-101-1, A-PE-101-4). Grafia VALUE-PARSED, lista CHIUSA:
///
///   - variabile ASSENTE   -> `DEFAULT_ON` (il flip del default cambia SOLO
///     quella costante, mai questa funzione)
///   - `PHPR_REG_LOWER=1`  -> ON  (opt-in esplicito)
///   - `PHPR_REG_LOWER=0`  -> OFF (opt-out esplicito; prima del contratto
///     `=0` ACCENDEVA il pass — `is_some()` presence-based, refutazione
///     capitale n.1 del Concilio WP-101)
///   - qualunque altro valore -> `DEFAULT_ON` + warning su stderr che nomina
///     la grammatica (mai un fallback silenzioso)
///
/// Letto UNA volta (OnceLock) e sigillato EAGER dai main via
/// `seal_reg_lower_mode()`; la unit-cache key porta il modo, così un'unità
/// compilata in un modo non può mai servire un processo nell'altro.
pub(crate) fn enabled() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| mode_from_env(std::env::var_os("PHPR_REG_LOWER").as_deref()))
}

/// Default di processo quando `PHPR_REG_LOWER` è ASSENTE. FLIPPATO a `true`
/// in S-100 punto 5 (promozione flag-on a default) coi gate del Concilio
/// WP-101 tutti verdi: corpus 1418 per NOME + diff per-test ZERO nei due
/// modi, parità server bimodale (sentinella estesa), coppia WP con bande
/// pre-registrate, batteria a modo ESPLICITO. L'opt-out è `PHPR_REG_LOWER=0`
/// (grammatica value-parsed sopra).
pub const DEFAULT_ON: bool = true;

/// La grammatica del contratto, pura e testabile senza toccare l'ambiente.
pub fn mode_from_env(raw: Option<&std::ffi::OsStr>) -> bool {
    let Some(v) = raw else { return DEFAULT_ON };
    match v.to_str() {
        Some("1") => true,
        Some("0") => false,
        _ => {
            eprintln!(
                "phpr: PHPR_REG_LOWER={v:?} fuori grammatica (accetta `1`=on, \
                 `0`=off, assente=default): uso il default"
            );
            DEFAULT_ON
        }
    }
}

/// Visit every jump address the op carries. The single authority both the
/// target-collection and the remap phase use — a new `Addr`-bearing variant
/// must be added HERE or the pass corrupts it.
///
/// A-HE-100-2 / KS-HE-101-2 (Concilio WP-101): match ESAUSTIVO, nessun
/// wildcard su `Op` — una variante NUOVA non compila finché non viene
/// classificata qui (stessa classe del dente S-96 in vm/liveness.rs). Il
/// gruppo «senza Addr» è la lista CHIUSA verificata a mano contro i payload
/// di bytecode.rs (unici campi `Addr` oggi: i 16 bracci sopra).
fn visit_addrs(op: &mut Op, f: &mut impl FnMut(&mut Addr)) {
    match op {
        Op::FillDefault { skip, .. } | Op::StaticGuard { skip, .. } => f(skip),
        Op::CatchMatch { body, .. } => f(body),
        Op::EndFinally { after } => f(after),
        Op::ParkJump(a)
        | Op::Jump(a)
        | Op::JumpIfFalse(a)
        | Op::JumpIfTrue(a)
        | Op::JumpIfNotNull(a)
        | Op::JumpIfNull(a) => f(a),
        Op::CmpJmp { addr, .. }
        | Op::CmpJmpConst { addr, .. }
        | Op::CmpJmpSS { addr, .. }
        | Op::CmpJmpSC { addr, .. }
        | Op::IncDecSlotJmp { addr, .. } => f(addr),
        Op::IterNext { end, .. } | Op::IterNextRef { end, .. } => f(end),
        // ---- lista chiusa: varianti SENZA Addr (niente da visitare) ----
        Op::PushConst { .. } | Op::Pop { .. } | Op::Dup { .. }
        | Op::LoadSlot { .. } | Op::LoadVar { .. } | Op::PushUndef { .. }
        | Op::StoreSlot { .. } | Op::ConcatAssignSlot { .. } | Op::Swap { .. }
        | Op::LoadGlobal { .. } | Op::StoreGlobal { .. } | Op::IncDecGlobal { .. }
        | Op::LoadSuperglobal { .. } | Op::StoreSuperglobal { .. }
        | Op::IncDecSuperglobal { .. } | Op::FetchDimList { .. }
        | Op::LoadGlobals { .. } | Op::GlobalsDynAssign { .. }
        | Op::CoerceParam { .. } | Op::CheckArity { .. } | Op::IncDecSlot { .. }
        | Op::BindRef { .. } | Op::StaticStore { .. } | Op::StaticAlias { .. }
        | Op::PushRef { .. } | Op::MakeRef { .. } | Op::PushArgPlace { .. }
        | Op::BindRefTo { .. } | Op::BindRefToChecked { .. } | Op::DerefTop { .. }
        | Op::MakeClosure { .. } | Op::MakeFcc { .. } | Op::CallValue { .. }
        | Op::CallNsFallback { .. } | Op::CallValueArgs { .. }
        | Op::CallNsFallbackArgs { .. } | Op::Throw { .. } | Op::Rethrow { .. }
        | Op::ParkReturn { .. } | Op::Binary { .. } | Op::BinaryAdd { .. }
        | Op::Unary { .. } | Op::Cast { .. } | Op::BinarySS { .. }
        | Op::BinarySSDst { .. } | Op::BinarySC { .. } | Op::BinarySCDst { .. }
        | Op::BinaryDst { .. } | Op::BinarySTDst { .. } | Op::BinaryTC { .. }
        | Op::BinarySCSC { .. } | Op::IncDecSlotPop { .. } | Op::PropGetSlot { .. }
        | Op::PropSetPop { .. } | Op::StringifySlot { .. }
        | Op::PropGetSlotRecv { .. } | Op::BinaryTCPropSetPop { .. }
        | Op::BinarySCSCDst { .. } | Op::LoadVarPushConst { .. }
        | Op::ConcatNConst { .. }
        | Op::ConcatN { .. } | Op::Echo { .. }
        | Op::Print { .. } | Op::Stringify { .. } | Op::ArrayInit { .. }
        | Op::ArrayPush { .. } | Op::ArrayInsert { .. }
        | Op::ArrayAppendSpread { .. } | Op::CallArgs { .. } | Op::FetchDim { .. }
        | Op::CoalesceFetchDim { .. } | Op::AssignPath { .. }
        | Op::AssignOpPath { .. } | Op::IncDecPath { .. } | Op::IssetPath { .. }
        | Op::EmptyPath { .. } | Op::UnsetPath { .. } | Op::Call { .. }
        | Op::DeclareFn { .. } | Op::DeclareClass { .. } | Op::DeclareTrait { .. }
        | Op::DeclareDeferred { .. } | Op::NewAnonDeferred { .. }
        | Op::CallBuiltin { .. } | Op::CallBuiltinSpread { .. }
        | Op::CallHostBuiltin { .. } | Op::CallHostBuiltinRef { .. }
        | Op::CallHostBuiltinOut { .. } | Op::CallHostBuiltinScanf { .. }
        | Op::CallArrayMultisort { .. } | Op::ConstFetch { .. }
        | Op::DefineConst { .. } | Op::CallBuiltinRef { .. }
        | Op::CallBuiltinRefSpread { .. } | Op::CallBuiltinRefCell { .. }
        | Op::Ret { .. } | Op::Yield { .. } | Op::YieldFrom { .. }
        | Op::IterInit { .. } | Op::IterInitRef { .. } | Op::IterPop { .. }
        | Op::Alloc { .. } | Op::This { .. } | Op::Clone { .. } | Op::Eval { .. }
        | Op::Include { .. } | Op::PropGet { .. } | Op::ThisPropGet { .. }
        | Op::PropSet { .. } | Op::PropOpSet { .. } | Op::PropIncDec { .. }
        | Op::PropIsset { .. } | Op::PropIssetFetchGate { .. }
        | Op::PropIssetDyn { .. } | Op::LoadVarDyn { .. } | Op::StoreVarDyn { .. }
        | Op::BindGlobalDyn { .. } | Op::ClassConstDynamic { .. }
        | Op::PropGetSilent { .. } | Op::PropGetDynamic { .. }
        | Op::PropGetDynamicSilent { .. } | Op::MatchError { .. }
        | Op::PropUnset { .. } | Op::MethodCall { .. } | Op::ThisMethodCall { .. }
        | Op::MethodCallArgs { .. } | Op::MethodCallDynamic { .. }
        | Op::MethodCallDynamicArgs { .. } | Op::MethodCallNamed { .. }
        | Op::CallNamed { .. } | Op::CallSpread { .. } | Op::InvokeMethod { .. }
        | Op::InstanceOf { .. } | Op::InstanceOfStatic { .. }
        | Op::InstanceOfDynamic { .. } | Op::InstanceOfBuiltin { .. }
        | Op::StaticCall { .. } | Op::HookCall { .. } | Op::ClosureStatic { .. }
        | Op::StaticCallArgs { .. } | Op::StaticCallDynamic { .. }
        | Op::StaticCallDynamicArgs { .. } | Op::StaticCallDynamicMethod { .. }
        | Op::StaticCallTargetDynamicMethod { .. }
        | Op::StaticPropGetDynName { .. } | Op::StaticPropSetDynName { .. }
        | Op::StaticCallDynamicMethodArgs { .. }
        | Op::StaticCallTargetDynamicMethodArgs { .. } | Op::ClassConst { .. }
        | Op::ClassConstDyn { .. } | Op::ClassConstFromValue { .. }
        | Op::EnumCase { .. } | Op::ClassNameStatic { .. }
        | Op::ClassNameScope { .. } | Op::AllocStatic { .. }
        | Op::AllocDynamic { .. } | Op::InvokeCtor { .. }
        | Op::InvokeCtorArgs { .. } | Op::InitProps { .. }
        | Op::StampThrowable { .. } | Op::StaticPropGet { .. }
        | Op::StaticPropSet { .. } | Op::StaticPropRef { .. }
        | Op::StaticPropOpSet { .. } | Op::StaticPropIncDec { .. }
        | Op::StaticPropGetDynamic { .. } | Op::StaticPropSetDynamic { .. }
        | Op::StaticPropOpSetDynamic { .. } | Op::StaticPropIncDecDynamic { .. }
        | Op::FieldAssign { .. } | Op::FieldAssignOp { .. }
        | Op::FieldIncDec { .. } | Op::FieldIsset { .. } | Op::FieldEmpty { .. }
        | Op::FieldUnset { .. } | Op::Fatal { .. } | Op::EmitNotice { .. }
        | Op::Exit { .. } | Op::SuppressBegin { .. } | Op::SuppressEnd { .. }
        | Op::Sweep { .. } | Op::Nop { .. } => {}
    }
}

/// A foldable `LoadVar` source: index fits u16 and the name const equals
/// `slot_names[slot]` (warning parity). `LoadSlot` is silent — never folded.
fn fold_slot(f: &Func, i: usize) -> Option<u16> {
    match &f.ops[i] {
        Op::LoadVar { slot, name } if *slot <= u16::MAX as u32 => {
            match &f.consts[*name as usize] {
                Const::Str(s)
                    if f.slot_names.get(*slot as usize).map(|n| &n[..]) == Some(s.as_bytes()) =>
                {
                    Some(*slot as u16)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// A foldable `PushConst` source (index fits u16).
fn fold_const(f: &Func, i: usize) -> Option<u16> {
    match &f.ops[i] {
        Op::PushConst(c) if *c <= u16::MAX as u32 => Some(*c as u16),
        _ => None,
    }
}

/// Mirror a comparison so its operands can swap sides (const to rhs).
/// Comparisons emit no coercion diagnostics and no operand-typed errors,
/// so the swap is unobservable. `None` = not mirrorable (Spaceship,
/// arithmetic — arithmetic is order-observable through the
/// "Unsupported operand types: <lhs> <op> <rhs>" TypeError).
fn mirror_cmp(b: BinOp) -> Option<BinOp> {
    Some(match b {
        BinOp::Eq | BinOp::NotEq | BinOp::Identical | BinOp::NotIdentical => b,
        BinOp::Lt => BinOp::Gt,
        BinOp::Le => BinOp::Ge,
        BinOp::Gt => BinOp::Lt,
        BinOp::Ge => BinOp::Le,
        _ => return None,
    })
}

/// The pass (called from `compile_body` behind [`enabled`]): scan for
/// fusable windows, rebuild `ops`/`lines`, remap every address.
pub(super) fn lower_func(f: &mut Func) {
    let n = f.ops.len();
    if n == 0 {
        return;
    }
    debug_assert_eq!(f.lines.len(), n, "lines parallel to ops");
    // Positions a window may not absorb: jump targets and exc boundaries.
    let mut blocked = vec![false; n + 1];
    {
        let mut mark = |a: Addr| {
            if (a as usize) <= n {
                blocked[a as usize] = true;
            }
        };
        for r in &f.exc_table {
            mark(r.start);
            mark(r.end);
            mark(r.target);
        }
        for op in &mut f.ops {
            visit_addrs(op, &mut |a| mark(*a));
        }
    }
    let mut new_ops: Vec<Op> = Vec::with_capacity(n);
    let mut new_lines = Vec::with_capacity(n);
    let mut map = vec![0u32; n + 1];
    let mut i = 0usize;
    while i < n {
        // S-109 F1 Neg-fold: [PushConst(c), Unary(Neg)] stessa riga, i+1 non
        // bersaglio — il const si NEGA IN TABELLA (consts-append) e resta un
        // solo PushConst. Vive QUI e non in fuse_window (che ha &Func: la
        // tabella const è mutabile solo nel driver). SOLO Int con
        // checked_neg()=Some (INT_MIN resta non fuso: la sua negazione
        // promuove a float via il funnel unario) e Float; la negazione
        // replica apply_unop_ovl a compile-time.
        if i + 1 < n && !blocked[i + 1] && f.lines[i + 1] == f.lines[i] {
            if let (Op::PushConst(c), Op::Unary(UnOp::Neg)) = (&f.ops[i], &f.ops[i + 1]) {
                let negated = match f.consts.get(*c as usize) {
                    Some(Const::Int(v)) => v.checked_neg().map(Const::Int),
                    Some(Const::Float(x)) => Some(Const::Float(-x)),
                    _ => None,
                };
                if let Some(k) = negated {
                    let nc = f.consts.len() as u32;
                    f.consts.push(k);
                    for k2 in i..i + 2 {
                        map[k2] = new_ops.len() as u32;
                    }
                    new_lines.push(f.lines[i]);
                    new_ops.push(Op::PushConst(nc));
                    i += 2;
                    continue;
                }
            }
        }
        let (op, w) = fuse_window(f, &blocked, i);
        for k in i..i + w {
            map[k] = new_ops.len() as u32;
        }
        new_lines.push(f.lines[i]);
        new_ops.push(op);
        i += w;
    }
    map[n] = new_ops.len() as u32;
    for op in &mut new_ops {
        visit_addrs(op, &mut |a| {
            if (*a as usize) <= n {
                *a = map[*a as usize];
            }
        });
    }
    for r in &mut f.exc_table {
        for a in [&mut r.start, &mut r.end, &mut r.target] {
            if (*a as usize) <= n {
                *a = map[*a as usize];
            }
        }
    }
    // S-100 A-MA-101-3, DECISIONE MISURATA (wp100-harness/hb2flip-premisura
    // .out: L=12,9 ns/occ, banda, sopra il pavimento 1,0; add.php del giudice
    // ha 1 residuo per iterazione): il flip del default non deve RITIRARE la
    // specializzazione H-B2 dai siti stack che le finestre non coprono — ogni
    // `Binary(Add)` sopravvissuto alle finestre diventa la forma
    // specializzata. Equivalenza provata da reg_lower_differential
    // (A-HE-100-3); tripwire: flag-on l'emissione non contiene MAI un
    // `Binary(Add)` generico (o forma fusa o `BinaryAdd`).
    for op in &mut new_ops {
        if matches!(op, Op::Binary(BinOp::Add)) {
            *op = Op::BinaryAdd;
        }
    }
    f.ops = new_ops;
    f.lines = new_lines;
}

/// Source shape of a Binary window, pre-resolved by the scanner.
enum BinKind {
    SS(u16, u16),
    SC(u16, u16),
    /// lhs from `LoadSlot` (silent read), rhs from the stack — the
    /// compound-assign prefix `LoadSlot(l), Swap` (S-106 leva H-A1).
    /// Fuses ONLY with an assign-and-discard tail: there is no bare
    /// `BinaryST` value form by choice (minimal lever, ha1-criterio.out).
    ST(u16),
    Stack,
}

/// The binary operator an op carries, seeing through the emission-time
/// `+` specialization (H-B2): `BinaryAdd` IS `Binary(Add)` by construction,
/// so the windows fuse both spellings — the production flag-on pipeline
/// only ever emits `Binary(Add)`, but the test battery (and any future
/// mixed pipeline) compiles with the specialized emission.
///
/// A-MA-101-2 (Concilio WP-101, salda A-MA-100-2): match ESAUSTIVO — una
/// variante NUOVA di `Op` (in particolare una futura specializzazione
/// Binary-like tipo `BinarySub`) NON COMPILA finché non viene classificata
/// qui: il ledger delle forme che le finestre fondono non decade in silenzio.
fn bin_op_of(op: &Op) -> Option<BinOp> {
    match op {
        Op::Binary(b) => Some(*b),
        Op::BinaryAdd => Some(BinOp::Add),
        // ---- lista chiusa: varianti che le finestre NON fondono ----
        // (le forme registro già abbassate — BinarySS/SC/Dst/CmpJmpSS/SC —
        // restano fuori PER SCELTA: il pass non ri-fonde il proprio output.)
        Op::PushConst { .. } | Op::Pop { .. } | Op::Dup { .. }
        | Op::LoadSlot { .. } | Op::LoadVar { .. } | Op::PushUndef { .. }
        | Op::StoreSlot { .. } | Op::ConcatAssignSlot { .. } | Op::Swap { .. }
        | Op::LoadGlobal { .. } | Op::StoreGlobal { .. } | Op::IncDecGlobal { .. }
        | Op::LoadSuperglobal { .. } | Op::StoreSuperglobal { .. }
        | Op::IncDecSuperglobal { .. } | Op::FetchDimList { .. }
        | Op::LoadGlobals { .. } | Op::GlobalsDynAssign { .. }
        | Op::FillDefault { .. } | Op::CoerceParam { .. } | Op::CheckArity { .. }
        | Op::IncDecSlot { .. } | Op::BindRef { .. } | Op::StaticGuard { .. }
        | Op::StaticStore { .. } | Op::StaticAlias { .. } | Op::PushRef { .. }
        | Op::MakeRef { .. } | Op::PushArgPlace { .. } | Op::BindRefTo { .. }
        | Op::BindRefToChecked { .. } | Op::DerefTop { .. }
        | Op::MakeClosure { .. } | Op::MakeFcc { .. } | Op::CallValue { .. }
        | Op::CallNsFallback { .. } | Op::CallValueArgs { .. }
        | Op::CallNsFallbackArgs { .. } | Op::Throw { .. } | Op::Rethrow { .. }
        | Op::CatchMatch { .. } | Op::EndFinally { .. } | Op::ParkReturn { .. }
        | Op::ParkJump { .. } | Op::Unary { .. } | Op::Cast { .. }
        | Op::Jump { .. } | Op::JumpIfFalse { .. } | Op::JumpIfTrue { .. }
        | Op::CmpJmp { .. } | Op::CmpJmpConst { .. } | Op::BinarySS { .. }
        | Op::BinarySSDst { .. } | Op::BinarySC { .. } | Op::BinarySCDst { .. }
        | Op::BinaryDst { .. } | Op::BinarySTDst { .. } | Op::BinaryTC { .. }
        | Op::BinarySCSC { .. } | Op::IncDecSlotPop { .. } | Op::IncDecSlotJmp { .. }
        | Op::PropGetSlot { .. } | Op::PropSetPop { .. } | Op::StringifySlot { .. }
        | Op::PropGetSlotRecv { .. } | Op::BinaryTCPropSetPop { .. }
        | Op::BinarySCSCDst { .. } | Op::LoadVarPushConst { .. }
        | Op::ConcatNConst { .. }
        | Op::CmpJmpSS { .. } | Op::CmpJmpSC { .. }
        | Op::ConcatN { .. } | Op::JumpIfNotNull { .. } | Op::JumpIfNull { .. }
        | Op::Echo { .. } | Op::Print { .. } | Op::Stringify { .. }
        | Op::ArrayInit { .. } | Op::ArrayPush { .. } | Op::ArrayInsert { .. }
        | Op::ArrayAppendSpread { .. } | Op::CallArgs { .. } | Op::FetchDim { .. }
        | Op::CoalesceFetchDim { .. } | Op::AssignPath { .. }
        | Op::AssignOpPath { .. } | Op::IncDecPath { .. } | Op::IssetPath { .. }
        | Op::EmptyPath { .. } | Op::UnsetPath { .. } | Op::Call { .. }
        | Op::DeclareFn { .. } | Op::DeclareClass { .. } | Op::DeclareTrait { .. }
        | Op::DeclareDeferred { .. } | Op::NewAnonDeferred { .. }
        | Op::CallBuiltin { .. } | Op::CallBuiltinSpread { .. }
        | Op::CallHostBuiltin { .. } | Op::CallHostBuiltinRef { .. }
        | Op::CallHostBuiltinOut { .. } | Op::CallHostBuiltinScanf { .. }
        | Op::CallArrayMultisort { .. } | Op::ConstFetch { .. }
        | Op::DefineConst { .. } | Op::CallBuiltinRef { .. }
        | Op::CallBuiltinRefSpread { .. } | Op::CallBuiltinRefCell { .. }
        | Op::Ret { .. } | Op::Yield { .. } | Op::YieldFrom { .. }
        | Op::IterInit { .. } | Op::IterNext { .. } | Op::IterInitRef { .. }
        | Op::IterNextRef { .. } | Op::IterPop { .. } | Op::Alloc { .. }
        | Op::This { .. } | Op::Clone { .. } | Op::Eval { .. }
        | Op::Include { .. } | Op::PropGet { .. } | Op::ThisPropGet { .. }
        | Op::PropSet { .. } | Op::PropOpSet { .. } | Op::PropIncDec { .. }
        | Op::PropIsset { .. } | Op::PropIssetFetchGate { .. }
        | Op::PropIssetDyn { .. } | Op::LoadVarDyn { .. } | Op::StoreVarDyn { .. }
        | Op::BindGlobalDyn { .. } | Op::ClassConstDynamic { .. }
        | Op::PropGetSilent { .. } | Op::PropGetDynamic { .. }
        | Op::PropGetDynamicSilent { .. } | Op::MatchError { .. }
        | Op::PropUnset { .. } | Op::MethodCall { .. } | Op::ThisMethodCall { .. }
        | Op::MethodCallArgs { .. } | Op::MethodCallDynamic { .. }
        | Op::MethodCallDynamicArgs { .. } | Op::MethodCallNamed { .. }
        | Op::CallNamed { .. } | Op::CallSpread { .. } | Op::InvokeMethod { .. }
        | Op::InstanceOf { .. } | Op::InstanceOfStatic { .. }
        | Op::InstanceOfDynamic { .. } | Op::InstanceOfBuiltin { .. }
        | Op::StaticCall { .. } | Op::HookCall { .. } | Op::ClosureStatic { .. }
        | Op::StaticCallArgs { .. } | Op::StaticCallDynamic { .. }
        | Op::StaticCallDynamicArgs { .. } | Op::StaticCallDynamicMethod { .. }
        | Op::StaticCallTargetDynamicMethod { .. }
        | Op::StaticPropGetDynName { .. } | Op::StaticPropSetDynName { .. }
        | Op::StaticCallDynamicMethodArgs { .. }
        | Op::StaticCallTargetDynamicMethodArgs { .. } | Op::ClassConst { .. }
        | Op::ClassConstDyn { .. } | Op::ClassConstFromValue { .. }
        | Op::EnumCase { .. } | Op::ClassNameStatic { .. }
        | Op::ClassNameScope { .. } | Op::AllocStatic { .. }
        | Op::AllocDynamic { .. } | Op::InvokeCtor { .. }
        | Op::InvokeCtorArgs { .. } | Op::InitProps { .. }
        | Op::StampThrowable { .. } | Op::StaticPropGet { .. }
        | Op::StaticPropSet { .. } | Op::StaticPropRef { .. }
        | Op::StaticPropOpSet { .. } | Op::StaticPropIncDec { .. }
        | Op::StaticPropGetDynamic { .. } | Op::StaticPropSetDynamic { .. }
        | Op::StaticPropOpSetDynamic { .. } | Op::StaticPropIncDecDynamic { .. }
        | Op::FieldAssign { .. } | Op::FieldAssignOp { .. }
        | Op::FieldIncDec { .. } | Op::FieldIsset { .. } | Op::FieldEmpty { .. }
        | Op::FieldUnset { .. } | Op::Fatal { .. } | Op::EmitNotice { .. }
        | Op::Exit { .. } | Op::SuppressBegin { .. } | Op::SuppressEnd { .. }
        | Op::Sweep { .. } | Op::Nop { .. } => None,
    }
}

/// Recognise the longest fusable window starting at `i`; `(op, width)` —
/// width 1 with the original op when nothing fuses.
fn fuse_window(f: &Func, blocked: &[bool], i: usize) -> (Op, usize) {
    let n = f.ops.len();
    let line = f.lines[i];
    let free = |j: usize| j < n && !blocked[j] && f.lines[j] == line;

    if let Some(a) = fold_slot(f, i) {
        // S-107 W6 (PRIMA delle finestre 3-op: la più lunga vince): l'albero
        // [LoadVar, PushConst, Binary, LoadVar, PushConst, Binary, Binary]
        // — due sottoespressioni slot⊚const che alimentano un Binary
        // (census arith: BinarySC;BinarySC;Binary(Sub), 50M/run). Nessuna
        // coda Dst per scelta (il giudice alimenta BinarySTDst).
        if (3..=6).all(|k| free(i + k)) && free(i + 1) && free(i + 2) {
            if let (Some(ca), Some(opa), Some(lb), Some(cb), Some(opb), Some(op)) = (
                fold_const(f, i + 1),
                bin_op_of(&f.ops[i + 2]),
                fold_slot(f, i + 3),
                fold_const(f, i + 4),
                bin_op_of(&f.ops[i + 5]),
                bin_op_of(&f.ops[i + 6]),
            ) {
                // S-108 lotto-2 W10: la coda BinarySTDst nella STESSA riga —
                // [LoadSlot(l), Swap, Binary(opd), tail-assign] subito dopo
                // l'albero: l'intero statement `$s opd= (…)` in un'op sola.
                // Se la coda non c'è, la finestra S-107 resta identica.
                if (7..=9).all(|k| free(i + k)) {
                    if let (Op::LoadSlot(l), Op::Swap) = (&f.ops[i + 7], &f.ops[i + 8]) {
                        if *l <= u16::MAX as u32 {
                            if let Some(opd) = bin_op_of(&f.ops[i + 9]) {
                                if free(i + 10) {
                                    if let Op::StoreSlot(dst) = &f.ops[i + 10] {
                                        if *dst <= u16::MAX as u32 {
                                            return (
                                                Op::BinarySCSCDst { opa, la: a, ca, opb, lb, cb, op, opd, l: *l as u16, dst: *dst as u16 },
                                                11,
                                            );
                                        }
                                    }
                                    if matches!(f.ops[i + 10], Op::Dup) && free(i + 11) && free(i + 12) {
                                        if let (Op::StoreSlot(dst), Op::Pop) = (&f.ops[i + 11], &f.ops[i + 12]) {
                                            if *dst <= u16::MAX as u32 {
                                                return (
                                                    Op::BinarySCSCDst { opa, la: a, ca, opb, lb, cb, op, opd, l: *l as u16, dst: *dst as u16 },
                                                    13,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return (
                    Op::BinarySCSC { opa, la: a, ca, opb, lb, cb, op },
                    7,
                );
            }
        }
        if free(i + 1) && free(i + 2) {
            // [LoadVar, LoadVar, Binary|CmpJmp]
            if let Some(b) = fold_slot(f, i + 1) {
                if let Some(op) = bin_op_of(&f.ops[i + 2]) {
                    return bin_dst(f, &free, i, 3, BinKind::SS(a, b), op);
                }
                match &f.ops[i + 2] {
                    Op::CmpJmp { op, addr, when } => {
                        return (
                            Op::CmpJmpSS { op: *op, l: a, r: b, addr: *addr, when: *when },
                            3,
                        )
                    }
                    _ => {}
                }
            }
            // [LoadVar, PushConst, Binary]
            if let Some(c) = fold_const(f, i + 1) {
                if let Some(op) = bin_op_of(&f.ops[i + 2]) {
                    return bin_dst(f, &free, i, 3, BinKind::SC(a, c), op);
                }
            }
        }
        // S-107 W3/W8: 2-op a testa LoadVar — la lettura fusa conserva la
        // parità warning (guardia fold_slot + `unit_slot_name` a runtime).
        if free(i + 1) {
            match &f.ops[i + 1] {
                // [LoadVar, PropGet] → PropGetSlot (bigram 60M nel giudice prop)
                Op::PropGet { name, ic } => {
                    return (
                        Op::PropGetSlot { slot: a, name: name.clone(), ic: ic.clone() },
                        2,
                    )
                }
                // [LoadVar, Stringify] → StringifySlot (interpolazione str/arr)
                Op::Stringify => return (Op::StringifySlot { slot: a }, 2),
                _ => {}
            }
        }
        // [LoadVar, CmpJmpConst] → slot vs const compare (mirror const-lhs)
        if free(i + 1) {
            if let Op::CmpJmpConst { op, cidx, addr, when, const_lhs } = &f.ops[i + 1] {
                if *cidx <= u16::MAX as u32 {
                    let op2 = if *const_lhs { mirror_cmp(*op) } else { Some(*op) };
                    if let Some(op2) = op2 {
                        return (
                            Op::CmpJmpSC {
                                op: op2,
                                slot: a,
                                cidx: *cidx as u16,
                                addr: *addr,
                                when: *when,
                            },
                            2,
                        );
                    }
                }
            }
        }
        // S-108 lotto-2 W13: [LoadVar, PushConst] — la coppia di push
        // argomenti (calls/arr/re) quando NESSUNA finestra più lunga ha
        // vinto. Guardia di non-interferenza: se all'op dopo il const c'è un
        // Binary sulla stessa riga, la coppia si LASCIA al braccio W5
        // ([PushConst, Binary] → BinaryTC); se dopo il const c'è un
        // [LoadVar foldabile, Cmp specchiabile] sulla stessa riga, il const
        // appartiene al fold SPECCHIO (S-109, azione-1 revisore S-108: su
        // quel pattern l'emissione torna lotto-1). W13 resta invece PRIMA
        // di F2 ([PushConst, ConcatN]): su [LoadVar, PushConst, ConcatN]
        // vince W13 come in lotto-2 — F2 fonde solo i const che nessuna
        // finestra precedente ha assorbito. Il lotto AGGIUNGE fusioni, non
        // cambia mai l'emissione dei lotti precedenti.
        if free(i + 1) {
            if let Some(c) = fold_const(f, i + 1) {
                let w5 = free(i + 2) && bin_op_of(&f.ops[i + 2]).is_some();
                let mirror = free(i + 2)
                    && free(i + 3)
                    && fold_slot(f, i + 2).is_some()
                    && bin_op_of(&f.ops[i + 3]).and_then(mirror_cmp).is_some();
                if !w5 && !mirror {
                    return (Op::LoadVarPushConst { slot: a, cidx: c }, 2);
                }
            }
        }
    }
    // [PushConst, LoadVar, Binary] — const written first: fold only a
    // mirrorable COMPARISON (order-free by construction). The WP-44
    // commutative-arith swap is deliberately absent (see module doc).
    if let Some(c) = fold_const(f, i) {
        if free(i + 1) && free(i + 2) {
            if let Some(a) = fold_slot(f, i + 1) {
                if let Some(op) = bin_op_of(&f.ops[i + 2]) {
                    if let Some(m) = mirror_cmp(op) {
                        return bin_dst(f, &free, i, 3, BinKind::SC(a, c), m);
                    }
                }
            }
        }
        // S-109 F2: [PushConst(Str), ConcatN] — la parte literal del join
        // entra nell'op di concatenazione (helper condiviso concat_n).
        // Guardia: SOLO Const::Str (il fast path all-Str di ConcatN resta
        // monomorfo); ConcatN è puro per costruzione — nessun helper
        // sospendibile in finestra (vincolo S-108).
        if free(i + 1) {
            if let Op::ConcatN(nn) = &f.ops[i + 1] {
                if matches!(f.consts.get(c as usize), Some(Const::Str(_))) {
                    return (Op::ConcatNConst { n: *nn, cidx: c }, 2);
                }
            }
        }
        // S-107 W5: [PushConst, Binary] ADIACENTI — lhs in PILA, const
        // SEMPRE rhs per costruzione (il PushConst precede immediatamente il
        // Binary ⇒ il const è il top). Nessuno swap, nessuna divergenza
        // d'ordine possibile (bigram prop: PushConst(1);BinaryAdd, 30M).
        if free(i + 1) {
            if let Some(op) = bin_op_of(&f.ops[i + 1]) {
                // S-108 lotto-2 W9b: la coda [PropSet, Pop] nella stessa
                // riga — `$o->p = <stack> op const;` intero (funnel BinaryTC
                // flat, poi l'entry PropSet DISCARD come ultimo passo).
                if free(i + 2) && free(i + 3) {
                    if let (Op::PropSet { name, ic }, Op::Pop) = (&f.ops[i + 2], &f.ops[i + 3]) {
                        return (
                            Op::BinaryTCPropSetPop { op, cidx: c, name: name.clone(), ic: ic.clone() },
                            4,
                        );
                    }
                }
                return (Op::BinaryTC { op, cidx: c }, 2);
            }
        }
    }
    // [LoadSlot, Swap, Binary] — RMW su slot (S-106 leva H-A1): il prefisso
    // lhs-da-slot del compound assign `$s <op>= expr`. EMENDAMENTO DICHIARATO
    // (S-106-R-3) della regola v3 «LoadSlot never folded»: quella regola
    // presumeva LoadSlot FREDDO e il dump del giudice arith la refuta (è il
    // read del compound assign nel corpo caldo). La lettura è SILENZIOSA per
    // contratto — nessun warning da risintetizzare ⇒ il fold è
    // diagnostic-safe. Fonde SOLO con coda assign-and-discard (BinKind::ST
    // senza coda non fonde: nessuna forma value).
    if let Op::LoadSlot(l) = &f.ops[i] {
        if *l <= u16::MAX as u32 && free(i + 1) && free(i + 2) {
            if matches!(f.ops[i + 1], Op::Swap) {
                if let Some(op) = bin_op_of(&f.ops[i + 2]) {
                    return bin_dst(f, &free, i, 3, BinKind::ST(*l as u16), op);
                }
            }
            // S-108 lotto-2 W9a: [LoadSlot(recv), LoadVar, PropGet] — la
            // testa RMW del giudice prop (`$o->c = $o->c …`): push silente
            // del ricevitore + la finestra W3 intera. La sospensione
            // hook/__get resta nell'ULTIMO helper del handler.
            if let Some(slot) = fold_slot(f, i + 1) {
                if let Op::PropGet { name, ic } = &f.ops[i + 2] {
                    return (
                        Op::PropGetSlotRecv { recv: *l as u16, slot, name: name.clone(), ic: ic.clone() },
                        3,
                    );
                }
            }
        }
    }
    // S-107 W1: [IncDecSlot, Pop] (+ Jump di back-edge) — il valore spinto
    // è SCARTATO, quindi pre/post collassano nella forma fusa e il gc_note
    // eliso è un no-op (il transiente è sempre uno scalare: ++/-- su
    // array/oggetto è TypeError prima di ogni push). Il trigramma col Jump
    // è la forma calda in TUTTI e sei i giudici.
    if let Op::IncDecSlot { slot, inc, pre: _ } = &f.ops[i] {
        if *slot <= u16::MAX as u32 && free(i + 1) && matches!(f.ops[i + 1], Op::Pop) {
            if free(i + 2) {
                if let Op::Jump(a) = &f.ops[i + 2] {
                    return (
                        Op::IncDecSlotJmp { slot: *slot as u16, inc: *inc, addr: *a },
                        3,
                    );
                }
            }
            return (Op::IncDecSlotPop { slot: *slot as u16, inc: *inc }, 2);
        }
    }
    // S-107 W2: [Dup, StoreSlot s, Pop] ≡ [StoreSlot s] — nessuna op nuova:
    // il duplicato transiente elide (stesso precedente delle code *Dst; il
    // gc_note del Pop cadeva sul clone transiente, che non esiste più).
    // Cattura le code assign-and-discard dietro produttori NON fusabili
    // (Call, CallBuiltin, Ret del chiamato — census calls/str).
    if matches!(f.ops[i], Op::Dup) && free(i + 1) && free(i + 2) {
        if let (Op::StoreSlot(s), Op::Pop) = (&f.ops[i + 1], &f.ops[i + 2]) {
            return (Op::StoreSlot(*s), 3);
        }
    }
    // S-107 W4: [PropSet, Pop] → PropSetPop (statement `$o->p = v;`).
    if let Op::PropSet { name, ic } = &f.ops[i] {
        if free(i + 1) && matches!(f.ops[i + 1], Op::Pop) {
            return (Op::PropSetPop { name: name.clone(), ic: ic.clone() }, 2);
        }
    }
    // Bare Binary: wins only with an assign-and-discard tail.
    if let Some(op) = bin_op_of(&f.ops[i]) {
        return bin_dst(f, &free, i, 1, BinKind::Stack, op);
    }
    (f.ops[i].clone(), 1)
}

/// Extend a Binary window over an assign-and-discard tail (`StoreSlot s` or
/// `Dup, StoreSlot s, Pop`) and emit the matching monomorphic variant. With
/// no tail: `SS`/`SC` push, a bare stack Binary stays as it is (nothing to
/// win).
fn bin_dst(
    f: &Func,
    free: &dyn Fn(usize) -> bool,
    i: usize,
    w: usize,
    kind: BinKind,
    op: BinOp,
) -> (Op, usize) {
    let j = i + w;
    let tail: Option<(u16, usize)> = if free(j) {
        match &f.ops[j] {
            Op::StoreSlot(s) if *s <= u16::MAX as u32 => Some((*s as u16, 1)),
            Op::Dup if free(j + 1) && free(j + 2) => {
                match (&f.ops[j + 1], &f.ops[j + 2]) {
                    (Op::StoreSlot(s), Op::Pop) if *s <= u16::MAX as u32 => {
                        Some((*s as u16, 3))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    };
    match (kind, tail) {
        (BinKind::SS(l, r), Some((dst, e))) => (Op::BinarySSDst { op, l, r, dst }, w + e),
        (BinKind::SS(l, r), None) => (Op::BinarySS { op, l, r }, w),
        (BinKind::SC(slot, cidx), Some((dst, e))) => {
            (Op::BinarySCDst { op, slot, cidx, dst }, w + e)
        }
        (BinKind::SC(slot, cidx), None) => (Op::BinarySC { op, slot, cidx }, w),
        // H-A1: senza coda la finestra ST NON fonde (nessuna forma value).
        (BinKind::ST(l), Some((dst, e))) => (Op::BinarySTDst { op, l, dst }, w + e),
        (BinKind::ST(_), None) => (f.ops[i].clone(), 1),
        (BinKind::Stack, Some((dst, e))) => (Op::BinaryDst { op, dst }, w + e),
        (BinKind::Stack, None) => (f.ops[i].clone(), 1),
    }
}

/// Whether `PHPR_DUMP_OPS` is set: dump every compiled unit's bytecode to
/// stderr. Compile-time-only diagnostic for this arc: diff a flag-off dump
/// against a flag-on dump to prove a stage's rewrite is a no-op (stage 1) or
/// inspect exactly what it rewrote (stage 2+).
fn dump_enabled() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PHPR_DUMP_OPS").is_some())
}

/// Dump a compiled unit's bytecode to stderr (gated on `PHPR_DUMP_OPS`).
/// Scope (A-HE-100-4, sanatoria RC-1): main, functions, closures, class
/// methods, property-HOOK bodies — every body that passes through the
/// `compile_body` funnel and that the lowering pass therefore rewrites
/// flag-on — plus the prop-init thunks (hand-built, NEVER lowered: declared
/// OUT of the pass, RC-2; dumped anyway so the production truth is visible).
/// Reflection/const/attribute thunks stay out: hand-built via `FnCompiler`
/// (`compile_const_thunk`, attribute thunks), the pass cannot touch them.
/// Hook order is sorted by property name — `prop_info` is a HashMap and the
/// dump feeds diff-based gates, so iteration order must be deterministic.
pub(super) fn dump_module_ops(m: &crate::bytecode::Module) {
    if !dump_enabled() {
        return;
    }
    let err = std::io::stderr();
    let mut w = err.lock();
    dump_module_to(&mut w, m);
}

/// The dump body, writer-parametric so the A-HE-100-4 coverage test can
/// assert the scope against the Module without touching process stderr.
pub(super) fn dump_module_to(w: &mut impl std::io::Write, m: &crate::bytecode::Module) {
    let _ = writeln!(w, "== unit {} ==", String::from_utf8_lossy(&m.file));
    dump_func(w, "{main}", &m.main);
    for f in &m.functions {
        dump_func(w, &format!("fn {}", String::from_utf8_lossy(&f.name)), f);
    }
    for (i, f) in m.closures.iter().enumerate() {
        dump_func(w, &format!("closure#{i}"), f);
    }
    for c in &m.classes {
        let cname = String::from_utf8_lossy(&c.name);
        for meth in &c.methods {
            let label = format!("{cname}::{}", String::from_utf8_lossy(&meth.name));
            dump_func(w, &label, &meth.func);
        }
        if let Some(pi) = &c.prop_init {
            dump_func(w, &format!("{cname}::{{prop-init}}"), pi);
        }
        let mut hooked: Vec<(&[u8], &crate::bytecode::PropHooks)> = c
            .prop_info
            .iter()
            .filter_map(|(n, i)| i.hooks.as_ref().map(|h| (&n[..], h)))
            .collect();
        hooked.sort_by(|a, b| a.0.cmp(b.0));
        for (pname, h) in hooked {
            let p = String::from_utf8_lossy(pname);
            if let Some(g) = &h.get {
                dump_func(w, &format!("{cname}::${p}::get"), g);
            }
            if let Some(s) = &h.set {
                dump_func(w, &format!("{cname}::${p}::set"), s);
            }
        }
    }
}

fn dump_func(w: &mut impl std::io::Write, label: &str, f: &Func) {
    let _ = writeln!(w, "-- {label} n_slots={} max_temps={} --", f.n_slots, f.max_temps);
    for (i, op) in f.ops.iter().enumerate() {
        let _ = writeln!(w, "{i:04} {op:?}");
    }
    for (i, c) in f.consts.iter().enumerate() {
        let _ = writeln!(w, "cst{i:03} {c:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin::Registry;
    use crate::bytecode::Module;

    /// Compile col modo ESPLICITO (S-100): la batteria di emissione non
    /// dipende dall'ambiente del processo — `compile` = modo OFF,
    /// `compile_on` = il funnel VERO col pass acceso (hook compresi,
    /// prop_init/thunk esclusi per costruzione: è compile_body a decidere).
    fn compile_mode(src: &[u8], reg_lower: bool) -> Module {
        let program = crate::lower_source(b"t.php", src).expect("lowers");
        crate::compile::compile_program_with_mode(&program, &Registry::default(), reg_lower)
            .expect("compiles")
    }

    fn compile(src: &[u8]) -> Module {
        compile_mode(src, false)
    }

    fn compile_on(src: &[u8]) -> Module {
        compile_mode(src, true)
    }

    // S-100: il vecchio helper `lowered()` (mirror A MANO del funnel,
    // fonte RC-2) è stato eliminato — il braccio flag-on dei test è ora il
    // funnel VERO via `compile_on` (`compile_program_with_mode`).

    /// A-HE-100-4: enumera OGNI corpo dal Module per DESTRUCTURING ESAUSTIVO
    /// — un campo nuovo di `Module` (o di `CompiledClass`/`PropHooks` sotto)
    /// che portasse corpi NON COMPILA finché non viene classificato qui
    /// (stessa classe del dente S-96 in vm/liveness.rs). Classificazione:
    /// - IN funnel `compile_body` (il pass li riscrive flag-on): main,
    ///   functions, closures, methods, hook get/set.
    /// - FUORI (costruiti a mano, il pass non li vede MAI): prop_init (RC-2),
    ///   const-thunk (`consts[].func`), attribute-thunk (new/args), enum-case.
    /// - Senza corpi compilati nel Module: i campi bound a `_` qui sotto
    ///   (hir non ancora compilato, indici, metadati).
    fn all_funcs(m: &Module) -> Vec<&Func> {
        let Module {
            main,
            functions,
            conditional_fns: _,
            fn_ci: _,
            conditional_classes: _,
            // hir non compilato: i corpi nascono al Declare, via compile_body
            conditional_traits: _,
            // ri-lowered a runtime: ripassa da compile_body al suo punto di esecuzione
            deferred: _,
            closures,
            classes,
            file: _,
            class_index: _,
            static_count: _,
            strict: _,
            // attribute-thunk: FnCompiler a mano, FUORI dal funnel
            const_attributes: _,
            elided: _,
        } = m;
        let mut all: Vec<&Func> = vec![main];
        all.extend(functions.iter().map(|f| f.as_ref()));
        all.extend(closures.iter());
        for c in classes {
            all.extend(c.methods.iter().map(|meth| &meth.func));
            for info in c.prop_info.values() {
                if let Some(crate::bytecode::PropHooks { get, set, backed: _ }) = &info.hooks {
                    all.extend(get.iter());
                    all.extend(set.iter());
                }
            }
            // FUORI dal funnel ma corpi reali: inclusi comunque — la batteria
            // flag-off asserisce «nessuna forma registro OVUNQUE», e per i
            // corpi fuori-funnel l'assenza deve valere in ENTRAMBI i modi.
            all.extend(c.prop_init.iter());
            all.extend(c.consts.iter().map(|k| &k.func));
        }
        all
    }

    /// La classe della fixture (il Module include il prelude: si cerca per NOME).
    fn zoo_class(m: &Module) -> &crate::bytecode::CompiledClass {
        m.classes
            .iter()
            .find(|c| &*c.name == b"C")
            .expect("fixture: classe C nel Module")
    }

    /// La fixture con UN corpo per specie: funzione, chiusura, metodo,
    /// hook get/set, prop-init non costante, const di classe.
    const BODY_ZOO: &[u8] = br#"<?php
function g($a){ $s=0; for($i=0;$i<9;$i++){ $s = $s + $i*3; } return $s+$a; }
$h = function($x){ return $x+1; };
class C {
  const K = 1;
  public $d = self::K + 1;
  public int $v = 0;
  public int $p { get { $s=0; for($i=0;$i<9;$i++){ $s = $s + $i*3; } return $s; } set { $this->v = $value * 2; } }
  public function m($a,$b){ $c=$a+$b; if($c>3){$c=$c*2;} return $c; }
}
echo g(1), ($h)(2), C::K;"#;

    /// A-HE-100-4: il dump copre ogni corpo del funnel `compile_body`
    /// (hook COMPRESI — il punto cieco refutato da RC-1) più il prop-init
    /// dichiarato fuori. La lista dei corpi viene dal Module (all_funcs,
    /// destructuring esaustivo), non da un elenco a mano.
    #[test]
    fn dump_scope_covers_every_funnel_body() {
        let m = compile(BODY_ZOO);
        assert!(zoo_class(&m).prop_init.is_some(), "fixture: il prop-init deve esistere");
        let mut buf = Vec::new();
        dump_module_to(&mut buf, &m);
        let out = String::from_utf8(buf).expect("dump utf8");
        for h in [
            "-- {main} ",
            "-- fn g ",
            "-- closure#0 ",
            "-- C::m ",
            "-- C::$p::get ",
            "-- C::$p::set ",
            "-- C::{prop-init} ",
        ] {
            assert!(out.contains(h), "dump privo del corpo `{h}`:\n{out}");
        }
    }

    /// RC-1 (Concilio WP-100): i corpi hook PASSANO da `compile_body`, il
    /// pass li riscrive flag-on — helper e dump devono vederlo. Controllo
    /// positivo: il get-hook con loop foldable mostra forme registro dopo
    /// il pass, e il chunk hook del dump le mostra pure.
    #[test]
    fn hooks_are_lowered_and_visible_in_the_dump() {
        let l = compile_on(BODY_ZOO);
        let hooks = zoo_class(&l)
            .prop_info
            .get(&b"p"[..])
            .and_then(|i| i.hooks.as_ref())
            .expect("fixture: hook su $p");
        let get = hooks.get.as_ref().expect("fixture: get hook");
        assert!(
            get.ops.iter().any(is_reg_form),
            "il pass non riscrive il corpo del get-hook: RC-1 di nuovo cieco\n{:?}",
            get.ops
        );
        // E il prop-init resta NON lowered anche nel helper (RC-2: fuori).
        let pi = zoo_class(&l).prop_init.as_ref().expect("prop-init");
        assert!(
            !pi.ops.iter().any(is_reg_form),
            "il funnel flag-on abbassa prop_init che la produzione non abbassa MAI (RC-2)"
        );
        let mut buf = Vec::new();
        dump_module_to(&mut buf, &l);
        let out = String::from_utf8(buf).expect("dump utf8");
        let chunk = out
            .split("-- C::$p::get ")
            .nth(1)
            .and_then(|r| r.split("\n-- ").next())
            .expect("chunk del get-hook nel dump");
        assert!(
            chunk.contains("BinarySC") || chunk.contains("BinarySS") || chunk.contains("CmpJmpSC"),
            "nessuna forma registro nel chunk hook del dump:\n{chunk}"
        );
    }

    /// Run and return the CLI-faithful stream (diagnostics inline) — the
    /// parity comparison must cover the warning text/order too.
    fn run(m: &Module) -> Vec<u8> {
        let reg = Registry::default();
        let out = crate::vm::run_module(m, &reg);
        assert!(out.fatal.is_none(), "unexpected fatal: {:?}", out.fatal);
        out.rendered
    }

    fn is_reg_form(o: &Op) -> bool {
        matches!(
            o,
            Op::BinarySS { .. }
                | Op::BinarySSDst { .. }
                | Op::BinarySC { .. }
                | Op::BinarySCDst { .. }
                | Op::BinaryDst { .. }
                | Op::CmpJmpSS { .. }
                | Op::CmpJmpSC { .. }
                // S-106 H-A1 + lotto S-107: il dente flag-off vale anche per
                // le forme nuove — la compilazione OFF non ne emette MAI una.
                | Op::BinarySTDst { .. }
                | Op::BinaryTC { .. }
                | Op::BinarySCSC { .. }
                | Op::IncDecSlotPop { .. }
                | Op::IncDecSlotJmp { .. }
                | Op::PropGetSlot { .. }
                | Op::PropSetPop { .. }
                | Op::StringifySlot { .. }
                // S-108 lotto-2: il dente flag-off copre anche queste.
                | Op::PropGetSlotRecv { .. }
                | Op::BinaryTCPropSetPop { .. }
                | Op::BinarySCSCDst { .. }
                | Op::LoadVarPushConst { .. }
                // S-109 lotto-3: e questa.
                | Op::ConcatNConst { .. }
        )
    }

    /// v3 shape: hot windows become the specialized monomorphic forms and
    /// no fused compare window survives un-rewritten; no register temps.
    #[test]
    fn stage2v3_rewrites_hot_windows() {
        let src = br#"<?php
            function f($a, $b) {
                $c = $a + $b;
                if ($a > $b) { $c = $c * 2; }
                if ($a == 3) { return -1; }
                if (3 < $b) { $c = $c + 1; }
                return $c . "s";
            }
            echo f(1, 2), f(4, 2), f(3, 0), f(1, 7);
            "#;
        let m = compile(src);
        let lm = compile_on(src);
        let lf = lm
            .functions
            .iter()
            .find(|f| f.name.as_ref() == b"f")
            .expect("fn f present")
            .as_ref();
        let has = |p: &dyn Fn(&Op) -> bool| lf.ops.iter().any(|o| p(o));
        assert!(has(&|o| matches!(o, Op::BinarySSDst { .. })), "$c=$a+$b: {:#?}", lf.ops);
        assert!(has(&|o| matches!(o, Op::CmpJmpSS { .. })), "$a>$b");
        assert!(has(&|o| matches!(o, Op::CmpJmpSC { .. })), "$a==3 / 3<$b (mirrored)");
        assert!(
            has(&|o| matches!(o, Op::BinarySCDst { .. })),
            "$c*2 / $c+1 (const fold with dst)"
        );
        // No compare window with a foldable LoadVar in front may survive.
        for (x, y) in lf.ops.iter().zip(lf.ops.iter().skip(1)) {
            assert!(
                !(matches!(x, Op::LoadVar { .. })
                    && matches!(y, Op::CmpJmpConst { .. } | Op::CmpJmp { .. })),
                "unfused compare window"
            );
        }
        for f in all_funcs(&lm) {
            assert_eq!(f.max_temps, 0, "v3 emits no temps");
        }
        assert_eq!(run(&m), run(&lm));
    }

    /// A compare whose lhs comes from the stack (no foldable producer) keeps
    /// the monomorphic WP-34 CmpJmpConst — no-elision rewrites are the
    /// measured v1 regression.
    #[test]
    fn stage2v3_stack_lhs_compare_keeps_cmpjmpconst() {
        let src = br#"<?php
            function g($a) { return $a + 1; }
            function h($a) { if (g($a) == 3) { return 1; } return 2; }
            echo h(2), h(5);
            "#;
        let m = compile(src);
        let lm = compile_on(src);
        let lh = lm
            .functions
            .iter()
            .find(|f| f.name.as_ref() == b"h")
            .expect("fn h present");
        assert!(
            lh.ops.iter().any(|o| matches!(o, Op::CmpJmpConst { .. })),
            "stack-lhs compare must stay CmpJmpConst: {:#?}",
            lh.ops
        );
        assert!(
            !lh.ops.iter().any(|o| matches!(o, Op::CmpJmpSS { .. } | Op::CmpJmpSC { .. })),
            "no fold available in h"
        );
        assert_eq!(run(&m), run(&lm));
    }

    /// S-97.1: a const-FIRST arithmetic window must NOT fold — the
    /// "Unsupported operand types" TypeError names its operands in order,
    /// so `3 + $x` and `$x + 3` are distinguishable when `$x` is an array.
    /// (This is the WP-44 v3 commutative swap, dropped on soundness.)
    #[test]
    fn stage2v3_const_first_arith_does_not_fold() {
        let src = br#"<?php $a=5; $b = 3 + $a; $c = 3 * $a; echo $b, ",", $c;"#;
        let m = compile(src);
        let lm = compile_on(src);
        assert!(
            !lm.main.ops.iter().any(|o| matches!(
                o,
                Op::BinarySC { .. } | Op::BinarySCDst { .. }
            )),
            "const-first arith must stay on the stack: {:#?}",
            lm.main.ops
        );
        assert_eq!(run(&m), run(&lm));
    }

    /// S-109 azione-2 revisore S-108 (DENTE, emissione INTESA pre-registrata):
    /// sullo stream [LoadVar, PushConst, LoadVar, Cmp-specchiabile] il const
    /// appartiene al fold SPECCHIO — W13 CEDE (guardia estesa S-109), il
    /// LoadVar di testa resta nudo e l'emissione è quella lotto-1
    /// (BinarySC specchiato). Prima della guardia W13 rubava il PushConst
    /// (de-ottimizzazione a valore identico, revisione.md S-108 pista 4).
    #[test]
    fn w13_cede_al_fold_specchio() {
        let src = br#"<?php
            function f($x, $y) { return $x + ($y ? 1 : 0); }
            $a=1; $b=2; echo f($a, 3 < $b);
            "#;
        let m = compile(src);
        let lm = compile_on(src);
        assert!(
            lm.main.ops.iter().any(|o| matches!(o, Op::BinarySC { .. })),
            "fold specchio atteso (BinarySC dal const-lhs): {:#?}",
            lm.main.ops
        );
        assert!(
            !lm.main.ops.iter().any(|o| matches!(o, Op::LoadVarPushConst { .. })),
            "W13 non deve rubare il PushConst del fold specchio: {:#?}",
            lm.main.ops
        );
        assert_eq!(run(&m), run(&lm));
    }

    /// S-109 lotto-3 (criterio s109-lotto3-criterio.out): il corpo str
    /// `$s = substr($s . "abc", -30)` emette ConcatNConst (F2) e il
    /// PushConst NEGATO senza Unary (F1); il modo OFF resta pila pura.
    #[test]
    fn lotto3_negfold_and_concatnconst() {
        // Stessa forma del giudice str ma su funzione UTENTE (il run() della
        // batteria non registra i builtin host): il PushConst del Neg segue
        // ConcatN — fuori dalla portata di W13, F1 fonde. (Un Neg il cui
        // const segue un LoadVar foldabile viene assorbito da W13 PRIMA:
        // valore identico, il Neg resta a runtime — dichiarato nel verbale.)
        let src = br#"<?php
            function keep($a, $b) { return $a . $b; }
            $s=''; for($i=0;$i<3;$i++){ $s = keep($s . "ab", -7); } echo $s;
            "#;
        let m = compile(src);
        let lm = compile_on(src);
        assert!(
            lm.main.ops.iter().any(|o| matches!(o, Op::ConcatNConst { .. })),
            "F2 atteso (ConcatNConst): {:#?}",
            lm.main.ops
        );
        assert!(
            !lm.main.ops.iter().any(|o| matches!(o, Op::Unary(UnOp::Neg))),
            "F1 atteso (Neg-fold via consts-append, niente Unary): {:#?}",
            lm.main.ops
        );
        // INT_MIN non si fonde: la negazione promuove a float nel funnel.
        let edge = br#"<?php echo -9223372036854775808;"#;
        let me = compile(edge);
        let lme = compile_on(edge);
        assert_eq!(run(&me), run(&lme));
        assert_eq!(run(&m), run(&lm));
    }

    /// v3 parity battery: control flow that emits `Addr`s (loops, if/else,
    /// try/catch/finally, foreach by value and by ref, static guard, param
    /// defaults, ?? / ?:), plus the diagnostic paths the folds must preserve
    /// (undefined-variable warning through slot_names, DivisionByZeroError
    /// at the fused op, references, self-assign, const-first mirror and
    /// const-first arith NON-fold, numeric-string coercions) — lowered
    /// output must equal stack output, and every remapped address must land
    /// inside the function.
    #[test]
    fn stage2v3_behavioral_parity_and_remap() {
        let snippets: &[&[u8]] = &[
            br#"<?php $s=0; for ($i=0; $i<10; $i++) { $s = $s + $i; } echo $s;"#,
            br#"<?php $i=0; while ($i < 5) { $i = $i + 1; if ($i == 3) continue; echo $i; } echo "|", $i;"#,
            br#"<?php $a=2; $b=3; try { echo $a % ($b - 3); } catch (\DivisionByZeroError $e) { echo "dbz"; } finally { echo "-f"; }"#,
            br#"<?php function g($x = 5) { static $n = 0; $n = $n + 1; return $x + $n; } echo g(), g(1), g();"#,
            br#"<?php $t = ['a'=>1,'b'=>2]; $s=''; foreach ($t as $k=>$v) { $s = $s . $k . ($v + 1); } echo $s;"#,
            br#"<?php $arr=[1,2,3]; foreach ($arr as &$v) { $v = $v * 2; } unset($v); echo $arr[0], $arr[1], $arr[2];"#,
            br#"<?php echo $u + 1; $q = $u2 . "x"; echo $q;"#,
            br#"<?php $a=1; $b=&$a; $c = $b + 1; $b = $b + 10; echo $a, ",", $c;"#,
            br#"<?php $a=1; $a = $a + 1; $a = 41 + $a > 42 ? $a * 10 : $a - 1; echo $a;"#,
            br#"<?php $x=null; $y = $x ?? 7; $z = $x ?: 9; echo $y, $z, 5 <=> 3, "10" == "1e1" ? "t" : "f";"#,
            br#"<?php $s="ab"; $n=2; if ($s == "ab" && $n > 1) { echo "y"; } if (3 == $n + 1) { echo "z"; }"#,
            br#"<?php $a=5; $b = 3 + $a; $c = 3 - $a; echo $b, ",", $c; if (3 < $a) { echo "m"; } if (3 <= $a) { echo "e"; } if ("x" == $a) { echo "s"; } else { echo "n"; }"#,
            br#"<?php $w = "7"; echo 3 + $w, 3 * $w, 10 - $w, "3" . $w; if (10 > $w) echo "g";"#,
        ];
        for src in snippets {
            let m = compile(src);
            let lm = compile_on(src);
            for (f, orig) in all_funcs(&lm).into_iter().zip(all_funcs(&m)) {
                let (new_n, old_n) = (f.ops.len(), orig.ops.len());
                let check = |a: Addr| {
                    assert!(
                        (a as usize) <= new_n || (a as usize) > old_n,
                        "addr {a} out of range (new {new_n}, old {old_n}) in {:?}",
                        f.name
                    );
                };
                let mut ops = f.ops.clone();
                for op in &mut ops {
                    visit_addrs(op, &mut |a| check(*a));
                }
                for r in &f.exc_table {
                    check(r.start);
                    check(r.end);
                    check(r.target);
                }
            }
            assert_eq!(
                run(&m),
                run(&lm),
                "lowered output diverges for {}",
                String::from_utf8_lossy(src)
            );
        }
    }

    /// The register forms must not widen the Op enum (every ops Vec pays a
    /// wider element — D-cache). Pinned so a future field addition trips
    /// this consciously.
    #[test]
    fn stage2v3_op_size_unchanged() {
        assert_eq!(std::mem::size_of::<Op>(), 48, "Op must not widen");
    }

    /// Dual-mode guard: in modo OFF (ESPLICITO — S-100: la batteria non ha
    /// più premesse ambientali, M5 è assorbita dal modo-parametro) la
    /// compilazione non emette MAI una forma registro né BinaryAdd da
    /// estensione, e il contratto di frame è invariato.
    #[test]
    fn stage2v3_flag_off_emits_no_register_forms() {
        let m = compile(br#"<?php function f($a,$b){ $c=$a+$b; if($c>3){$c=$c*2;} return $c; } echo f(1,2);"#);
        for f in all_funcs(&m) {
            assert_eq!(f.max_temps, 0);
            assert!(
                !f.ops.iter().any(is_reg_form),
                "flag-off compile must stay stack-based"
            );
        }
    }

    /// L'entry di PRODUZIONE (`compile_program`) stampa il modo del
    /// PROCESSO (`enabled()`, contratto value-parsed): il test vale in
    /// QUALUNQUE modo giri la batteria — niente falso verde stesso-modo,
    /// e il flip del default non inverte nessuna premessa (KS-HO-101-3).
    #[test]
    fn production_entry_follows_process_mode() {
        let src = br#"<?php $s=0; for($i=0;$i<9;$i++){ $s = $s + $i*3; } echo $s;"#;
        let program = crate::lower_source(b"t.php", src).expect("lowers");
        let m = crate::compile::compile_program(&program, &Registry::default()).expect("compiles");
        let has_reg = m.main.ops.iter().any(is_reg_form);
        assert_eq!(
            has_reg,
            enabled(),
            "compile_program non segue il modo di processo del contratto \
             (enabled()={}, forme registro nel main={})",
            enabled(),
            has_reg
        );
    }

    /// Il contratto di modo (S-100 punto 1): grammatica value-parsed, lista
    /// chiusa, testata PURA (nessun ambiente toccato). Il caso `=0` è la
    /// trappola che ha motivato il contratto: sotto `is_some()` accendeva.
    #[test]
    fn mode_contract_grammar_is_value_parsed() {
        use std::ffi::OsStr;
        assert_eq!(mode_from_env(None), DEFAULT_ON, "assente => default nominato");
        assert!(mode_from_env(Some(OsStr::new("1"))), "`=1` => ON");
        assert!(!mode_from_env(Some(OsStr::new("0"))), "`=0` => OFF, MAI presence");
        // Fuori grammatica: default (con warning su stderr, non asseribile qui).
        assert_eq!(mode_from_env(Some(OsStr::new(""))), DEFAULT_ON);
        assert_eq!(mode_from_env(Some(OsStr::new("on"))), DEFAULT_ON);
        assert_eq!(mode_from_env(Some(OsStr::new("true"))), DEFAULT_ON);
    }

    /// Post-flip (S-100 punto 5) il default nominato è ON: chi lo
    /// ri-invertisse deve dichiararsi QUI e ri-derivare denti e launcher
    /// (KS-HO-101-3) — il braccio OFF resta collaudato a ogni rotazione
    /// (KS-HE-101-3) via `PHPR_REG_LOWER=0` esplicito.
    #[test]
    fn mode_contract_default_is_on_post_flip() {
        assert!(DEFAULT_ON, "default ri-invertito: ri-derivare denti anti-putenv, launcher e batteria PRIMA di spedire");
    }

    /// A-HE-102-1 (Concilio WP-102, S-101 punto 5): il braccio OFF
    /// in-process emette l'emissione di PRODUZIONE anche per l'add RESIDUO
    /// DI PILA — il sito che `emit_binary` decideva col globale `enabled()`
    /// invece di `ctx.reg_lower` (fix A-HO-102-1 @ b618e3a). Sotto OFF il
    /// sito è `Binary(Add)` generico e `BinaryAdd` NON esiste; sotto ON,
    /// stesso sorgente, il generico scompare (tripwire zero-`Binary(Add)`).
    #[test]
    fn in_process_off_arm_emits_production_stack_add() {
        use crate::hir::BinOp;
        // `g($a+$b+$c)`: il primo add lascia il risultato in pila
        // (argomento) — la classe residua che l'estensione H-B2 riscrive
        // SOLO flag-on.
        let src = br#"<?php function g($x){ return $x; } $a=1;$b=2;$c=3; echo g($a+$b+$c);"#;
        let count = |m: &Module, pred: &dyn Fn(&Op) -> bool| -> usize {
            all_funcs(m)
                .iter()
                .map(|f| f.ops.iter().filter(|o| pred(o)).count())
                .sum()
        };
        // Produzione OFF (H-B2, S-98): emit_binary emette DIRETTAMENTE
        // `BinaryAdd` quando `!ctx.reg_lower` — il bug A-HO-102-1 (lettura
        // di `enabled()` col default ON) faceva cadere il sito nel ramo
        // `Binary(Add)` generico. Il dente pinna la polarità VERA.
        let off = compile(src);
        assert!(
            count(&off, &|o| matches!(o, Op::BinaryAdd)) >= 1,
            "OFF: l'add di pila e' BinaryAdd di produzione (H-B2 flag-off)"
        );
        assert_eq!(
            count(&off, &|o| matches!(o, Op::Binary(BinOp::Add))),
            0,
            "OFF: nessun Binary(Add) generico (il sintomo del bug emit_binary)"
        );
        let on = compile_on(src);
        assert_eq!(
            count(&on, &|o| matches!(o, Op::Binary(BinOp::Add))),
            0,
            "ON: tripwire zero-Binary(Add) (ogni add e' forma fusa o BinaryAdd)"
        );
    }

    /// A-HE-103-1 (Concilio WP-103, S-102 punto 5): BODY_ZOO — il tripwire
    /// zero-`Binary(Add)` esteso ai corpi FUORI funnel. `ctx.reg_lower` è UN
    /// campo per l'intero Module, ma il pass registro visita solo i corpi
    /// del funnel `compile_body`: prop_init (costruito a mano, RC-2) non lo
    /// vede MAI.
    /// ATTESA SCRITTA PRIMA (dalla lettura di emit_binary + RC-2):
    /// - OFF: `!ctx.reg_lower` ⇒ emit_binary emette `BinaryAdd` diretto
    ///   OVUNQUE, prop_init compreso ⇒ zero `Binary(Add)` nel modulo intero.
    /// - ON: emit_binary emette `Binary(Add)` generico contando sul pass; in
    ///   prop_init il pass non passa ⇒ il generico SOPRAVVIVE. L'invariante
    ///   «zero-Binary(Add) modulo intero» sotto ON è FALSO fuori funnel: il
    ///   residuo si pinna PER NOME (carve-out prop_init), non si nasconde.
    ///   Semantica invariata (il generico è corretto, solo non specializzato).
    #[test]
    fn body_zoo_off_funnel_add_polarity() {
        use crate::hir::BinOp;
        let src = br#"<?php
            class Z {
                const K = 5;
                public $p = self::K + 1;
            }
            $z = new Z(); echo $z->p;"#;
        let count_in = |fs: &[&Func], pred: &dyn Fn(&Op) -> bool| -> usize {
            fs.iter().map(|f| f.ops.iter().filter(|o| pred(o)).count()).sum()
        };
        let off = compile(src);
        assert_eq!(
            count_in(&all_funcs(&off), &|o| matches!(o, Op::Binary(BinOp::Add))),
            0,
            "OFF: nessun Binary(Add) generico in NESSUN corpo (prop_init compreso)"
        );
        let on = compile_on(src);
        // Carve-out per NOME: il residuo generico sotto ON vive SOLO in
        // prop_init (fuori funnel). I corpi del funnel restano a zero.
        let funnel: Vec<&Func> = {
            let mut v: Vec<&Func> = vec![&on.main];
            v.extend(on.functions.iter().map(|f| f.as_ref()));
            v.extend(on.closures.iter());
            for c in &on.classes {
                v.extend(c.methods.iter().map(|m| &m.func));
            }
            v
        };
        assert_eq!(
            count_in(&funnel, &|o| matches!(o, Op::Binary(BinOp::Add))),
            0,
            "ON: zero Binary(Add) nei corpi DEL funnel"
        );
        let prop_inits: Vec<&Func> =
            on.classes.iter().filter_map(|c| c.prop_init.as_ref()).collect();
        assert!(
            count_in(&prop_inits, &|o| matches!(o, Op::Binary(BinOp::Add))) >= 1,
            "ON: il residuo fuori-funnel esiste ed e' PINNATO qui (prop_init, RC-2); \
             se questo assert diventa rosso il pass ha iniziato a visitare \
             prop_init — aggiornare il carve-out PER NOME"
        );
    }

    /// A-KL-102-3 riscritto per R-HE-103-1 (Concilio WP-103, S-102 punto 5):
    /// la vecchia metà «stessa emissione» era `f(x)==f(x)` — confrontava
    /// `compile_mode(src, a)` con `compile_mode(src, b)` DOPO aver asserito
    /// `a == b`: pinnava il DETERMINISMO del compilatore, non «assente ≡ =1»
    /// (copertura fabbricata: un sito ambientale residuo colorerebbe i due
    /// bracci allo stesso modo). Qui resta SOLO il contenuto reale del dente:
    /// la riga di grammatica (A-HE-103-4). La coppia assente↔`=1` VERA vive
    /// in SOTTOPROCESSO col dump-diff: php-cli/tests/absent_eq_one.rs
    /// (A-HE-103-3).
    #[test]
    fn absent_env_resolves_like_explicit_one() {
        use std::ffi::OsStr;
        assert_eq!(
            mode_from_env(None),
            mode_from_env(Some(OsStr::new("1"))),
            "assente e `=1` devono risolvere lo stesso modo (grammatica value-parsed)"
        );
    }
}
