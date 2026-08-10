# WP_SESSION_127 — leva «stampo» SPEDITA + cure composer (DUE pin: s127, s127b)

**In una frase**: ogni classe ora fotografa una volta la propria tabella di
default e ogni `new` la clona (−20% su crea+distruggi un oggetto); poi, su
richiesta utente, il composer «muto» non è stato solo diagnosticato ma CURATO:
`composer install` completa su phpr con vendor/ costruito, e il Fatal muto
non esiste più.

**SCOREBOARD** (pin **s127b ccb63dca**f565cffc; micro R=5 dal gate s127b):
**arith 5,3 ↓ · prop 5,6 = · calls 4,9 ↑(*) · str 4,2 = · arr 3,2 = · re 2,6 =**
((*) bordo run-to-run, da osservare) · oggetti: **objalloc 7,7× · churn 8,9×**
(A/B s127 +250,0) · rif WP full = 1,815–1,896 (S-125 @ s124) · **leve spedite:
1 (L-OL1-F1)** + ondata-2 fedeltà · incidenti: +1 apparato (LSP). 2026-08-10/11.
## Esiti secchi
1·az.rev. S-126 #1-#5 TUTTE EVASE; **additività churn CHIUSA 3,4 ns bilaterale**
  (Δins 320 + Δdrop 90; drop differito GRATIS per l'oracle; interp diluiva: 12,3×).
2·**L-OL1-F1 «stampo»** end-to-end: census 9 alloc/oggetto → forma (OnceCell,
  snapshot al Ret del thunk, default COW) → sonde IDENTICHE → admission2
  fuori-predizione DIAGNOSTICATO (modello −2+w, 6/6) → **A/B +250,0 (−20,4%)**,
  8 guardie ok (EMENDA dichiarata: guardie sui giudici scalati) → gate pieni →
  **PIN s127 834f5e01**. Corpus EMENDATO 1415→**1414**: bug69534 flippa VERDE
  (int(8)→int(2)==oracle, stesso meccanismo COW).
3·**Bisezione compoff CHIUSA e POI CURATA (ondata-2, richiesta utente)**:
  §3.19-bis dispatch dinamico exec/system/passthru/proc_open (+warning by-ref
  oracle-fedele) · §3.19-ter display_errors=stderr = destinazione · +FILTER_
  FLAG_EMAIL_UNICODE · +drift SimpleXML lista cased · +iconv dichiarata (patto
  test-driven) con costanti. Sonde bilaterali IDENTICHE (out+err+rc);
  `composer --version` stdout BYTE-ID; **`composer install` rc=0 vendor_ok**.
  Gate pieni di nuovo → **PIN s127b ccb63dca** + server bc95ba71. Residui
  nominati: §3.19-quater (log CLI→stderr assente) · §3.19-quinquies (phpcs
  config-set) · iconv superficie parziale. Voce mappa compoff RIAPERTA.
4·az.rev. S-127 (lente processo): attestazione rc corretta (az.1 eseguita;
  la catena s127b cita già il SUO file rc); #2-#5 in NEXT_SESSION.
## ⭐ Lezioni (max 3)
- ⭐⭐ Un «fuori predizione» spiegato da UN modello su TUTTE le celle è una
  scoperta (il thunk costava 2 alloc), non un fallimento della leva.
- ⭐⭐ Una bisezione chiusa può diventare cura nella stessa sessione se ogni
  strato ha la sua sonda bilaterale: 5 strati pelati (exec→stderr→FILTER→
  SimpleXML→iconv) ciascuno con verdetto oracle-fedele PRIMA del successivo.
- ⭐ Una guardia con bande nate su ALTRI giudici morde per metrica, non per merito.
