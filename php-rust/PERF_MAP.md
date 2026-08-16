# PERF_MAP — phpr vs PHP oracle 8.5.7, mappa multi-workload

Aggiornata: **2026-08-16 sera (S-146)** · pin phpr **s145 a89faf32** + server
**s145 4a9adc51** (invariato; S-146 = coppia WP saldata + concilio B3 +
guardia dimrmw: **regressione FR1 CONFERMATA +3,00 ns/iter su m-dimrmw 10×,
5/5, soglia 1,33 ⇒ leva FR1 in ISTRUTTORIA — dimread resta;** verdetto
`wp146-harness/s146-ab-dimrmw-verdetto.out`; deliberato concilio VINCOLANTE
in `wp146-harness/concilio/sintesi.md`) (leva **L-FR1 «dim-read fuso a chiave costante» SPEDITA**:
peephole `PropGetSlot;PushConst(k);FetchDim` → `PropDimGetConst` in place,
PropIc condivisa, composito intatto come fallback per costruzione; hit =
elemento through-borrow, l'`Rc<PhpArray>` della prop NON viene clonato.
Giudice NUOVO m-dimread 3M iter: A/B R=5 **D=+16,7 ns/iter (60,0→43,3,
−28%) segni 7/7**, rumore drop-1 0, soglia 4; guardie m-dimrmw +0,01s=1 tick
DICHIARATO al limite di risoluzione, m-diminc/arr/prop verdi; disasm run_loop
A 71694 istr/5988 bl → B 72489/6014 agli atti; bilaterale: oracle 10 ns/iter
⇒ rapporto dim-read 6,0×→4,3×. Catena promo piena rc=0: batteria 1747/0/2
con inventario = baseline s125 + SOLO dente rczval dichiarato (t1 rosso su
ancora test census, t2 rosso su nomi VOLATILI compile-fail (line N) — emende
dichiarate agli atti), corpus 1414×2 nomi+contenuto+off↔on, fixture 9/9,
micro R=5, ORM 16 nomi==baseline, hk 0E/0F; conferma post-pin = IDENTITÀ di
byte col braccio B giudicato (ricetta riprodotta ×2). **Coppia WP al pin
nuovo DOVUTA → S-146 p.1**; verdetti in `wp145-harness/`) ·
storico S-142 (pin **s142 bba8a734** + server **eeb284b6**, leva **L-RD1
«teardown array inline» SPEDITA**: Drop for
PhpArray drena Packed/Hashed con match esaustivo, niente call per-elemento
sul cammino eseguito — disasm agli atti: il bl residuo è unwind-only; A/B
S-141 D=+5,0 AL BORDO con segni 7/7, conferma post-pin S-142 D=+5,0 al CENTRO
banda segni 5/5; catena piena rc=0 incl. ORM 3E/13F per NOME; invarianza
semantica VERIFICATA: parità Hashed A==B byte-id, nesting ~74–76k invariato;
divergenza PRE-esistente catalogata §3.22 unset-elemento/__destruct differito;
quota ORM dal census rd1_*: v. wp142-harness/s142-census-verdetto.out) ·
storico S-140 (HC1 «hint-check senza clone»: borrow-first
in coerce_or_check_hint, ramo Ref invariato; giudice NUOVO m-hintcall 7,3×
bilaterale, D=+6,7 su 6 check/iter, catena promo completa incl. gate ORM
3E/13F per NOME; census ORM: 35,6M hint-check ≈ 0,13% suite ⇒ HC1 non muove
Doctrine — REPERTO S-140: profilo SUITE = CHURN 32% vs DIMPROP 6%, 44% dei
clone INLINE da run_loop → filone TakeSlot. COPPIA WP @ s140 FATTA: on-only
1,765–1,777 N=6 COMPATIBILE — **banda_ON 0,033 CONFERMATA cross-finestra,
az.rev. S-139 #1 CHIUSA**; objmap 43,4 → piano gc-cycle-collector; dbal/ORM
riferimento S-139: dbal 8,15–8,23 ind., ORM 8,59–8,71) ·
metodo: user CPU, pavimenti per-binario, N per voce come indicato; criteri pre-registrati in
`wp125-harness/s125-criterio-{pair,mappa}.md` e `wp126-harness/s126-criterio-{orm,mappa2}.md`
(+ emenda S-127: **cifra canonica = NETTO-pavimento**, raw companion; gate contesa in ictx/s);
cifre dai verdetti `.out`. Regola di lettura: rapporti PER workload, MAI aggregato.

