//! S-95.0 leva A-ZV1 — i contatori del MECCANISMO per «il clone che muore
//! subito».
//!
//! Perché esiste: la predizione di `wp95-harness/design95-leva-zval.md` è
//! firmata su un MECCANISMO (quanti valori con `Rc` vengono materializzati da
//! uno slot), non su un cronometro. Bak, consulenza S-95.0: *«il contatore del
//! meccanismo prima dell'orologio — se la taglia non si muove, la leva non ha
//! agito, e qualunque Δ tempo viene da altro»*. Senza questi numeri il
//! confronto prima/dopo non è difendibile.
//!
//! Convenzione identica a `op-census`/`gc-census`: compilato SOLO dietro la
//! feature `zval-census`, che nessuna build di parità accende. Il binario di
//! release non contiene questo modulo.

use std::sync::atomic::{AtomicU64, Ordering};

use php_types::Zval;

/// Materializzazioni totali da uno slot (`read_slot`), qualunque variante.
pub static SLOT_READS: AtomicU64 = AtomicU64::new(0);
/// Quelle che hanno clonato una variante che porta un `Rc`: sono le uniche
/// che pagano refcount++ seguito da refcount-- quando la copia muore subito.
pub static SLOT_READS_RC: AtomicU64 = AtomicU64::new(0);
/// Materializzazioni evitate dalla leva (fast path servito per riferimento).
/// Prima della leva vale 0 per costruzione: è il controllo positivo che
/// distingue «la leva ha agito» da «il tempo è cambiato per altro».
pub static SLOT_READS_AVOIDED: AtomicU64 = AtomicU64::new(0);

// ----- S-95.0 A-ZV2 fase F1 (design95-liveness.md) -----
/// Esecuzioni di `LoadSlot`/`LoadVar` il cui sito è un ULTIMO USO secondo
/// l'analisi di [`super::liveness`]: letture che la leva `TakeSlot` potrebbe
/// spostare invece di clonare. SOLA MISURA: nessuna emissione cambia.
pub static WOULD_TAKE: AtomicU64 = AtomicU64::new(0);
/// Il sottoinsieme di [`WOULD_TAKE`] il cui valore porta un `Rc`: il
/// NUMERATORE della regola a tre bande di design95-liveness.md §P1
/// (il confronto è con `slot_reads_rc`, stessa esecuzione).
pub static WOULD_TAKE_RC: AtomicU64 = AtomicU64::new(0);
/// Siti statici `LoadSlot`/`LoadVar` visti dall'analisi (una volta per
/// funzione analizzata, per processo). Advisory: pesa i siti, non le esecuzioni.
pub static SITES_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Quanti di quei siti sono ultimi usi.
pub static SITES_MOVABLE: AtomicU64 = AtomicU64::new(0);
// ----- F2: il perimetro conservativo (design95-liveness.md, predizione P2) -----
/// Esecuzioni movibili che SOPRAVVIVONO ai predicati di rinuncia F2.
pub static WOULD_TAKE_SAFE: AtomicU64 = AtomicU64::new(0);
/// Il sottoinsieme rc di [`WOULD_TAKE_SAFE`]: il numeratore di P2 contro
/// `would_take_rc` (≥60% o la leva vale meno della sua complessità).
pub static WOULD_TAKE_SAFE_RC: AtomicU64 = AtomicU64::new(0);
/// Il sottoinsieme di [`WOULD_TAKE_SAFE`] il cui valore è una STRINGA: le
/// stringhe non hanno distruttori osservabili, quindi è la parte del canale
/// che un eventuale `TakeSlot` ristretto per tipo (F3) prenderebbe senza
/// toccare l'ordine dei `__destruct` — il rischio più insidioso dell'elenco.
pub static WOULD_TAKE_SAFE_STR: AtomicU64 = AtomicU64::new(0);
/// A-MS-97-1 (Concilio WP-97): il sottoinsieme di [`WOULD_TAKE_SAFE`] che a
/// RUNTIME regge un [`Zval::Ref`]. La lezione di S-95.0 è che la rinuncia
/// STATICA non vede il tipo a runtime: uno slot sul lato INTERNO di una
/// closure by-ref porta un `Ref` che `param_by_ref` non copre. Questo
/// contatore misura quanto grande è quel buco PRIMA di scrivere l'opcode: un
/// `TakeSlot` col guard di tipo lo pagherebbe come fallback, e senza il numero
/// il controllo positivo di F4 (takes + fallback = safe predetto) sarebbe
/// vacuo per costruzione.
pub static WOULD_TAKE_SAFE_REF: AtomicU64 = AtomicU64::new(0);
/// Siti che restano movibili sotto il perimetro F2.
pub static SITES_SAFE: AtomicU64 = AtomicU64::new(0);

