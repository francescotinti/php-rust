# S-142 criterio conferma post-pin L-RD1 (pre-registrato PRIMA di ogni run S-142)

1. **Giudice**: m-arrdrop (wp141-harness, versione emendata d6b77b7), N=1e7 dal sorgente.
2. **Coppia**: A = stash `phpr-s140` (f2708b75) · B = pin s142 (post-catena); R=5 ABAB,
   user CPU al netto dei pavimenti per-binario; pgrep rust-analyzer/cargo PRIMA.
3. **Attesa**: D=+5,0 dall'A/B S-141; **banda di conferma = 5,0** (stessa banda del verdetto:
   max(4,0; drop-1; banda-layout 2,0+2 tick)) ⇒ compatibile se D ∈ [0,0; 10,0] **con segni ≥4/5**.
4. **STRETTEZZA (az.rev. S-141 #5, pre-registrata)**: il confronto con la soglia è STRETTO (>).
   D == bordo (al tick 1,0) ⇒ verdetto «**AL BORDO ⇒ replica**» (una replica R=5, verdetto sul
   cumulato dei segni), MAI «SOPRA»/«FUORI». D col segno opposto ≥2/5 ⇒ conferma FALLITA,
   pin resta ma la leva torna «non confermata» a verbale.
5. **Guardie**: nessuna (conferma osservativa post-pin; le guardie hanno già giudicato l'A/B).
6. Sonde semantiche (az.rev. #3/#4) NON sono misure: parità Hashed = byte-compare vs oracle
   (qualunque diff = STOP catena); nesting = soglia di sfondamento A vs B documentata, o rientro.
