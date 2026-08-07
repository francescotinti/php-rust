//! S-95.0 A-ZV2 fase F1 — analisi di ULTIMO USO per slot, in SOLA MISURA
//! (`wp95-harness/design95-liveness.md`).
//!
//! Risponde a una sola domanda, per ogni `LoadSlot`/`LoadVar` di ogni funzione
//! compilata: *dopo questa lettura, esiste un cammino che rilegge lo slot prima
//! che venga riscritto?* Se no, la lettura è un ultimo uso e il valore si
//! POTREBBE spostare (`TakeSlot`) invece di clonarlo. Qui non si sposta nulla:
//! il risultato alimenta solo i contatori `would_take*` di [`super::zvalcensus`].
//!
//! PERIMETRO F1 (voluto, non dimenticato — design95-liveness.md): questa è
//! l'analisi dataflow NUDA. I modi in cui PHP osserva uno slot fuori dal
//! flusso lineare — `compact()`/`extract()`/`get_defined_vars()`, `$$x`,
//! `eval`/`include`, closure by-ref, generatori, `Zval::Ref` condivisi,
//! l'ordine dei distruttori — sono i predicati di RINUNCIA della fase F2:
//! la differenza fra il conteggio F1 e quello F2 è «quanto costa la prudenza».
//!
//! Direzione degli errori di modello (documentata perché è una scelta):
//! - una SCRITTURA non modellata (es. `extract`, `StoreVarDyn`) rende vivo ciò
//!   che è morto → SOTTOconta i movibili (conservativo, accettato);
//! - una LETTURA non modellata sovraconta: le uniche letture fuori modello
//!   sono esattamente quelle del perimetro F2 (per nome, qui sopra);
//! - gli op `*Global(s)` toccano gli slot di `frames[0]`: contarli come USO
//!   dello slot locale `s` è corretto nel main e spurio (ma sottocontante,
//!   quindi innocuo) dentro una funzione.
//!
//! Convenzione identica a `zvalcensus`: compilato SOLO dietro `zval-census`,
//! nessun bit del binario di parità cambia.

use crate::bytecode::{DimBase, FieldBase, Func, Op};

/// Esito dell'analisi su una funzione: `movable[i]` è vero sse `ops[i]` è un
/// `LoadSlot`/`LoadVar` la cui lettura è un ultimo uso (F1, senza rinunce F2);
/// `movable_safe[i]` applica in più il PERIMETRO CONSERVATIVO F2.
pub struct Analysis {
    pub movable: Vec<bool>,
    /// F2: movibile E fuori da ogni predicato di rinuncia (vedi [`renounce`]).
    pub movable_safe: Vec<bool>,
    /// Siti statici `LoadSlot`/`LoadVar` totali nella funzione.
    pub sites_total: u64,
    /// Quanti di quei siti sono marcati movibili.
    pub sites_movable: u64,
    /// Quanti restano movibili sotto il perimetro F2.
    pub sites_safe: u64,
}

/// Insieme di slot come bitset a parole. Le funzioni PHP hanno decine di slot,
/// non migliaia: la semplicità batte la raffinatezza in una build di misura.
#[derive(Clone, PartialEq)]
struct Bits(Vec<u64>);

impl Bits {
    fn new(nbits: usize) -> Self {
        Bits(vec![0; nbits.div_ceil(64)])
    }
    fn set(&mut self, i: u32) {
        self.0[(i / 64) as usize] |= 1 << (i % 64);
    }
    fn clear(&mut self, i: u32) {
        self.0[(i / 64) as usize] &= !(1 << (i % 64));
    }
    /// Fuori intervallo = falso (i bitset F2 possono essere più stretti di
    /// quelli dell'analisi: mai indicizzare nel vuoto in una build di misura).
    fn get(&self, i: u32) -> bool {
        self.0.get((i / 64) as usize).is_some_and(|w| w & (1 << (i % 64)) != 0)
    }
    fn set_all(&mut self) {
        for w in &mut self.0 {
            *w = !0;
        }
    }
    fn or_assign(&mut self, o: &Bits) {
        for (a, b) in self.0.iter_mut().zip(&o.0) {
            *a |= *b;
        }
    }
}

/// Effetto di un op sul dataflow degli slot. `edges` sono i successori NON di
/// fall-through; `fall` dice se `ip+1` è successore. Ogni edge porta il suo
/// insieme di def PER-ARCO (es. `IterNext` definisce `value`/`key` solo sul
/// ramo che entra nel corpo, non su quello di uscita).
struct Effect {
    uses: Vec<u32>,
    defs: Vec<u32>,
    uses_all: bool,
    fall: bool,
    /// Def che valgono solo sull'arco di fall-through (IterNext/IterNextRef).
    fall_defs: Vec<u32>,
    /// (target, def-per-arco)
    edges: Vec<(usize, Vec<u32>)>,
}

fn dim_base_use(b: &DimBase, uses: &mut Vec<u32>) {
    match b {
        DimBase::Local(s) | DimBase::Global(s) => uses.push(*s),
        DimBase::Superglobal(_) => {}
    }
}

