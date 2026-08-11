# PERF_MAP — phpr vs PHP oracle 8.5.7, mappa multi-workload

Aggiornata: **2026-08-11 (S-128)** · pin phpr **s127b ccb63dca** ·
metodo: user CPU, pavimenti per-binario, N per voce come indicato; criteri pre-registrati in
`wp125-harness/s125-criterio-{pair,mappa}.md` e `wp126-harness/s126-criterio-{orm,mappa2}.md`
(+ emenda S-127: **cifra canonica = NETTO-pavimento**, raw companion; gate contesa in ictx/s);
cifre dai verdetti `.out`. Regola di lettura: rapporti PER workload, MAI aggregato.

## Workload reali

| workload | rapporto phpr/oracle | N | note |
|---|---|---|---|
| **WordPress full-suite** | **1,758–1,909** | 4/lato (16 celle) | S-128 @ pin s127b (post cbargs2+stampo); phpr 782–809 s (S-125: 802–828, ↓ su OGNI gamba); intervallo allargato in alto dal denominatore (spread oracle 5,1%, gamba off1 veloce 423,6 s); parità per NOME (solo `wp_is_stream data set #2` ×4); peak mem 2,299–2,374× (phpr 1828–1894 MiB); verdetto `wp128-harness/pair-out/cross-ratios.out` |
| **WordPress gruppo media** | **2,447–2,539** | 4 | user-only; S-128 @ s127b (S-125: 2,485–2,518); parità 0 nomi ×4 |
| **symfony http-foundation** (1854) | **2,547–2,559** (raw 2,55–2,57) | 2/lato | S-126; canonica sul CONTEGGIO diff 17 nomi = 0,92% ≤1% (≥3 nomi sono unit puri, NON famiglia `php -S` — emenda S-127); sys alto (I/O) |
| **symfony http-kernel** (1665 test) | **4,29–4,32** | 2/lato | parità 0E/0F; contesa ok |
| **doctrine/collections** (242) | **8,22 net** (raw 6,20) | 2/lato | S-126; INDICATIVA: oracle netto 0,09 s (denominatore sotto-scala); parità 0/0 |
| **doctrine/dbal** (3929, sqlite) | **8,57–8,60** (raw 8,29–8,33) | 2/lato | S-126; fail-set stabile 10 nomi (0,25% ≤1% ⇒ canonica; Portability+parser unicode a catalogo); ictx assoluti nel verdetto, gate in ictx/s da emenda S-127 |
| **doctrine/orm** (3484 test) | **8,51–8,56** | 2/lato | oracle con `memory_limit=-1` (§3.14); parità fail-set 16 nomi |
| **composer install OFFLINE** | **1,863–1,891 net** (raw 1,820–1,847) | 2/lato | S-128 @ s127b, PRIMA misura col numeratore vivo (cure ondata-2); composer ESTRATTO, vendor_ok bilaterale, contesa ok (ictx/s); floors 0,07/0,06; sys≈user (I/O-denso, ~2,3 s/lato); residuo phpcs config-set (§3.19-quinquies); verdetto `wp128-harness/s128-compoff-verdetto.out` |

## Micro-categorie (R=5, pin s127b; tappa ≤3×)

| arith | prop | calls | str | arr | re |
|---|---|---|---|---|---|
| 5,3 | 5,6 | 4,9(*) | **4,2** | **3,2** | **2,6** ✅ |

(*) calls +0,2 vs s127: bordo del run-to-run, da osservare in S-128.

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
  default COW — objalloc −20,4% (7,7×), churn 8,9×; corpus bug69534 flippa VERDE
  (stesso meccanismo). Prossimo segmento nominabile dal churn residuo: Δins 320 ns
  (insert su array di proprietà) — vedi wp127-harness/s127-submicro-letture.md.
- Aperture per NOME: `evalcls` **316,9×** (cliff compile-per-classe; serve strumento di densità
  prima di ogni leva) · `refl` **42,4×** · re +2,00 alloc/iter.

## Voci da misurare (per NOME)

lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
DBAL: catalogare i 10 nomi Portability/parser-unicode in PHPR_DIVERGENCES.
