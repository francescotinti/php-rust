# s129-ab-f4-lettura.md — F4 prelude-gate: AVVERSA PER CRITERIO, direzione firmata; rientro S-130

**Esito formale**: smoke R=2 PROMOSSA (D=+71,7, soglia 4,0) → conferma R=5 AVVERSA
(rc=5): giudice D=+66,7 SOTTO soglia 70,0 e guardia objmap D=−6,7 oltre la banda
default −4. Verdetto avverso committato PRIMA di questa lettura; revert eseguito
(riproduce il pin **ccb63dca AL BYTE**; php-server ripristinato dallo stash
**bc95ba71** dopo il relink di workspace).

## Diagnosi (dai raw, nessuna nuova misura)
1. **Il segno e la magnitudine della leva sono FIRMATI, non caduti**: B più veloce
   sul giudice in 7/7 coppie tra smoke e R=5 (D +71,7 e +66,7, contro UB modello
   73) e su objchurn in 7/7 (D +80,0 e +86,7). Nessun segno opposto in alcun run.
2. **La soglia 70 è un artefatto della formula**: rumoreB = range PIENO su R=5;
   la serie B del giudice è 3,79/3,83/3,81/3,83/**4,00** — un singolo outlier
   (ultima coppia) gonfia il range a 70 ns. Con le prime 4 coppie il rumore è
   ~13 ns e D=+66,7 sarebbe ~5× sopra. La formula era pre-registrata: vale, e
   si emenda SOLO rieseguendo il criterio emendato (REGOLE §3 / rev. S-112).
3. **objmap −6,7 NON è meccanismo**: il suo loop non contiene statement
   FieldAssign prop-rooted — il gate F4 vi costa un solo `matches!` fallito
   (sub-ns). La serie B ha outlier 0,59 (mediane 0,55 vs 0,53). È il layout su
   una categoria SENZA banda storica (default 4 dichiarato): identico genere del
   morso «banda-layout» già visto (S-103/S-104). Serve la sua banda v2, non un
   verdetto meccanicistico.

## Allocazione (keep-partial-wins: il criterio alloca, non demolisce)
S-130 rientra su F4 con criterio EMENDATO e PRE-REGISTRATO PRIMA di ogni run:
(a) rumore del giudice robusto all'outlier singolo (dichiarare la formula, p.es.
trimmed range drop-1, simmetrica su A e B); (b) banda objmap/objalloc/objchurn
fondata (banda-layout v2 o SL propria misurata su più run del pin), NON default;
(c) stesso giudice, stesse guardie, stesso segno; (d) census F4 già 11/11
PRED-OK e resta valido (nessun cambio di forma). Il codice della forma è nel
commit f4143a6 (revertito da 706f9b8-parent): si ri-applica con `git revert` del
revert o cherry-pick, SENZA riscriverlo.
