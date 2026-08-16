# s149-decisione-bt1 — aritmetica di decisione (criterio s149-criterio-bt1.md p.6 + s149-criterio-pair.md p.5) — scritta PRIMA dell'esito A/B

1. Prezzi PROPRI (s149-sonda-pair-verdetto.out; banda = [min,max] 2 repliche,
   cal 0,480 sottratto e DICHIARATO): pair16 6,37–6,38 · pair32 6,77 ·
   pair48 11,21–11,27 · splitoff3 19,06–20,09 ns/coppia (replica 5,0% >2%:
   NON usabile come chiave di decisione senza t3 — qui NON è la chiave).
2. Bersaglio BT1 (tetto su binario census, tranche-4): other=130,15M
   (conservativo, bersaglio-solo) · n=275,0M (tetto pieno: la forma dominante
   IGNORE_ARGS+limit=2 salta anche le alloc attribuite del perimetro).
   Segmento pertinente = mix taglie ≤16..≤48 (hist s149) ⇒ banda prezzo
   [pair16_min; pair48_max] = [6,37; 11,27] ns.
3. Attesa: conservativa 130,15M × 6,37 = **0,83 s** · alta 275,0M × 11,27 =
   **3,10 s** (prezzo = solo coppia malloc+free: PAVIMENTO per evento — il
   lavoro di costruzione frame non prezzato qui, dichiarato).
4. SCALA S-146 (risoluzione coppia ORM 0,26–0,30 s; ≥2× ≈ 0,6 s): l'attesa
   BASSA 0,83 s ≥ 2× ⇒ **SCOMMESSA SUITE AMMESSA** già sul bersaglio
   conservativo col prezzo minimo. PRE-REGISTRO: al prossimo pin la coppia
   ORM deve dare direzione ↓ fuori dal rumore (denominatore KS-146-1
   0,293 s); attesa dichiarata 0,8–3,1 s su ~42 s netti phpr (≈2–7%).
5. Secondarie (non-BT1): pop-diretti CallHostBuiltin = 21,6M × splitoff3
   ~20 ns ≈ 0,43 s ⇒ banda 1×–2×: SOLO fetta micro-judged, e la chiave
   splitoff3 richiede replica t3 della sonda prima di ogni criterio; args-Vec
   user-call (vecargs 13,1M × ~20 ns ≈ 0,26 s) ⇒ ≈1×: zero codice
   bersaglio-solo.
6. La leva BT1 procede all'A/B (s149-ab-bt1.sh) col giudice micro; la
   promozione resta vincolata a t3 coppia WP (criterio bt1 p.7).
