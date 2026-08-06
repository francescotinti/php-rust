//! S-102 punto 4 (Concilio WP-103, A-BA-103-1 + A-GR-103-3) — census della
//! MECCANICA DELLA PILA OPERANDI, per SITO-OPCODE e per PRIMITIVA.
//!
//! Perché esiste: il profilo inline-aware S-101 attribuisce ~26,6% del tempo
//! phpr agli accessor `Vec<Zval>` dentro `run_loop`, ma l'attribuzione a
//! campioni sui simboli inlined è DICHIARATA porosa (confine col 21,2%
//! dispatch che perde nei due sensi; errore fattore ~2 a SEGNO IGNOTO,
//! R-GR-103-1). L'unico arbitro è il CONTEGGIO: quanti transiti di pila
//! (push/pop/peek/len/elem) fa DAVVERO ogni op del giudice per iterazione.
//! VIETATO derivarne attesi via quota%×T (KS-GR-103-1 ≡ KS-BA-103-3): il
//! costo/transito farà fede SOLO da un Δ_A/B ÷ transiti contati.
//!
//! Convenzione identica a `zvalcensus`: SOLO dietro `zval-census`, nessuna
//! build di parità la accende; dump appeso al file `PHPR_ZVAL_CENSUS` (riga
//! propria `stackcensus …`, la riga storica resta byte-identica).

use std::sync::atomic::{AtomicU64, Ordering};

/// Siti-opcode strumentati: gli op del loop del giudice prop.php (13 specie)
/// + `Other` per ogni transito fuori perimetro che si volesse taggare.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Site {
    Pop = 0,
    Swap = 1,
    LoadSlot = 2,
    LoadVar = 3,
    PushConst = 4,
    IncDecSlot = 5,
    BinaryAdd = 6,
    BinaryDst = 7,
    CmpJmpSC = 8,
    Sweep = 9,
    Jump = 10,
    PropGet = 11,
    PropSet = 12,
    Other = 13,
}
pub const SITES: usize = 14;
pub const SITE_NAMES: [&str; SITES] = [
    "Pop", "Swap", "LoadSlot", "LoadVar", "PushConst", "IncDecSlot",
    "BinaryAdd", "BinaryDst", "CmpJmpSC", "Sweep", "Jump", "PropGet",
    "PropSet", "Other",
];

/// Primitive di pila a livello SORGENTE (gli `as_slice`/`ptr::read` del
/// profilo sono l'interno compilato di queste): `Push` = `Vec::push`,
/// `Pop` = `Vec::pop` (+`expect`), `Peek` = `last`/`last_mut`,
/// `Len` = `len()` esplicito, `Elem` = accesso a elemento (`swap`, index).
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Prim {
    Push = 0,
    Pop = 1,
    Peek = 2,
    Len = 3,
    Elem = 4,
    // S-103 drop-census (KS-BA-104-3 ≡ RC-LE-104-1): la fine-vita di uno
    // Zval nell'arm, per SPECIE — la specie è il canale della leva H-C2
    // (fast-out scalare). Classificata col predicato ESISTENTE
    // `is_gc_container` (KS-MA-104-1: mai un secondo predicato).
    DropS = 5,
    DropC = 6,
}
pub const PRIMS: usize = 7;
pub const PRIM_NAMES: [&str; PRIMS] =
    ["push", "pop", "peek", "len", "elem", "drop_s", "drop_c"];

/// Esecuzioni per sito-opcode (chiude il ledger: transiti = Σ conteggi e
/// l'assert conteggi↔dump confronta ops/iter col dump `{main}`).
static OPS: [AtomicU64; SITES] = [const { AtomicU64::new(0) }; SITES];
/// Transiti per (sito, primitiva).
static ST: [[AtomicU64; PRIMS]; SITES] = [const { [const { AtomicU64::new(0) }; PRIMS] }; SITES];

/// Una esecuzione dell'arm `site`.
#[inline]
pub fn note_op(site: Site) {
    OPS[site as usize].fetch_add(1, Ordering::Relaxed);
}

/// `n` transiti della primitiva `prim` al sito `site` (sul SENTIERO eseguito:
/// fast e slow path si contano dove eseguono, mai staticamente).
#[inline]
pub fn note(site: Site, prim: Prim, n: u64) {
    ST[site as usize][prim as usize].fetch_add(n, Ordering::Relaxed);
}

/// Una fine-vita di Zval al sito `site`, classificata per specie (S-103
/// drop-census). Si annota nel punto dell'arm dove la vita del valore
/// finisce (pop scartato, operando consumato, bersaglio sovrascritto,
/// temporaneo a fine arm) — sul sentiero ESEGUITO.
#[inline]
pub fn note_drop(site: Site, is_container: bool) {
    let p = if is_container { Prim::DropC } else { Prim::DropS };
    ST[site as usize][p as usize].fetch_add(1, Ordering::Relaxed);
}

/// Righe di dump: una per sito eseguito (`stackcensus site=… ops=… push=…`)
/// + una riga di totali per primitiva.
pub fn dump_lines() -> String {
    let mut out = String::new();
    let mut tot = [0u64; PRIMS];
    for s in 0..SITES {
        let ops = OPS[s].load(Ordering::Relaxed);
        let row: Vec<u64> = (0..PRIMS).map(|p| ST[s][p].load(Ordering::Relaxed)).collect();
        for (p, v) in row.iter().enumerate() {
            tot[p] += v;
        }
        if ops == 0 && row.iter().all(|&v| v == 0) {
            continue;
        }
        out.push_str(&format!(
            "stackcensus site={} ops={} push={} pop={} peek={} len={} elem={} drop_s={} drop_c={}\n",
            SITE_NAMES[s], ops, row[0], row[1], row[2], row[3], row[4], row[5], row[6]
        ));
    }
    out.push_str(&format!(
        "stackcensus_tot push={} pop={} peek={} len={} elem={} drop_s={} drop_c={}",
        tot[0], tot[1], tot[2], tot[3], tot[4], tot[5], tot[6]
    ));
    out
}