## Workload reali

| workload | rapporto phpr/oracle | N | note |
|---|---|---|---|
| **WordPress full-suite** | **ON-ONLY CANONICO 1,733–1,823** (S-146 @ **pin s145**; N=6 coppie proprie, **6/6 PULITE**; COMPATIBILE col rif S-142 1,765–1,788 su banda_ON 0,036 — attesa FERMO rispettata, L-FR1 non muove WP; **banda finestra 0,090 INEDITA (3–7× le precedenti) con zero gambe sporche ⇒ banda_ON canonica post-S-146 = 0,090 (unione 4 finestre) — APERTURA: capirla prima di usarla come banda di merito**) | **6/6 gambe pulite** (t1) | S-146 @ s145; parità per NOME 6/6 (solo `wp_is_stream #2`); **REPLICA PEAK SENZA inserzione (az.rev. S-142 #2 SALDATA): esito MISTO ⇒ nessuna firma**; pattern S-140 (leg1 bassa) NON riprodotto; osservata DERIVA DISCENDENTE gambe tarde 1843→1831→1808→1773 correlata ai rapporti (leg6 min su peak E rapporto); verdetto `wp146-harness/s146-pair-verdetto-t1.out` (storico S-142: bimodale 1744–1850, verdetto `wp142-harness/s142-pair-verdetto-t1.out`) |
| **WordPress gruppo media** | **2,460–2,547 CANONICA user-only** (S-146 @ s145, 6 gambe pulite; companion 2,391–2,491; leg5 2,547 di NUOVO la massima — ricorrenza osservativa: S-142 leg5 2,524) | 6 | S-146 @ s145 |
| **symfony http-foundation** (1854) | **2,547–2,559** (raw 2,55–2,57) | 2/lato | S-126; canonica sul CONTEGGIO diff 17 nomi = 0,92% ≤1% (≥3 nomi sono unit puri, NON famiglia `php -S` — emenda S-127); sys alto (I/O) |
| **symfony http-kernel** (1665 test) | **4,29–4,32** | 2/lato | parità 0E/0F; contesa ok |
| **doctrine/collections** (242) | **8,22 net** (raw 6,20) | 2/lato | S-126; INDICATIVA: oracle netto 0,09 s (denominatore sotto-scala); parità 0/0 |
| **doctrine/dbal** (3929, sqlite) | **8,15–8,23 net** (raw 7,90–7,97) | 2/lato | **S-139 RIMISURATA @ pin s138** (verdetto `wp139-harness/s139-rimisura-verdetto.out`; floors 0,06/0,19): fail-set stabile 10 nomi == baseline (0,25% ≤1% ⇒ canonica); vs 8,36–8,45 @ s134: **direzione ↓ INDICATIVA (az.rev. S-139 #5: ENTRAMBE le gambe oracle SEGNALATE al gate ictx — l'adiudicazione stesso-lato <1% è precedente S-135 ma applicata fuori criterio; declassata da «lieve ↓» a indicativo)**; summary phpr VUOTA (classe S-126 #3, fail-set dai .failnames) |
| **doctrine/orm** (3484 test) | **8,59–8,71 net** | 2/lato | **S-139 RIMISURATA @ pin s138** (stesso verdetto; oracle `memory_limit=-1` §3.14; parità 16 nomi == baseline; phpr1 ictx segnalata ma stesso-lato <0,2% ⇒ valida): vs 8,43–8,56 @ s134 ⇒ **FERMO/lieve ↑** — REPERTO pre-registrato (criterio p.6): le TRE leve dim-write s135→s138 (AP1+FD1+RMW) NON muovono la suite (l'attesa ↓ è FALSIFICATA: `$this->elements[$k]=$v` non è fetta misurabile del tempo ORM, o il perimetro FD1 lì non morde) ⇒ la prossima leva si sceglie sul profilo SUITE (churn clone/drop, insert/lookup — come già indicava S-135) |
| **composer install OFFLINE** | **1,863–1,891 net** (raw 1,820–1,847) | 2/lato | S-128 @ s127b, PRIMA misura col numeratore vivo (cure ondata-2); composer ESTRATTO, vendor_ok bilaterale, contesa ok (ictx/s); floors 0,07/0,06; sys≈user (~2,3 s/lato) ⇒ **cifra user-only NON confrontabile col full (user+sys): su user+sys sarebbe ~1,3** (rev. S-128 az.5); residuo phpcs config-set (§3.19-quinquies); verdetto `wp128-harness/s128-compoff-verdetto.out` |

## Micro-categorie (R=5, pin s145 dalla catena promo L-FR1; tappa ≤3×)

| arith | prop | calls | str | arr | re | hintcall | dimread |
|---|---|---|---|---|---|---|---|
| 5,5 | 5,5 | 4,8 | 4,3 | **3,2** | **2,5** ✅ | **7,3** (S-140, non rimis.) | **4,3** (m-dimread NUOVO, 43,3 vs 10 ns/iter) |

(S-145: tutte le voci entro 1 tick dai rif s142 5,5·5,6·4,7·4,2·3,2·2,6 —
la fusione non tassa i freddi; rif storici s138: 5,6 · 5,6 · 4,8 · 4,3 ·
3,2 · 2,6.)

RMW (giudici leva S-138, A/B + conferma post-pin): **m-dimrmw 320→146,7
ns/iter (D=+173,3)** · **m-diminc 270→113,3 (D=+156,7)**.
**HC1 (S-140, pin s140)**: m-hintcall6 D=+6,7 — **QUALIFICA rev. S-140: tick
di quantizzazione 3,3 ⇒ 6,7±3,3; evidenza portante = conferma post-pin +10,0
+ segni 5/5 (t2) e 5/5 (conferma)**; ~1,1–1,7 ns/check; census ORM 35,6M
check ≈ 0,13% suite (guadagno reale = prezzo×conteggi: HC1 non muove le
suite — REPERTO; az.rev. #5: census preventivo della quota PRIMA di spedire).

calls: la (*) di s127 è SCIOLTA in S-129 (phpr netto IDENTICO 2,14 s; si muove
solo il denominatore oracle 0,43–0,44). re 2,5/2,6 = run-to-run del denominatore.

Allocazioni/iter vs oracle: arith/prop/calls 0=0 · **str 2,00=2,00 (PARITÀ, S-125)** ·
arr 2,05≈2,03 · re 7,00 vs 5,00 (+2, apertura per NOME).

## Micro-ORM (S-136 sul pin s136 POST-leva-FD1 — verdetto s136-submicro; evalcls/refl da S-126)

| evalcls (compile/classe via eval) | refl | objchurn | └ objalloc | └ objdatains | └ objdropdef | └ objallocni | └ objmap |
|---|---|---|---|---|---|---|---|
| **316,9** (2,38 ms vs 7,5 µs) | **42,4** | 7,0→**6,7** (1180,0 ns, collaterale FD1 −86,7) | **6,4** (810,0) | 6,5→**5,9** (963,3 ns, −96,7 = FD1; riconc. A/B 13,4 ≤ 26,7) | **7,5** | 8,1→**7,9** (736,7; l'osservazione +13,3 di S-135 rientra) | **11,7** (116,7 ns, fermo) |

Profilo ORM phpr (indizio unilaterale): churn visibile multi-% (Zval clone/drop, slot_of,
gc_note/sweep/collect_cycles, insert/lookup, malloc/free); compile ≤~1% leaf, reflection <0,5%.

## Lettura (direzione+indizio, NON attribuzioni firmate — REGOLE §4)

- Il gap **cresce con la densità di lavoro-motore puro**: WP ~1,8 ≈ compoff ~1,9 ≪ hf 2,6 ≪ hk 4,3 ≪
  dbal 8,6 ≈ ORM 8,5 (cifre net). WP, compoff e hf sono diluiti da I/O; le suite object-dense mostrano il soffitto.
- **dbal 8,6 conferma ORM 8,5 senza mock-eval pesante** ⇒ il driver è il lavoro-oggetti, non il
  sentiero compile: coerente con l'istruttoria (compile ≤1% leaf nel run reale).
- **L-OL1-F1 «stampo» SPEDITA (S-127, pin s127 834f5e01)**: template Props per classe,
  default COW — objalloc −20,4% (7,7×), churn 8,9×. **S-129: MODELLO DEL TEMPO seg.3
  CHIUSO** (s129-modello-tempo.md): statement Field* ≈300–340 ns QUASI INVARIANTE per
  forma (oracle 23–37; locale 170); torta per-passo (chiusura 96%): **E−E2
  (dispatch+prop_step) ~155 ns (52%)** — residuo per sottrazione; l'attribuzione
  «resolve-per-NOME» è INDIZIO (profilo+disasm), sonda diretta E1a dovuta in S-130
  (rev. S-129) · preludio byref/indirect/lazy ~73 ns (25%, sondato E corroborato
  dall'A/B F4) · walk interno 48 (16%);
  i 2 alloc residui = n.clone() del nome in byref_hook_root+field_lazy_root (census
  22/22). **F4 «prelude-gate» SPEDITA S-130** (criterio emendato pre-registrato:
  rumore trimmed drop-1 simmetrico + bande fondate 6,7/6,7/13,3): smoke +80,0 →
  R=5 D=+80,0 vs soglia 16,7, direzione 14/14 cumulata, promozione piena rc=0 →
  **pin s130 0fdf1c49**. **Sonda E1a S-130** (s130-e1a-lettura.md): controllo
  objalloc k=4 svela le resolve del CTOR (criterio p.2 EMENDATO a verbale S-131).
  **MODELLO PROP_STEP S-131** (s131-propstep-lettura.md, chiusura 93–94%): E−E2
  166,9 = prop_step interno 130,7 (guardie 49,4 · defer 37,0 · key+op 34,3 ·
  borrow 1,5 · altro 8,5) + dispatch 36,3; resolve statement 40,3 su 5 siti
  enumerati per NOME (3× prop_key + prop_key_read + prop_indirect_guard ≈0);
  ctor 70,8 (17,7/resolve, più care). **E1-KO «resolve-once» SPEDITA S-131**
  (criterio pre-registrato: smoke +45,0 → R=5 D=+23,3 vs soglia 13,3, guardie
  9/9, promozione rc=0) → **pin s131 ff66cb84**. **L-LO1 «lookup-once» SPEDITA
  S-132** (criterio pre-registrato con soglia az.rev. #1 = spread-batch 10,0:
  smoke +23,3 → R=5 D=+20,0, riconciliazione 3,3 in banda e dentro UB 30,
  guardie 9/9, promozione rc=0) → **pin s132 6af6e497**: UN accesso alla
  props-map nel ramo non-leaf (slot WP-29 dalla resolve, fallback by-name).
  **Leva ctor «resolve-once» SPEDITA S-133** (sonda a soli conteggi conferma lo
  split 2+2 magic_applies/fallback-PropSet; criterio pre-registrato soglia
  max(4, drop-1, spread-batch 6,7): smoke +36,7/+31,7 → R=5 objalloc D=+46,7
  [DICHIARATO fuori UB 35,4, +11,3 non ripartita] + objdatains D=+30,0,
  guardie 8/8, promozione rc=0) → **pin s133 c87439a9**: UNA resolve hoistata
  post-hook in prop_set_entry, condivisa dal magic-check e dal blocco
  key/slot/IC; hooked-set resta a zero resolve.
  **Leva «IC non-plain» SPEDITA S-134** (eccedenza s133 prima NOMINATA dal
  disasm — seconda lookup dipendente back-to-back, `s134-eccedenza-lettura.md`;
  criterio pre-registrato coi componenti non prezzati DICHIARATI per nome,
  soglie spread-batch s133 26,7/13,3: smoke +150,0/+130,0 → R=5 objalloc
  D=+136,7 + objdatains D=+133,3, riconciliazioni in banda, guardie 8/8,
  promozione rc=0 su catena a 9 gate) → **pin s134 61896da1**: bit NP/TY nei
  2 bit alti dello slot del PropIc; fill dal cammino pieno SOLO con fatti di
  classe provati (no set/virtual-hook, no `__set` — load-bearing per il
  typed-unset —, asym ok, non readonly, key==name); il hit salta resolve +
  magic-probe + asym/readonly/hook-lookup e MANTIENE coercizione typed,
  presenza slot e typed_refs per-scrittura.
  Residui NOMINATI: dispatch 36,3 · contabilità del non-resolve residuo da
  RI-DERIVARE sul pin s134 (il hit IC copre parte dei ~60 ns/statement) ·
  cammini non cacheabili per costruzione (readonly, private mangled, `__set`
  presente, slot assente).
  **Leva «AP1 fast-path» SPEDITA S-135** (scelta dai numeri: bisezione objmap
  → canale dominante = macchineria dim-set 183,3 = 77%, poi modello del tempo
  AssignPath su m3: arm 66,8, path_op 52,2 = 78%, walk-plumbing 38,4,
  chiusura 86% INCOMPLETO dichiarato; criterio pre-registrato con UB
  FALSIFICABILE 47,7 = prezzi misurati; A/B r1 rc=5 agli atti — guardia
  objalloc su banda sotto-fondata — emenda rev. S-112: guardie alla formula
  del giudice; r2 R=5 objmap D=+56,7 ≤ 57,7, riconc. smoke 6,7 ≤ 10,
  guardie 9/9, promozione rc=0) → **pin s135 6518a1e1**: nel braccio
  AssignPath, caso 1-chiave/no-append/base GIÀ Array = specializzazione
  letterale del cammino pieno (coerce → make_mut → set_returning_displaced →
  gc_note → push), tutto il resto al pieno invariato; sonda dim-write residua
  (objdatains 2 resolve/iter sul cammino prop-dim, fuori perimetro) a
  catalogo. Sonda conteggi S-135: eccedenza S-134 ATTRIBUITA (5 canali 2→0,
  depr 0→0 falsificato, layout escluso; `s135-eccedenza-chiusura.md`).
  **Leva «FD1 fast-path dim-write su prop» SPEDITA S-136** (dal reperto sonda:
  2 resolve/iter su `$e->data['k']=$i` → lowering `FieldAssign{[Prop,Index]}`
  verificato col dump; modello tempo FieldAssign su m-dimwrite, chiusura 94%:
  arm 118,2 = walk_driver 37,2 · leaf 18,9 · plumbing 17,6 · prop_step_altro
  14,4 · guardia 11,3 · resolve 6,7 · dispatch 7,0 · pop 4,5; criterio con UB
  falsificabile 69,6 = somma canali bypassati; A/B R=5 objdatains D=+83,3,
  soglia 13,3, riconc. smoke 1,6, **FUORI-UB +0,4 DICHIARATO** — eccedenza
  +13,7 non ripartita, sonda dovuta; guardie 10/10, `re` morsa allo smoke
  rientrata a R=5 col drop-1 vero; promozione rc=0) → **pin s136 1e14793e**:
  cella PropIc su `Op::FieldAssign`, fast path `[Prop,Index]` con
  `field_write_walk` RIUSATO sul child Array (leaf identico per costruzione)
  + driver-loop replicato; fill dal ramo F4 a esito Ok coi fatti di classe
  (slot key==name, non readonly, asym ok; hooks esclusi da F4). Perimetro
  fuori: child Ref/Str/assente, nkeys≠1, unset-prop, readonly, asym-negata.
  **Eccedenza FD1 CHIUSA (S-138)**: disasm refuta l'artefatto-inlining (bl +63
  = timer); sonda arm-only v2 (inerzia 0,000) dà arm 51,9 pulito; **A/B pin
  s135↔s136 sul giudice del modello: D_mdw 63,3 vs UB 69,6 IN BANDA** — la
  «eccedenza» era aritmetica CROSS-GIUDICE (D 83,3 misurato su objdatains, arm
  su m-dimwrite); coerenza-arm 51,9+63,3=115,2 ≈ 118,2. Dim-write SBLOCCATO.
  **Leva «FD1-ext RMW» SPEDITA S-138** (criterio s138-criterio-rmw.md: cella IC
  su FieldAssignOp/FieldIncDec; fast = admission FD1 + peek entry + op silente
  {Add,Sub,Mul}×{Long,Double} + field_write_walk riusato; fill dal ramo piano
  via field_prelude_skip; il pieno pagava DUE walk + preludio. A/B R=5:
  m-dimrmw D=+173,3, m-diminc D=+156,7, guardie 7/7, objdatains ±0,0;
  fuori-modello +110 ATTRIBUITO con sonda monobinaria kill-switch (scarto
  +3,7/17,3: arm_full 266,9 − arm_fast 89,9 = 177,0 ≈ D); promo rc=0 + conferma
  post-pin in banda 5,0/5,0) → **pin s138 fa17dabd + server a9aded45**.
  Aperture per NOME: FieldRead/dim-read IC (famiglia sbloccata) · divergenze
  RMW del pieno (undefined-key, float-key, str-increment, overloaded-notice) ·
  14% modello AssignPath (86%) · **objmap «valore-oggetto» 43,4 ATTRIBUITO
  (S-137, census) al round-trip GC nota→sweep→demozione — leva note-time
  REFUTATA (precedente WP-21), cura = piano gc-cycle-collector**.
- Aperture per NOME: `evalcls` **316,9×** (cliff compile-per-classe; serve strumento di densità
  prima di ogni leva) · `refl` **42,4×** · re +2,00 alloc/iter.

## Voci da misurare (per NOME)

lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
DBAL: catalogare i 10 nomi Portability/parser-unicode in PHPR_DIVERGENCES.
