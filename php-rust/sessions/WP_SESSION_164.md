# WP_SESSION_164 — coppia t14+ORM assolta al pin s163; incidente census CURATO; indagine arith CHIUSA (era l'arrotondamento); leva L-AL3 caduta a verdetto
**In una frase**: i benchmark sul motore promosso ieri sono tutti buoni e
stabili, il "peggioramento" aritmetico che inseguivamo da due sessioni si è
rivelato un artefatto di arrotondamento del cronometro (il motore è fermo al
centesimo su tre versioni), e la leva candidata è stata provata fino in fondo
ed è caduta con meccanismo nominato: riciclare una scatoletta di memoria non
paga, perché l'allocatore mimalloc fa già quel lavoro gratis.
**SCOREBOARD** (pin s163 fea4a2d040a0d8d0 + server 8d76d6f129bfd4af INVARIATO):
arith 5,5 = (VERO de-quantizzato ~5,43 su s161/s162/s163: nota CHIUSA) · prop
5,5 = · calls 4,7 = · str 4,2 = · arr 3,2 = · re 2,6 = · WP t14 mediana
1,761 COMPATIBILE 6/6 pulite (**banda_ON 0,011 = record**) · media
2,341-2,450 · ORM [7,066;7,111] VALIDO (attesa-AU1 COMPATIBILE tetto ~0) ·
dbal [7,459;7,491] riserva ictx RICORRENTE (3ª coppia) · corpus 1412×2
invariato (nessun edit residuo) · **leve spedite: 0 — 1 TENTATA con A/B
COMPLETO e CADUTA A VERDETTO (ritmo rispettato)** · incidenti: +1 (S-164 #1,
curato in sessione).

## Esiti secchi
1·Coppia t14 rc=0 (predicato anti-flare PRE-registrato: 1° giro pulito) + ORM
  rc=0 al 2° giro (rc=8 leg1: flare innescato dalla MIA bonifica in finestra,
  dichiarato); dbal parità stabile, 1 gamba ictx-oracle per l'istruttoria.
2·Census AU1 RIESEGUITO dallo script, rc=0 Δ=600000 ESATTO ⇒ incidente S-163
  CURATO. Incidente S-164 #1: sed senza mappa wp163→wp164 ⇒ scritto nel
  harness chiuso (copia-gate verificato PARZIALE); bonificato, originali
  s163 salvi in git (a4954127).
3·Indagine arith CHIUSA rc=0: fase 1 forense (phpr 2,35×3 FERMO; outlier =
  oracle 0,44 di s161) + fase 2 de-quantizzata N=250M su stash byte-verificati
  (max−min 0,080; rapporti VERI 5,426/5,454/5,417). Companion: creep +0,6
  ns/iter cumulato da s158 sotto soglia, a verbale.
4·L-AL3 CADUTA (verbale s164-al3-STOP.md): smoke D=+0,0; census Δ=199998
  (rc=5 FUORI ATTESA di 1; spiegazione post-hoc dichiarata: buffer del Vec
  del pool) ⇒ p.3b non pagante ⇒ revert AL BYTE (fea4a2d0). unreachable! ×2
  tornano in aperture. Gemello: 1° build SENZA ricetta morso dal gate
  (1492be21), con ricetta al byte; php-server NON riproduce (661b490c) ⇒
  canonico ripristinato dallo stash pinnato.

## ⭐ Lezioni (max 3)
- ⭐⭐ un'alloc mimalloc su fast path vale ~0 ns visibili: il coeff per-sito
  7,0±3,0 impacchettava alloc+dispatch delle leve promosse — la classe
  «Box/Vec-pooling puro senza cambio di dispatch» è RIDIMENSIONATA.
- ⭐⭐ un giudice a 2 decimali su un netto di 0,43 ha quanto ~0,13 di
  rapporto: due sessioni di "indagine dovuta" per un artefatto — la nota
  su un tick si apre SOLO dopo il de-quantizzo (REGOLE §3 già lo diceva).
- ⭐ la RICETTA è parte del gemello (build nudo ⇒ hash diverso, gate morso);
  e il copia-gate si verifica sul diff INTERO + grep dei path di harness.
