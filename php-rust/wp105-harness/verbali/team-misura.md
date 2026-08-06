# Team-MISURA (Leijen + Gregg) — Concilio WP-105, fase 2
Relatore: team-MISURA. Fonti: verbale-7-leijen.md, verbale-9-gregg.md.

## (a) Regola di lettura composta A/B R=7 — PRE-REGISTRATA prima del verdetto
1. **Audit del tetto PRIMA della lettura** (KS-LE-105-2): se il tetto 51,96 è un range tarato in fase-1 a R diverso da 7, il confronto range-vs-tetto è auto-VOID per costruzione (spread monotono non-decrescente in R, R5 Leijen). In tal caso la magnitudine si legge su IQR/percentili, mai sul range (A-LE-105-4).
2. **Verdetti co-primari**: (i) magnitudine con dispersione R-coerente; (ii) sign test — 7/7 B>A ⇒ p one-sided 0,0078 firma la DIREZIONE anche se la magnitudine annega nel rumore. Direzione senza magnitudine non autorizza bisect (KS-LE-105-3: nessun bisect finché banda < effetto atteso).
3. **STOP** (A-GR-105-2 + KS-LE-105-3, convergenti): R=7 è l'ULTIMO tentativo full-peak. Qualunque VOID ⇒ metrica dichiarata inadatta; MAI terzo rerun; si passa alla misura per-fase con design pre-registrato = A-LE-105-5 (high-water event-level GA_LIVE/GA_PEAK + peak per finestra phys; il peak fisico resta cifra di riferimento, non giudice).
4. **Sequenza** (A-GR-105-4, accettata da Leijen): la LETTURA del verdetto è atto breve, ammessa in apertura S-104; il RIDISEGNO per-fase, se VOID, va DOPO la leva H-C2, mai prima.

## (b) Cifra H-D — cosa resta in S-104
La firma «1 alloc × 32,0 B» regge SOLO lato alloc e SOLO via argomento del soffitto (media 31,999993, max 32 ⇒ ≤20 eventi sotto-taglia); lato free è ASSUNTA (gfree_note senza istogramma). Prima del SiteTag: **A-LE-105-1** (istogramma free path + delta per-bucket ESATTI, 7 orfani inclusi) e **A-LE-105-2** (attese byte-per-tipo pre-registrate: 32 = Vec 2×Zval o RcBox+16; 40 = Rc<RefCell<Zval>> ⇒ ret_cell escluso PER LAYOUT). Criterio: KS-LE-105-1 (niente «N B esatti» senza soffitto + free istogrammato, pena declassamento a media) + KS-GR-105-2/A-GR-105-1 (ogni cifra per-iter cita l'N EMESSO dal run). Il SiteTag resta il giudice; il layout è attesa, non verdetto. H-D in slot 3, dopo H-C2.

## (c) Banda tra-sere — protocollo emendato
R-GR-105-1: due punti dello stesso giorno di calendario valgono UNO. A-GR-105-3: banda nominabile solo con ≥3 punti su ≥2 giorni distinti; fino ad allora KS-GR-104-1 pieno. Igiene S-104: terzo punto su GIORNO distinto. Dispersione sempre IQR/percentili.

## (d) Anomalia «2 sessioni a rapporti fermi» — nell'ordine
Scrivere testuale: «Contatore anomalia = 2 (soglia raggiunta, non superata). **KS-GR-105-1**: se S-104 chiude senza l'A/B della leva H-C2 ESEGUITO (qualunque verdetto), contatore = 3 ⇒ anomalia DICHIARATA; WP-106 apre con riallocazione obbligatoria fondamentali-first.» Conseguenza d'ordine: H-C2 punto 1 non negoziabile; lettura R=7 atto breve; ridisegno per-fase e H-D subordinati.
