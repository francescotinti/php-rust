# s156-smoke-atteso-hd2host — attesi BLIND (scritti PRIMA di ogni run; verifica da SECONDO attore dovuta prima del run)

1. **Parità**: su `m-hostargs.php` A e B stampano ENTRAMBI `HA-OK 20000000`
   (10M iter × 2 true); ogni altra categoria: output A==B byte-identici,
   pena STOP LEVA (rc=2).
2. **Smoke R=2 (giudice hostargs)**: D=A−B POSITIVO su entrambe le coppie
   (early-stop a segno opposto). Banda di grandezza DICHIARATA
   **[+7; +21] ns/iter** (attesa ≈ +13,8 = 2 args-Vec × miheap 6,9;
   mezzo-pair di margine per lato). Dentro banda ⇒ si prosegue a R=5 con
   DSM=D_smoke; segno + ma fuori banda ⇒ si prosegue DICHIARANDO (la
   riconciliazione UB del criterio p.4 arbitra al R=5).
3. **Disasm (criterio p.7)**: bl-count di `run_loop` registrato su A e B
   PRIMA del giudizio; atteso: variazione LOCALIZZATA (call al nuovo
   dispatch slice); |Δbl| > 20 = reperto da dichiarare (lezione H-C2
   S-104: flip inliner), non gate.
4. **Guardie (solo-regressione, nessun morso atteso)**: backtrace24 atteso
   D ∈ [0; +10] ns/iter (debug_backtrace è convertito: 1 args-Vec/iter in
   meno ≈ +6,9); obj* e le sei attese piatte (|D| < soglia propria).
5. **rc attesi**: quiescenza 0 · smoke ab-out/<tag>.rc = 0 · R=5
   ab-out/<tag>.rc = 0 (SOPRA SOGLIA) se la leva morde; 4 (SOTTO) = leva
   caduta, revert al byte; 2/5/9 = STOP dichiarati.
