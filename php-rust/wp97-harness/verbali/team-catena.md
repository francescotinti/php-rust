# Team-catena (Klabnik · Hejlsberg · Pedersen) — nota di riconciliazione WP-97

## Convergenze
1. **Provenienza come precondizione dei gate**: KS-SK-97-1 (canale env-git A-SK-93..97 / denti T27-T30 chiuso PRIMA di spedire F3, altrimenti gate provisional per NOME) e A-PP-97-1 (identity con `git status --porcelain` + re-hash post-run; header dei `.out` trascritti dall'identity, mai a memoria) sono la stessa tesi su due livelli: nessun PASS di parità è verdict-grade senza catena env+albero pulita.
2. **Smoke → test committati**: A-SK-97-3 (fixture con conteggi `would_take*` esatti + negativi back-edge/`global`/`use(&$x)`/`compact`) e A-PP-97-4 (caso per-richiesta: `__destruct` a ridosso di `request_end()`, generatore sospeso oltre il confine) — un controllo non ripetibile a macchina non è un controllo.
3. **Invarianti presidiate a compile-time**: A-SK-97-2 (match esaustivo di `effect()`, via il `_ => {}`) e A-AH-97-4 (static assert `size_of::<Op>() == 48` nello stesso commit dell'opcode) condividono il principio: la variante futura deve rompere la build, non decadere in silenzio.
4. **F3 = analisi di compilazione, identità strutturale**: A-AH-97-1/3 e RC-2 (mai cache keyed-by-pointer: ABA = take sbagliato; KS-AH-97-3 reject) con KS-SK-97-2 (TakeSlot guarda il TIPO: `Zval::Ref` si de-referenzia) e KS-PP-97-1 (divergenza ordine distruttori ⇒ restrizione a `_str`).
5. **Banda F4 non usabile com'è**: RC-1/A-AH-97-5 (banda LORDA, serve A/A per il costo dispatch), A-SK-97-4 (grade per campo: conteggi VERDICT, guadagni DERIVED-ESTIMATE; margine 2,8%), A-PP-97-5 (42 eventi da nominare prima di F4).
6. **Suite per NOME**: A-PP-97-3 riafferma la regola di progetto; coerente col gate-per-nome di Klabnik.

## Conflitti
- **Validità di F2 oggi**: Klabnik PASS con riserve (aritmetica e banda ALTA robuste, identity committati); Pedersen RESPINTO IN PARTE — provenance F2 REFUTATA (header «HEAD ee3f551» contraddetto da fb0599b, build da albero non committato, irriproducibile). Hejlsberg non contesta la misura ma refuta F3 come scritto.
- **Determinismo**: Klabnik lo tratta come robusto con ignoto nominato; Pedersen lo declassa a «coppia identica, N=1» (A-PP-97-5).
- **Riparazione F2**: Klabnik → riconteggio dopo A-SK-97-1 (buco `NewAnonDeferred`); Pedersen → riparazione della provenance/header. Complementari ma con oggetto diverso.

## Priorità proposte per WP-96
1. Provenance: A-PP-97-1 (correggere `zvalcensus-f2.out`) + riconteggio F2 post A-SK-97-1, PRIMA di F3.
2. A-SK-97-2 + A-SK-97-3/A-PP-97-4 (match esaustivo; smoke promossi a test, incluso caso per-richiesta).
3. F3 solo come da A-AH-97-1/2/3, con KS-AH-97-3, KS-SK-97-2, KS-PP-97-1 armati.
4. KS-SK-97-1: chiudere A-SK-93..97 o dichiarare i gate F3 provisional per NOME.
5. Prima di F4: A-AH-97-5 (A/A), A-SK-97-4, A-PP-97-5; robustezza summer A-PP-97-2 e suite per nome A-PP-97-3.
