# Criterio PRE-registrato — leva S-111 «hot-cluster dispatch» (REGOLE §3; committato PRIMA della misura)

1. **Forma**: pre-match caldo in testa a run_loop su 8 op (CmpJmpSC, BinarySCSCDst, IncDecSlotJmp, Sweep, PropGetSlot, PropGetSlotRecv, BinaryTCPropSetPop, BinarySTDst), corpi estratti in metodi `#[inline(always)]` chiamati da ENTRAMBI i siti (match grande intatto) — semantica identica per costruzione. Famiglia discriminata: «lo scatter degli handler caldi nel corpo da 288 KB affama il frontend» (tail-call escluso: niente TCO garantita in Rust stabile; un trampolino conserva l'unico sito indiretto e non discrimina).
2. **Giudici A/B**: arith e prop (bersaglio, firmati S-110); arr (controllo di specificità); calls (sentinella della tassa del filtro: op fuori dal set caldo).
3. **Metodo**: ricetta run-micro (user CPU netto pavimenti PER-binario), A/B interleaved ABAB, R=5, stesso sorgente; smoke R=2 con early-stop a segno opposto. A = pin 92909544 (stash), B = build leva.
4. **Segno atteso**: arith ↓ E prop ↓.
5. **Soglia per bersaglio**: Δ mediana ≥ max(4 ns/iter; rumore ABAB osservato; banda-layout 0,67 ns/iter) — N emesso dal sorgente (arith 50M ⇒ 4 ns/iter = 0,20 s; prop 30M ⇒ 0,12 s). Guardie: |Δ arr| e |Δ calls| < stessa soglia della rispettiva categoria.
6. **TETTO (az.4 revisore S-110)**: guadagno massimo credibile ×1,48 su arith (9,3→~6,3); un esito OLTRE il tetto = sospetto errore di misura, si indaga prima di celebrare.
7. **Verdetto (leva = DISCRIMINATORE)**: FAMIGLIA-CONFERMATA (entrambi i bersagli sotto soglia col segno giusto, guardie dentro) · FAMIGLIA-REFUTATA (|Δ| < soglia su entrambi i bersagli ⇒ la fame frontend NON viene dallo scatter dei caldi) · MISTO (tutto il resto: si nomina e non si promuove).
8. **Protocollo run_loop**: disasm bl-count prima/dopo (pin: 71.992 istr, bl 5849, br 22, blr 2).
9. **Post-leva (qualunque verdetto)**: contro-lettura delivery con `wp110-harness/s110-l1i-run.sh` + PRIMA lettura dei giudici held-out (`wp111-harness/heldout/run-heldout.sh`, R=5) — i held-out giudicano la GENERALIZZAZIONE della famiglia, non la promozione micro.
10. **Admission**: batteria `cargo test --release` + parità d'output micro+held-out sul binario B prima dell'A/B; se la leva spedisce: pin SOLO via `scripts/pin-phpr.sh`, corpus per NOME ×2 modi, coppia WP dovuta.
