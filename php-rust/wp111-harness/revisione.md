# Revisione S-111 — lente SEMANTICA (revisore singolo, REGOLE §7)

**Verifiche sui raw**: criterio 451d747 e congelamento fdbe5c8 anteriori alla leva f36c028 (heldout/ mai toccata dopo il freeze: diff vuoto); diff leva letto riga per riga contro gli arm originali; `git diff 451d747..HEAD -- crates/php-runtime/src/vm/run.rs` VUOTO; binario deployato = 929095448e823cb5 = pin; loop: nessuna coda dopo il match grande, ip+1 e op-census PRIMA del pre-match ⇒ `continue` equivalente al fall-through, sospensioni PropGet/hook intatte (ritorno a ip+1 come nella sequenza non fusa). Spot-run dei tre giudici a N=16000: output byte-identici oracle↔phpr.

**«Identica per costruzione»: REGGE.** L'unica differenza è l'ordine scn!↔note_* in BinarySTDst; scn!/dcn!/note_* sono TUTTI contatori `fetch_add` Relaxed (stackcensus.rs:78-97, zvalcensus.rs), nessuno stream ordinato, e i valori letti ai siti di nota sono immutati ⇒ dump identici anche in build census. Neo cosmetico: il commento «op che arrivano da sentieri freddi» è falso (gli arm caldi del match grande sono codice morto: il pre-match intercetta tutto).

**Giudici held-out: REGGONO con due tare.** poly: parità GARANTITA dalla semantica (addendi diadici, |somma|≪2^53, ordine fissato), non fortuna — ma il pattern periodo-16 è imparabile da un predittore moderno: polimorfismo di dispatch reale, «branch imprevedibili» sovra-enunciato. err: warning REALE e per-iterazione su phpr (verificato senza `@`: nessun folding), quindi non vacuo; però phpr attribuisce il warning alla RIGA SBAGLIATA (riga d'uso successiva: 6 vs 4 oracle) — divergenza nascosta proprio dal `@`. wploop: forma filtri-array pre-WP_Hook, accettabile e dichiarabile.

**Sovra-enunciati.** (1) «Lo scatter dei caldi NON è il motore» in WP_SESSION_111/GAP_TREND perde il qualificatore: la prova portante (residuo delivery ~0,295 con handler adiacenti) è quota per-motore = SOLO direzione (REGOLE §3, nota presente nei .out ma non negli enunciati). (2) Lezione 3 «un pre-filtro tassa ogni op fredda»: calls +11,4%/arr +8,4% sono JOINT (pre-filtro + inliner flippato + br 22→36), non ripartiti. (3) «Regrediti tutti e tre»: err Δ+0,07 s contro spread ≤0,06, rapporto 2,6→2,6 — entro rumore, nessuna soglia held-out pre-registrata.

**Revert: REGGE** (sorgente al byte, binario dallo stash = pin; non-riproducibilità della build fresca già dichiarata).

## Azioni
1. Emendare WP_SESSION_111 e riga WP-111 di GAP_TREND: «lo scatter non è il motore» → direzione, limitata alla forma pre-filtro provata.
2. Prima di iscrivere la lezione «tassa del filtro»: A/B con pre-filtro che non intercetta nulla, per separare costo del test dal flip dell'inliner.
3. Catalogare in PHPR_DIVERGENCES_FROM_PHP.md la riga errata del warning «A non-numeric value encountered».
4. Nota dichiarata nel README held-out: poly periodo-16 (dispatch polimorfo sì, branch imprevedibili no); err non certifica la diagnostica sotto `@`.
5. Declassare err a «invariato entro spread» o pre-registrare una soglia held-out prima della prossima lettura.