std::thread_local! {
    /// Cache per-funzione dell'analisi di ultimo uso. Chiave: (indirizzo della
    /// `Func`, indirizzo del suo `ops`, lunghezza) — il doppio ancoraggio rende
    /// una collisione da riuso d'indirizzo un evento da coincidenza doppia,
    /// accettabile in una build di sola misura.
    static LIVENESS: std::cell::RefCell<
        std::collections::HashMap<(usize, usize, usize), std::rc::Rc<super::liveness::Analysis>>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Nota una esecuzione di `LoadSlot`/`LoadVar` all'op `ip` di `func`, PRIMA
/// della materializzazione (la cella è ancora nello slot). Analizza la
/// funzione alla prima visita e conta se questo sito è un ultimo uso.
#[inline]
pub fn note_slot_load_site(func: &crate::bytecode::Func, ip: usize, cell: &Zval) {
    let key = (
        func as *const _ as usize,
        func.ops.as_ptr() as usize,
        func.ops.len(),
    );
    let analysis = LIVENESS.with(|c| {
        let mut c = c.borrow_mut();
        std::rc::Rc::clone(c.entry(key).or_insert_with(|| {
            let a = super::liveness::analyze(func);
            SITES_TOTAL.fetch_add(a.sites_total, Ordering::Relaxed);
            SITES_MOVABLE.fetch_add(a.sites_movable, Ordering::Relaxed);
            SITES_SAFE.fetch_add(a.sites_safe, Ordering::Relaxed);
            std::rc::Rc::new(a)
        }))
    });
    if analysis.movable.get(ip).copied().unwrap_or(false) {
        WOULD_TAKE.fetch_add(1, Ordering::Relaxed);
        if zval_holds_rc(cell) {
            WOULD_TAKE_RC.fetch_add(1, Ordering::Relaxed);
        }
        if analysis.movable_safe.get(ip).copied().unwrap_or(false) {
            WOULD_TAKE_SAFE.fetch_add(1, Ordering::Relaxed);
            if zval_holds_rc(cell) {
                WOULD_TAKE_SAFE_RC.fetch_add(1, Ordering::Relaxed);
            }
            if matches!(cell, Zval::Str(_)) {
                WOULD_TAKE_SAFE_STR.fetch_add(1, Ordering::Relaxed);
            }
            // A-MS-97-1: il buco che la rinuncia statica non vede.
            if matches!(cell, Zval::Ref(_)) {
                WOULD_TAKE_SAFE_REF.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Il valore porta un `Rc`? Solo per queste varianti clone/drop costano un
/// aggiornamento di refcount; sulle altre sono una copia di parola. Discrimina
/// i numeratori `slot_reads_rc` e `would_take_rc`.
pub(super) fn zval_holds_rc(v: &Zval) -> bool {
    match v {
        Zval::Undef | Zval::Null | Zval::Bool(_) | Zval::Long(_) | Zval::Double(_) => false,
        // `Ref` clona il valore INTERNO: il costo sta lì, non nel wrapper.
        Zval::Ref(r) => zval_holds_rc(&r.borrow()),
        _ => true,
    }
}

// ----- S-101 punto 2 (ordine WP-102 §2): census dinamico specie×sito×canale
// sul percorso PROPRIETÀ. Arbitra le TRE predizioni pre-registrate di
// `wp101-harness/hc-census-predizioni.out` (P1 specie dei valori, P2 canale
// ricevitore, P3 attribuzione gc_note) — scritte PRIMA di questi contatori.
// Stessa convenzione del resto del modulo: SOLO build di strumentazione.

/// Valori transitati dal canale di LETTURA proprietà (`PropGet`/`ThisPropGet`,
/// IC-hit + fallback + fast-path WP-25 + lettura generale). P1.
pub static PROPGET_VAL: AtomicU64 = AtomicU64::new(0);
/// Il sottoinsieme refcounted ([`zval_holds_rc`]) di [`PROPGET_VAL`].
pub static PROPGET_VAL_RC: AtomicU64 = AtomicU64::new(0);
/// Valori transitati dal canale di SCRITTURA proprietà (`PropSet`, IC-hit +
/// fast-path WP-25 + prop_init). P1.
pub static PROPSET_VAL: AtomicU64 = AtomicU64::new(0);
/// Il sottoinsieme refcounted di [`PROPSET_VAL`].
pub static PROPSET_VAL_RC: AtomicU64 = AtomicU64::new(0);
/// Operandi (lhs+rhs) transitati da `BinaryDst`. P1.
pub static BINDST_OPND: AtomicU64 = AtomicU64::new(0);
/// Il sottoinsieme refcounted di [`BINDST_OPND`].
pub static BINDST_OPND_RC: AtomicU64 = AtomicU64::new(0);

/// Canale RICEVITORE (P2): clone di un handle `Rc<Object>` fatti da
/// `LoadVar`/`LoadSlot` (il push di `$o` in pila via `read_slot`).
pub static RECV_CLONE_LOAD: AtomicU64 = AtomicU64::new(0);
/// Canale RICEVITORE (P2): `obj.deref_clone()` dentro `PropGet`/`PropSet`/
/// fallback quando il target è un `Object` (bump Rc del ricevitore).
/// I DROP corrispondenti non hanno un sito contabile (fine-arm): per
/// conservazione drop_handle = clone_handle su un micro stazionario.
pub static RECV_CLONE_PROP: AtomicU64 = AtomicU64::new(0);
/// `Op::Pop` che droppa un handle `Object` (la parte del traffico ricevitore
/// che muore esplicitamente in pila, con la sua `gc_note`).
pub static RECV_DROP_POP: AtomicU64 = AtomicU64::new(0);

/// Ogni chiamata a `Vm::gc_note` (contata NEL corpo: cattura tutti i siti). P3.
pub static GCNOTE_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Chiamate a `gc_note` con argomento NON-refcounted (braccio `_ => {}`:
/// il costo è la chiamata+match, non il bookkeeping). Predizione Stogov.
pub static GCNOTE_SCALAR: AtomicU64 = AtomicU64::new(0);
/// Chiamate a `gc_note` con argomento `Object` (borrow + flag DESTRUCTED +
/// eventuale insert nel gc_buf). Predizione Matsakis.
pub static GCNOTE_OBJ: AtomicU64 = AtomicU64::new(0);
/// Le chiamate taggate al sito `Op::Pop` (una per pop riuscito).
pub static GCNOTE_SITE_POP: AtomicU64 = AtomicU64::new(0);
/// Le chiamate taggate al sito `PropSet` sul VECCHIO valore sovrascritto.
/// Il residuo `total - pop - propset_old` = altri siti (Sweep, teardown, …).
pub static GCNOTE_SITE_PROPSET_OLD: AtomicU64 = AtomicU64::new(0);

/// Specie×canale sul percorso proprietà: `chan` 0=PropGet, 1=PropSet,
/// 2=BinaryDst (operando).
#[inline]
pub fn note_prop_val(chan: u8, v: &Zval) {
    REGISTERED.call_once(|| unsafe {
        libc::atexit(dump_at_exit);
    });
    let rc = zval_holds_rc(v);
    let (t, trc) = match chan {
        0 => (&PROPGET_VAL, &PROPGET_VAL_RC),
        1 => (&PROPSET_VAL, &PROPSET_VAL_RC),
        _ => (&BINDST_OPND, &BINDST_OPND_RC),
    };
    t.fetch_add(1, Ordering::Relaxed);
    if rc {
        trc.fetch_add(1, Ordering::Relaxed);
    }
}

/// `LoadVar`/`LoadSlot`: la cella che sta per essere clonata in pila.
#[inline]
pub fn note_recv_load(cell: &Zval) {
    if matches!(cell, Zval::Object(_)) {
        RECV_CLONE_LOAD.fetch_add(1, Ordering::Relaxed);
    }
}

/// `PropGet`/`PropSet`/fallback: il target APPENA clonato con `deref_clone`.
#[inline]
pub fn note_recv_clone_prop(target: &Zval) {
    if matches!(target, Zval::Object(_)) {
        RECV_CLONE_PROP.fetch_add(1, Ordering::Relaxed);
    }
}

/// `Op::Pop`: il valore appena poppato (che sta per essere `gc_note`'d).
#[inline]
pub fn note_pop(v: &Zval) {
    GCNOTE_SITE_POP.fetch_add(1, Ordering::Relaxed);
    if matches!(v, Zval::Object(_)) {
        RECV_DROP_POP.fetch_add(1, Ordering::Relaxed);
    }
}

/// Corpo di `Vm::gc_note`: ogni chiamata, con la specie dell'argomento.
#[inline]
pub fn note_gcnote(v: &Zval) {
    REGISTERED.call_once(|| unsafe {
        libc::atexit(dump_at_exit);
    });
    GCNOTE_TOTAL.fetch_add(1, Ordering::Relaxed);
    match v {
        Zval::Undef | Zval::Null | Zval::Bool(_) | Zval::Long(_) | Zval::Double(_) => {
            GCNOTE_SCALAR.fetch_add(1, Ordering::Relaxed);
        }
        Zval::Object(_) => {
            GCNOTE_OBJ.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

/// Sito `PropSet`: la `gc_note` sul vecchio valore sovrascritto.
#[inline]
pub fn note_gcnote_site_propset_old() {
    GCNOTE_SITE_PROPSET_OLD.fetch_add(1, Ordering::Relaxed);
}

/// Riga S-101 SEPARATA (il formato della riga storica resta intatto: il
/// gate cifre la parsa così com'è).
pub fn dump_line_s101() -> String {
    format!(
        "zvalcensus_s101 propget_val={} propget_val_rc={} propset_val={} propset_val_rc={} bindst_opnd={} bindst_opnd_rc={} recv_clone_load={} recv_clone_prop={} recv_drop_pop={} gcnote_total={} gcnote_scalar={} gcnote_obj={} gcnote_site_pop={} gcnote_site_propset_old={}",
        PROPGET_VAL.load(Ordering::Relaxed),
        PROPGET_VAL_RC.load(Ordering::Relaxed),
        PROPSET_VAL.load(Ordering::Relaxed),
        PROPSET_VAL_RC.load(Ordering::Relaxed),
        BINDST_OPND.load(Ordering::Relaxed),
        BINDST_OPND_RC.load(Ordering::Relaxed),
        RECV_CLONE_LOAD.load(Ordering::Relaxed),
        RECV_CLONE_PROP.load(Ordering::Relaxed),
        RECV_DROP_POP.load(Ordering::Relaxed),
        GCNOTE_TOTAL.load(Ordering::Relaxed),
        GCNOTE_SCALAR.load(Ordering::Relaxed),
        GCNOTE_OBJ.load(Ordering::Relaxed),
        GCNOTE_SITE_POP.load(Ordering::Relaxed),
        GCNOTE_SITE_PROPSET_OLD.load(Ordering::Relaxed),
    )
}

static REGISTERED: std::sync::Once = std::sync::Once::new();

extern "C" fn dump_at_exit() {
    dump_exit();
}

#[inline]
pub fn note_slot_read(is_rc: bool) {
    // La stampa si registra alla prima nota, così il modulo è auto-contenuto e
    // non serve toccare il `main` di ogni binario. Il costo del `Once` esiste
    // solo nelle build di strumentazione.
    REGISTERED.call_once(|| unsafe {
        libc::atexit(dump_at_exit);
    });
    SLOT_READS.fetch_add(1, Ordering::Relaxed);
    if is_rc {
        SLOT_READS_RC.fetch_add(1, Ordering::Relaxed);
    }
}

#[inline]
pub fn note_avoided() {
    SLOT_READS_AVOIDED.fetch_add(1, Ordering::Relaxed);
}

/// Riga unica, formato ascii-nudo (A-BG66: nessun separatore di migliaia),
/// così il raw entra nel corpus del gate cifre senza post-elaborazione.
pub fn dump_line() -> String {
    format!(
        "zvalcensus slot_reads={} slot_reads_rc={} slot_reads_avoided={} would_take={} would_take_rc={} would_take_safe={} would_take_safe_rc={} would_take_safe_str={} would_take_safe_ref={} sites_total={} sites_movable={} sites_safe={}",
        SLOT_READS.load(Ordering::Relaxed),
        SLOT_READS_RC.load(Ordering::Relaxed),
        SLOT_READS_AVOIDED.load(Ordering::Relaxed),
        WOULD_TAKE.load(Ordering::Relaxed),
        WOULD_TAKE_RC.load(Ordering::Relaxed),
        WOULD_TAKE_SAFE.load(Ordering::Relaxed),
        WOULD_TAKE_SAFE_RC.load(Ordering::Relaxed),
        WOULD_TAKE_SAFE_STR.load(Ordering::Relaxed),
        WOULD_TAKE_SAFE_REF.load(Ordering::Relaxed),
        SITES_TOTAL.load(Ordering::Relaxed),
        SITES_MOVABLE.load(Ordering::Relaxed),
        SITES_SAFE.load(Ordering::Relaxed),
    )
}

/// Scrive i contatori a fine processo. `PHPR_ZVAL_CENSUS` è un **path**: la
/// riga viene APPESA a quel file.
///
/// Perché non su stderr (misurato, non temuto): il workload reale lancia
/// processi figli e i test ne catturano lo stderr — la riga di census del
/// figlio diventava `PHPUnit\Framework\Exception` e la prima misura è uscita
/// con 15 errori spuri. Uno strumento che parla sul canale che il misurato
/// legge non misura: partecipa. L'append da più processi è voluto: la somma
/// del workload include i figli, che eseguono PHP quanto il padre.
///
/// La env si legge QUI, a fine processo, mai nel percorso caldo (lezione
/// A-TH-73 di S-94.0).
/// S-140 leva HC1 «hint-check senza clone» — contatori del MECCANISMO
/// (convenzione A-ZV1: il controllo positivo distingue «la leva ha agito»
/// da «il tempo è cambiato per altro»).
/// Chiamate a `coerce_or_check_hint` (qualunque esito).
pub static HINT_CHECKS: AtomicU64 = AtomicU64::new(0);
/// Il sottoinsieme il cui valore porta un `Rc` ([`zval_holds_rc`]): prima
/// della leva OGNUNA paga un `deref_clone` che muore a fine check.
pub static HINT_CHECKS_RC: AtomicU64 = AtomicU64::new(0);
/// Check serviti dal cammino borrow-first SENZA clone (leva HC1). Prima
/// della leva vale 0 per costruzione.
pub static HINT_AVOIDED: AtomicU64 = AtomicU64::new(0);

/// Nota una chiamata a `coerce_or_check_hint` col valore in ingresso.
#[inline]
pub fn note_hint_check(v: &Zval) {
    REGISTERED.call_once(|| unsafe {
        libc::atexit(dump_at_exit);
    });
    HINT_CHECKS.fetch_add(1, Ordering::Relaxed);
    if zval_holds_rc(v) {
        HINT_CHECKS_RC.fetch_add(1, Ordering::Relaxed);
    }
}

/// Nota un check servito senza clone (solo la build con la leva lo tocca).
#[inline]
pub fn note_hint_avoided() {
    HINT_AVOIDED.fetch_add(1, Ordering::Relaxed);
}

/// Riga S-140 SEPARATA (le righe storiche restano byte-identiche).
pub fn dump_line_s140() -> String {
    format!(
        "zvalcensus_s140 hint_checks={} hint_checks_rc={} hint_avoided={}",
        HINT_CHECKS.load(Ordering::Relaxed),
        HINT_CHECKS_RC.load(Ordering::Relaxed),
        HINT_AVOIDED.load(Ordering::Relaxed),
    )
}

pub fn dump_exit() {
    use std::io::Write;
    let Some(path) = std::env::var_os("PHPR_ZVAL_CENSUS") else { return };
    if path.is_empty() {
        return;
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", dump_line());
        let _ = writeln!(f, "{}", dump_line_s101());
        // S-140: riga contatori hint-check (leva HC1) — riga NUOVA.
        let _ = writeln!(f, "{}", dump_line_s140());
        // S-142: contatori del meccanismo L-RD1 — riga NUOVA, SOLO quando la
        // build monta anche mem-census (i simboli non esistono altrimenti).
        #[cfg(feature = "mem-census")]
        {
            let (ra, re_, rt) = php_types::memcensus::rd1_counters();
            let _ =
                writeln!(f, "zvalcensus_s142 rd1_arrays={ra} rd1_elems={re_} rd1_tombs={rt}");
        }
        // S-102: righe del census pila operandi (modulo separato).
        let _ = writeln!(f, "{}", super::stackcensus::dump_lines());
        // S-102 (A-LE-103-1): gamba alloc a mem-census DIRETTO — byte e
        // CONTEGGI dal global_allocator contante (0/0 se questa build non
        // monta CountingMi: il campo dice anche QUALE build ha scritto).
        let (ab, fb) = php_types::memcensus::alloc_counters();
        let (an, fn_) = php_types::memcensus::alloc_event_counters();
        let _ = writeln!(
            f,
            "alloccensus galloc_bytes={ab} gfree_bytes={fb} galloc_n={an} gfree_n={fn_}"
        );
        // S-103 H-D (A-LE-104-1): realloc DISAGGREGATO + istogramma
        // size-class degli alloc puri — righe NUOVE, la riga storica resta
        // byte-identica (da S-103 galloc/gfree NON includono più i realloc).
        let (rn, ro, rnew) = php_types::memcensus::realloc_counters();
        let _ = writeln!(f, "realloccensus n={rn} old_bytes={ro} new_bytes={rnew}");
        let h = php_types::memcensus::alloc_histogram();
        let _ = writeln!(
            f,
            "allochist le16={} le32={} le48={} le64={} le96={} le128={} le256={} le512={} le1k={} le4k={} gt4k={}",
            h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7], h[8], h[9], h[10]
        );
        // S-104 H-D (A-LE-105-1): il free-hist misurato — riga NUOVA, le
        // righe storiche restano byte-identiche.
        let fh = php_types::memcensus::free_histogram();
        let _ = writeln!(
            f,
            "freehist le16={} le32={} le48={} le64={} le96={} le128={} le256={} le512={} le1k={} le4k={} gt4k={}",
            fh[0], fh[1], fh[2], fh[3], fh[4], fh[5], fh[6], fh[7], fh[8], fh[9], fh[10]
        );
        // S-105 H-D gate G2: l'arità vista da bind_params — riga NUOVA, le
        // righe storiche restano byte-identiche.
        let ar = php_types::memcensus::arity_histogram();
        let _ = writeln!(
            f,
            "argarity a0={} a1={} a2={} a3={} a4={} ge5={}",
            ar[0], ar[1], ar[2], ar[3], ar[4], ar[5]
        );
    }
}
