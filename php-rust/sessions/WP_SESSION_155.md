# WP_SESSION_155 — coppia t6 + ORM a CAVALLO di 7× (gamba migliore 6,97); istruttoria CE1 chiusa; gdc NON pagante; pin INVARIATO s154
**In una frase**: WordPress fermo (atteso), Doctrine ancora più giù (gamba
migliore 6,97×), spiegato al byte il residuo del controllo-esistenza-classi
(costo fisso di chiamata, non la cura), e uccisa coi numeri la leva in coda.
**SCOREBOARD** (pin INVARIATO s154 bddc050320a6af4c + server b3cf348f69739edc):
arith 5,5 → · prop 5,6 → · calls 4,8 → · str 4,3 → · arr 3,3 → · re 2,5 →
(micro/corpus/batteria non rieseguiti: pin invariato, valori promo s154) ·
WP t6 mediana 1,771 COMPATIBILE · media 2,456–2,510 · **ORM 6,972–7,053 ↓ (a
CAVALLO di 7, rett. rev.)** · dbal 7,385–7,422 ↓ · **leve spedite: 0 (tentate
0) — ANOMALIA dichiarata** (misure DOVUTE + istruttorie; gdc refutata prima
del criterio) · incidenti 19 (=).

## Esiti secchi
1·Coppia t6 @ s154 rc=0: mediana 1,771 ∈ [1,738;1,799], 6/6 pulite, banda_ON
  0,022 record, parità 6/6, peak 6/6 BASSE, deriva assente; attesa CE1
  «piccola/nulla» RISPETTATA.
2·ORM @ s154 rc=0: net 6,972–7,053 (Δ +0,41/+0,50 GIÙ fuori rumore) · dbal
  7,385–7,422; parità 16== e dbal 10 stabile; summary dbal phpr vuota
  (estrazione rotta, az.rev.#4→S-156).
3·Sonda ce-count rc=0: k_post=1 (attesa 0 FUORI) → istruttoria CHIUSA al
  sorgente: 1×32 B = args-Vec di pop_keys attribuito al nome (scope s149
  prima di pop_keys); controlli fe-count e ce-true (autoload=true, az.rev.#2)
  k=1 b=16,0 ESATTI ⇒ **CE1 0-alloc su ENTRAMBI i rami** (>64 B esclusi;
  canale == H-D S-103).
4·OLTRE-attesa ORM: micro-fondata 0,05–0,11 s ≪ Δ; meccanismo NOMINATO:
  resolve_class_autoload = funnel 11 siti — census al probe s155 (3e6b5008 ×2).
5·gdc-count rc=0: per_classe=3,0 fisso=1 ESATTI + terzo punto C=2352
  k=7057==predetto (az.rev.#3: linearità a scala ORM) ⇒ k_ORM=7180, ~636
  chiamate, 0,031 s ≪ 0,293 ⇒ **fetta NON pagante, declassata**.
6·Az.rev. S-154 recepite (§3 emendata; pin-phpr --braccio; gate a rischio
  pre-dichiarati) · T2/A2: raccomandazione SOSPENDERE, ratifica utente · CI:
  corpus-FAIL d'ambiente su 3 test backtrace (nei gate di record passano).

## ⭐ Lezioni (max 3)
- ⭐⭐ Le attese-conteggio su census per-NOME devono includere il PLUMBING
  dell'attribuzione (scope prima di pop_keys): k=0 mal posto, cura sana.
- ⭐⭐ Un fit da 4 run UCCIDE una leva in coda a costo ~zero: gdc 3
  alloc/classe ma ~636 chiamate ⇒ 0,03 s (terzo punto == predetto).
- ⭐ Una cura su un FUNNEL batte l'attesa calcolata sul solo nome motivante:
  attese per-nome ≠ attese per-funnel.
