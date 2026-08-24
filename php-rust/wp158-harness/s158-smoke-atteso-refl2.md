# s158-smoke-atteso-refl2 — attesi BLIND (scritti PRIMA di ogni run; verifica da SECONDO attore dovuta prima del run)

1. **Parità**: su `m-refl.php` A e B stampano ENTRAMBI `RF-OK 20000000`
   (10M iter × 2 esiti veri); `fx-refl.php` e ogni altra categoria: output
   A==B byte-identici, pena STOP LEVA (rc=2).
2. **Smoke R=2 (giudice refl)**: D=A−B POSITIVO su entrambe le coppie
   (early-stop a segno opposto). Banda di grandezza DICHIARATA
   **[+7; +21] ns/iter** (attesa ≈ +13,8 = 2 alloc × miheap 6,9; mezzo-UB di
   margine per lato). Dentro banda ⇒ R=5 con DSM=D_smoke; segno + fuori banda
   ⇒ si prosegue DICHIARANDO (la riconciliazione UB p.4 arbitra al R=5).
3. **Identità bracci**: A ricostruito == 76787303716acd4e AL BYTE (atteso a
   cache calda, reperto promo S-157); byte diverso ⇒ arbitrato a CONTENUTO con
   REGIONI PRE-REGISTRATE (az.rev. S-157 #3): SOLO LC_UUID (16 B) + stringa
   data build/banner mimalloc (≤32 B) + firma code-sign (2×32 B); qualunque
   byte FUORI regione ⇒ STOP; esito nel verbale s158-gemelloA-identita.out
   PRIMA del giudizio. B = hash nuovo dichiarato al run (header con hash
   MISURATI, mai stringhe fisse).
4. **Guardie (solo-regressione, nessun morso atteso; comparatore STRETTO)**:
   missload attesa piatta D ∈ [−4; +4] (try_autoload NON toccato; presidio
   L-AL1) · hostargs attesa piatta (il match slice cresce 6→12 nomi: shift
   atteso sotto soglia, si dichiara se |D| sfiora il bordo) · backtrace24
   piatta (debug_backtrace resta in slice, corpo invariato) · obj* e le sei
   piatte (|D| < soglia propria).
5. **RAMO «MORSO ALLO SMOKE» (az.rev. S-157 #5, VINCOLANTE)**: smoke con
   rc=5 (guardia morde) ⇒ STOP del cammino diretto; UNICA prosecuzione
   ammessa: arbitrato DEDICATO del morso (derivato a N con tick ≤ soglia/4,
   copia-gate PRIMA del run) e SOLO a morso refutato si passa al R=5;
   arbitrato che conferma ⇒ leva ferma, si dichiara. rc=2/9 ⇒ STOP secco.
6. **Disasm**: bl-count run_loop — Δ PICCOLO atteso possibile (il match slice
   inlined cresce di 6 nomi; precedente S-156: +19); Δ a verbale, non gate.
7. **rc attesi**: quiescenza 0 · smoke ab-out/<tag>.rc = 0 · R=5 = 0 (SOPRA
   SOGLIA) se la leva morde; 4 = leva caduta, revert al byte; 2/5/9 = STOP
   (per 5 vale SOLO il ramo p.5).
