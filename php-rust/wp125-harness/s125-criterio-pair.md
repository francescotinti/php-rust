# Criterio S-125 p.1 — rimisura full/media WP sul pin s124 (trigger REGOLE §4)

1. Oggetto: RIMISURA del riferimento (non leva): pin s124 c5ba2573a23adf69 vs oracle 8.5.7, stessa sera.
2. R: coppia bimodale N=2 per modo, 4 gambe INTERCALATE off1→on1→off2→on2 (ricetta pair109 INVARIATA, az. rev. S-119 §1).
3. Giudice/arbitro: `s125-pair-intercal.sh` → `cross-ratios.out` meccanico dai `.time` (user+sys full; user-only media canonico GAP_TREND). Committato PRIMA del primo run (vincolo rev. S-124 #1).
4. Segno atteso: full ≤ 1,810–1,889 (S-120) — direzione ↓ da str/arr/re promosse; l'esito REGISTRA il nuovo riferimento qualunque sia (nessuna soglia di promozione: non è una leva).
5. Parità per NOME attesa: media 0 nomi ×4; full diff == SOLO `wp_is_stream data set #2` (divergenza nota S-119/S-120). Diverso ⇒ indagine PRIMA di registrare.
6. gate_void=1 su qualunque gamba ⇒ verdetto NULLO su quella gamba (KS-KL-101-3).
7. Peak footprint phpr registrato (rif. 1862–1983 MiB); uploads via guardia Gregg R7; DB reset per run; run DETACHED sequenziale con marker `.done`.
8. Ogni attribuzione del Δ full a PhpStr resta «direzione+meccanismo, magnitudine non ripartita» (REGOLE §4): nessun A/B full off-patch stasera.
