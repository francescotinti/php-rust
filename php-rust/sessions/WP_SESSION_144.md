# WP_SESSION_144 — B progettata su carta + istruttoria CHIUSA: il maggiorante è 2,38% (B confermata PER MISURA) e l'oracle dice che il bersaglio vivo è il churn Rc, non il memcpy

**In una frase**: abbiamo scritto il progetto della via B con le regole firmate
prima dei dati; poi le due misure ordinate dai revisori hanno detto che gli
oggetti restano sotto il 2,4% delle allocazioni (la via B regge senza appello)
e che il PHP originale paga quasi lo stesso memcpy di phpr — quindi B deve
attaccare il ciclo clone/drop dei valori condivisi, non lo spostamento di byte.

**SCOREBOARD** (pin s142 bba8a734+eeb284b6 INVARIATO, ripristinato da stash
post-probe; micro non rimisurate = per costruzione): **arith 5,5 → · prop 5,6 →
· calls 4,7 → · str 4,2 → · arr 3,2 → · re 2,6 →** (hintcall 7,3 →) · WP rif
1,765–1,788 (fermo) · **leve perf spedite: 0 — DICHIARATO: istruttoria
vincolante deliberata dal concilio S-143 (seconda sessione consecutiva: la
prossima leva è DOVUTA)** · incidenti 15 (nessun nuovo).

## Esiti secchi
1·**p.1 progettazione B** (`s144-progettazione-B.md` + `s144-criterio-B.md`,
  committati PRIMA dei dati): reperto — la NICHE è GIÀ attiva
  (`Option<Zval>`==16, assert array.rs) ⇒ B = solo clone/drop: fette B1
  «uniform-rc», B2 «root-at-decrement», B3 condizionale; KS armonizzati.
2·**p.2a tranche-2** (probe 4ab4aa9a, ×2, r1==r2 ESATTO, parità rc=0):
  rczval 1,01% (4,74M nascite) · vecargs 2,79% · objsynth 48 ·
  **quota_obj_max_loose 2,38% < 25% ⇒ B sola/B-poi-A CONFERMATA PER MISURA**
  (il maggiorante ipotetico ~8–15% del revisore S-143 è REFUTATO: le scale
  propget/recv_clone erano CLONI, non nascite). Funnel `zcell` tipato:
  enumerazione siti chiusa dal COMPILATORE. Golden-test parser PASS 2/2.
3·**p.2d profilo oracle** (2 rep, sentinelle CLEAN, emenda v2 dichiarata:
  69% dei campioni = thread workqueue PARCHEGGIATI, esclusi): memops 7,8–8,1%
  vs 12,6% ⇒ rapporto 62–66% ≥50% ⇒ **FUORI BUDGET** (severa). churn_zval:
  il revisore ha morso la prima dicitura «robusto» (giudice top-of-stack
  quasi-vacuo per inlining) ⇒ **az.2 eseguita in chiusura col SUO criterio:
  whole-stack 0,26/0,24pp ≪ 5,15pp ⇒ IN BUDGET CHIUSO PER MISURA** (caveat
  inlining-totale dichiarato; S-129 a corredo). Gate Stogov R4 assolto:
  **bersaglio vivo di B = churn Rc + gc-nota, NON il memcpy**.
4·Az.rev. S-143: #1 quota_obj_max FATTA · #2 other 42,1% attribuito, residuo
  57,9% dichiarato fuori-budget + tranche-3 nominata (growth-alloc hashbrown)
  · #3 emenda verdetto FATTA · #4 golden-test FATTO · #5 objsynth=48 FATTO.
  Voce c (sonda prezzi) NON aperta → S-145 p.1. Az.rev. S-142 #2 residua.

## ⭐ Lezioni (max 3)
- ⭐⭐ Un maggiorante «sotto ipotesi» si chiude solo misurando: 8–15% atteso,
  2,38% misurato — le scale di CLONI non predicono le NASCITE (Rc::clone
  non alloca).
- ⭐⭐ `sample` conta anche i thread PARCHEGGIATI: su binario multi-thread il
  denominatore va emendato ai campioni ATTIVI; se due letture divergono su
  un gate, si applica la SEVERA (errore asimmetrico verso il non-promuovere).
- ⭐ Un funnel TIPATO (parametro `Zval`) vincola il payload NEI SITI CONVERTITI
  (rett. revisore az.4: la chiusura resta la ricerca esaustiva; il dente CI
  che vieta il pattern fuori dal funnel è a S-145).
