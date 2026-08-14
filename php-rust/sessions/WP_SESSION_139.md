# WP_SESSION_139 — coppia @ s138 + banda_ON fondata + rimisura ORM/dbal (REPERTO) + az.rev. chiusa + CI viva

**In una frase**: il confronto WordPress sul motore promosso è fermo come
previsto e ora ha una banda fondata su 5 misure; le suite Doctrine dicono che
le tre leve di scrittura-array NON le muovono (reperto che orienta la prossima
leva); il collaudo RMW è chiuso; la CI locale e quella GitHub sono risanate.

**SCOREBOARD** (pin s138 INVARIATI; micro dal gate promo S-138, non rimisurate):
**arith 5,6 = · prop 5,6 = · calls 4,8 = · str 4,3 = · arr 3,2 = · re 2,6 =**
· **WP full ON-ONLY 1,752–1,785 (N=5) CANONICO · banda_ON FONDATA 0,033** ·
media 2,470–2,486 · dbal 8,15–8,23 ↓ · ORM 8,59–8,71 fermo/↑ REPERTO ·
**leve spedite: 0 (ANOMALIA dichiarata: la finestra è andata a 2 misure
canoniche obbligatorie + apparato CI + az.rev.)** · incidenti 14 (=).

## Esiti secchi
1·**Coppia t1 rc=0**: 6 gambe TUTTE ON, 5 pulite (leg6 SPORCA ictx 267% =
  proprio anomalo 1,810 escluso: la firma morde giusto); COMPATIBILE rif S-137
  su 0,041 ULTIMO USO ⇒ RMW non muove WP (atteso); parità 6/6; quiescenza 7/7;
  **peak 1831–1849 OSSERVAZIONE** (+80 MiB, candidate per NOME in PERF_MAP).
2·**Rimisura ORM/dbal rc=0** (ricetta S-135, chain): ORM **8,59–8,71 FERMO**
  vs 8,43–8,56 → **REPERTO pre-registrato: AP1+FD1+RMW non parlano alla
  suite** (attesa ↓ FALSIFICATA) ⇒ prossima leva dal profilo SUITE; dbal
  8,15–8,23 ↓ lieve; parità ORM 16==baseline, dbal 10 stabili.
3·**Az.rev. S-138 #1..#5 CHIUSE**: fixtures CALDE H1–H8 + v.21 coda mono-classe;
  byte-id 3 file (2 warning undefined-key A CATALOGO esclusi per NOME, zero
  NUOVE); gate hit build `ic-stats`: **21/9/23/11 PASS** (21 == modello 7×3);
  **equivalenza rw PROVATA** (unico consumatore prop_indirect_guard allo step
  Prop, mai raggiunto dal fast; probe uninit-typed su sito CALDO byte-id).
4·**CI**: locale — runner.lock PID-based + rottura atomica mv + trap
  owner-check + filtro `._*` (causa S-138: staleness a OROLOGIO su runner
  LEGITTIMO >6h); GitHub (richiesta utente) — ci.yml dei concili WP-78..83
  era INERTE in sottodirectory → radice: build+batteria Linux VERDI ~10′; le
  corsie -D vive svelano bit-rot census → 3 fix (dettaglio in NEXT_SESSION).

## ⭐ Lezioni (max 3)
- ⭐⭐ Una corsia di gate INERTE è peggio di nessuna: certifica silenzio. Il
  primo run vivo ha trovato in 10′ il bit-rot che 5 concili volevano prevenire.
- ⭐⭐ Il byte-id vs oracle su fixture CON divergenze a catalogo si pre-registra
  col filtro per NOME (il cmp secco era il criterio sbagliato, emendato).
- ⭐ La staleness di un lock si giudica sul PID vivo, mai a orologio.
