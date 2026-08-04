# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — WP-97

## VERDETTO

**PASS CON RISERVE VINCOLANTI** su S-95.0 (F1+F2 sola misura) e sul
programma §WP-96/F3. La decisione di banda ALTA regge: ho riverificato le
derivazioni dei raw (47,11%→2,12/3,06; 42,33%→1,91/2,75; 18,65%→0,84/1,21 —
aritmetica corretta) e la conclusione è robusta fino a un canale ≈2,8%
(ALTA cade solo se il canale reale è < 1,3/0,4711). Identity committati
(`f1-out/f1.identity`: sha census + sha parità + head + epoch), raw
per-processo in-repo, append multi-processo documentato, env letta solo a
exit. MA la matrice `effect()` ha UN buco reale e ZERO test a macchina.

## Emendamenti

- **A-SK-97-1 (buco di matrice)**: `Op::NewAnonDeferred` ri-valuta gli
  argomenti del costruttore «nel bridged scope del chiamante»
  (bytecode.rs §deferred): legge i locali per nome a runtime, come `eval`.
  Non è in `effect()` (cade nel `_ => {}`) né in `renounce()` (che copre
  solo Eval/Include/dyn/observes_scope/generatori). Va aggiunto al
  whole-renounce (con `DeclareDeferred` prudenziale) e F2 va ricontata
  PRIMA di F3 (delta atteso ~0, ma il conto si fa, non si presume).
- **A-SK-97-2 (decadimento silenzioso)**: eliminare il `_ => {}` di
  `effect()`: match esaustivo con elenco esplicito dei no-effect, così
  ogni variante FUTURA dell'enum `Op` con effetti su slot rompe la
  compilazione della build census invece di venire classificata
  no-effect in silenzio. L'invariante di testata oggi non è presidiata
  da nulla.
- **A-SK-97-3 (smoke → test)**: gli smoke di S-95.0 sono stati manuali e
  NON committati (nessun `#[cfg(test)]` in liveness.rs, nessun
  riferimento nei test d'integrazione). Il negativo che ha morso
  (slot vivo sul back-edge non contato) è oggi irripetibile a macchina.
  Prima o nello stesso commit di F3: fixture con conteggi `would_take*`
  ESATTI attesi + i negativi (back-edge, `global`, `use(&$x)`,
  `compact`) come test cargo/phpt.
- **A-SK-97-4 (grade misto nei raw)**: i `.out` sono grade=VERDICT ma i
  campi `guadagno_cpu_atteso_*` derivano dal canale 4,5–6,5% che è una
  STIMA di prof95 non riproducibile dal raw. Marcare il grade per campo
  (conteggi=VERDICT, guadagni=DERIVED-ESTIMATE), pinnare la derivazione
  del canale in un raw proprio, scrivere il margine di robustezza (2,8%).
  I 42 eventi di divergenza dal before restano un ignoto nominato: se in
  F4 crescono d'ordine, non è più «rumore».

## Kill-switch

- **KS-SK-97-1**: se S-96.0 spedisce F3 (commit che CAMBIA l'emissione e
  il binario) con il canale env di git ancora aperto, i PASS di parità F3
  NON sono verdict-grade. La clausola timebox «si spedisce l'oggetto e
  l'apparato torna in coda» NON si applica ai gate di un commit che tocca
  l'emissione: lì A-SK-93..97 (denti T27-T30) va chiusa prima, o i gate
  si dichiarano provisional per NOME.
- **KS-SK-97-2**: se il handler `TakeSlot` non guarda il TIPO a runtime
  (un `Zval::Ref` si de-referenzia, mai si sposta — il lato interno delle
  closure by-ref è INVISIBILE alla `Func` del callee, `param_by_ref` non
  lo copre), o se il riconteggio post A-SK-97-1 porta
  `safe_su_would_take_pct` sotto 60, F3 si ferma e la banda si ri-deriva.

## Refutazioni capitali

**Nessuna sul verdetto di banda** (ALTA sopravvive ai margini). Refutate
però due affermazioni: (1) la testata di liveness.rs — «le uniche letture
fuori modello sono ESATTAMENTE quelle del perimetro F2» — è FALSA:
`NewAnonDeferred` è fuori modello e fuori perimetro; (2) «gli smoke
bastano come controllo positivo/negativo» — no: un controllo che non può
più mordere non è un controllo, è un ricordo. L'apparato A-SK-93..97 è
correttamente prioritizzato e NON retro-blocca S-95.0 (sola misura, sha
di parità pinnato indipendentemente dall'env del gate) — ma vedi
KS-SK-97-1 per F3.
