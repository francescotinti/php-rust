# design91-ledger.md — DELIBERA UNICA di formato ledger (S-91.0 p3, Concilio WP-92 T5)

Sei emendamenti toccano le STESSE DUE grammatiche di ledger
(battery-attempts + campaign). Emessi separatamente sarebbero tre
revisioni di formato in una sessione, ognuna delle quali invalida i
checker che la leggono — qui sono UNA revisione (grammar v2), coi
checker aggiornati nello stesso commit (T5 team-cifre, vincolante).

## Grammatica v2 — battery-attempts.ledger (righe `battery=91pre` in poi)

Riga: `attempt_epoch=<s> battery=<id> rev=<short> esito=<E> [k=v ...] writer=<W> [sha256=<64hex>]`

- **A-AH58 `writer=`**: OGNI riga porta `writer=script:<sha16(script)>`
  (emessa da `att_row`) oppure `writer=operator`. `esito=ABORT` è legale
  SOLO con `writer=operator` — un ABORT è un atto dell'operatore. Lo
  script 91pre TRAPpa INT/TERM/HUP e ledgera l'ABORT DA SÉ (il segnale È
  l'atto dell'operatore, registrato a macchina) e rileva l'HEAD-move
  prima di OGNI riga terminale (il failure mode dell'a1 S-90.0, dove la
  riga ABORT fu scritta a mano con grammatica identica a quelle di
  att_row: la provenienza era una convenzione, non un campo).
- **A-AH59 ancora su OGNI esito**: FAIL, REFUSE e ABORT portano
  `sha256=<sha256(OUT-parziale)>` — il triangolo attempts↔OUT, chiuso
  sul PASS da A-AH54, chiude su ogni esito (mai «assente con motivo»).
- Esiti ammessi: {PASS, FAIL, REFUSE, ABORT} — chiuso, il checker
  rifiuta il resto.

**Checker aggiornato nello stesso commit**: `battery-equivalence.sh`
enforce la v2 su TUTTE le righe delle battery `9[1-9]*` alla
consumazione (KS-AH-92-1: riga senza writer= valido ⇒ consumazione
VOID; KS-AH-92-2: FAIL/REFUSE/ABORT senza ancora ⇒ battery VOID). Le
righe 89pre/90pre restano come scritte (grammar v1, dichiarato: la
storia non si rigiudica).

## Grammatica v2 — campaign ledger (`m91.campaign.ledger` in poi)

- **A-AH60 `campaign_sha=` sulle righe verdict**: ogni riga
  `phase=verdict` porta il `campaign_sha=` letto dalla riga
  `phase=start` — il verdetto è legato all'attempt per CONTENUTO, non
  solo per numero.
- **A-BG58 `reason=` autosufficiente**: le righe verdict/supersede
  portano `reason=requalify:<blocco>:<old>-><new>` (es.
  `requalify:VCKPT:want-1/3->1/1`) — il ledger da solo rigenera la
  storia g(n)→g(n+1) senza aprire i verdict file (KS-BG-92-2: reason
  non ricostruibile ⇒ riga non autosufficiente, supersede invalido
  alla campagna successiva).
- **A-PP-65 autorizzazione del giudice**: ogni cambio di `judge_sha`
  dopo `phase=start` richiede una riga
  `phase=authorize judge_sha=<nuovo> gG=<atto>` PRIMA della riga
  verdict che lo usa (KS-PP-92-2: verdict con judge_sha ≠ pinned senza
  riga di autorizzazione ⇒ generazione VOID). In S-90.0 g1→g2 avvenne
  con `supersede_of=g1` ma NESSUNA riga autorizzava la sostituzione
  del giudice: la catena gG era fuori banda.
- **A-SK-72 supersessione provata** (attuato lato gate in p2/D): una
  citazione di generazione non-massima è legale SOLO se il ledger
  committato porta la prova (riga `esito=FAIL` per quella generazione
  o `supersede_of=g<G>`); il wording sulla riga citante non decide
  nulla; la regex di citazione vede anche le forme senza `.out`.

**Attuazione**: la grammatica campaign v2 vive in
`measure91-campaign.sh`/`verdict91.sh` (canale iterazione 3, p5) —
NESSUNA campagna m91 può partire con la grammatica v1; il giudice
verdict91 verifica authorize/campaign_sha/reason come denti propri.

## Scope e non-retroattività

La v2 governa `battery=91pre`+ e `m91`+. I ledger storici (m89: g1/g2
con `esito=FAIL` senza righe supersede; m90: `supersede_of=g1`) restano
la prova A-SK-72 così come sono — il gate li legge come committati.
