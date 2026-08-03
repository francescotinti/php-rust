# Verbale sedia 9 — Gregg (metodologia di misura + MANDATO INVERSO) — Concilio WP-95

**VERDETTO: APPROVATA CON EMENDAMENTI** — la migliore sessione d'oggetto da WP-90: due refutazioni vere sull'oggetto (malloc_huge cumulativo; LEVER-2 nulla) e un canale nuovo quantificato (preludio 39,5 MB). Ma una cifra è sopravvalutata di grado e una supersessione formale manca.

## §BILANCIO D'OGGETTO (cosa sappiamo OGGI che ieri non sapevamo)
1. I sei huge per worker hanno UN nome: sei chunk in raddoppio (x2+16, firma bumpalo) di UNA arena — il parse del PRELUDIO stdlib — e sono TUTTI liberati dentro la prima richiesta. Ieri erano «riserva fissa mai liberata»; oggi sono uno spike transiente.
2. malloc_huge nei dump m90 è un CUMULATIVO travestito da retained (current==total==peak su 20/20 raw, anche a exit_collect_mi). Il canale stat huge di m90 NON è consumabile come retained.
3. LEVER-2 (mi_collect post-preludio) non muove lo slope (delta 21837 B/worker, 0,12%), coerente con arena committed invariata nei raw m90.
4. Il preludio costa per-PROCESSO: arena 39534144 B (coda inutilizzata 13738592), e su hello il CLI pinnato sta a 44630520 vs oracle 10093048 = **4,42×**.
5. Corollario negativo: lo slope ~18,8 MB/worker resta SENZA nome — il maggior candidato nominato (39,9 MB huge) è stato esonerato.

**Il 4,42× ridisegna le priorità?** Sì, come candidato n.1, ma con cautela: GAP_TREND dice «riferimento resta WP-85» da 7 sessioni — la media ~3,0-3,1 non ha un «prima» fresco. Il delta hello (44,6−10,1 ≈ 34,5 MB) è dell'ordine dell'arena (39,5 MB): la leva per-file (peak ~74 KB) porterebbe hello-class verso ~1,1-1,5×, e poiché il corpus è dominato da test hello-sized la media può scendere sotto 3 in un colpo. Ma la priorità si DICHIARA solo dopo la coppia full stessa-sera: senza baseline aggiornata la predizione-misurata (WP-48) non ha giudice.

## Q1 — Il probe slope è una «misura»? NO, è uno SCREEN.
1 run per W, 2 punti W, zero varianza. La risoluzione dichiarabile è lo spread tra le due build strumentate: ctrl 18814309 vs baseline b048b697 18852538 = **38229 B > delta 21837 B**. Il verdetto legale è «effetto < ~0,2% alla risoluzione del probe», non «delta = 21837». Consumabile SOLO in coppia con la coerenza m90 (arena committed invariata). Cifre NON consumabili: slope 18814309 (build mem-census, W∈{1,4}, 1 run — mai nel ledger come slope); delta 21837 come pin. Il 4,42× = MAGNITUDINE (1 run per lato), non pin.

## Q2 — Supersessione huge-worker.out: SÌ, formale.
La conclusione «non viene MAI liberata … riserva FISSA» è refutata; anche la nota-canale (righe 6-8: «current==total NON è un contatore monco: è assenza di free») è refutata — il decremento esiste nel codice ma non morde su questo canale. Atto: UNA riga SUPERSEDED-IN-PART in testa al file (stesso commit della riga ledger, per NOME delle due clausole, puntatore a huge-sites.out). Il corpo NON si riscrive: è evidenza storica. I numeri per-worker (39911424, 6 blocchi) restano validi come CUMULATIVO.

## Q3 — Rischio d'oggetto più trascurato ORA
Ogni derivata di m90 che ha consumato malloc_huge come retained (repair90-estimators, VCOV 0,778, decomposizione b) è ora SOSPETTA e nessuno l'ha censita. La coverage marginale e l'«invisibile» 4,48 MB/worker vanno ricalcolati sapendo che 39,9 MB del censito erano transienti.

## Q4 — Ordine S-94.0 (FONDAMENTALI-first)
1. Supersessione huge-worker.out + audit derivate m90 contaminate (carta, breve).
2. **Coppia full/media stessa-sera** (contatore fermo a WP-85: baseline PRIMA della leva).
3. Campagna m91 (probe on-thread, heap=<ptr>): attribuzione slope 18,8 MB per NOME.
4. Leva arene per-file con predizione scritta (hello 44,6→~11 MB) + gate parità completi.
5. battery61 nativo (criterio 5).

## Emendamenti
- **A-BG-71**: riga SUPERSEDED-IN-PART in testa a huge-worker.out, due clausole per NOME, stesso commit della riga ledger; corpo intatto.
- **A-BG-72**: malloc_huge m90 declassato a CUMULATIVO ovunque; audit per NOME delle derivate che l'hanno trattato come retained, esito a ledger.
- **A-BG-73**: probe slope = grado ADVISORY-SCREEN; risoluzione = spread inter-build (38229 B); verdetto legale «effetto < 0,2%», pin del delta vietato.
- **A-BG-74**: 4,42× e 39534144 = MAGNITUDINE legata a identità build; pin solo dopo R≥3.
- **A-BG-75**: ⏱ FONDAMENTALI aggiornato (S-93.0 = prime misure nuove); obbligo coppia full in S-94.0 PRIMA della leva per-file.

## KS
- **KS-BG-95-1**: la risoluzione di un probe è lo spread tra le sue stesse baseline — se lo spread supera il delta, il probe è uno screen, non una misura.
- **KS-BG-95-2**: un rapporto su UN workload (4,42× su hello) è una magnitudine del canale, non una priorità di roadmap, finché la media non è rimisurata.

## Refutazioni capitali: SÌ
(a) huge-worker.out: conclusione «mai liberata» E nota-canale «assenza di free» refutate — supersessione obbligatoria. (b) La qualifica «REFUTATA CON MISURA» di LEVER-2 è refutata nel GRADO: è refutazione da screen+coerenza, non da misura.
