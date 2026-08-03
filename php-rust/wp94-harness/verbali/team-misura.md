# TEAM-MISURA — relazione di fase 2 (Concilio WP-94)

Relatore: team-misura. Fonti vincolanti: verbale-2-matsakis, verbale-6-pedersen, verbale-5-bak.
Gradi dichiarati dalle sedie: Matsakis **PASS-CONDIZIONATO, 1 refutazione capitale SÌ**;
Pedersen **PASS CON RISERVE, capitali NO**; Bak **CONFERMATO con emendamenti, capitali NO**.

## CONVERGENZE

1. **Stessa proposizione, due sedie**: l'implicazione «pre==post==0 ∧ arr_pre==arr_post ⇒
   window clean» (main.rs:273) è FALSA nel modello di memoria puro. Matsakis Q3 e Pedersen
   Q1(b) la refutano indipendentemente con controesempi distinti.
2. **Stessa causa**: nessun ordering compra *freshness*; la coherence read-read non impone di
   leggere l'ultimo valore. Il buco non è di ordering.
3. **Stessa protezione reale**: ciò che schermerebbe la finestra è il **protocollo** (driver
   sequenziale, unico client = la richiesta census che risponde prima di `dispatch`), non la
   coppia di contatori.
4. **Stesso statuto della coppia**: tripwire fail-closed NECESSARIA, non prova sufficiente
   (Matsakis) = declassata ad ADVISORY senza dichiarazione in-banda (Pedersen).
5. **Emendamenti gemelli**: A-MS-59 ≡ A-PP-70 sul testo del commento; KS-MS-94-1 ≡ KS-PP-94-1.
6. **Universo del testimone ristretto** ai soli DISPATCHED (Pedersen Q1c) — coerente con il
   cfg-split verificato sano da Matsakis Q2 sugli inc.
7. **Bak non tocca la coppia**: il suo perimetro (A-DL-59) è indipendente e non è intaccato
   dalla refutazione.

## CONFLITTI

- **C1 — grado.** Matsakis registra capitale (la *sufficienza formale dichiarata* cade);
  Pedersen no (nessuna riga misurata è falsa: si declassa un'implicazione). Composizione del
  team: **è la stessa refutazione**; la divergenza è su cosa conta come capitale (Matsakis
  giudica la dichiarazione, Pedersen l'esito misurato). Registriamo il grado più severo per il
  claim testuale, il più mite per i dati m9x già raccolti: **nessun dato va ritirato, il
  commento sì**.
- **C2 — SeqCst come via alternativa.** Pedersen offre «o si passa a SeqCst sui sei op»;
  Matsakis nega esplicitamente che qualunque rafforzamento compri freshness. **Composizione:
  SeqCst NON è sostituto della sequenzialità** — chiude solo il buco *extra* di Pedersen (load
  Relaxed di `census_arrivals_now`, worker_pool.rs:313). Resta igiene consigliata, non
  condizione.
- **C3 — asimmetria mecanica/grado.** Il controesempio di Pedersen è strettamente più forte
  (anche arr_post può mancare l'arrivo) ma porta il grado più mite. Da registrare come tale.

## GRADO DEL TESTIMONE ATTUATO

**ADVISORY.** Diventa **verdict-grade** solo con questo elenco CHIUSO, tutto in-banda e
leggibile a macchina dalla riga di fase:

1. Enforcement sequenziale del driver **dichiarato in-band** nella riga (non in un commento).
2. Trigger VOID **nominato** per dispatch-Err sul canale **mem-census** (A-MS-60/KS-MS-94-2):
   `rows==N` non esiste su quel canale, ARRIVALS resta gonfiata.
3. Universo del testimone dichiarato = sole richieste DISPATCHED (A-PP-70/Q1c).
4. Residuo dichiarato: dec dopo la send sulla oneshot, non dopo la write sul socket ⇒ free del
   body precedente dentro una finestra «pulita» (A-PP-69 → A-PP-73: O(1) alloc, taglia ∝ body,
   census esenti per costruzione).
5. Grammatica del ledger sana: sentinella su `$now` vuoto (A-PP-71) e `deferred=` dichiarato
   DERIVATO con etichetta propria per i denti non-gate (A-PP-72).

Fuori elenco (nessuno è condizione): SeqCst, rafforzamenti di ordering, doppio campo derivato.

## GRADO DEL JOIN A-DL-59

**CONSUMABILE come analisi, non come oracolo.** Ricomputo indipendente al byte su 20/20 run e
~260 valori, zero mismatch; identità vis≈bin-committed verificata per-run (dev max +0,172%
contro margine 3% del test committed-vs-used). Consumo ammesso: **refutazione dell'ipotesi
page-slack di Leijen** e collocazione dello slack DENTRO il visibile — coerenza interna,
semi-tautologica. Conclusione da declassare (A-BB73-1): «l'invisibile vive **fuori dai bin del
heap visitato**», non «ad arena/chunk»; abandoned segments, huge/OS-direct e metadata restano
indistinti, discriminazione al canale barrier A-DL-57/58. Drift dump#1↔pwork da dichiarare con
cifre (A-BB73-2: w16.r2 +1.048.576 B, w16.r3 +4.194.304 B; 18/20 identici).

## PRIORITÀ PER S-93.0

P1. Punti 1-2 dell'elenco chiuso (riga in-banda + trigger VOID mem-census): senza questi ogni Δ
di fase nasce ADVISORY.
P2. Punti 3-5 (dichiarazioni + sentinella `$now` + `deferred` derivato).
P3. A-BB73-1/2/3 nel testo di A-DL-59 (riformulazione, cifre del drift, identità come test).
P4. A-MS-61 (lsp_check: policy antenato-assente, deprecation al punto di scoperta, `lc`
ASCII-only, Err al posto di `unreachable!`) — fuori perimetro misura, da instradare al team
competente prima del wiring di fase 2.
