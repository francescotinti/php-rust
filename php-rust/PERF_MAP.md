# PERF_MAP — phpr vs PHP oracle 8.5.7, mappa multi-workload

Aggiornata: **2026-08-11 (S-129)** · pin phpr **s127b ccb63dca** ·
metodo: user CPU, pavimenti per-binario, N per voce come indicato; criteri pre-registrati in
`wp125-harness/s125-criterio-{pair,mappa}.md` e `wp126-harness/s126-criterio-{orm,mappa2}.md`
(+ emenda S-127: **cifra canonica = NETTO-pavimento**, raw companion; gate contesa in ictx/s);
cifre dai verdetti `.out`. Regola di lettura: rapporti PER workload, MAI aggregato.

## Workload reali

| workload | rapporto phpr/oracle | N | note |
|---|---|---|---|
| **WordPress full-suite** | **1,758–1,805 PULITO** (gate ictx/s per gamba, S-129: leg1-off S-128 E leg3-off nuova SEGNALATE ed ESCLUSE — set INVARIATO anche con mediana PER MOTORE, addendum rev. S-129; entrambe prime-di-sequenza, indizio warm-up; coppie proprie pulite **1,765–1,805** user+sys, **1,772–1,813** user-only, N=3) | 3 gambe pulite (9 celle) | S-129 @ pin s127b; phpr 782–796 s; parità per NOME invariata; peak 1828–1880 MiB sulle pulite; verdetto `wp129-harness/s129-pair-legoff-verdetto.out` |
| **WordPress gruppo media** | **2,447–2,463 CANONICA user-only** (gambe pulite; companion user+sys 2,408–2,419; az.rev. S-128 #3: una sola media) | 3 | S-129 @ s127b; le gambe segnalate davano 2,506–2,539 |
| **symfony http-foundation** (1854) | **2,547–2,559** (raw 2,55–2,57) | 2/lato | S-126; canonica sul CONTEGGIO diff 17 nomi = 0,92% ≤1% (≥3 nomi sono unit puri, NON famiglia `php -S` — emenda S-127); sys alto (I/O) |
| **symfony http-kernel** (1665 test) | **4,29–4,32** | 2/lato | parità 0E/0F; contesa ok |
| **doctrine/collections** (242) | **8,22 net** (raw 6,20) | 2/lato | S-126; INDICATIVA: oracle netto 0,09 s (denominatore sotto-scala); parità 0/0 |
| **doctrine/dbal** (3929, sqlite) | **8,57–8,60** (raw 8,29–8,33) | 2/lato | S-126; fail-set stabile 10 nomi (0,25% ≤1% ⇒ canonica; Portability+parser unicode a catalogo); ictx assoluti nel verdetto, gate in ictx/s da emenda S-127 |
| **doctrine/orm** (3484 test) | **8,51–8,56** | 2/lato | oracle con `memory_limit=-1` (§3.14); parità fail-set 16 nomi |
| **composer install OFFLINE** | **1,863–1,891 net** (raw 1,820–1,847) | 2/lato | S-128 @ s127b, PRIMA misura col numeratore vivo (cure ondata-2); composer ESTRATTO, vendor_ok bilaterale, contesa ok (ictx/s); floors 0,07/0,06; sys≈user (~2,3 s/lato) ⇒ **cifra user-only NON confrontabile col full (user+sys): su user+sys sarebbe ~1,3** (rev. S-128 az.5); residuo phpcs config-set (§3.19-quinquies); verdetto `wp128-harness/s128-compoff-verdetto.out` |

## Micro-categorie (R=5, pin s127b; tappa ≤3×; gate S-129)

| arith | prop | calls | str | arr | re |
|---|---|---|---|---|---|
| 5,3 | 5,6 | 5,0 | **4,2** | **3,2** | **2,5** ✅ |

calls: la (*) di s127 è SCIOLTA in S-129 (phpr netto IDENTICO 2,14 s; si muove
solo il denominatore oracle 0,43–0,44). re 2,5/2,6 = run-to-run del denominatore.

Allocazioni/iter vs oracle: arith/prop/calls 0=0 · **str 2,00=2,00 (PARITÀ, S-125)** ·
arr 2,05≈2,03 · re 7,00 vs 5,00 (+2, apertura per NOME).

## Micro-ORM (S-126 istruttoria; S-127 post-leva L-OL1-F1 — verdetti s127-submicro + s127-ab)

| evalcls (compile/classe via eval) | refl | objchurn | └ objalloc (new+ctor+drop) | └ objmap (insert map) |
|---|---|---|---|---|
| **316,9** (2,38 ms vs 7,5 µs) | **42,4** | 10,2→**8,9** | 9,6→**7,7** (976,7 vs 126,7 ns; additività chiusa 3,4 ns) | 17,3 |

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
  22/22). **F4 «prelude-gate» TENTATA S-129**: census 11/11, smoke +71,7 PROMOSSA,
  R=5 D=+66,7 AVVERSA per criterio (soglia 70 da outlier singolo; objmap −6,7 =
  layout su banda default) — direzione firmata 7/7, revert al byte; **rientro S-130
  con criterio emendato** (s129-ab-f4-lettura.md); poi resolve-once (E1) col modello.
- Aperture per NOME: `evalcls` **316,9×** (cliff compile-per-classe; serve strumento di densità
  prima di ogni leva) · `refl` **42,4×** · re +2,00 alloc/iter.

## Voci da misurare (per NOME)

lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
DBAL: catalogare i 10 nomi Portability/parser-unicode in PHPR_DIVERGENCES.
