# Team «semantica-confine» — Concilio WP-99 (Stogov · Pedersen · Leijen)

## 1. Convergenze — GUARDIE dovute PRIMA di promozione/estensione del flag

**Mirror dei confronti (Stogov RC-1/RC-2)** — «i compare non lanciano» è FALSO (GMP `__cmp`→ValueError, BcMath `opCoerce` branda Left/Right). Il mirror vive su TRE contingenze da pinnare in fixture, non in commenti:
1. gate `operand_cmp_ok`/`bc_str_wellformed` devia le malformate al braccio *uncomparable* simmetrico;
2. il messaggio GMP è position-free PER CASO;
3. dispatch a NOME ESATTO (`overload_receiver`): le sottoclassi non ridefiniscono `__cmp`.
Invariante da scrivere: **antisimmetria di `__cmp` + errori position-free sul percorso compare**. Guardie: property-test `eval(op,a,b)==eval(mirror(op),b,a)` su fast E funnel (matrice tag × NaN/±0.0/i64::MAX±1/stringhe), snippet GMP/Number flag-on≡flag-off INCLUSI i messaggi (A-ST-99-2); contratto nome-warning WP-65 esercitato da fixture flag-on via unit-cache/{main}, o `debug_assert!` in `reg_load_slot` (A-ST-99-1); le SETTE fixture-trappola AssignOp (RHS-first, typed-ref, `.=`, overflow, ordine warning, `gc_note`, mai commutare) scritte PRIMA di qualunque fold (A-ST-99-3). KS-ST-99-1: allargare `overload_receiver` a instanceof con mirror attivo = unsound silenzioso.

**Flag (Pedersen R1)** — `enabled()` è un OnceLock lazy: `putenv("PHPR_REG_LOWER=1")` da una richiesta può decidere il modo dell'INTERO worker prima della prima compile. Guardie: lettura EAGER a confine nominato (bootstrap Vm) + test cargo che putenv non flippa il modo (A-PE-99-1); `reg=` nel vocabolario uc_log — un A/B on/off è valido SOLO come due processi a env fissato allo spawn (A-PE-99-2, KS-PE-99-2); `.unwrap_or(b"")` in `reg_load_slot` → fail-loud, mai warning col nome vuoto (A-PE-99-3).

**Confini di layout (Leijen)** — dente `N_OPS < 256` accanto al pin 48B (A-LE-99-1, KS-LE-99-2: scatta ⇒ ridisegno prima di ogni misura); «−42% opcode = guadagno footprint» è REFUTATO (dispatch dinamici ≠ byte statici): la promozione flag-on si regge sulla CPU soltanto.

## 2. Debiti di parità pendenti (per urgenza)

1. **Server restapi 3508 + option 413 per NOME sotto `env -i`** (PHPR_REG_LOWER assente dalla lista) + sentinella output-capture. **BLOCCA** qualunque uso/misura del server su 832568a72b925dd1 (KS-PE-99-1 = VOID): H-A2 è nel binario INCONDIZIONATAMENTE, l'emissione flag-off è cambiata, il collaudo è DOVUTO per la regola n.2. Non blocca però S-98.0 se l'oggetto resta CLI (H-B1) — rientra in ordine solo quando si tocca il server; timebox mezza sessione.
2. **Coppia peak `/usr/bin/time -l` agganciata al prossimo collaudo parità WP** (A-LE-99-2, zero run aggiuntivi; m90 fermo, deriva non misurata). NON blocca H-B1 (attesa footprint-neutrale) ma **BLOCCA la promozione a baseline** (KS-LE-99-1). Correzione immediata del pin server in NEXT_SESSION (A-PE-99-4, R3: due fonti di verità).

## 3. Conflitti fra le sedie

Nessun conflitto sostanziale. Unica tensione di sequenza: Pedersen rende il debito server esigibile «al primo uso», Leijen aggancia il peak «al prossimo collaudo WP» — si compongono: un solo collaudo server per NOME che acquisisce ANCHE la coppia peak nello stesso run soddisfa entrambi.

## 4. Priorità S-98.0

1. A-PE-99-4 (pin server in NEXT_SESSION) — un minuto, subito.
2. A-PE-99-1 + A-PE-99-3 (flag eager + dente putenv; fail-loud) — piccoli, chiudono R1.
3. A-ST-99-2 property-test mirror + fixture GMP/Number (con A-ST-99-1 nome-warning).
4. A-LE-99-1 dente N_OPS<256 accanto al pin 48B.
5. A-ST-99-3 (sette trappole) SOLO se S-98.0 apre il fold AssignOp; il collaudo server+peak SOLO se si tocca il server. Apparato in ordine solo se blocca l'oggetto; timebox mezza sessione.
