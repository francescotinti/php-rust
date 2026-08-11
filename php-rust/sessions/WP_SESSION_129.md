# WP_SESSION_129 — modello del tempo chiuso + F4 tentata (avversa per criterio) + riferimento pulito

**In una frase**: abbiamo finalmente MISURATO dove finiscono i ~300 ns che ogni
scrittura su proprietà-array costa in più dell'oracle (metà in ricerche per nome,
un quarto in controlli-hook mai necessari), provato una leva che taglia quel quarto
(più veloce in 7 run su 7 ma bocciata dalla soglia per un outlier), e ripulito il
riferimento WordPress dalle gambe di misura contaminate: 1,758–1,805.

**SCOREBOARD** (pin **s127b ccb63dca INVARIATO** + server bc95ba71; micro gate S-129):
**arith 5,3 = · prop 5,6 = · calls 5,0 (sciolta, era 4,9*) · str 4,2 = · arr 3,2 = ·
re 2,5 ≈** · **WP full rif PULITO = 1,758–1,805** (era 1,758–1,909; gate ictx/s) ·
media CANONICA 2,447–2,463 · **leve perf spedite: 0 — TENTATA 1 (F4, A/B completo:
smoke PROMOSSA +71,7, R=5 AVVERSA per criterio, direzione firmata 7/7)**;
incidenti: 1 apparato (dump tprobes assente su categoria a zero statement). 2026-08-11.

## Esiti secchi
1·**MODELLO DEL TEMPO seg.3 CHIUSO** (criterio 2c8fb2f PRIMA; census 22/22 → pin
  R=5 → per-passo chiusura 95-96% → disasm → lettura): Δstatement 300–340 ns quasi
  INVARIANTE per forma; torta: **E−E2 dispatch+prop_step 155 ns (52%; residuo per
  sottrazione — «resolve-per-NOME» è indizio, sonda E1a in S-130, rev.)** · preludio
  byref/lazy/indirect 73 (25%, sondato+corroborato) · walk 48 (16%); 2 alloc residui =
  n.clone() del nome (mod.rs:14333, 12876). Overwrite(+4)/2°insert(+3) SPIEGATI.
2·**F4 prelude-gate**: criterio → codice f4143a6 → census 11/11 → smoke +71,7
  PROMOSSA → R=5 D=+66,7 SOTTO soglia 70 (rumore da UNA run 4,00 vs 3,79–3,83) +
  objmap −6,7 su banda default (gate mai eseguito lì: layout). Verdetto avverso
  committato PRIMA; revert riproduce **ccb63dca AL BYTE**; server ripristinato
  bc95ba71. Rientro S-130: criterio emendato (rumore robusto + bande fondate).
3·Gate micro R=5: **calls 5,0 SCIOLTA** (phpr netto IDENTICO 2,14; muove solo
  l'oracle 0,43–0,44); leva sbloccata (REGOLE §4).
4·Az.rev. S-128 TUTTE: #1 gate ictx/s nella ricetta · #2 leg3-off → riferimento
  PULITO (le 2 segnalate sono entrambe prime-di-sequenza ⇒ apertura warm-up leg)
  · #3 media unica canonica etichettata · #4 riconciliazione F1 a verbale.
## ⭐ Lezioni (max 3)
- ⭐⭐ Un modello del tempo per-passo (sonde con conteggi deterministici + chiusura
  della somma) PREDICE la leva: UB 73 → misura +71,7/+66,7. È l'inverso di F2.
- ⭐⭐ Una soglia col rumore = range PIENO muore su un outlier singolo; una guardia
  su categoria senza banda storica morde per layout, non per meccanismo. Le bande
  si fondano PRIMA del prossimo A/B — e il verdetto avverso resta avverso.
- ⭐ Una sonda atexit registrata al primo evento non scatta MAI sulla categoria a
  zero eventi: il baseline della strumentazione va collaudato come caso proprio.
