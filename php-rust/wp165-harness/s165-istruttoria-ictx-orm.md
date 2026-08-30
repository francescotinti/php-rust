# S-165 p.2 — istruttoria dbal ictx-oracle (MATURA, 3 coppie) + banda sentinella oracle ORM

## A. dbal ictx-oracle: firma NOMINATA = DENOMINATORE, non contaminazione
Dati (dai verdetti s162/s163/s164, ictx ASSOLUTI vs rate):
- s162: abs oracle 318/96 · phpr 70/62 → rate 217,8/69,1 vs 8,1/7,1
- s163: abs oracle 110/66 · phpr 58/60 → rate 80,9/48,2 vs 6,7/6,9
- s164: abs oracle 121/139 · phpr 163/192 → rate 87,1/101,5 vs 18,7/22,2
1. Gli ictx ASSOLUTI dei due motori sono dello STESSO ordine (oracle 66-318,
   phpr 58-192); la gamba dbal oracle dura ~1,37 s contro ~8,7 s phpr ⇒ a
   parità di switch assoluti il rate del braccio corto esce 6-13×. Il gate
   S-127 (>1,5× della mediana sui 4 rate) morde STRUTTURALMENTE il braccio
   veloce: 6/6 gambe oracle segnalate in 3 coppie, 0/6 phpr, senza alcun
   effetto sul ratio_net (dbal stabile 7,39-7,57).
2. Componente reale residua: il transitorio di 1ª gamba (oracle1 3,3× oracle2
   in s162, 1,7× in s163) SPARISCE col rodaggio non giudicante montato in
   s164 (121 vs 139): coerente con page-in/opcache della prima corsa.
3. **EMENDA PROPOSTA** (per il criterio della PROSSIMA coppia, REGOLE §3: si
   emenda rieseguendo il criterio emendato — nessuna coppia dovuta in S-165):
   la segnalazione ictx si giudica PER MOTORE (>1,5× della mediana del
   PROPRIO motore tra le gambe della stessa coppia, come già fa il pair WP)
   e il rodaggio resta montato. Gli assoluti restano a verbale.
   VERDETTO ISTRUTTORIA: riserva ictx-oracle dbal = ARTEFATTO del gate;
   nessuna contaminazione dimostrata nelle 3 coppie (ratio_net insensibile).

## B. Banda NUMERICA sentinella oracle ORM (az.rev. S-164 #3) — PRE-registrata
Osservato (3 coppie quiete, oracle_net/gamba): s162 4,87/4,90 · s163
4,85/4,84 · s164 4,86/4,85 → 6 valori in [4,84;4,90], mediana 4,855.
ORA_REF=4,885 sta SOPRA 5/6 osservazioni: un vero drift −0,03 produce da
solo Δ_norm ≈ −0,21 (rilievo #4 S-164).
1. **BANDA SENTINELLA** (vincolante dalla prossima coppia): oracle_net/gamba
   in **[4,83;4,94]** (min/max osservato ±1 tick 0,01). Gamba fuori banda =
   «drift oracle»: Δ_norm della coppia NON GIUDICANTE, istruttoria dedicata.
2. **ISTRUTTORIA DRIFT (S-165, misura)**: R=5 oracle-only ORM in finestra
   quieta (quiescenza rc=0, lock s165, sentinella LS), stesso lancio della
   gamba oracle della coppia, floor phpunit --version med3. Criterio
   PRE-registrato: se |mediana_R5 − ORA_REF| > 0,03 (= 6 tick del giudice a
   2 decimali su ~4,9 s) ⇒ **RI-FONDAZIONE ORA_REF dichiarata** alla mediana
   R=5 (a verbale, con la storia); altrimenti ORA_REF=4,885 resta e la banda
   p.B1 fa da sentinella. Esito in s165-orm-oracle-r5-verdetto.out.
