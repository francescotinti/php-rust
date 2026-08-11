# s129-riconciliazione-f1.md — az.rev. S-128 #4: collaudo F1 riconciliato coi raw s127

**Reperto** (revisione S-128, lente MISURA): il criterio S-128 p.2 ordinava «−1,00
ESATTO» con riferimento pre-F1 13−9=4, ma il verdetto S-128 misura 12−7=5
(objalloc −2 vs s127-admission) e dichiara [F1-OK] con «atteso 5,00» — criterio e
script sembravano in disaccordo.

**Riconciliazione dai raw** (nessuna nuova misura richiesta):
1. `s127-admission-verdetto.out` (baseline PRE-F1): objalloc 9 · allocni 7 ·
   datains 13 · dropdef 10 · map 1 · churn 14.
2. `s127-admission2-verdetto.out` (candidato F1, stesso epoch): 7 · 5 · 12 · 8 · 1 · 13
   → D = −2/−2/−1/−2/0/−1, dichiarato FUORI PREDIZIONE rispetto al «−1,00» della
   forma, **con diagnosi PRIMA dell'A/B** in `s127-admission2-diagnosi.md`:
   modello **Δ = −2 + w** (thunk saltato = −2: array `[]` + frame-thunk fuori pool;
   w = +1 clone COW solo se il micro scrive nel default) — spiega 6/6.
3. Il collaudo differito S-128 usava come EXP la colonna «leva» di admission2
   (7/5/12/8/1/13): il pin s127b la riproduce ESATTA (12−7=5). Quindi:
   **l'indagine ordinata ERA a verbale** (fatta in S-127, punto 2) e **il testo del
   criterio S-128 p.2 era la parte errata** (frase «−1,00 esatto / rif 13−9=4»
   rimasta dalla predizione pre-diagnosi), non il numero del verdetto.

**Esito**: verdetto S-128 VALIDO; Δins_alloc=5 confermato coerente con la catena
s127-admission → admission2 → diagnosi −2+w → pin s127b. Errore di processo
classificato: frase di criterio stantia (wording), non misura. Nessuna emenda di
cifre. (Il criterio S-129 p.2 nasce già coi riferimenti di admission2 e con
l'emenda PropSet dichiarata pre-run.)
