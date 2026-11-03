# Revisione adversariale S-188 Tappa 1 (t1) — lente MISURA

Generato con assistenza AI (Claude Fable 5). Revisore: agente fresco, senza il contesto della sessione; mandato REGOLE §7 (≤400 parole, trovare ciò che invalida il claim).

**Claim**: le pagine WordPress costano al server phpr (pin s183, php-server modo cli-server = `phpr -S`) 9,4–13,1× la user-CPU/richiesta di PHP 8.5.7 `php -S` con opcache; senza cache sui due lati 3,7–4,3×; la unit cache di phpr evita 2,1–4,6×, l'opcache 4,9–16,3×.

## Rilievi del revisore (testuali, ordinati per gravità)

1. **Igiene violata, non rilevata: un client HTTP estraneo (Chrome, tab wp-admin) ha colpito la porta 8080 durante il run di record.** Evidenza nei `server.log` (attesi 248 GET/boot): c2-P 672, c2-Ooff 672, c3-P0 672 (424 GET statici `wp-includes/js/*`, 106 distinti ×4, dentro i blocchi K); c1-P ha 1 `/wp-admin/admin-ajax.php` (PHP+DB, heartbeat) in rest-W. Il verdetto dichiara «igiene ok, 0 VOID» perché la guardia copre solo cargo/rustc/phpt-runner e il tripwire `Connections` è cieco agli statici. Aggravante: la cella mediana pubblicata di P/front (293,42, c2) e di P0/front (689,68, c3) È quella contaminata. Stima del danno ≤212×~0,02–0,3 ms su 8,8 s di blocco (<1 %, sotto il rumore fra boot): la CIFRA non si sposta, ma per REGOLE §3 è «record senza prova d'igiene» = incidente. Riparazione: gate `#GET nel server.log == 8+6·(W+K)` ⇒ boot VOID; rerun t2.
2. **La sonda admin-ajax spiega il picco c1-P 475 MiB** (final.ru 479 MB vs 341 MB c2): UNA richiesta PHP fuori canale ha lasciato +140 MB residenti fino al teardown. Non tocca la CPU misurata, ma è un segnale di ritenzione da aprire a registro.
3. **Frequenza effettiva 2,09 GHz identica su OGNI boot, entrambi i lati** (cicli/CPU-s da `time -l`; M4 = P 4,4 / E 2,9 GHz): il server gira clampato/E-core (processo daemonizzato). Simmetrico ⇒ direzione salva, ma IPC diversa (phpr 3,86 vs oracle 2,92) rende il rapporto dipendente dalla classe di core: la cifra 9,4× è «a 2,09 GHz». Riparazione: dichiarare la classe di core nel criterio.
4. **mysqld fuori dal metro.** La canonica è Δuser del solo processo server; «costo della pagina servita» include mysqld (stesse K+1 connessioni, ma Questions/CPU non misurate). Un pavimento comune di pochi ms/req abbassa il rapporto (es. 5 ms ⇒ front 8,3×). Riparazione: snapshot rusage di mysqld per blocco + Δ`Questions`.
5. **Equivalenza «php-server cli-server = phpr -S» asserita, non misurata.** Un boot `phpr -S` di controllo chiuderebbe il punto.
6. **P0/Ooff «entrambi compilano» è asimmetrico**: `opcache.enable=0` toglie anche l'ottimizzatore Zend, `PHPR_UNIT_CACHE=0` solo la cache unità; 3,7–4,3 resta direzione, non «gap del motore».
7. Minori: drift wall positivo in-blocco su P front c1/c2 (+7–9 ms su 30 req, ≈2,5 %) ⇒ W=10 al limite; instr/req concorda in direzione (P/O 11,9–17,1 vs CPU 9,4–13,1, stabile ×3 cicli); opcache con `validate_timestamps=On`, JIT off (da dichiarare); DISABLE_WP_CRON simmetrico ma il rapporto è «senza cron»; sentinella LS attiva (rust-analyzer, Serena) tollerata dal gate hog, non dal criterio.

**Verdetto del revisore: REGGE con riserve** — rapporti e direzione reggono (contaminazione sotto il rumore), ma t1 non è citabile come record pulito finché l'incidente #1 non è dichiarato e un t2 con gate sul conteggio GET non lo conferma.

## Risposta (sessione, 03/11 02:20)

- #1 ACCOLTO, incidente dichiarato (criterio p.11; verbale). Evidenza verificata: `pair-out/t1/c{2-P,2-Ooff,3-P0}/server.log` 672 righe di richiesta, `c1-P` 1 POST admin-ajax. Le schede del browser dell'utente non sono controllabili dalla sessione ⇒ difesa nell'harness: gate richieste = 8 + 6·(W+K), boot sporco SCARTATO PER INTERO e ripetuto (max 3), prove in `<boot>.void<n>`. Collaudo del gate: dry3 con una richiesta estranea iniettata di proposito.
- #2 a registro (apertura per NOME nel NEXT: ritenzione +140 MiB dopo una richiesta admin-ajax).
- #3 ACCOLTO: GHz e IPC per boot nel verdetto (companion), classe di core dichiarata (p.11c).
- #4 ACCOLTO: rusage di mysqld + ΔQuestions per blocco; canonica invariata (solo server), companion «pagina servita» stampato.
- #5 ACCOLTO: braccio `Pphpr` (`phpr -S` del pin) in ogni ciclo.
- #6 ACCOLTO: etichetta DIREZIONE sulla riga P0/Ooff.
- #7 ACCOLTO in parte: W=20; opcache ini in testa; «senza cron» e sentinella LS restano dichiarati, non curati.
- Esito t2: vedi `s188-wp-server-pair-verdetto-t2.out` e verbale.
