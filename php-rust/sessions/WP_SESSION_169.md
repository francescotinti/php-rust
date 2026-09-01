# WP_SESSION_169 — az.rev. S-168 eseguite (stessa conversazione): DISPATCH puro 1,75 ns/op = oracle intero; il divario è il CORPO dei handler (banale 5,6; fuso ~27,4, ~17,5 non nominati); c3 fissata con mutante casuale ⇒ mispredict REFUTATO firmato
**In una frase**: abbiamo misurato quanto costa il solo «passare all'istruzione
successiva» (1,75 ns, quanto l'oracle spende per un'istruzione INTERA) e scoperto
che il resto del tempo — quattro quinti — se ne va dentro il corpo di ogni
istruzione, anche delle più banali: è lì, non nel dispatcher né nel compilatore,
che la prossima rotta deve guardare; la decisione formale (R4) resta all'utente.
**SCOREBOARD** (pin INVARIATO **s166 phpr 092dcff431bef876 + server
caa4e4b2638686a9**): arith 5,4 = · prop 5,5 = · calls 4,8 = · str 4,2 = ·
arr 3,2 = · re 2,5 = · WP 1,746-1,749 (rif.) · ORM [7,023;7,053] (rif.) ·
**leve spedite: 0 — sanzionato ⚖️ (kill S-168 ⇒ delibera R4 prima di ogni
codice)** · incidenti: **1** (ENOSPC: 22 `.ktrace` temporanei di xctrace =
10 GB in $TMPDIR, curato con purge nel copione) + 2 difetti curati in corsa
(path E2 del copione derivato ⇒ colonna e2 invalida per m5/m7, dq validi;
residuo `wp168-harness` nel build script sfuggito a copia-gate v2).

## Esiti secchi (criterio s169-criterio.md pre-registrato, emende p.6-7 dichiarate)
1·**m5** (8 Nop/iter, dump collaudato): D=−14,00 ⇒ **dispatch puro 1,75 ns/op
  (±0,05)**; e2 14,7 = 2×1,75 + 2 corpi ⇒ **corpo CmpJmpSC/IncDecSlotJmp ≈5,6
  ns/op** (oracle 1,8 TOTALI/op) · **m7** (m2 senza guardia) +5,60 ⇒ guardia
  tupla 0,4 (sotto rumore) · **m4b R=9 +2,88** (Sweep/iter ≈2,9: 1,75+1,1) ·
  **m3 R=9 −2,60** riproducibile (ristrutturazione > hoist: CONFUSO; m3-puro
  non scrivibile in safe Rust) · A=m0, tutti bl/output collaudati.
2·Decomposizione dq 46,7: loop 14,7 · Sweep 2,9 · BinarySCSCDst 1,75 + ~27,4
  corpo (consts 4,7 + BinOp 5,2-5,6 nominati; **~17,5 NON nominati**: guardie
  Undef/Ref, read_slot clone, 4 funnel, reg_store_slot+gc_note, bounds).
  Per meccanismo nominato 43%; per esclusione «corpo handler» 61%.
3·**xctrace-2**: rbranchmut (LCG bit 30) alza SOLO c3 su ENTRAMBI (phpr
  0,007→0,084; oracle 0,024→0,357) ⇒ c3=discarded NON circolare ⇒ **mispredict
  REFUTATO firmato** (phpr 0,007 vs oracle 0,024) · mutante c0 (crc32) FALLITO
  (alza c1: catena di dipendenza = stallo) ⇒ c0=useful solo per esclusione
  esaustiva (4 categorie, 3 fissate) — positivo c0 ancora dovuto.
4·Per la DELIBERA R4 (utente): Σ nominata 9,92 · Σ grezza 12,9 · tetto di ogni
  rotta «meno op / dispatch più leggero» = 1,75 ns/op; bersaglio grosso = corpo di
  OGNI handler (accesso slot, guardie, Result/`?`, gc_note, clone/drop).

## ⭐ Lezioni (max 3)
- ⭐⭐ il collaudo di parità A==B deve controllare l'output ATTESO, non solo
  l'uguaglianza (due errori identici passano: colonna e2 vacua per m5/m7).
- ⭐⭐ copia-gate v2 passa una riga con etichetta stantia E corrente insieme
  (`wp168-harness` accanto a `s169`): il gate va per TOKEN, non per riga. ⭐ xctrace lascia `.ktrace` da GB in $TMPDIR: purge nel copione, sempre.
