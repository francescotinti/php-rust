# WP_SESSION_162 — rimisura AL2 a registro; coppia t12 6/6 + banda_ON fondata + ORM rivalidata; leva L-AM2 PROMOSSA → pin NUOVO s162
**In una frase**: dato il prezzo proprio alla leva di ieri (7 ns a chiamata
mancata), rifatta la coppia di benchmark in una notte finalmente quieta (tutte
le gambe buone, ORM di nuovo misurabile), e spedita una leva grossa: `array_map`
con callback a stringa non alloca più nome e argomenti a ogni elemento (−39%).
**SCOREBOARD** (pin NUOVO s162 phpr 20c63af44bfd077a + server f6d4a63b23b963da):
arith 5,5 ↑tick · prop 5,5 ↓tick · calls 4,8 ↓ · str 4,2 ↓ · arr 3,1 ↓ ·
re 2,5 ↓ (rif s161: 5,3/5,6/4,9/4,3/3,3/2,6; tick arith da sorvegliare, guardia
A/B verde D=-0,2) · WP t12 mediana 1,767 COMPATIBILE 6/6 pulite · **banda_ON
FONDATA=0,018** · media 2,426-2,445 · ORM [7,035;7,086] VALIDO · dbal
[7,391;7,440] con riserva · corpus 1412×2 · batteria 1748/0/2 (cap 25847/7726) ·
**leve spedite: 1 (L-AM2 PROMOSSA) + rimisura AL2** · incidenti 19+2 (0 nuovi).

## Esiti secchi
1·Rimisura AL2 stash FERMI rc=0: **autoload 7,0±3,0** (5/5, coerente R=5) —
  rev. S-161 #1/#2 ASSOLTE, TABELLA PER-SITO a cifre proprie in PERF_MAP.
2·Coppia t12 rc=0 COMPATIBILE 1,767, 6/6 pulite (quiete dichiarata uptime/top);
  banda_ON fondata (tentativo t11 CHIUSO). ORM rc=0 in finestra quieta:
  sentinella contaminazione NEGATIVA, rimisura s161 ASSOLTA; companion assoluto
  MIGLIORA [+0,45;+0,73]s (rev. #3, tensione s161 risolta); attesa-AF1 NON
  risolta per AMPIEZZA (a cavallo di RES), coerente col tetto ~0.
3·Leva L-AM2 (string-callable UTENTE k=1): smoke +67,5 FUORI banda [4;19]
  SOPRA ⇒ census: **Δ=2 alloc/elemento ESATTE** (args-Vec + to_vec del nome;
  attesa p.5 contraddiceva p.1 dello stesso criterio: errore di pre-registro
  DICHIARATO) ⇒ R=5 **D=+65,0** (167→102, rumore 1/1), 18/18 guardie, disasm
  Δ=0 ⇒ PROMOZIONE rc=0 SENZA rettifiche; conferma post-pin +68,0 CIFRA PIENA
  (rumore 1,0 ≪ attesa/2). Quinto sito: **strmap 65,0±1,0**. §3.26 a catalogo
  (2 divergenze PRE-esistenti, gate invarianza fx-sm-div).
4·Gemello A: 2 tentativi in target DEDICATO scartati prima d'ogni uso (PATH
  cotto nel binario); canonico = pin AL BYTE. AppleDouble bonificati (13.073).

## ⭐ Lezioni (max 3)
- ⭐⭐ la riproducibilità del build è PATH-SENSIBILE: il gemello si costruisce
  nel target CANONICO del pin (un target dedicato dà un hash stabile ma
  inconfrontabile — 2 build sprecate).
- ⭐⭐ un'attesa census si pre-registra CONTANDO le alloc del bundle p.1, non
  per analogia: string-callable = 2 alloc/elemento (vec + nome), closure = 1
  — il criterio si contraddiceva da solo e l'arbitrato l'ha smascherato.
- ⭐ la sentinella «oracle vs SUO rif» morde in entrambi i versi: a s161 ha
  annullato la cifra (oracle gonfio), a s162 l'ha VALIDATA (-1,2% = quiete).
