# s161-criterio-al2.md — leva «L-AL2: loader autoload k=1 senza args-Vec» (residuo DICHIARATO di L-AL1, NEXT_SESSION p.4a; PRE-REGISTRATO prima di ogni edit/misura)

0. **Census-quota (gate pre-registrato, ESEGUITO su atti s149)**: il residuo
   vive in `try_autoload` (mod.rs:12124): `call_callable(loader,
   vec![arg.clone()])` per iterazione loader = 1 passaggio CLOSURE-vec/miss
   (k=1). Quota ORM DICHIARATA ~ZERO: il loader Composer è ARRAY-callable
   `[ClassLoader,'loadClass']`, NON ammesso dal fast path — leva
   MICRO-JUDGED come L-AL1 (attesa suite sotto-risoluzione, si dichiara);
   quota WP mai censita (loader closure plausibili, si dichiara). Il VALORE
   della leva oltre il micro: terzo sito di collaudo del coeff closure-vec
   17,0±2,0 della sonda s161 (famiglia SDOPPIATA) — predizione ESATTA
   falsificabile.
1. Oggetto: in `try_autoload`, quando il loader è closure ANONIMA
   `simple_call && n_params==1` (stessa ammissione L-AM1/L-AF1), la chiamata
   per-miss va via `call_closure_one(&cl, arg.clone())` RIUSATO — si rimuove
   il SOLO passaggio args-Vec+dispatch call_callable. Ammissione PER-LOADER
   (la lista è LIVE, S-71.2: non hoistabile; costo dichiarato: match variante
   + named-check + 2 load per iterazione). OGNI altra forma di loader
   (string, array-callable, named closure, arità≠1, hint) resta su
   `call_callable` INVARIATO per costruzione. Edit SOLO mod.rs (+~20 righe)
   + dente loc mod.rs PRE-dichiarato (cap 25810 → salita dichiarata in
   loc_dente.rs nel braccio B).
2. Giudice: `m-missload.php` (wp158-harness, quello delle guardie s158-s160:
   N=10.000.000, 1 class_exists MISS/iter con autoloader closure no-op —
   AMMESSA: anonima, 1 param senza hint/by_ref/variadic ⇒ simple_call).
   Parità marcatore `ML-OK 10000000` bilaterale. N dichiarato dal sorgente.
3. A/B: braccio A = GEMELLO dal tree corrente s160 (§7-bis), atteso ==pin
   ceeb6e76 AL BYTE anche a freddo (precedenti s159/s160 N=3); byte diverso
   ⇒ arbitrato a CONTENUTO regioni s158, si dichiara. B = tree + edit p.1.
   R=5 ABAB, floor3, mediane, rumore drop-1 (matematica s158 INVARIATA);
   soglia giudice = max(4, drop-1). Stash bracci SOLO via pin-phpr.sh
   --braccio. Build SOLO a catena p.2 conclusa (fatta: pair t11 + ORM done).
4. **UB dal modello SDOPPIATO (sonda s161) — bivio MECCANICO pre-registrato**
   (fix rev. S-160 #4: la formula è UNA e sta QUI): coeff closure-vec =
   17,0±2,0 ⇒ D atteso ∈ [15,0; 19,0]. Nel verdetto: D > 19,0+rumore ⇒
   FUORI-UB SOPRA (reperto, sonda dovuta); D < 15,0−rumore (ma sopra soglia)
   ⇒ SOTTO-MODELLO (reperto: componente non rimossa, si dichiara); altrimenti
   DENTRO il modello — PRIMA leva della famiglia closure-vec col coeff
   PROPRIO.
5. **BANDA SMOKE VINCOLANTE** (EREDITATA, denti NEL giudice): smoke R=2
   early-stop a segno opposto; D_smoke ∈ [8; 22] ⇒ R=5; fuori ⇒ rc=6
   arbitrato census PRIMA del R=5; D_smoke < soglia ⇒ STOP.
6. Guardie R=5 a SOLO-REGRESSIONE, comparatore STRETTO, soglie EREDITATE
   s160: **arrfilter (presidio L-AF1, NUOVA)** + arrmap (presidio L-AM1) +
   refl + **hostargs (hit-path class_exists: il cammino hit NON deve
   regredire)** + backtrace24 + obj*6 + sei micro = 17 guardie.
7. Semantica: output A==B su OGNI categoria pena rc=2; return del loader
   ignorato in entrambi i cammini; propagazione eccezioni identica
   (Result); cursori LIVE e guard-key INTATTI (l'edit tocca SOLO la
   chiamata). Fixture: il contratto autoload/backtrace §3.25 è già
   presidiato da sonda-bt (gate promo, pin==stash); fixture bilaterale
   fx-al2 NON necessaria per il criterio (nessuna forma nuova osservabile:
   si dichiara).
8. Disasm: la leva NON tocca run.rs; bl-count `run_loop` A vs B registrato
   (atteso Δ=0; Δ≠0 = reperto, non gate).
9. Promozione SOLO se R=5 SOPRA SOGLIA e guardie 17/17: catena piena via
   s161-promozione.sh (copia dal s160 CON le lezioni #21: H proprio, OUT
   NUOVA, path verificati POSITIVI, marcatori pretesi) — batteria + corpus
   1412×2 + fixture + micro R=5 + ORM 16 nomi + hk 0E/0F.
10. Esiti a FILE: rc autoritativo `ab-out/al2.rc` (smoke: `al2smoke.rc`);
    verdetto `s161-al2-verdetto.out` (smoke: `s161-al2smoke-verdetto.out`);
    identità gemello `s161-gemelloA-identita.out`. FINESTRA: macchina in uso
    interattivo a fine serata DICHIARATO — quiescenza rc=0 obbligatoria in
    testa a ogni run di misura; i build non misurano.
