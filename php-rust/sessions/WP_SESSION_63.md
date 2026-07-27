# WP_SESSION_63 — STUB-ELISION SPEDITA (contratto v2 default-ON): peak full −46,0% (3,90→2,11GB), net_tot compile-side −74,7%, footprint media 4,1×→~3,0×, CPU intatta, TRE gate pieni verdi (OFF/ON/FLIP)

> ⚡ **WP-63 (2026-07-27 notte, `13eeee0`→`deb54b8`)** — rotta Hejlsberg
> dal decision point M2 di WP-62, **K1 RATIFICATO dall'utente in
> apertura** ("CONFERMO"); design63 = recepimento integrale della
> sintesi a 9 (COUNCIL_WP63_REVIEWS); scaletta Klabnik E0-E6 eseguita
> INTERA in una sessione. RULEBOOK §4 (contratti v1/v2 + monotonia +
> regola reloc) FIRMATO dall'utente; PHPR_DIVERGENCES §3.8 depositata.

## E0 — ratifica, quota, metro

- **G1 ri-derivata** (probe63-fresh.sh: n=6 include/bersaglio stesso
  processo, scarto prima — paga lo stub-interning one-time 212KB —
  mediana 2..6, righe di regime BYTE-STABILI): quota **89,3%**
  (definizione reference-image) / **87,5%** (conservativa) ≫ 60% ⇒
  KG1 NON scattato. Basi di regime: version 23.945 (proper 4.094) ·
  modules 66.131 · loader 128.455.
- **Metro**: R3 = counted/net **0,557** su unit multi-include (residuo
  = ritenzioni fuori-Module nella finestra compile) ⇒ **KG2: banda
  NET→PHYS degradata a counted-only, verdetto SOLO su coppie
  misurate**. L1 molte-piccole (30k fn, 916B/fn net): phys/net via
  peak-differential = 3,9 CONTAMINATO dal transiente HIR/AST (il peak
  cattura il lowering che la finestra net esclude) — forma inadatta ai
  bin standing; fanno fede le coppie reali OFF/ON.

## La leva (E1-E3, commit `b55c763`+`5e06924`)

Contratto v2: le classi che il VM già linka E l'intero prefisso seed
(incluse le ricompile dei condizionali di unit precedenti) NON si
materializzano nel Module; `class_index` per-unit non ritenuto;
`conditional_classes` ribasato al retained; provenienza tipata
`Module.elided: Option<u32>`. **Raffinazione di forma (a mandato
invariato)**: baking degli id globali AL LINK col walker esistente su
remap PROGRAM-space che replica alla lettera `unit_class_remap`
(registrato→esistente / identity con name-check / append) ⇒ bytecode
ritenuto BYTE-IDENTICO al legacy per costruzione; emit-time baking
rifiutato (avrebbe toccato ogni sito di emissione). Bordi pinnati:
eval/deferred catturano `seed_len` PRE-accumulo; ramo identity
disallineato = LOUD `elide_align_miss` (panic nei census); hit path =
double-check retained-space (il ramo identità-posizionale MUORE, P2 —
i riferimenti seed baked sono guardati dal fingerprint).
`alloc_stdclass` → tabella VM. Kill-switch diagnostico
`PHPR_UNIT_CACHE=0` (controllo cache-off dei probe P1).

## Gate (tre PIENI, tutti verdi)

- **OFF (delta zero E3)** e **ON (E4)** e **FLIP (E5)**: corpus **1421
  IDENTICO** · refl **290 IDENTICO** · ORM 3484 **3E/13F IDENTICO** ·
  hk 1665 **0E/0F** · sentinelle 5 assi BYTE-ID · cargo **1645/0**
  (anche col default ON) · sentinelle63 3/3 · reverse-order 2F =
  order-dep PRE-ESISTENTI (fail-set IDENTICO al gemello phpr-wp62).
