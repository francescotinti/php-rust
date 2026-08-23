# s157-smoke-atteso-al1 — attesi BLIND (scritti PRIMA di ogni run; verifica da SECONDO attore dovuta prima del run)

1. **Parità**: su `m-missload.php` A e B stampano ENTRAMBI `ML-OK 10000000`
   (10M iter, sempre miss); ogni altra categoria: output A==B byte-identici,
   pena STOP LEVA (rc=2).
2. **Smoke R=2 (giudice missload)**: D=A−B POSITIVO su entrambe le coppie
   (early-stop a segno opposto). Banda di grandezza DICHIARATA
   **[+10; +31] ns/iter** (attesa ≈ +20,7 = 3 alloc × miheap 6,9; mezzo-UB
   di margine per lato). Dentro banda ⇒ R=5 con DSM=D_smoke; segno + fuori
   banda ⇒ si prosegue DICHIARANDO (la riconciliazione UB p.5 arbitra al R=5).
3. **Identità bracci**: A ricostruito == 42efea3e34feb390 AL BYTE (atteso);
   byte diverso ⇒ arbitrato a CONTENUTO 48 B LC_UUID+firma (emenda S-154),
   esito DICHIARATO prima del giudizio; B = hash nuovo dichiarato al run.
4. **Guardie (solo-regressione, nessun morso atteso; comparatore STRETTO
   pre-registrato)**: hostargs attesa piatta D ∈ [−4; +4] (hit-path non
   toccato: solo firma con Option in più, inline atteso); backtrace24 attesa
   piatta (debug_backtrace non passa da try_autoload); obj* e le sei attese
   piatte (|D| < soglia propria).
5. **Disasm**: bl-count run_loop atteso Δ=0 (nessun edit run.rs); Δ≠0 =
   reperto dichiarato, non gate.
6. **rc attesi**: quiescenza 0 · smoke ab-out/<tag>.rc = 0 · R=5 = 0 (SOPRA
   SOGLIA) se la leva morde; 4 = leva caduta, revert al byte; 2/5/9 = STOP.
