# WP_SESSION_117 — pipeline A′ SPEDITA (lto=fat+cgu=1, determinismo al byte) · L-A PROMOSSA al 4° giudizio (la «tassa calls» era layout) · pin s117

**In una frase**: cambiata la fabbrica del programma (compilazione ottimizzata e
riproducibile bit-per-bit), il righello è diventato abbastanza fine da assolvere
il ritocco sulle proprietà bocciato tre volte — la sua presunta tassa su un
altro giudice era un artefatto d'impaginazione della vecchia build — e il
rapporto sulle proprietà scende da 7,6× a 5,9×.

**SCOREBOARD** (pin s117 **1656580e**, micro R=5 sul pin; frecce vs pin s112):
**arith 5,4 ↓ · prop 5,9 ↓↓ (7,6) · calls 4,8 ↓ · str 5,3 = · arr 3,8 ↓ ·
re 3,4 =** · held-out 6,2·2,5·5,4 · WP full/media NON rimisurati (rif. 1,867 /
2,632). **Leve perf spedite: 2 (A′ pipeline + L-A)** — sblocca 4 sessioni a
zero. 2026-08-08 · Fable 5 · 9582271→23a9087.

## Esiti secchi
1·A′ stadio-1 (criterio PRE 9582271 PRIMA di Cargo.toml; sorgente HEAD byte-id
al pin): determinismo ×2 prima prova ROTTA per 4 byte `__cstring` = `__TIME__`
di mimalloc (diagnosi per-sezione; `.text` GIÀ identico) → ricetta emendata
`SOURCE_DATE_EPOCH=0` DICHIARATA, criterio rieseguito da capo: hash-FILE
IDENTICO ×2. Gate PIENI rc=0: batteria 1742/0/2 inventario per NOME identico,
corpus 1415 ×2 modi == congelato s109, off↔on zero diff. Uplift mediano
+2,17% ⇒ KS-A (congiunzione, sciolta nel criterio) NON scatta.
2·Ri-banda N=2 (zavorre s114/s115-2 RICOSTRUITE sotto A′, admission anti-forgia
LTO: dump ON/OFF + taglia run_loop): banda-v2 = 0,80·3,33·**0,50**·7,50·6,67·
**0,00**; globale 7,50 > 5 ⇒ claim «ripara-metro» NON passa (dichiarato, A′
vale come leva velocità) MA calls 5,50→0,50 e re 10→0. Held-out N=2 ≤0,08 s.
3·Rigiudizio L-A sotto A′ (cherry-pick -n, criterio con quanto-guard e floor
mediano di 3): prop **+33,00** 5/5 (soglia 4,00) · guardia calls **−0,50 =
1 quanto, DENTRO banda** ⇒ la tassa −6,5 di S-114..116 era di QUELLA build
(ipotesi layout FIRMATA, N layout=2) · held-out 3/3 ok (spread_pin pubblicato)
⇒ **PROMOSSA**. §6 pieno: re-hash morde il churn del test-relink → neutralizzato
via determinismo (build ricetta ⇒ H1 al byte); pin s117 via pin-phpr.sh.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Il metro si ripara per categoria, non per proclama**: la stessa leva bocciata 3 volte è promossa quando banda(calls) scende 5,50→0,50 — una «tassa sistematica» replicata su un solo layout era la firma della build, non del codice.
- ⭐⭐ **Il determinismo è uno strumento, non solo un gate**: ha permesso di curare la ricetta (SOURCE_DATE_EPOCH) senza indebolire il criterio hash-file, e di neutralizzare il churn del test-relink provando il ritorno a H1 al byte.
- ⭐ **Sotto fat-LTO una zavorra può svanire in silenzio**: l'admission delle nulle (dump ON/OFF al byte + taglia run_loop) è il dente che distingue banda misurata da forgia fallita.
