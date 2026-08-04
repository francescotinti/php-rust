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

/// Materializzazioni totali da uno slot (`read_slot`), qualunque variante.
pub static SLOT_READS: AtomicU64 = AtomicU64::new(0);
/// Quelle che hanno clonato una variante che porta un `Rc`: sono le uniche
/// che pagano refcount++ seguito da refcount-- quando la copia muore subito.
pub static SLOT_READS_RC: AtomicU64 = AtomicU64::new(0);
/// Materializzazioni evitate dalla leva (fast path servito per riferimento).
/// Prima della leva vale 0 per costruzione: è il controllo positivo che
/// distingue «la leva ha agito» da «il tempo è cambiato per altro».
pub static SLOT_READS_AVOIDED: AtomicU64 = AtomicU64::new(0);

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
        "zvalcensus slot_reads={} slot_reads_rc={} slot_reads_avoided={}",
        SLOT_READS.load(Ordering::Relaxed),
        SLOT_READS_RC.load(Ordering::Relaxed),
        SLOT_READS_AVOIDED.load(Ordering::Relaxed),
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
