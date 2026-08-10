# WP_SESSION_125 — full rimisurato (1,815–1,896: PhpStr non lo muove) · fusione = MISURA · banda v2-s125 · cbargs refutata

**In una frase**: rimisurata l'intera suite WordPress col motore nuovo — il
salto delle micro (stringhe +39%) NON si vede sul carico reale, fermo a ~1,85×
dall'oracle: il collo vero vive altrove, e ora abbiamo bande e controlli per cercarlo.

**SCOREBOARD** (pin **s124 c5ba2573** INVARIATO; micro non rimisurate, = S-124):
**arith 5,5 = · prop 5,6 = · calls 4,7 = · str 4,2 = · arr 3,2 = · re 2,5 =** ·
rif WP **full = 1,815–1,896 (RIMISURATO, sovrapposto a 1,810–1,889)** · **leve
perf spedite: 0** (1 tentata: cbargs, A/B smoke, REFUTATA). 2026-08-10 · Fable 5 · 92d5e9b→8b23231.

## Esiti secchi
1·**p.1 coppia full/media** (criterio 92d5e9b PRIMA del run): 4 gambe intercalate
rc=0, parità per NOME ×4 (solo wp_is_stream #2). **full 1,815–1,896 SOVRAPPOSTO
a S-120** ⇒ il salto micro str/arr/re non muove il full oltre lo spread; media
2,485–2,518 (lieve ↓); peak 1838–1959 MiB. → REPORT_GAP_125.
2·**p.2a controllo ±zval STESSO head** (az. rev. S-123 #1): **6/6 esatto** —
prop zvclone A(fusa)=3,00 vs B(unfused)=5,00 ⇒ **fusione +2,00 = MISURA**
(inferenza S-123 chiusa); alloc invariate A↔B ovunque.
3·**p.2b banda layout v2-s125 POST-PATCH** (az. rev. S-124 #4): SL = arith 0,44
· prop 0,40 · calls 0,60 · str 0,47 · arr 1,31 · re 2,03 ns/iter — sostituiscono
le v2 S-123 (K=5 latino ciclico, pin ripristinato al byte).
4·**Leva L-HD2 cbargs** (criterio+arbitri 75a0caf PRIMA di ogni run): admission
census **PASS str 3,00→2,00 = PARITÀ alloc oracle** (6/6); smoke R=2 **−5,04/
−5,41 segno opposto 2/2** ⇒ **REFUTAZIONE della FORMA** (init [Zval;4]+drain >
coppia malloc+free; run_loop +1284 B, bl 5906→5924); revert 0755f0e verificato
al byte. Canale args-CallBuiltin APERTO (forma senza init/drain da disegnare).
5·**p.3 istruttoria prop oltre-i-cloni** consegnata: 5 RefCell + 3 probe IC +
3 copie Long per iter; C1 single-borrow nominato (vincolo banda 0,40).
6·Incidenti d'apparato (2, contati): (a) mv dei ratios TRACCIATI wp109 ⇒ PRE
census morso (recupero rieseguito); (b) smoke sul giudice fam≥5 con R=2 ⇒ rc=3
garantito, revert automatico saltato (fatto a mano, al byte). ⚠️ Serena 1.6→1.7
in-sessione: LSP rotto fino al riavvio.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Le micro sature non pagano più sul full**: +39% su str micro, full fermo
  nell'intervallo — il prossimo sforzo va sul PROFILO del carico WP, non su
  un'altra leva micro.
- ⭐⭐ **Census centrato + tempo peggiorato = costo della FORMA** (S-105 redux):
  la refutazione con alloc-parità non chiude il canale, ordina la forma nuova.
- ⭐ **Lo smoke deve avere il SUO lettore**: un giudice che pretende fam≥5
  chiamato con R=2 dà rc=3 garantito e salta i rami di sicurezza a valle.
