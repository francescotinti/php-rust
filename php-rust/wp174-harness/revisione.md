# Revisione S-174 — lente SEMANTICA (revisore singolo, sola lettura)

## Verdetto: REGGE CON RILIEVI

Il claim «semantica preservata per costruzione» regge sul codice; ciò che NON regge è la prova sperimentale che lo accompagna (mutante e fixture), dichiarata più forte di quanto gli artefatti mostrino.

### Ciò che regge (letto in `crates/php-runtime/src/vm/run.rs` a 883cb598)
- **Un solo testo**: `sweep_idle` (630-637) è l'unico predicato; il handler `Op::Sweep` lo richiama (7002) e sotto `!noop` fa solo `gc_sweep` (7011-7013); sotto IN_DESTRUCTOR il handler non fa nulla e il predicato risponde vero: equivalente.
- **Predicato letto DOPO l'effetto**: nei cinque siti (2144-2150, 2256-2264, 4716-4723, 4885-4893, 4939-4957) tra la scrittura `*x = r` su un Long già Long e la lettura del predicato non c'è drop, `gc_note`, chiamata PHP né cambio di frame; in P4 la lettura è guardata da `in_place` (4954) perché `write_property_at` può notare l'old (4949). `main`, `gc_enabled`, `gc_sweep_bound`, `gc_light_demoted` sono letti nello stesso istante in cui li avrebbe letti il handler.
- **Back-edge fuso** (2303-2311) replica verbatim il handler CmpJmpSC (2343-2353): stesso `long_cmp_i64`, stesso `when`/`addr`, `a+1` = avanzamento di default (1541). `run_loop` non ha controlli per-dispatch oltre la profondità pila (1484-1541); `declare(ticks)` non è implementato (`lower/stmt.rs:209`); un diag nel sentiero lento dell'incremento passa per `raise_diagnostic` SINCRONO (`mod.rs` 5620-5623), quindi il confronto fuso vede lo stato dopo l'eventuale handler utente. Census: B spento sotto op/gc-census, C sotto op-census; CmpJmpSC non ha contatori gc: coerente.

### Rilievi
1. **Fixture v2 NON bilaterale su disco.** `ab-out/fx-sw1-oracle.out:32` = `p1-loop: 6` (v1); `ab-out/s174-mutsw/ref.out:32` = `p1-loop: 6 3` (v2). La riga di s174-verdetto.out «fixture v2 88 righe bilaterale oracle==pin==C» non ha artefatto (88 = righe di OUTPUT; il file ha 75 righe). La v2 sarà collaudata solo dal gate `bilat fxsw1` della promozione differita (s174-promozione.sh:212).
2. **Mutante MS: correzione post-corsa a senso invertito.** stdst-dtor-loop/for-dtor/p3-chain sono passati da «attese rotte» a «DEVONO restare intatte» (s174-mutante-sw.sh, INTS). Non sono intatti per dominio ma NON DISCRIMINANTI (nessun `echo` nel corpo del loop: la nota è drenata dal light sweep di mk() o dallo Sweep del `for` senza cambiare l'ordine osservabile, fx-sw1.php:18,36,69). Se si fossero rotti sotto MS sarebbe stata prova di dominio, non violazione: la classe giusta è «a verdetto». Conseguenza: MS prova lo scavalco solo su statement lineari (BinarySTDst, P3, light, window); NESSUNA forma a loop (i giudici!), NESSUN sito P1/P4 (p1-chain/p4-chain intatti a verdetto) e NESSUNA gamba «pressione GC» del predicato (pressure/collected: il totale raccolto è invariante al momento) è provata dal mutante.
3. **Criterio-bis**: legittimo (REGOLE §3, commit 12 s prima del lancio, attesa [2;6] già in p.3) ma la regola «nominato su ALMENO un bersaglio» è stata scelta sapendo che arith-dq non nomina; corsa 2 più rumorosa (prop B' 2,87, Z' 1,93) a soglia ferma 4. Il bersaglio originario resta «solo direzione»: va detto così anche nella riga roadmap.

### Azioni S-175
1. PRIMA della promozione: fx-sw1 v2 bilaterale (oracle==pin==C) con artefatto in ab-out; correggere «88 righe» in «88 righe di output / 75 di fixture».
2. Fixture v3: `echo` dentro i corpi di stdst-dtor-loop, for-dtor, p3-chain, p1-chain, p4-chain (blocco discriminante per iterazione); riportarli in ATTS; rerun MS con SOLO=1 sui binari conservati.
3. Blocco discriminante per la gamba «pressione»: `gc_status()['runs']` letto DENTRO il loop con radici ≥ bound, atteso ROTTO sotto MS.
4. Nel copione mutante: regola scritta «blocco non discriminante = a verdetto, mai intatto atteso».
5. Promozione con Data ≥10G, gate pieni + coppia WP/ORM come da criterio p.5.

## Replica della sessione (S-174, dichiarata; azioni residue = az.rev. S-175)
- Rilievo 1 CHIUSO: fixture v4 (87 righe; output 140) bilaterale oracle==pin==B==C con artefatti `ab-out/fx-sw1-{oracle,A,B,C}.out`; la dicitura «88 righe» era di output (corretto nei verdetti).
- Rilievo 2 CHIUSO (corse 3 e 4, SOLO=1 sui binari conservati): blocchi a loop con `echo` nel corpo (scavalco osservabile per iterazione) e coppie per SITO: MS rompe stdst-dtor-loop-echo, for-dtor-echo, p3-site, p1-site, p4-site, scsc-site (i 5 siti) e lascia INTATTI p3-site-ctl (statement intermedio non fuso) e p1-site-typed (classe con prop tipizzata: IC set non riempita, perimetro S-173) — `s174-mutante-sw-verdetto.out` rc=0. Scoperte nel farlo: `$l = $l + (…)` NON è BinarySCSCDst (serve `+=`, come arith-dq); la classe P della fixture era tipizzata ⇒ il probe P1/P4 non era mai preso. Regola «non discriminante = a verdetto» scritta nel copione (azione 4). Gamba «pressione GC»: NON provata, az.rev. S-175 (azione 3).
- Rilievo 3 ACCOLTO: arith-dq resta «solo direzione» in scoreboard, roadmap e NEXT; alla promozione la conferma post-pin su arith-dq è attesa SOLO SEGNO.