- **Sentinelle63 (E2, KK1'+KS-S2)**: S1 pomo/polyfill · S6 statics ·
  K2 generator attraverso include · KE-a ordine enumerazione (md5
  set+ordine) · KE-b reflection · eval-extends (bordo seed_len) ·
  KE-d condizionale+extends · KE-c redeclare · S3 prelude-fn — tutte
  byte-id OFF vs ON con `elide>0` asserito e sequenza `unit_fp`
  IDENTICA (nuovo evento `fp` nel log).
- **P1 Pedersen**: P1-a sul binario wp62 = **primo avvistamento
  hit_cross>0** (buco colmato PRIMA della leva); P1-b su moduli elisi
  verde incluso byte-id vs cache-off; P1-c cross-rotta resta miss fp;
  P1-d degrada a miss fp e recupera.

## Misure (giudici pre-registrati)

- **--list-tests OFF→ON (memgc63 back-to-back)**: net_tot 648.774.801
  (riproduce WP-61 alla cifra) → **330.372.107 = −49,1%**; version
  −82,7%/inc (**KE2 ✓ ≥80%**); `pfx_stub=0` (L3) · elided 1931/1931 ·
  `elide_align_miss=0` · compilens lower ~flat, compile −34% (B7) ·
  **peak footprint 1,438→1,059GB = −26,4%** (drain/net 1,19× — sopra
  banda [0,75-1,10] nella DIREZIONE Leijen a.i: rounding per-bin).
- **census63-full (flag ON) MASTER**: **net_tot 1.973,3MB →
  499,9MB = −74,7% (−1.473MB counted)** — dentro la banda predetta
  [850MB…1,5GB] al bordo alto; per-include: version 22,6K (−94,8%),
  modules 65,3K (−86,5%, predetto 65,0K), loader 127,7K (−76,6%,
  predetto 126,0K); aggregato retained 117,4MB vs 102,1MB predetti =
  **+15,0% (bordo interno ±15%, KS63-1 ✓; version singolarmente FUORI
  a 3,4× — attribuzione colonne, a verbale)**; tag=reloc:
  skipped_classes=0, **unexpected=0**; S5 ✓. ⚠️ colonne prefixsum ora
  dominate dalla mappa eager TRANSIENTE (free fuori-seg): net è il
  giudice, non le colonne.
- **Coppia full run50 (stessa-sera, new=flip vs old=phpr-wp62)**:
  fail-set **88 BYTE-ID su entrambe = run33** ✓ · CPU 781,32 vs
  783,80u = **−0,32%** (KS63-3 non scatta) · **peak fisico 3,896 →
  2,105GB = −46,0%**.
- **Gap media (coppia singola, spread G3 dichiarato)**: oracle
  21,01u/393,3MB · phpr 54,14u/**1.170,3MB** ⇒ CPU **2,58×** ·
  footprint **2,98×** (riferimento strutturale 4,1× → **~3,0× da
  confermare con protocollo ≥3+3**).

## E6 — ri-quota cache (chiusa col metro) + KS63-4

- Dup residuo master post-elisione = **138,4MB** counted (<150MB) ·
  compile-CPU 15,4s = **1,97%** del full (<2%) ⇒ per Hejlsberg d la
  **cache re-link RESTA CHIUSA** (entrambe borderline: a verbale per
  il concilio). Nota: ways_evictions 2226 invariato.
- **KS63-4 SCATTATO**: residuo per-include O(seed) (version 22,6K vs
  proper vero ~4-6K) >30% ⇒ **tranche 2 NOMINATA** (slot_names prefix,
  fn Vec prefix, costo CPU mappa eager — lower tot 32,2s = 4,1% full):
  fronte NON chiuso, quota da design64.

## Stash e binari

Release = **phpr-wp63 (`1666e1b4ed4899a5…`)**, additivo accanto a wp62;
census **phpr-memgc63 (`515b9760…`)** con tag=reloc + compilens.
Harness: `wp63-harness/{design63.md,probe63-fresh.sh,sentinels63.sh,
probe63-p1.sh,gate63.sh (off|on|flip),census63-listtests.sh,
calib63-manysmall.sh,build-memgc63.sh,orchestrate63.sh}` + out-dir.

## ⭐ Lezioni

- ⭐⭐ **La forma minima della leva è "non materializzare", non
  "ricablare"**: replicare al link le decisioni identiche del remap
  legacy ha reso il bytecode ritenuto byte-identico PER COSTRUZIONE —
  tre gate pieni verdi alla prima esecuzione, zero siti di emissione
  toccati. L'alternativa "id globali a emit-time" avrebbe toccato
  DeclareClass/self::/thunk con rischio divergenza a beneficio zero.
- ⭐⭐ **eval/deferred accumulano il seed PRIMA del compile**: catturare
  `seed_len` pre-accumulo è ciò che salva le classi proprie dell'eval
  dall'elisione — il bordo era invisibile finché non si è letto
  l'ordine reale delle chiamate (accumulate → stub_mask → compile).
- ⭐⭐ **Il peak-differential NON misura i bin standing** (L1): su unit
  function-heavy il picco è dominato dal transiente HIR/AST che la
  finestra net esclude per costruzione — phys/net 3,9 era il
  lowering, non il rounding. Le coppie reali OFF/ON stessa-sera sono
  l'unico metro onesto del NET→PHYS.
- ⭐ Un probe con più include muore alla PRIMA divergenza fatale: i
  casi expected-fatal (redeclare, prelude-fn) vanno in coppie
  DEDICATE o silenziano il resto della matrice.
- ⭐ `grep -c` con 0 match esce 1 e rompe le catene `&&` dei waiter.
- ⭐ Il detector "cache-off" deve cercare i HIT, non hit|miss: a cache
  spenta i miss si loggano comunque.

## Prossimo (WP-64) — vedi NEXT_SESSION §WP-64
