# WP_SESSION_124 — PhpStr single-alloc PROMOSSA (pin s124) · str 4,2 · arr 3,2 · re 2,5

**In una frase**: ogni stringa PHP costava due allocazioni (contatore + byte
separati) — ora è un blocco unico stile zend_string, e tre categorie su sei
fanno il salto più grande da mesi: stringhe 5,3→4,2, array 3,7→3,2, regex
2,5 (le allocazioni per iterazione di array sono ora a PARITÀ con PHP).

**SCOREBOARD** (pin **s124 c5ba2573** @ fb140d1; micro R=5 sul pin; frecce vs
s120/S-123): **arith 5,5 = · prop 5,6 ↑ (5,5) · calls 4,7 ↓ (4,8) · str 4,2 ↓↓
(5,3) · arr 3,2 ↓↓ (3,7) · re 2,5 ↓↓ (2,8)** · rif WP **full = 1,810–1,889**
(non rimisurato) · **leve perf spedite: 1** (PhpStr single-alloc, strutturale).
2026-08-09/10 · Fable 5 · bcf1d61→(chiusura).

## Esiti secchi
1·**Criterio PRIMA** (bcf1d61): modello del costo SOSTITUTIVO prezzato (refcount
=Rc, PartialEq ptr-fast-path, regrow realloc, Vec-fed=2→2 dichiarato) + metro v3
(BSTOR/ZAVORRA RITIRATI, az. rev. S-123 #3-4).
2·**Patch** (43d7610): ZStr = blocco unico {rc,hash,len,cap}+bytes, funnel
zstr.rs, 7 siti Rc-API + builder ConcatN; 4 test nuovi dichiarati.
3·**Admission census** run1: str −2,00 e arr −2,03 ESATTI, re 0,00 MANCATA ⇒
ridiagnosi: gruppi Caps Vec-fed (il caso 2→2 del modello). Emendazione
(2d2c180): CapMatch.text → ZStr slice-fed nei 5 costruttori. Run2: **6/6
esatte** (str −2 · re −3 · arr −2,03); Δalloc vs oracle: str +1 · arr +0,02 · re +2.
4·**A/B alternato R=5** run1 (b4cec406): str +35,51 (5/5) MA guardia calls
SFONDATA (−3,94, 0/5) ⇒ B2 (fb140d1): drop path outlined #[cold] come
Rc::drop_slow. Run2 (c5ba2573): **str +39,42 · arr +29,38 · re +50,48 (tutti
5/5) · calls +0,81 (flip = canale icache FIRMATO)** ⇒ PROMOZIONE rc=0.
5·**Gate**: batteria 1746/0/2 (inventario IDENTICO + 4 dichiarati) · corpus
nomi==congelato+golden+off↔on zero · fixture verdi · **ORM 3484 3E/13F per
NOME == baseline · http-kernel 1665 0E/0F** · server re-pin **f9526be3**.
±zval-census stesso head (az. rev. S-123 #1) RINVIATO dichiarato a S-125.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Il modello del costo sostitutivo scritto PRIMA trasforma la predizione
  mancata in diagnosi immediata**: re 0,00 era il caso Vec-fed già prezzato —
  5ª leva alloc-removal, prima che NON cade, e l'emendazione era una riga per sito.
- ⭐⭐ **Un tipo che sostituisce Rc deve replicarne anche la STRUTTURA di
  codegen**: Drop inline con morte outlined #[cold]; inlinearla ha sfondato
  calls (−3,94, 0/5) e il flip a +0,81 dopo l'outline firma il canale icache (WP-104).
- ⭐ **L'admission a predizioni di conteggio è un arbitro pre-tempo economico**:
  due giri di emendazione consumati sul census (30 s l'uno), zero A/B sprecati.
