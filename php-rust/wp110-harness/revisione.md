# Revisione S-110 — lente MISURA (revisore singolo, REGOLE §7)

**Verifiche fatte sui raw**: ratios dai .time quadrano (447,60/835,50 → 1,867; derivazione meccanica confermata); parità per NOME confermata (diff = solo wp_is_stream ×2); parse l1i rieseguito indipendentemente.

## Claim 1 (coppia in banda) — NON invalidato, ma SOVRA-ENUNCIATO
La banda [1,81;1,88] è pre-registrata legittimamente (ded4a74 prima della run), però la sua larghezza (+0,038) è EREDITATA per analogia dal cap S-109, mai derivata da rumore misurato. La prova interna che la morde: la gamba OFF passa 1,911→1,869 tra le due sere (Δ=−0,042, PIÙ della larghezza banda) e le due gambe si muovono in direzioni OPPOSTE rispetto ai riferimenti (ON +0,025, OFF −0,042): il rumore tra-sere per-gamba è ≥ della banda intera. Con N=1 per gamba e ricetta non-ABAB (ammessa come collaudo in-famiglia, non come A/B fine), 1,867 NON distingue «rumore» da «regressione ~1,4% del lotto-3». Enunciato sostenibile: «il lotto-3 costa AL PIÙ ~2% sul full», non «non paga». «Il divario bimodale si chiude a rumore» (ON~OFF) è post-hoc: nessuna banda propria pre-registrata.

## Claim 2 (tesi frontend) — firma valida, due difetti di misura + un tetto taciuto
1. **N mal etichettato**: `finestre_per_run` nel verdetto conta le RIGHE (nome×finestra), 4× le finestre vere (oracle arith r1: 677 finestre, non 2708). Lettera di «N dal sorgente» rispettata, sostanza no.
2. **Spread R=3 non pubblicato**: ricalcolato qui — stretto ovunque tranne oracle-prop r3 delivery 0,0103 vs 0,0170/0,0171 (−39%); la mediana regge, direzione invariata, ma andava nel verbale.
3. **9,75× è una CIFRA da contrasto tra binari** (REGOLE §3): difendibile solo come soglia pre-registrata superata (≥2× con controllo arr 1,04×), non come magnitudine che viaggia in MEMORY/README.
4. **Tetto della leva**: dalle quote per-motore (lecite), delivery phpr arith = 0,325 ⇒ azzerarla TUTTA vale al massimo ×1,48 ⇒ arith 9,3→~6,3×. La firma frontend non promette la chiusura del gap: threaded-dispatch ha un tetto dichiarabile oggi.

## Azioni
1. Riformulare nei file di rotazione: «lotto-3 ≤ +0,038 (~2%) sul full»; retrocedere «si chiude a rumore» a osservazione senza banda.
2. Caratterizzare il rumore tra-sere della coppia (3 serate, stesso pin, spread pubblicato) e derivarne la prossima banda invece di ereditarla.
3. Emendare il verdetto l1i: etichetta righe→finestre (/4) e mediane per-run pubblicate (outlier oracle-prop r3 nominato).
4. Nel criterio S-111 pre-registrare il tetto ×1,48 su arith come banda attesa della leva threaded-dispatch.
5. Nelle sintesi che viaggiano declassare 9,75×/5,96× a «soglia ≥2× superata con margine».

---
## Recepimento (S-110, stessa sessione)
- **Az.1 RECEPITA**: WP_SESSION_110 + GAP_TREND riformulati («costa al più ~2%»;
  «ON~OFF stasera» retrocesso a osservazione senza banda propria).
- **Az.3 RECEPITA**: emendamento DICHIARATO in coda a s110-l1i-verdetto.out
  (finestre = righe/4; outlier oracle-prop r3 delivery 0,0103 nominato).
- **Az.5 RECEPITA**: MEMORY declassata a quote per-motore + «soglia ≥2×
  superata»; README già in quote per-motore (32,5% vs 3,3%).
- **Az.4 NOMINATA in S-111** (ordine §1: tetto ×1,48 nel criterio della leva).
- **Az.2 A S-111** (tre serate stesso pin: lavoro di misura, non di verbale).
