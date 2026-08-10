# Criterio S-128 p.1 — rimisura full/media WP sul pin s127b (rif vecchio di 2 pin)

1. Oggetto: RIMISURA del riferimento (non leva): pin s127b ccb63dcaf565cffc vs oracle 8.5.7, stessa sera; due leve accumulate dal rif (cbargs2 + stampo L-OL1-F1).
2. R: coppia bimodale N=2 per modo, 4 gambe INTERCALATE off1→on1→off2→on2 (ricetta pair109 INVARIATA).
3. Giudice/arbitro: `s128-pair-intercal.sh` → `cross-ratios.out` meccanico dai `.time` (user+sys full; user-only media canonico GAP_TREND). Questo criterio è committato PRIMA del codice dell'orchestratore (az.rev. S-127 #5, commit distinti).
4. Segno atteso: full ≤ 1,815–1,896 (S-125 @ s124) — direzione ↓ attesa da cbargs2+stampo; l'esito REGISTRA il nuovo riferimento qualunque sia (rimisura, nessuna soglia di promozione).
5. Parità per NOME attesa: media 0 nomi ×4; full diff == SOLO `wp_is_stream data set #2` (divergenza nota S-119/S-120). Diverso ⇒ indagine PRIMA di registrare.
6. gate_void=1 su qualunque gamba ⇒ verdetto NULLO su quella gamba (KS-KL-101-3).
7. Peak footprint phpr registrato (rif. 1862–1983 MiB); uploads via guardia Gregg R7; DB reset per run; run DETACHED sequenziale con marker `.done`; nessun'altra run pesante in parallelo.
8. Il verdetto cita SOLO il file rc scritto dal proprio script: `wp128-harness/pair-out/pair-intercal.done` (az.rev. S-127 #2).
9. Nessuna misura con indicizzatore LSP in volo: pgrep rust-analyzer quiescente (CPU ~0) verificato PRIMA del lancio (incidente S-127).