fn field_base_use(b: &FieldBase, uses: &mut Vec<u32>) {
    match b {
        FieldBase::Local(s) | FieldBase::Global(s) => uses.push(*s),
        FieldBase::Superglobal(_) | FieldBase::This => {}
    }
}

/// Classifica `op`. `park_targets` sono TUTTI i bersagli di `ParkJump` della
/// funzione: un `EndFinally` può riprendere uno qualunque di quei salti
/// parcheggiati, quindi li riceve tutti come successori (conservativo).
fn effect(op: &Op, park_targets: &[usize]) -> Effect {
    let mut e = Effect {
        uses: Vec::new(),
        defs: Vec::new(),
        uses_all: false,
        fall: true,
        fall_defs: Vec::new(),
        edges: Vec::new(),
    };
    match op {
        // ----- letture di slot (i due bersagli della leva + i lettori puri) -----
        Op::LoadSlot(s) => e.uses.push(*s),
        Op::LoadVar { slot, .. } => e.uses.push(*slot),
        // Forme registro (WP-44 v3, riarmate S-97.1): leggono gli operandi
        // per indice (uso) e le forme *Dst scrivono `dst` per intero (def,
        // stessa classificazione di StoreSlot). Le CmpJmp* portano un arco.
        Op::BinarySS { l, r, .. } => {
            e.uses.push(*l as u32);
            e.uses.push(*r as u32);
        }
        Op::BinarySSDst { l, r, dst, .. } => {
            e.uses.push(*l as u32);
            e.uses.push(*r as u32);
            e.defs.push(*dst as u32);
        }
        Op::BinarySC { slot, .. } => e.uses.push(*slot as u32),
        Op::BinarySCDst { slot, dst, .. } => {
            e.uses.push(*slot as u32);
            e.defs.push(*dst as u32);
        }
        Op::BinaryDst { dst, .. } => e.defs.push(*dst as u32),
        // S-107 lotto superistruzioni: letture per indice (uso), la coppia
        // IncDec* è lettura+scrittura sullo stesso slot; IncDecSlotJmp è un
        // Jump incondizionato (fall=false, arco verso addr).
        Op::BinarySCSC { la, lb, .. } => {
            e.uses.push(*la as u32);
            e.uses.push(*lb as u32);
        }
        // S-106 H-A1 / S-108 lotto-2 — classificazione EMENDATA in S-108: la
        // build zval-census non compilava da S-106 (BinarySTDst/BinaryTC/
        // PropSetPop mai classificati qui — il dente A-TH-97-2 morde solo
        // quando la feature viene compilata, e nessuna sessione l'ha ricompilata
        // dopo H-A1). Dichiarato nel verbale lotto-2 (wp108-harness).
        Op::BinarySTDst { l, dst, .. } => {
            e.uses.push(*l as u32);
            e.defs.push(*dst as u32);
        }
        Op::BinarySCSCDst { la, lb, l, dst, .. } => {
            e.uses.push(*la as u32);
            e.uses.push(*lb as u32);
            e.uses.push(*l as u32);
            e.defs.push(*dst as u32);
        }
        Op::PropGetSlotRecv { recv, slot, .. } => {
            e.uses.push(*recv as u32);
            e.uses.push(*slot as u32);
        }
        Op::LoadVarPushConst { slot, .. } => e.uses.push(*slot as u32),
        Op::PropGetSlot { slot, .. } | Op::StringifySlot { slot } => {
            e.uses.push(*slot as u32)
        }
        Op::IncDecSlotPop { slot, .. } => {
            e.uses.push(*slot as u32);
            e.defs.push(*slot as u32);
        }
        Op::IncDecSlotJmp { slot, addr, .. } => {
            e.uses.push(*slot as u32);
            e.defs.push(*slot as u32);
            e.fall = false;
            e.edges.push((*addr as usize, Vec::new()));
        }
        Op::CmpJmpSS { l, r, addr, .. } => {
            e.uses.push(*l as u32);
            e.uses.push(*r as u32);
            e.edges.push((*addr as usize, Vec::new()));
        }
        Op::CmpJmpSC { slot, addr, .. } => {
            e.uses.push(*slot as u32);
            e.edges.push((*addr as usize, Vec::new()));
        }
        Op::MatchError(s) => {
            e.uses.push(*s);
            e.fall = false;
        }
        Op::IterInitRef(s) => e.uses.push(*s),
        Op::PushRef(s) => e.uses.push(*s),
        Op::MakeClosure { captures, .. } => {
            for c in captures.iter() {
                e.uses.push(c.src);
            }
        }

        // ----- scritture pure -----
        Op::StoreSlot(s) => e.defs.push(*s),
        Op::StaticAlias { slot, .. } => e.defs.push(*slot),
        Op::CallHostBuiltinOut { out_slot, out_slot2, .. } => {
            e.defs.extend(out_slot.iter().copied());
            e.defs.extend(out_slot2.iter().copied());
        }
        Op::CallHostBuiltinScanf { out_slots, .. } => {
            e.defs.extend(out_slots.iter().flatten().copied());
        }

        // ----- lettura+scrittura sullo stesso slot -----
        Op::ConcatAssignSlot(s) | Op::IncDecSlot { slot: s, .. } | Op::CoerceParam { slot: s, .. } => {
            e.uses.push(*s);
            e.defs.push(*s);
        }
        Op::CallBuiltinRef { slot, .. }
        | Op::CallBuiltinRefSpread { slot, .. }
        | Op::CallHostBuiltinRef { slot, .. } => {
            e.uses.push(*slot);
            e.defs.push(*slot);
        }
        Op::CallArrayMultisort { arg_slots, .. } => {
            for s in arg_slots.iter().flatten() {
                e.uses.push(*s);
                e.defs.push(*s);
            }
        }

        // ----- alias fra due posti nominati -----
        Op::BindRef { target, source } => {
            dim_base_use(source, &mut e.uses);
            match target {
                DimBase::Local(t) => e.defs.push(*t),
                DimBase::Global(t) => e.uses.push(*t),
                DimBase::Superglobal(_) => {}
            }
        }
        Op::BindRefTo { base, steps } | Op::BindRefToChecked { base, steps } => {
            // Base senza step: sovrascritta per intero (def). Con step: si
            // naviga DENTRO il contenitore tenuto dallo slot (uso).
            if steps.is_empty() {
                match base {
                    FieldBase::Local(t) => e.defs.push(*t),
                    FieldBase::Global(t) => e.uses.push(*t),
                    FieldBase::Superglobal(_) | FieldBase::This => {}
                }
            } else {
                field_base_use(base, &mut e.uses);
            }
        }

        // ----- op che passano DENTRO il contenitore di uno slot (uso, mai def) -----
        Op::MakeRef { base, .. } | Op::PushArgPlace { base, .. } => field_base_use(base, &mut e.uses),
        Op::FieldAssign { base, .. }
        | Op::FieldAssignOp { base, .. }
        | Op::FieldIncDec { base, .. }
        | Op::FieldIsset { base, .. }
        | Op::FieldEmpty { base, .. }
        | Op::FieldUnset { base, .. } => field_base_use(base, &mut e.uses),
        Op::AssignPath { base, .. } | Op::AssignOpPath { base, .. } | Op::IncDecPath { base, .. } => {
            dim_base_use(base, &mut e.uses)
        }
        Op::IssetPath { base, .. } | Op::EmptyPath { base, .. } => dim_base_use(base, &mut e.uses),
        Op::UnsetPath { base, nkeys } => {
            if *nkeys == 0 {
                match base {
                    DimBase::Local(s) => e.defs.push(*s),
                    DimBase::Global(s) => e.uses.push(*s),
                    DimBase::Superglobal(_) => {}
                }
            } else {
                dim_base_use(base, &mut e.uses);
            }
        }

        // ----- $GLOBALS per slot: nel main sono gli slot correnti (uso) -----
        Op::LoadGlobal(s) | Op::IncDecGlobal { slot: s, .. } => e.uses.push(*s),
        // La scrittura di un global NON deve uccidere lo slot locale omonimo
        // dentro una funzione (sarebbe un sovraconteggio): uso, mai def.
        Op::StoreGlobal(s) => e.uses.push(*s),
        // Nome dinamico / snapshot dell'intero scope: ogni slot è potenzialmente letto.
        Op::LoadGlobals | Op::GlobalsDynAssign | Op::BindGlobalDyn => e.uses_all = true,

        // ----- controllo di flusso -----
        Op::Jump(a) => {
            e.fall = false;
            e.edges.push((*a as usize, Vec::new()));
        }
        Op::JumpIfFalse(a)
        | Op::JumpIfTrue(a)
        | Op::JumpIfNotNull(a)
        | Op::JumpIfNull(a)
        | Op::CmpJmp { addr: a, .. }
        | Op::CmpJmpConst { addr: a, .. }
        | Op::ParkJump(a) => {
            // ParkJump di per sé cade oltre (il salto avverrà all'EndFinally),
            // ma dare l'arco anche qui è solo conservativo.
            e.edges.push((*a as usize, Vec::new()));
        }
        Op::FillDefault { slot, skip } => {
            e.uses.push(*slot);
            e.edges.push((*skip as usize, Vec::new()));
        }
        Op::StaticGuard { skip, .. } => e.edges.push((*skip as usize, Vec::new())),
        Op::CatchMatch { var, body, .. } => {
            // `var` è definita solo sull'arco preso verso il corpo del catch.
            let edge_defs: Vec<u32> = var.iter().copied().collect();
            e.edges.push((*body as usize, edge_defs));
        }
        Op::IterNext { value, key, end } | Op::IterNextRef { value, key, end } => {
            // value/key sono definiti solo entrando nel corpo (fall-through);
            // sull'arco di uscita lo slot conserva l'ultimo valore (il gotcha
            // documentato di PHP), quindi lì NIENTE def.
            e.fall_defs.push(*value);
            e.fall_defs.extend(key.iter().copied());
            e.edges.push((*end as usize, Vec::new()));
        }
        Op::EndFinally { after } => {
            e.edges.push((*after as usize, Vec::new()));
            for t in park_targets {
                e.edges.push((*t, Vec::new()));
            }
        }

        // ----- terminatori -----
        Op::Ret | Op::Throw | Op::Rethrow | Op::Exit { .. } | Op::Fatal(_) => e.fall = false,

        // ----- nessun effetto sugli slot PER INDICE, fall-through -----
        // A-TH-97-2 / A-SK-97-2 (Concilio WP-97): l'elenco e' ESPLICITO e il
        // match ESAUSTIVO. Il vecchio `_ => {}` era un buco di soundness a
        // futura memoria: qualunque variante NUOVA di `Op` che leggesse o
        // scrivesse uno slot sarebbe stata classificata «nessun effetto» in
        // silenzio, e l'invariante di testata non era presidiata da nulla.
        // Ora una variante nuova NON COMPILA finche' qualcuno non la
        // classifica a mano — il decadimento silenzioso diventa rumoroso.
        //
        // Audit una-tantum dell'elenco di oggi (A-TH-97-2, richiesto):
        // `CallBuiltinRefCell` NON porta uno slot — la cella by-ref sta sulla
        // pila, prodotta da `MakeRef`, la cui base e' gia' marcata in
        // `renounce()`; `NewAnonDeferred`/`DeclareDeferred` rileggono i
        // locali del chiamante per NOME e non per indice, quindi la loro
        // sede e' la rinuncia INTERA (A-SK-97-1); `Yield`/`YieldFrom`,
        // `Eval`/`Include`, `LoadVarDyn`/`StoreVarDyn` sono qui perche' il
        // loro effetto e' sull'intera funzione, non su uno slot nominato, e
        // vive anch'esso in `renounce()`.
        Op::Alloc { .. }
        | Op::AllocDynamic { .. }
        | Op::AllocStatic { .. }
        | Op::ArrayAppendSpread { .. }
        | Op::ArrayInit { .. }
        | Op::ArrayInsert { .. }
        | Op::ArrayPush { .. }
        | Op::Binary { .. }
        // S-101: `BinaryAdd` (S-100, H-B2) è la forma pura-pila di
        // `Binary(Add)`: stessi effetti (nessuno slot per indice).
        | Op::BinaryAdd
        // S-107/S-108: forme pure-pila (const inlined, nessuno slot per
        // indice) — classificate nell'emendamento S-108 (vedi sopra).
        | Op::BinaryTC { .. }
        | Op::BinaryTCPropSetPop { .. }
        | Op::Call { .. }
        | Op::CallArgs { .. }
        | Op::CallBuiltin { .. }
        | Op::CallBuiltinRefCell { .. }
        | Op::CallBuiltinSpread { .. }
        | Op::CallHostBuiltin { .. }
        | Op::CallNamed { .. }
        | Op::CallNsFallback { .. }
        | Op::CallNsFallbackArgs { .. }
        | Op::CallSpread { .. }
        | Op::CallValue { .. }
        | Op::CallValueArgs { .. }
        | Op::Cast { .. }
        | Op::CheckArity { .. }
        | Op::ClassConst { .. }
        | Op::ClassConstDyn { .. }
        | Op::ClassConstDynamic { .. }
        | Op::ClassConstFromValue { .. }
        | Op::ClassNameScope { .. }
        | Op::ClassNameStatic { .. }
        | Op::Clone { .. }
        | Op::ClosureStatic { .. }
        | Op::CoalesceFetchDim { .. }
        | Op::ConcatN { .. }
        // S-109 F2: come ConcatN — solo pila, nessun effetto slot.
        | Op::ConcatNConst { .. }
        | Op::ConstFetch { .. }
        | Op::DeclareClass { .. }
        | Op::DeclareDeferred { .. }
        | Op::DeclareFn { .. }
        | Op::DeclareTrait { .. }
        | Op::DefineConst { .. }
        | Op::DerefTop { .. }
        | Op::Dup { .. }
        | Op::Echo { .. }
        | Op::EmitNotice { .. }
        | Op::EnumCase { .. }
        | Op::Eval { .. }
        | Op::FetchDim { .. }
        | Op::FetchDimList { .. }
        | Op::HookCall { .. }
        | Op::IncDecSuperglobal { .. }
        | Op::Include { .. }
        | Op::InitProps { .. }
        | Op::InstanceOf { .. }
        | Op::InstanceOfBuiltin { .. }
        | Op::InstanceOfDynamic { .. }
        | Op::InstanceOfStatic { .. }
        | Op::InvokeCtor { .. }
        | Op::InvokeCtorArgs { .. }
        | Op::InvokeMethod { .. }
        | Op::IterInit { .. }
        | Op::IterPop { .. }
        | Op::LoadSuperglobal { .. }
        | Op::LoadVarDyn { .. }
        | Op::MakeFcc { .. }
        | Op::MethodCall { .. }
        | Op::MethodCallArgs { .. }
        | Op::MethodCallDynamic { .. }
        | Op::MethodCallDynamicArgs { .. }
        | Op::MethodCallNamed { .. }
        | Op::NewAnonDeferred { .. }
        | Op::Nop { .. }
        | Op::ParkReturn { .. }
        | Op::Pop { .. }
        | Op::Print { .. }
        | Op::PropGet { .. }
        | Op::PropGetDynamic { .. }
        | Op::PropGetDynamicSilent { .. }
        | Op::PropGetSilent { .. }
        | Op::PropIncDec { .. }
        | Op::PropIsset { .. }
        | Op::PropIssetDyn { .. }
        | Op::PropIssetFetchGate { .. }
        | Op::PropOpSet { .. }
        | Op::PropSet { .. }
        | Op::PropSetPop { .. }
        | Op::PropUnset { .. }
        | Op::PushConst { .. }
        | Op::PushUndef { .. }
        | Op::StampThrowable { .. }
        | Op::StaticCall { .. }
        | Op::StaticCallArgs { .. }
        | Op::StaticCallDynamic { .. }
        | Op::StaticCallDynamicArgs { .. }
        | Op::StaticCallDynamicMethod { .. }
        | Op::StaticCallDynamicMethodArgs { .. }
        | Op::StaticCallTargetDynamicMethod { .. }
        | Op::StaticCallTargetDynamicMethodArgs { .. }
        | Op::StaticPropGet { .. }
        | Op::StaticPropGetDynName { .. }
        | Op::StaticPropGetDynamic { .. }
        | Op::StaticPropIncDec { .. }
        | Op::StaticPropIncDecDynamic { .. }
        | Op::StaticPropOpSet { .. }
        | Op::StaticPropOpSetDynamic { .. }
        | Op::StaticPropRef { .. }
        | Op::StaticPropSet { .. }
        | Op::StaticPropSetDynName { .. }
        | Op::StaticPropSetDynamic { .. }
        | Op::StaticStore { .. }
        | Op::StoreSuperglobal { .. }
        | Op::StoreVarDyn { .. }
        | Op::Stringify { .. }
        | Op::SuppressBegin { .. }
        | Op::SuppressEnd { .. }
        | Op::Swap { .. }
        | Op::Sweep { .. }
        | Op::This { .. }
        | Op::ThisMethodCall { .. }
        | Op::ThisPropGet { .. }
        | Op::Unary { .. }
        | Op::Yield { .. }
        | Op::YieldFrom { .. } => {}
    }
    e
}

