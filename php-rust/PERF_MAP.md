# PERF_MAP — phpr vs PHP oracle 8.5.7, mappa multi-workload

Aggiornata: **2026-08-10 (S-125)** · pin phpr **s125 002e6cc1** · metodo: user CPU,
pavimenti per-binario, N per voce come indicato; criteri pre-registrati in
`wp125-harness/s125-criterio-{pair,mappa}.md`; cifre dai verdetti `.out`.
Regola di lettura: rapporti PER workload, MAI aggregato (un aggregato è diluito).

## Workload reali

| workload | rapporto phpr/oracle | N | note |
|---|---|---|---|
| **WordPress full-suite** | **1,815–1,896** | 4/lato (16 celle) | pin s124 pre-cbargs2 (effetto leva ≤~2%); parità per NOME; peak mem ~2,7× |
| **WordPress gruppo media** | **2,485–2,518** | 4 | user-only |
| **symfony http-kernel** (1665 test) | **4,29–4,32** | 2/lato | parità 0E/0F; contesa ok |
| **doctrine/orm** (3484 test) | **8,51–8,56** | 2/lato | oracle con `memory_limit=-1` (phpr non applica il limite, §3.14 — emenda criterio p.7); parità fail-set 16 nomi |

## Micro-categorie (R=5, pin s125; tappa ≤3×)

| arith | prop | calls | str | arr | re |
|---|---|---|---|---|---|
| 5,5 | 5,6 | 4,7 | **4,2** | **3,2** | **2,6** ✅ |

Allocazioni/iter vs oracle: arith/prop/calls 0=0 · **str 2,00=2,00 (PARITÀ, S-125)** ·
arr 2,05≈2,03 · re 7,00 vs 5,00 (+2, apertura per NOME).

## Lettura (direzione+indizio, NON attribuzioni firmate — REGOLE §4)

- Il gap **cresce con la densità di lavoro-motore puro**: WP 1,85 ≪ hk 4,3 ≪ ORM 8,5.
  WP è il caso MIGLIORE perché diluito da I/O (MySQL, filesystem) ed estensioni
  native; le suite pure-engine mostrano il soffitto vero.
- **ORM 8,5× è il segnale più grosso della mappa**: indiziati (da istruire in
  S-126) i mock PHPUnit generati via eval (sentiero di COMPILE per classe),
  reflection, e il churn oggetti/UoW — coerente con micro prop 5,6 e calls 4,7.
- hk 4,3× ≈ le micro str/calls: workload stringhe/closure senza I/O che diluisce.
- ictx: gamba orm-phpr1 segnalata (contesa >1,5× mediana) ma le due gambe ORM
  concordano allo 0,5% ⇒ cifra tenuta.

## Voci da misurare (S-126, per NOME)

DBAL (3769) · http-foundation · collections/lexer/inflector/event-manager ·
Composer install OFFLINE (rete esclusa) · wp-cli · PHPUnit-self.
