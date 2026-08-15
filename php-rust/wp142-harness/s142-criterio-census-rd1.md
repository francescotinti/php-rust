# S-142 criterio census arrays/elems al teardown (criterio L-RD1 p.7 — pre-registrato PRIMA di ogni edit/build della probe)

1. **Oggetto**: CONTEGGI (mai tempo) del meccanismo esatto di L-RD1 sulla SUITE
   doctrine/orm: `rd1_arrays` = ingressi in `PhpArray::drop` con repr non vuota ·
   `rd1_elems` = elementi drenati (Packed + Hashed, tombstoni ESCLUSI dal conto
   elems ma contati a parte `rd1_tombs`). Contatori NUOVI sotto la feature
   `zval-census` (stessa disciplina S-95/S-97), dump nelle righe `zvalcensus_s142`.
2. **Pin non toccato** (EMENDA DICHIARATA in-sessione dopo DUE STOP hash: la
   byte-identità post-edit è irraggiungibile per costruzione — ogni edit sposta
   i numeri di riga dei panic-location embedded, anche con codice interamente
   cfg-gated; v1 39cf/3067, v2 7679). Gate SOSTITUITO: (i) contatori TUTTI
   sotto `#[cfg(feature="mem-census")]` (nel pin il simbolo non esiste);
   (ii) il PIN resta lo stash `phpr-s142` bba8a734 IMMUTABILE, la dir canonica
   si RIPRISTINA dallo stash a fine lavori census (hash verificato);
   (iii) HEAD ≠ sorgente-pin dichiarato in NEXT_SESSION (stato normale tra
   sessioni); il prossimo pin ripercorre la catena piena come sempre.
3. **Probe**: build feature `zval-census,mem-census`, target ISOLATO
   `/Volumes/Extreme Pro/Claude/phpr-census-target` — mai parità/A-B, mai stash.
4. **Run**: script COPIA DICHIARATA di s141-census.sh (manifest copia-gate);
   2 repliche orm-work su APFS, PHPR_ZVAL_CENSUS raw datato, watchdog 1800 s;
   smoke: dump contiene `rd1_arrays`, altrimenti rc=8. Gate validità: parità
   fail-set per NOME vs baseline16 su entrambe; scarto r1/r2 >1% dichiarato.
5. **Quota (INDIZIO, REGOLE §4)**: quota = (rd1_elems × prezzo_elem) / 42,5 s,
   prezzo_elem 0,5–1,0 ns = lato-basso modello S-141 (~0,8 ns/elem, D=+5,0/6
   elem) — cross-giudice DICHIARATO, mai cifra promossa. Companion:
   rd1_arrays × costo fisso call-glue risparmiata (2–5 ns).
6. **Decisione pre-registrata**: la quota si SCRIVE nel verbale di spedizione
   L-RD1 qualunque sia (era attesa ~1–2%); quota <0,3% ⇒ dichiarare «guadagno
   ORM sotto risoluzione della coppia suite» nel dossier p.3; ≥0,3% ⇒ citarla
   come canale residuo. Nessun gate: la leva è già promossa dalla catena.
7. Sentinelle non-gate: pgrep rustc/cargo/rust-analyzer per finestra; misura
   SOLO a catena s142 conclusa (mai edit coi build in volo).