/// Builtin che OSSERVANO lo scope del chiamante per nome: la loro presenza
/// rinuncia all'intera funzione (design95-liveness.md, elenco F2).
fn observes_scope(name: &[u8]) -> bool {
    // A-DS-97-5 / A-MS-97-5 (Concilio WP-97): `debug_zval_refcount` mancava
    // pur essendo nell'elenco F2 di design95-liveness.md — osserva il
    // refcount di un valore, che e' esattamente cio' che lo spostamento
    // cambia. `debug_zval_dump` per la stessa ragione.
    const NAMES: [&[u8]; 9] = [
        b"compact",
        b"extract",
        b"get_defined_vars",
        b"debug_backtrace",
        b"debug_print_backtrace",
        b"debug_zval_refcount",
        b"debug_zval_dump",
        b"func_get_args",
        b"func_get_arg",
    ];
    NAMES.iter().any(|n| name.eq_ignore_ascii_case(n))
}

/// F2 — il perimetro conservativo. Restituisce `(rinuncia_funzione,
/// slot_rinunciati)`: la prima scatta sui modi in cui PHP vede TUTTI i locali
/// fuori dal flusso (eval/include, `$$x`, `compact` & co., generatori — lo
/// stato sopravvive alla sospensione); il secondo sui singoli slot che
/// possono diventare CONDIVISI (`Zval::Ref`): `&$x`, `global $x`, `static $x`,
/// closure `use (&$x)`, `foreach ... as &$v`, parametri by-ref — spostare un
/// valore da uno slot condiviso sarebbe osservabile altrove.
fn renounce(func: &Func) -> (bool, Bits) {
    let mut nbits = (func.n_slots + func.max_temps) as usize;
    // Le stesse difese di larghezza di `analyze`.
    for op in &func.ops {
        if let Op::LoadSlot(s) | Op::LoadVar { slot: s, .. } = op {
            nbits = nbits.max(*s as usize + 1);
        }
    }
    let mut slots = Bits::new(nbits.max(func.param_by_ref.len()));
    let mut whole = func.is_generator;
    let mut mark = |s: u32, b: &mut Bits| {
        if (s as usize) < nbits.max(func.param_by_ref.len()) {
            b.set(s);
        }
    };
    for (i, by_ref) in func.param_by_ref.iter().enumerate() {
        if *by_ref {
            mark(i as u32, &mut slots);
        }
    }
    for op in &func.ops {
        match op {
            Op::Eval | Op::Include { .. } => whole = true,
            Op::LoadVarDyn | Op::StoreVarDyn | Op::BindGlobalDyn | Op::GlobalsDynAssign | Op::LoadGlobals => {
                whole = true
            }
            // A-SK-97-1 (Klabnik, Concilio WP-97 — buco di matrice): gli
            // argomenti del costruttore di `NewAnonDeferred` si RI-VALUTANO
            // «nel bridged scope del chiamante» (bytecode.rs §deferred): legge
            // i locali per NOME a runtime, esattamente come `eval`, e cadeva
            // nel wildcard di entrambe le funzioni. `DeclareDeferred` e' la
            // stessa strada di ri-lowering: rinuncia per prudenza, non perche'
            // sia provato che legga lo scope.
            Op::NewAnonDeferred { .. } | Op::DeclareDeferred { .. } => whole = true,
            Op::CallBuiltin { name, .. }
            | Op::CallBuiltinSpread { name, .. }
            | Op::CallHostBuiltin { name, .. }
            | Op::CallHostBuiltinRef { name, .. }
            | Op::CallHostBuiltinOut { name, .. }
            | Op::CallHostBuiltinScanf { name, .. }
            | Op::CallBuiltinRef { name, .. }
            | Op::CallBuiltinRefSpread { name, .. }
            | Op::CallBuiltinRefCell { name, .. } => {
                if observes_scope(name) {
                    whole = true;
                }
            }
            Op::PushRef(s) | Op::IterInitRef(s) => mark(*s, &mut slots),
            Op::StaticAlias { slot, .. } => mark(*slot, &mut slots),
            Op::IterNextRef { value, key, .. } => {
                mark(*value, &mut slots);
                if let Some(k) = key {
                    mark(*k, &mut slots);
                }
            }
            Op::BindRef { target, source } => {
                for b in [target, source] {
                    if let DimBase::Local(s) = b {
                        mark(*s, &mut slots);
                    }
                }
            }
            Op::MakeRef { base, .. }
            | Op::PushArgPlace { base, .. }
            | Op::BindRefTo { base, .. }
            | Op::BindRefToChecked { base, .. } => {
                if let FieldBase::Local(s) = base {
                    mark(*s, &mut slots);
                }
            }
            Op::MakeClosure { captures, .. } => {
                for c in captures.iter() {
                    if c.by_ref {
                        mark(c.src, &mut slots);
                    }
                }
            }
            // A-TH-97-2 / A-SK-97-2: esaustivo anche qui. Una variante nuova
            // che rende CONDIVISO uno slot non deve poter entrare in silenzio
            // dal wildcard — il perimetro conservativo e' proprio la cosa che
            // non si accorge di essere diventata meno conservativa.
            Op::Alloc { .. }
            | Op::AllocDynamic { .. }
            | Op::AllocStatic { .. }
            | Op::ArrayAppendSpread { .. }
            | Op::ArrayInit { .. }
            | Op::ArrayInsert { .. }
            | Op::ArrayPush { .. }
            | Op::AssignOpPath { .. }
            | Op::AssignPath { .. }
            | Op::Binary { .. }
            // S-101: forma pura-pila di `Binary(Add)` (S-100, H-B2) — opera
            // solo sulla pila, non rende CONDIVISO alcuno slot.
            | Op::BinaryAdd
            // Forme registro (S-97.1): leggono per VALORE e scrivono per
            // intero via il write-through di StoreSlot — nessuna delle
            // sette rende uno slot CONDIVISO (il Ref-handling resta dentro
            // il funnel generico, che non installa alias).
            | Op::BinarySS { .. }
            | Op::BinarySSDst { .. }
            | Op::BinarySC { .. }
            | Op::BinarySCDst { .. }
            | Op::BinaryDst { .. }
            | Op::CmpJmpSS { .. }
            | Op::CmpJmpSC { .. }
            // S-106 H-A1 + lotti S-107/S-108 — classificazione EMENDATA in
            // S-108 (stessa scoperta dell'effect() qui sopra: la build
            // zval-census non compilava da S-106). Stessa ragione delle
            // forme registro: helper condivisi, nessun alias installato.
            | Op::BinarySTDst { .. }
            | Op::BinaryTC { .. }
            | Op::BinarySCSC { .. }
            | Op::IncDecSlotPop { .. }
            | Op::IncDecSlotJmp { .. }
            | Op::PropGetSlot { .. }
            | Op::PropSetPop { .. }
            | Op::StringifySlot { .. }
            | Op::PropGetSlotRecv { .. }
            | Op::BinaryTCPropSetPop { .. }
            | Op::BinarySCSCDst { .. }
            | Op::LoadVarPushConst { .. }
            | Op::Call { .. }
            | Op::CallArgs { .. }
            | Op::CallArrayMultisort { .. }
            | Op::CallNamed { .. }
            | Op::CallNsFallback { .. }
            | Op::CallNsFallbackArgs { .. }
            | Op::CallSpread { .. }
            | Op::CallValue { .. }
            | Op::CallValueArgs { .. }
            | Op::Cast { .. }
            | Op::CatchMatch { .. }
            | Op::CheckArity { .. }
            | Op::ClassConst { .. }
            | Op::ClassConstDyn { .. }
            | Op::ClassConstDynamic { .. }
            | Op::ClassConstFromValue { .. }
            | Op::ClassNameScope { .. }
            | Op::ClassNameStatic { .. }
            | Op::Clone { .. }
            | Op::ClosureStatic { .. }
            | Op::CmpJmp { .. }
            | Op::CmpJmpConst { .. }
            | Op::CoalesceFetchDim { .. }
            | Op::CoerceParam { .. }
            | Op::ConcatAssignSlot { .. }
            | Op::ConcatN { .. }
            | Op::ConcatNConst { .. }
            | Op::ConstFetch { .. }
            | Op::DeclareClass { .. }
            | Op::DeclareFn { .. }
            | Op::DeclareTrait { .. }
            | Op::DefineConst { .. }
            | Op::DerefTop { .. }
            | Op::Dup { .. }
            | Op::Echo { .. }
            | Op::EmitNotice { .. }
            | Op::EmptyPath { .. }
            | Op::EndFinally { .. }
            | Op::EnumCase { .. }
            | Op::Exit { .. }
            | Op::Fatal { .. }
            | Op::FetchDim { .. }
            | Op::FetchDimList { .. }
            | Op::FieldAssign { .. }
            | Op::FieldAssignOp { .. }
            | Op::FieldEmpty { .. }
            | Op::FieldIncDec { .. }
            | Op::FieldIsset { .. }
            | Op::FieldUnset { .. }
            | Op::FillDefault { .. }
            | Op::HookCall { .. }
            | Op::IncDecGlobal { .. }
            | Op::IncDecPath { .. }
            | Op::IncDecSlot { .. }
            | Op::IncDecSuperglobal { .. }
            | Op::InitProps { .. }
            | Op::InstanceOf { .. }
            | Op::InstanceOfBuiltin { .. }
            | Op::InstanceOfDynamic { .. }
            | Op::InstanceOfStatic { .. }
            | Op::InvokeCtor { .. }
            | Op::InvokeCtorArgs { .. }
            | Op::InvokeMethod { .. }
            | Op::IssetPath { .. }
            | Op::IterInit { .. }
            | Op::IterNext { .. }
            | Op::IterPop { .. }
            | Op::Jump { .. }
            | Op::JumpIfFalse { .. }
            | Op::JumpIfNotNull { .. }
            | Op::JumpIfNull { .. }
            | Op::JumpIfTrue { .. }
            | Op::LoadGlobal { .. }
            | Op::LoadSlot { .. }
            | Op::LoadSuperglobal { .. }
            | Op::LoadVar { .. }
            | Op::MakeFcc { .. }
            | Op::MatchError { .. }
            | Op::MethodCall { .. }
            | Op::MethodCallArgs { .. }
            | Op::MethodCallDynamic { .. }
            | Op::MethodCallDynamicArgs { .. }
            | Op::MethodCallNamed { .. }
            | Op::Nop { .. }
            | Op::ParkJump { .. }
            | Op::ParkReturn { .. }
            | Op::Pop { .. }
            | Op::Print { .. }
            | Op::PropGet { .. }
            | Op::PropGetDynamic { .. }
            | Op::PropGetDynamicSilent { .. }
            | Op::PropGetSilent { .. }
            | Op::PropIncDec { .. }
            | Op::PropIsset { .. }
            | Op::PropIssetDyn { .. }
            | Op::PropIssetFetchGate { .. }
            | Op::PropOpSet { .. }
            | Op::PropSet { .. }
            | Op::PropUnset { .. }
            | Op::PushConst { .. }
            | Op::PushUndef { .. }
            | Op::Ret { .. }
            | Op::Rethrow { .. }
            | Op::StampThrowable { .. }
            | Op::StaticCall { .. }
            | Op::StaticCallArgs { .. }
            | Op::StaticCallDynamic { .. }
            | Op::StaticCallDynamicArgs { .. }
            | Op::StaticCallDynamicMethod { .. }
            | Op::StaticCallDynamicMethodArgs { .. }
            | Op::StaticCallTargetDynamicMethod { .. }
            | Op::StaticCallTargetDynamicMethodArgs { .. }
            | Op::StaticGuard { .. }
            | Op::StaticPropGet { .. }
            | Op::StaticPropGetDynName { .. }
            | Op::StaticPropGetDynamic { .. }
            | Op::StaticPropIncDec { .. }
            | Op::StaticPropIncDecDynamic { .. }
            | Op::StaticPropOpSet { .. }
            | Op::StaticPropOpSetDynamic { .. }
            | Op::StaticPropRef { .. }
            | Op::StaticPropSet { .. }
            | Op::StaticPropSetDynName { .. }
            | Op::StaticPropSetDynamic { .. }
            | Op::StaticStore { .. }
            | Op::StoreGlobal { .. }
            | Op::StoreSlot { .. }
            | Op::StoreSuperglobal { .. }
            | Op::Stringify { .. }
            | Op::SuppressBegin { .. }
            | Op::SuppressEnd { .. }
            | Op::Swap { .. }
            | Op::Sweep { .. }
            | Op::This { .. }
            | Op::ThisMethodCall { .. }
            | Op::ThisPropGet { .. }
            | Op::Throw { .. }
            | Op::Unary { .. }
            | Op::UnsetPath { .. }
            | Op::Yield { .. }
            | Op::YieldFrom { .. } => {}
        }
    }
    (whole, slots)
}

