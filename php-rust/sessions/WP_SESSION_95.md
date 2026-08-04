# WP_SESSION_95 — S-95.0: A-ZV2, l'analisi di ultimo uso che ha dato i numeri (F1+F2)

**In una frase**: abbiamo costruito uno strumento che, senza cambiare nulla
del programma, conta quante volte il motore copia un valore che non servirà
mai più — e il conteggio dice che quasi metà di quelle copie si può evitare,
quindi la strada scelta per velocizzare vale la pena di essere percorsa.

**Data**: 2026-08-04 (pomeriggio; la mattina della stessa giornata aveva
prodotto la ricognizione di profiling, le consulenze Bak/Stogov e il design
A-ZV2 — questa sessione è l'ESECUZIONE di F1+F2).

## Oggetto ed esito

L'oggetto era la fase F1 di A-ZV2 (`wp95-harness/design95-liveness.md`):
calcolare l'ultimo uso per slot su ogni funzione compilata e CONTARE le
letture spostabili, senza cambiare una sola emissione; decidere poi con la
regola a tre bande di §P1 (sul GUADAGNO atteso, non sulla percentuale).
La banda è risultata ALTA su entrambi gli estremi → eseguita anche F2
(perimetro conservativo), che era il passo successivo dichiarato.

- **F1 (commit b82c446 + fb0599b)**: `vm/liveness.rs` — dataflow
  all'indietro per slot (use/def per variante dell'enum `Op`, archi di
  salto + `exc_table`, def per-arco su `IterNext`/`CatchMatch`,
  `EndFinally` verso tutti i `ParkJump`), contatori `would_take*` in
  `zvalcensus`, hook nei due arm `LoadSlot`/`LoadVar` di `run_loop`.
  TUTTO dietro la feature `zval-census`: il binario di parità
  d5ce86e3342f3926 è INVARIATO (riverificato a ogni passo).
  Cifre in `wp95-harness/zvalcensus-f1.out` (VERDICT, suite media
  identica al riferimento, 16 processi): la frazione di letture rc che
  sono ultimi usi sta in `would_take_rc_su_slot_reads_rc_pct`; il
  guadagno CPU derivato sta in `guadagno_cpu_atteso_pct_min/_max` —
  **entrambi gli estremi sopra la soglia alta** → strada lunga F2→F3→F4.
  P3 derivata scritta in design95-liveness.md §P3 come promesso ex-ante.
- **F2 (commit ee3f551 + ba12519)**: predicati di rinuncia per FUNZIONE
  (generatori, eval/include, `$$x`/`LoadGlobals`, compact/extract/
  get_defined_vars/debug_backtrace/func_get_args), per SLOT (tutto ciò
  che può rendere lo slot condiviso: `&$x`, `global`, `static`,
  `use(&$x)`, foreach by-ref, param by-ref, MakeRef/BindRefTo su base
  locale), per REGIONE protetta (prudenza doppia: gli archi exc sono già
  nel CFG F1). Cifre in `wp95-harness/zvalcensus-f2.out`:
  `safe_su_would_take_pct` mostra che la prudenza taglia molto meno del
  40% → **P2 SODDISFATTA**; il perimetro F2 intero resta in banda ALTA
  (`guadagno_cpu_atteso_safe_pct_min/_max`); il solo nucleo stringhe
  (`guadagno_cpu_atteso_str_pct_min/_max`) cade in banda MEDIA.
- **Determinismo**: i contatori F1 sono riprodotti IDENTICI nel run F2
  (`nota-determinismo` in zvalcensus-f2.out) — conteggi, non campioni.
- **Controlli del meccanismo** (Bak: il contatore prima dell'orologio):
  smoke a mano con negativo che MORDE (slot vivo sul back-edge di un
  loop NON contato; `global`/`use(&$x)`/`compact` tolgono esattamente i
  siti attesi), scalari esclusi dal numeratore `_rc`, output
  byte-identico all'oracle su entrambi gli smoke.

## ⭐ Lezioni

- ⭐⭐ **La direzione dell'errore di modello va scelta, non subita**: nel
  dataflow F1 ogni scrittura non modellata SOTTOconta (innocuo) e le
  letture non modellate sono ESATTAMENTE l'elenco delle rinunce F2 — così
  la differenza F1−F2 è il prezzo della prudenza, non un errore ignoto.
- ⭐⭐ **Un PASS del gate prima del commit è vacuo per lo stato dopo il
  commit**: il corpus si legge da HEAD, quindi la sequenza giusta è
  commit locale → morso del gate sulla cardinalità nuova → budget alzato
  con delibera NELLO STESSO commit → push solo a PASS (fatto due volte,
  F1 e F2).
- ⭐⭐ **La rinuncia statica non vede il tipo a runtime**: uno slot che
  regge un `Zval::Ref` sul lato INTERNO di una closure by-ref sfugge ai
  predicati statici — quindi il futuro `TakeSlot` DEVE guardare il tipo a
  runtime (un `Ref` si de-referenzia, mai si sposta), e il taglio `_str`
  è l'unico che non anticipa mai un `__destruct` (le stringhe non ne
  hanno).
- ⭐ Il run strumentato del media group è DETERMINISTICO al contatore
  esatto su decine di milioni di eventi: la divergenza residua rispetto
  al before della mattina è di poche decine di eventi (la cifra sta nella
  `nota` di zvalcensus-f1.out) e viene dalla suite, non dallo strumento.

## NON fatti (dichiarati)

- **A-SK-93..97** (env -i + allowlist chiusa, denti T27-T30): la prima
  voce d'apparato di S-95.0 NON è stata eseguita — l'oggetto aveva
  priorità (FONDAMENTALI-first) e la falla non bloccava F1/F2 (i PASS di
  questa sessione sono stati prodotti in ambiente pulito, senza env di
  git iniettate). Resta PRIMA voce d'apparato di S-96.0.
- F3 (opcode `TakeSlot`) e F4 (coppia della sera): sono il §WP-96.
- Probe slope v2 e attribuzione slope: invariati da WP-94.

## Stato binari e processi

- phpr parità: **d5ce86e3342f3926 INVARIATO** (stash esistente, nessun
  ri-stash). php-server: f8f4295a1dcdb627 (invariato, non toccato).
- Build di strumentazione in `/Volumes/Extreme Pro/Claude/
  phpr-census-target/` (F1: 6728f8826e60a59e; F2: 2a321e3b345ba799),
  MAI in `~/Claude/php-rust-output`.
- Nessun processo orfano a fine sessione; uploads ripristinati dalla
  guardia (backup tar conservati, vedi progress dei run).
