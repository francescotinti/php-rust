# WP_SESSION_121 — L-ST1 REFUTATA dalla regola pre-registrata · grado PIENO server s120 · az. rev. S-120 4/4

**In una frase**: la leva sulle stringhe ha tolto l'allocazione prevista ma il
tempo è PEGGIORATO e la regola scritta in anticipo l'ha fermata senza sconti;
il server ha superato il collaudo completo nei due modi, e la doppia misura
mostra che il guadagno regex di ieri non si vede su WordPress intero.

**SCOREBOARD** (pin s120 885d2c64 INVARIATO — micro non rimisurate; vs S-120):
**arith 5,5 = · prop 5,5 = · calls 4,8 = · str 5,3 = · arr 3,7 = · re 2,8 =**
· rif WP **full = 1,810–1,889** (resta; ABAB pin-to-pin dentro rumore) ·
**leve perf spedite: 0** (1 TENTATA con A/B pieno e REFUTATA: ritmo
rispettato). 2026-08-09 · Fable 5 · ae92801→26c484e.

## Esiti secchi
1·**L-ST1 (str-alloc) REFUTATA sul tempo** (criterio PRE ae92801): scratch
args-Vec di CallBuiltin; istruttoria che SOMMA (5 = 2 concat + 1 args-Vec +
2 substr); census **str 5,00→4,00 ESATTO**; admission 6/6+6/6; smoke str
**−5,00/−7,50 concorde** > banda-zavorra 2,50 ⇒ **early-stop p.7 + revert
p.9, NESSUNA deroga** (prima applicazione della precedenza az. rev. #3).
Candidato 2e1eda8d reperto; release al pin AL BYTE. Zavorra str N=3 = 2,50
(debito saldato); NOMINATA: banda-LAYOUT non coperta dalla zavorra run-to-run.
2·**Grado PIENO server s120**: off E on rc=0 voids=0 (option 413 e restapi
3508 IDENTICI per NOME). Re-pin chiusura riproduce **6b822369 AL BYTE**
@ 26c484e, phpr invariato (lezione S-120 applicata).
3·**Az. rev. S-120 4/4**: fixture preg per NOME (gate PASS; **§3.18**:
`(?J)` dup → false; `__phprbg` utente nascosto) · colonna arr a denominatore
DICHIARATO (D2 per-op-int 6,1M: **+2,02/op**) · precedenza smoke↔banda nel
criterio (applicata) · **ABAB s119↔s120**: L-RE1 su WP **NON ripartibile**
(D −2,16/−14,57 s dentro spread intra-pin 18,73 s; parità failnames 4/4).

## ⭐ Lezioni (max 3)
- ⭐⭐ **La precedenza scritta PRIMA morde anche quando dispiace**: dopo due
  smoke «bugiardi», la regola pre-registrata ha fermato la terza leva SENZA
  full — il criterio decide, non l'entusiasmo.
- ⭐⭐ **Un alloc in meno non è un guadagno**: il bookkeeping dello scratch
  (take/drain/restore) costa più del malloc 32 B di mimalloc — le leve alloc
  si giudicano sul TEMPO; il census firma solo il meccanismo.
- ⭐ **Un micro-guadagno promosso può essere invisibile sul full**: +100 ns/iter
  su re non muove WP sopra il rumore N=2 — lo dice solo l'ABAB tra-pin.