/// Analizza una funzione compilata. Costo O(ops × slot-words × iterazioni):
/// build di misura, la semplicità è un pregio.
pub fn analyze(func: &Func) -> Analysis {
    let ops = &func.ops;
    let n = ops.len();
    let park_targets: Vec<usize> = ops
        .iter()
        .filter_map(|o| match o {
            Op::ParkJump(a) => Some(*a as usize),
            _ => None,
        })
        .collect();
    let effects: Vec<Effect> = ops.iter().map(|o| effect(o, &park_targets)).collect();

    // Larghezza del bitset: gli slot dichiarati più i temp registro, allargata
    // se un op referenzia oltre (difensivo: mai indicizzare fuori).
    let mut nbits = (func.n_slots + func.max_temps) as usize;
    for e in &effects {
        for s in e.uses.iter().chain(&e.defs).chain(&e.fall_defs) {
            nbits = nbits.max(*s as usize + 1);
        }
    }

    // Archi eccezionali: ogni op dentro una regione protetta può saltare al
    // suo handler (nessun def d'arco).
    let mut exc_edges: Vec<Vec<usize>> = vec![Vec::new(); n];
    for r in &func.exc_table {
        let (start, end) = (r.start as usize, (r.end as usize).min(n));
        for slot_edges in exc_edges.iter_mut().take(end).skip(start) {
            slot_edges.push(r.target as usize);
        }
    }

    let mut live_in: Vec<Bits> = (0..n).map(|_| Bits::new(nbits)).collect();
    let mut live_out: Vec<Bits> = (0..n).map(|_| Bits::new(nbits)).collect();

    // Punto fisso all'indietro. La terminazione è garantita: i live set solo crescono.
    let mut changed = true;
    while changed {
        changed = false;
        for i in (0..n).rev() {
            let e = &effects[i];
            let mut out = Bits::new(nbits);
            if e.fall && i + 1 < n {
                let mut t = live_in[i + 1].clone();
                for d in &e.fall_defs {
                    t.clear(*d);
                }
                out.or_assign(&t);
            }
            for (tgt, edefs) in &e.edges {
                if *tgt < n {
                    let mut t = live_in[*tgt].clone();
                    for d in edefs {
                        t.clear(*d);
                    }
                    out.or_assign(&t);
                }
            }
            // A-TH-97-1 (Hoare, Concilio WP-97 — REFUTAZIONE CAPITALE): il
            // contributo dell'arco eccezionale NON deve subire il kill delle
            // def di `i`. Quando l'eccezione parte, la def puo' non essere
            // ancora avvenuta: `CallHostBuiltinOut { out_slot: $m }` che lancia
            // un TypeError PRIMA di scrivere l'out lascia `$m` col vecchio
            // valore, e il `catch` lo legge. Fondendo il contributo exc DENTRO
            // `out` e poi sottraendo le def, `$m` spariva da `live_out` del
            // lettore precedente e diventava «spostabile»: con `TakeSlot` il
            // catch avrebbe visto `Undef` dove Zend stampa il valore.
            // Quindi: `out` (che e' il giudice della movibilita') porta il
            // contributo exc, ma `inb` lo riceve DOPO il kill, mai prima.
            let mut inb = out.clone();
            for d in &e.defs {
                inb.clear(*d);
            }
            for tgt in &exc_edges[i] {
                if *tgt < n {
                    out.or_assign(&live_in[*tgt]);
                    inb.or_assign(&live_in[*tgt]);
                }
            }
            if e.uses_all {
                inb.set_all();
            }
            for u in &e.uses {
                inb.set(*u);
            }
            if inb != live_in[i] {
                live_in[i] = inb;
                changed = true;
            }
            live_out[i] = out;
        }
    }

    // F2: il perimetro conservativo sopra il dataflow F1. `in_region[i]` è la
    // rinuncia sulle regioni protette (design95: un salto non locale può
    // rendere vivo ciò che il flusso lineare dava per morto — l'analisi QUI
    // modella già quegli archi, ma la prudenza F2 rinuncia lo stesso e la
    // differenza F1−F2 dice quanto costa).
    let (whole_renounced, slots_renounced) = renounce(func);
    let mut in_region = vec![false; n];
    for r in &func.exc_table {
        let (start, end) = (r.start as usize, (r.end as usize).min(n));
        for f in in_region.iter_mut().take(end).skip(start) {
            *f = true;
        }
    }

    let mut movable = vec![false; n];
    let mut movable_safe = vec![false; n];
    let mut sites_total = 0u64;
    let mut sites_movable = 0u64;
    let mut sites_safe = 0u64;
    for (i, op) in ops.iter().enumerate() {
        let s = match op {
            Op::LoadSlot(s) => *s,
            Op::LoadVar { slot, .. } => *slot,
            _ => continue,
        };
        sites_total += 1;
        if !live_out[i].get(s) {
            movable[i] = true;
            sites_movable += 1;
            if !whole_renounced && !slots_renounced.get(s) && !in_region[i] {
                movable_safe[i] = true;
                sites_safe += 1;
            }
        }
    }
    Analysis { movable, movable_safe, sites_total, sites_movable, sites_safe }
}
