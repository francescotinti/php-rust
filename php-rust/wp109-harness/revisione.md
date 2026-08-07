# Revisione S-109 — revisore singolo, lente PROCESSO (REGOLE §7)

## Verdetto sul claim

Il claim regge nell'ossatura: l'ordine è stato eseguito, i due criteri principali sono committati prima delle misure (5b7ea5b < 5e2713d; 6534584 19:40 < 35d9ff1 19:58) e gli emendamenti sono dichiarati. Ma è indebolito nel punto che il claim rivendica di più: l'admission pre-registrata è FALLITA (rc=1) ed è stata riscritta fino al PASS nella stessa finestra, e l'intero arco implementazione→admission v1→v2→smoke→ABAB→promozione vive in UN solo commit — la precedenza «criterio emendato PRIMA della misura» non è attestabile da git.

## Debolezza più grave

**Monocommit 35d9ff1 + emendamento admission v1→v2.** Prove: il criterio 6534584 (punto 6) pretende «gli altri CINQUE IDENTICI» sul dump; s109-admission-v1.out dà admission_rc=1 su tutti e cinque; la v2 ri-collocata su {main} passa; e .rs, script admission, entrambi i verdetti, ab-runs e ab-verdetto stanno TUTTI in 35d9ff1 (19:58, 18 minuti dopo il criterio) — violando «commit+push a ogni passo» (REGOLE §10) e rendendo il messaggio «prima di ogni riga di codice» insostenibile su quella timeline (231 righe su 5 file + build release + due admission + smoke + ABAB R=5 in 18'). Perimetro perso dalla v2: il prelude ON dei sei giudici DIVERGE (lo prova la v1) e nessun gate enumera che la divergenza sia SOLO fusione F1/F2 legittima; inoltre il dump str intero mostra 73 Unary(Neg) non fusi, invisibili al {main}.

Secondarie: (e) la batteria attesta il SORGENTE (lib ricompilata 9a2dbfa1), non il byte del pin 92909544 ripristinato — churn dichiarato, ma il session file elenca «batteria» tra i gate del pin senza la distinzione; (f) il grado server resta valido per la coppia 443ae42f+3b3d25e2, coppia rotta dal lotto-3 — nominato, ma «tocca solo l'emissione CLI» è impreciso; (d) l'rc-da-pipe fu LANCIATO e poi fermato: è una morsicatura, il verbale la conta come «sfiorata»; (g) w9a caso B parcheggiato è conforme (§9, bilaterale pre-esistente) solo se il gate stesso lo dichiara; REGOLE §5 dice ancora «corpus 1417» contro il 1415 reale.

## Azioni

1. Admission in due commit: script+attese PRIMA della build del candidato; ogni emendamento = commit proprio prima della run emendata.
2. Estendere l'admission v2: diff del prelude ON enumerato («diverge SOLO per» conteggi F1/F2 attesi), non solo {main}.
3. Nel pin-verdetto separare per iscritto gate-del-sorgente (batteria) da gate-del-byte (corpus/fixture/micro), o rieseguire la batteria sul binario stashato.
4. Aggiornare la riga corpus di REGOLE al fail-set congelato corrente.
5. Registrare l'rc-da-pipe come quarta morsicatura, non come evitamento.

---
## Recepimento (S-109, stessa sessione)

- **Azione 5 RECEPITA SUBITO**: il lancio c'è stato — è la QUARTA morsicatura
  (fermata prima del verdetto, ma morsicatura): session file e veto in
  NEXT_SESSION corretti.
- **Azione 4 RECEPITA SUBITO**: REGOLE §5 aggiornata al fail-set corrente
  (1415, congelato per NOME) con emendamento dichiarato.
- **Azione 3 RECEPITA a verbale**: la batteria di S-109 attesta il SORGENTE
  (la lib ricompilata), non il byte dello stash; corpus/fixture/micro
  attestano il BYTE (92909544 verificato FAIL-CLOSED nei gate). Distinzione
  ora scritta qui e nel session file; la scelta tra «batteria sul binario
  stashato» e «dichiarazione permanente» va a S-110.
- **Azioni 1-2 NOMINATE in S-110** (ordine §3): admission bipartita per
  commit + enumerazione del diff del prelude (conteggi F1/F2 attesi).
- Sul monocommit: fatto reale, nessuna difesa — la sequenza interna
  (criterio→v1 FAIL→emendamento v2→smoke→ABAB) è ricostruibile SOLO dai
  timestamp dei file .out, non da git. La regola «commit+push a ogni passo»
  ha ceduto alla velocità della finestra; vincolo per S-110 in testa
  all'ordine.
