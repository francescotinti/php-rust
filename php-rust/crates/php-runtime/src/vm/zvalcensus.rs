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
            std::rc::Rc::new(a)
        }))
    });
    if analysis.movable.get(ip).copied().unwrap_or(false) {
        WOULD_TAKE.fetch_add(1, Ordering::Relaxed);
        if zval_holds_rc(cell) {
            WOULD_TAKE_RC.fetch_add(1, Ordering::Relaxed);
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
        "zvalcensus slot_reads={} slot_reads_rc={} slot_reads_avoided={} would_take={} would_take_rc={} sites_total={} sites_movable={}",
        SLOT_READS.load(Ordering::Relaxed),
        SLOT_READS_RC.load(Ordering::Relaxed),
        SLOT_READS_AVOIDED.load(Ordering::Relaxed),
        WOULD_TAKE.load(Ordering::Relaxed),
        WOULD_TAKE_RC.load(Ordering::Relaxed),
        SITES_TOTAL.load(Ordering::Relaxed),
        SITES_MOVABLE.load(Ordering::Relaxed),
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
pub fn dump_exit() {
    use std::io::Write;
    let Some(path) = std::env::var_os("PHPR_ZVAL_CENSUS") else { return };
    if path.is_empty() {
        return;
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", dump_line());
    }
}
