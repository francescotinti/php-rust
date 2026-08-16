# WP_SESSION_145 — sonda-B chiude la via B com'era disegnata (KS-B4: il collo è il pavimento per-movimento) e la leva FR1 «dim-read fuso» è SPEDITA con pin nuovo s145

**In una frase**: abbiamo misurato che il costo del ciclo clone/drop è per il
69,5% il puro pavimento «sposta e smista» (non i contatori né le note GC),
quindi le fette B progettate non si aprono e la palla torna al concilio; nel
frattempo una leva concreta sulle letture `$obj->prop['chiave']` — leggere
l'elemento senza clonare l'array di mezzo — ha tagliato il 28% del costo di
quella lettura ed è stata promossa a pin con tutti i gate verdi.

**SCOREBOARD** (pin NUOVO s145 a89faf32+4a9adc51, micro R=5 dalla catena):
**arith 5,5 → · prop 5,5 ↓ · calls 4,8 ↑ · str 4,3 ↑ · arr 3,2 → · re 2,5 ↓**
(tutte entro 1 tick dai rif s142; hintcall 7,3 non rimis.; **dimread 4,3
NUOVA** — era 6,0 pre-leva) · WP rif 1,765–1,788 (fermo; **coppia DOVUTA al
pin nuovo → S-146 p.1**) · **leve perf spedite: 1 (L-FR1)** · incidenti 15
(nessun nuovo).

## Esiti secchi
1·**p.1 SONDA-B** (verdetto s145-sonda-b-verdetto.out; prezzi t2 record,
  t1/t3 dichiarati; conteggi ORM ×2 parità rc=0): TRE numeri firmati —
  **memcpy 69,5% · inc-dec 14,1% · nota 16,4%** ⇒ **KS-B4 SCATTA: B1/B2 NON
  si aprono, filone conteggi (B3) TORNA AL CONCILIO**. Pair a banda
  zcell 8,71–8,80 / arr0 11,57–11,79 ns (Hoare §7.4 ora MISURA). gcnote
  238,6M == dossier S-141 ESATTO.
2·**p.2 LEVA L-FR1 SPEDITA** (criterio pre-registrato; peephole in place,
  fallback per costruzione): m-dimread **60,0→43,3 ns/iter, D=+16,7 (−28%),
  segni 7/7**, guardie verdi (RMW +1 tick dichiarato), disasm agli atti;
  catena promo rc=0 (batteria 1747/0/2 + dente, corpus 1414×2, fixture 9/9,
  micro, ORM 16 nomi, hk 0E/0F) → **pin s145**; conferma post-pin = identità
  di byte col braccio giudicato.
3·**p.3 az.rev. S-144**: #1 CHIUSA (riparse simmetrico committato+golden:
  emenda v2 riprodotta ESATTA, phpr idle=0 verificato 4/4, memops FUORI /
  churn IN confermati) · #4 CHIUSA (dente rczval-funnel VERDE in batteria).
  p.4 str-provenienza NON aperta (tempo) → resta in coda.

## ⭐ Lezioni (max 3)
- ⭐⭐ Le àncore relative di un dente (N_OPS−k) non si emendano con replace
  testuali: «N_OPS - 2» morde «N_OPS - 23» (prefisso) — si riscrivono per
  NOME, e il dente stesso l'ha provato mordendo due volte.
- ⭐⭐ Un inventario per NOME muore sui nomi VOLATILI: i compile-fail VmGate
  portano il numero di riga nel nome e ogni edit del file li rinomina —
  normalizzare la parte volatile PRIMA del confronto (classe S-96).
- ⭐ Target dir condivisa tra worktree avvelena il fingerprint cargo: il
  «braccio A» era il binario B con un altro nome — l'ha svelato il gate
  hash+dump, il braccio di contrasto pretende target DEDICATA.
