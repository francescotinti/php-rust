# PERF_MAP — phpr vs PHP oracle 8.5.7, mappa multi-workload

Aggiornata: **2026-08-10 (S-127, emenda cifra canonica)** · pin phpr **s125 002e6cc1** ·
metodo: user CPU, pavimenti per-binario, N per voce come indicato; criteri pre-registrati in
`wp125-harness/s125-criterio-{pair,mappa}.md` e `wp126-harness/s126-criterio-{orm,mappa2}.md`
(+ emenda S-127: **cifra canonica = NETTO-pavimento**, raw companion; gate contesa in ictx/s);
cifre dai verdetti `.out`. Regola di lettura: rapporti PER workload, MAI aggregato.

## Workload reali

| workload | rapporto phpr/oracle | N | note |
|---|---|---|---|
| **WordPress full-suite** | **1,815–1,896** | 4/lato (16 celle) | pin s124 pre-cbargs2 (effetto leva ≤~2%); parità per NOME; peak mem ~2,7× |
| **WordPress gruppo media** | **2,485–2,518** | 4 | user-only |
| **symfony http-foundation** (1854) | **2,547–2,559** (raw 2,55–2,57) | 2/lato | S-126; canonica sul CONTEGGIO diff 17 nomi = 0,92% ≤1% (≥3 nomi sono unit puri, NON famiglia `php -S` — emenda S-127); sys alto (I/O) |
| **symfony http-kernel** (1665 test) | **4,29–4,32** | 2/lato | parità 0E/0F; contesa ok |
| **doctrine/collections** (242) | **8,22 net** (raw 6,20) | 2/lato | S-126; INDICATIVA: oracle netto 0,09 s (denominatore sotto-scala); parità 0/0 |
| **doctrine/dbal** (3929, sqlite) | **8,57–8,60** (raw 8,29–8,33) | 2/lato | S-126; fail-set stabile 10 nomi (0,25% ≤1% ⇒ canonica; Portability+parser unicode a catalogo); ictx assoluti nel verdetto, gate in ictx/s da emenda S-127 |
| **doctrine/orm** (3484 test) | **8,51–8,56** | 2/lato | oracle con `memory_limit=-1` (§3.14); parità fail-set 16 nomi |
| **composer install OFFLINE** | phpr **NULLA** (voce aperta) | 2/lato | phar/`__halt_compiler` assente; rimisura con composer 2.10 ESTRATTO abortita dallo smoke: **rc=255 silente** (§3.19 aggravata) — bisezione S-127 |

## Micro-categorie (R=5, pin s125; tappa ≤3×)

| arith | prop | calls | str | arr | re |
|---|---|---|---|---|---|
| 5,5 | 5,6 | 4,7 | **4,2** | **3,2** | **2,6** ✅ |

Allocazioni/iter vs oracle: arith/prop/calls 0=0 · **str 2,00=2,00 (PARITÀ, S-125)** ·
arr 2,05≈2,03 · re 7,00 vs 5,00 (+2, apertura per NOME).

## Micro-ORM (istruttoria S-126, R=5 — `wp126-harness/s126-orm-micro{,2}-verdetto.out`)

| evalcls (compile/classe via eval) | refl | objchurn | └ objalloc (new+ctor+drop) | └ objmap (insert map) |
|---|---|---|---|---|
| **316,9** (2,38 ms vs 7,5 µs) | **42,4** | **10,3** | **9,9** (1220 vs 123 ns; ~67% del churn) | 17,3 (~10%) |

Profilo ORM phpr (indizio unilaterale): churn visibile multi-% (Zval clone/drop, slot_of,
gc_note/sweep/collect_cycles, insert/lookup, malloc/free); compile ≤~1% leaf, reflection <0,5%.

## Lettura (direzione+indizio, NON attribuzioni firmate — REGOLE §4)

- Il gap **cresce con la densità di lavoro-motore puro**: WP 1,85 ≪ hf 2,6 ≪ hk 4,3 ≪
  dbal 8,6 ≈ ORM 8,5 (cifre net, emenda S-127). WP e hf sono diluiti da I/O; le suite object-dense mostrano il soffitto.
- **dbal 8,6 conferma ORM 8,5 senza mock-eval pesante** ⇒ il driver è il lavoro-oggetti, non il
  sentiero compile: coerente con l'istruttoria (compile ≤1% leaf nel run reale).
- **LEVA NOMINATA: L-OL1 ciclo-di-vita oggetto** (objalloc 9,9× = 67% del churn) →
  `wp126-harness/s126-leva-nominata.md` (criterio A/B pre-scritto; esecuzione S-127).
- Aperture per NOME: `evalcls` **316,9×** (cliff compile-per-classe; serve strumento di densità
  prima di ogni leva) · `refl` **42,4×** · re +2,00 alloc/iter.

## Voci da misurare (per NOME)

compoff (in coda stanotte) · lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
DBAL: catalogare i 10 nomi Portability/parser-unicode in PHPR_DIVERGENCES.
