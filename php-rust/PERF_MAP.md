# PERF_MAP — phpr vs PHP oracle 8.5.7, mappa multi-workload

Aggiornata: **2026-08-13 (S-134)** · pin phpr **s134 61896da1** (leva IC non-plain SPEDITA sopra la ctor resolve-once; WP full/media RIMISURATI in S-134 @ pin s134; hf/hk/dbal/ORM/compoff restano @ pin precedenti — DUE leve consecutive parlano alle suite object-dense: rimisura dbal/ORM DOVUTA in S-135) ·
metodo: user CPU, pavimenti per-binario, N per voce come indicato; criteri pre-registrati in
`wp125-harness/s125-criterio-{pair,mappa}.md` e `wp126-harness/s126-criterio-{orm,mappa2}.md`
(+ emenda S-127: **cifra canonica = NETTO-pavimento**, raw companion; gate contesa in ictx/s);
cifre dai verdetti `.out`. Regola di lettura: rapporti PER workload, MAI aggregato.

## Workload reali

| workload | rapporto phpr/oracle | N | note |
|---|---|---|---|
| **WordPress full-suite** | **ON-ONLY CANONICO 1,769** (S-134 @ pin s134; **N=2 coppie proprie CONCORDI 1,769/1,769 — prima banda propria**; off 1,748–1,789 N=2; misto pulito 1,730–1,791; quiescenza rc=0 ×5, CI sospesa via lock) | **4/4 gambe pulite** (16 celle; prima coppia senza esclusioni dal s131) | S-134 @ s134; parità per NOME 4/4 (solo `wp_is_stream #2`); **peak 1818–1884 MiB (bordo alto +80 vs s131/s132 PERSISTE, da tenere d'occhio)**; verdetto `wp134-harness/s134-pair-verdetto-t1.out`; le leve ctor+IC-non-plain NON muovono WP (coerente: mordono il cammino non-plain, profilo ORM) |
| **WordPress gruppo media** | **2,405–2,467 CANONICA user-only** (companion user+sys 2,324–2,411, 4 gambe pulite) | 4 | S-134 @ s134 |
| **symfony http-foundation** (1854) | **2,547–2,559** (raw 2,55–2,57) | 2/lato | S-126; canonica sul CONTEGGIO diff 17 nomi = 0,92% ≤1% (≥3 nomi sono unit puri, NON famiglia `php -S` — emenda S-127); sys alto (I/O) |
| **symfony http-kernel** (1665 test) | **4,29–4,32** | 2/lato | parità 0E/0F; contesa ok |
| **doctrine/collections** (242) | **8,22 net** (raw 6,20) | 2/lato | S-126; INDICATIVA: oracle netto 0,09 s (denominatore sotto-scala); parità 0/0 |
| **doctrine/dbal** (3929, sqlite) | **8,36–8,45 net** (raw 8,10–8,17) | 2/lato | **S-135 RIMISURATA @ pin s134** (verdetto `wp135-harness/s135-rimisura-verdetto.out`; pavimento PER-workspace 0,06/0,19): fail-set stabile 10 nomi (0,25% ≤1% ⇒ canonica); vs baseline s125-pin 8,57–8,60: phpr assoluto −4% MA oracle −3% (drift ambiente) ⇒ **rapporto FERMO dentro il rumore**; gamba phpr2 segnalata ictx/s ma stesso-lato <3% ⇒ valida; cattura summary dbal-phpr VUOTA (classe S-126 #3, fail-set dai .failnames) |
| **doctrine/orm** (3484 test) | **8,43–8,56 net** | 2/lato | **S-135 RIMISURATA @ pin s134** (stesso verdetto; oracle `memory_limit=-1` §3.14; parità 16 nomi == baseline): vs 8,51–8,56 @ s125 ⇒ **INVARIATO** — REPERTO pre-registrato (criterio p.6): 6 leve object s127→s134 (objalloc micro −20% e −14%) NON muovono il rapporto suite (phpr −2,3 s assoluti ≈ −5%, oracle −4% drift): il typed-set/ctor è fetta minore del churn ⇒ la prossima leva si sceglie sul profilo SUITE (insert/lookup, clone/drop) |
| **composer install OFFLINE** | **1,863–1,891 net** (raw 1,820–1,847) | 2/lato | S-128 @ s127b, PRIMA misura col numeratore vivo (cure ondata-2); composer ESTRATTO, vendor_ok bilaterale, contesa ok (ictx/s); floors 0,07/0,06; sys≈user (~2,3 s/lato) ⇒ **cifra user-only NON confrontabile col full (user+sys): su user+sys sarebbe ~1,3** (rev. S-128 az.5); residuo phpcs config-set (§3.19-quinquies); verdetto `wp128-harness/s128-compoff-verdetto.out` |

## Micro-categorie (R=5, pin s134; tappa ≤3×; gate promozione S-134)

| arith | prop | calls | str | arr | re |
|---|---|---|---|---|---|
| 5,4 | 5,5 | 5,0 | **4,2** | **3,2** | **2,5** ✅ |

calls: la (*) di s127 è SCIOLTA in S-129 (phpr netto IDENTICO 2,14 s; si muove
solo il denominatore oracle 0,43–0,44). re 2,5/2,6 = run-to-run del denominatore.

Allocazioni/iter vs oracle: arith/prop/calls 0=0 · **str 2,00=2,00 (PARITÀ, S-125)** ·
arr 2,05≈2,03 · re 7,00 vs 5,00 (+2, apertura per NOME).

## Micro-ORM (S-134 sul pin s134 POST-leva-IC-non-plain — verdetto s134-submicro; evalcls/refl da S-126)

| evalcls (compile/classe via eval) | refl | objchurn | └ objalloc | └ objdatains | └ objdropdef | └ objallocni | └ objmap |
|---|---|---|---|---|---|---|---|
| **316,9** (2,38 ms vs 7,5 µs) | **42,4** | 8,2→**7,4** (1303,3 ns) | 7,5→**6,6** (813,3, −133,4 = IC non-plain) | 7,2→**6,4** (1066,7, −116,6) | 8,9→**7,9** | 9,4→**7,9** | **17,3** = |

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
- Aperture per NOME: `evalcls` **316,9×** (cliff compile-per-classe; serve strumento di densità
  prima di ogni leva) · `refl` **42,4×** · re +2,00 alloc/iter.

## Voci da misurare (per NOME)

lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
DBAL: catalogare i 10 nomi Portability/parser-unicode in PHPR_DIVERGENCES.
