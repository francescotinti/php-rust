# s145-sonda-b-modello — modello operativo PRE-REGISTRATO della sonda-B (regola madre: s144-criterio-B.md p.2–3, GIÀ firmata; questo file la ISTANZIA prima di ogni dato)

## Due probe, contrasti mai incrociati sul tempo
- **Probe PREZZI** (`--features sonda-price`, feature NUOVA che gate SOLO il
  builtin `__phpr_sonda_b`): nessun contatore census attivo ⇒ i cammini
  clone/drop/gc_note prezzati sono BYTE della forma di parità. Tutti i
  contrasti di tempo (seg−cal, seg_classe−seg_scalar) vivono DENTRO questo
  binario (monobinaria per costruzione).
- **Probe CONTEGGI** (`--features "mem-census zval-census"`, classe tranche-2):
  SOLO conteggi deterministici su SUITE ORM ×2, r1==r2 atteso ESATTO, parità
  per NOME vs baseline16. MAI cifre di tempo da qui.
- La partizione combina prezzo×conteggio: lecito (nessun contrasto di tempo
  tra binari; il conteggio non è tempo). Stessa forma deliberata in S-143
  (census voce a + sonda prezzi voce c).

## Segmenti di prezzo (loop N fisso, `Instant`, cal sottratto; ogni iter =
## clone+drop ⇒ il segmento prezza la COPPIA per-movimento)
mv_scalar (Long) · mv_str (ZStr condivisa) · mv_arr (Rc<PhpArray> condiviso) ·
mv_obj (Rc<RefCell<Object>> condiviso) · note_scalar (gc_note su Long: guard
inline) · note_cont_repeat (gc_note su Object GIÀ bufferizzato) · pair_zcell
(zcell(Long): alloc+free vera) · pair_arr0 (Rc::new(PhpArray vuoto)).
N: 200M per mv_*/note_*, 20M per pair_*. Ripetizione ×2 (criterio p.2),
r1≈r2 ≤1% per chiave; oltre ⇒ dichiara e replica (p.9).

## Conteggi (probe CONTEGGI, suite ORM)
- `s145.clone_{scalar,str,arr,obj,ref,rcother}_n`: impl Clone MANUALE di Zval
  sotto `mem-census` (senza feature resta `derive(Clone)`: parità intatta).
- `s145.gcnote_cont_n` (nuovo, zvalcensus): note con `is_gc_container()`
  vero; scalar/total già contati (S-101).
- nascite pair dai contatori esistenti (ch_*, s144.rczval_n): contesto del
  prezzo pair, FUORI dal denominatore della partizione.

## Partizione (parser committato + golden PRIMA del run; denominatore = somma
## delle tre componenti, dal sorgente della sonda — criterio p.2)
- memcpy  = mv_scalar × Σ clone_n                     (dispatch+move+glue)
- inc-dec = (mv_str−mv_scalar)×str + (mv_arr−mv_scalar)×arr
            + (mv_obj−mv_scalar)×(obj+ref+rcother)    [ref/rcother al prezzo
            obj: stessa inc/dec di RcBox — DICHIARATO]
- nota    = note_scalar×(gcnote_total−gcnote_cont) + note_cont_repeat×gcnote_cont
- Approssimazioni DICHIARATE: ogni clone paga UN drop (coppia per-movimento;
  il drop delle nascite sta nel prezzo pair) · sovrapprezzo first-note NON
  prezzato, nominato (bound: nascite obj) · pair obj non prezzato (bound:
  ch_obj nascite ≪ movimenti).
- Decisione = s144-criterio-B.md p.3 sulle TRE quote (≥60% inc-dec+nota ⇒
  B1→B2 · ≥60% memcpy ⇒ B3/concilio · nessuno ⇒ contributo assoluto).
- Prezzi pair_zcell/pair_arr0 = numeri FIRMATI a sé (mandato Hoare §7.4),
  mai dentro le quote.

## Igiene
Lock `/private/tmp/phpr-measure.lock` per l'intera finestra · quiescenza
cargo/rustc/rust-analyzer (KILL RA) per i SOLI run di prezzo · sentinelle
stampate non-gate per il census (S-143 p.1) · smoke a esito ESATTO per chiave
(prezzi: tutte le chiavi presenti e >0; conteggi: ≥1 su smoke145.php) ·
probe MAI pinnabili · pin s142 ripristinato da stash a fine sonda.
