# WP_SESSION_172 — LEVA L-SL2 «forma sigillata Long» fetta 2 = PROP PROMOSSA: prop-dq 72,6→52,9 ns/iter (3,78× l'oracle contro 5,19×), micro prop 5,2→3,8; az.rev. S-171 chiuse (mutante abortivo rc=0); pin s172
**In una frase**: l'idea che in S-171 ha dimezzato il ciclo aritmetico (conti sugli interi «nudi»,
senza valori temporanei) è stata applicata alle due istruzioni calde dell'accesso alle proprietà
degli oggetti: il ciclo di prova passa da 5,2 a 3,8 volte l'oracle e il pin nuovo ha superato i
gate; la fixture della sessione scorsa era per metà «vuota» e ora presidia davvero il fast path.
**SCOREBOARD** (pin NUOVO **s172 phpr 5f2dff7d17ebed79 + server 812f7962952da67f**):
**arith 2,7 = · prop 3,8 ↓↓ (5,2) · calls 4,7 ↓ (4,8) · str 4,1 = · arr 3,0 ↓ (3,2) · re 2,5 =**
· coppia t18 DOVUTA: lanciata 00:23 (WP→ORM in daemon; esito in wp172-harness/pair-out e orm-out,
letto in chiusura se concluso, altrimenti istruttoria in apertura S-173) · **leve spedite: 1
(L-SL2, bracci P1 e P2)** · incidenti: **2** (#1 build C == B al byte: `git archive` dà l'mtime del
commit, cargo nel target condiviso non ricompila — adattamento mai collaudato, fermato dalla
guardia hash, emenda touch; #2 push manuale durante la catena in collisione col push di
pin-phpr.sh → rc=1 sul solo push, ripresa dal corpus-gate con pre-condizioni) · dente run.rs
7091→7200 DICHIARATO (+109) dopo un morso in promozione (tentativo 1 rc=101, come S-171) · §3.30
catalogata (default `float $f = 1` resta int, pre-esistente) · coda CI: 12 job trattenuti dal lock.

## Esiti secchi (criterio wp172-harness/s172-criterio.md pre-registrato; verdetti s172-leva-verdetto.out, s172-verdetto.out, s172-promo-verdetto.out)
1·**Az.rev. S-171** (wp171-harness/s172-azrev-verdetto.out): mutante M1 (+1 sul fast path) rompe 9
  righe = tutte le attese + dst==src; M2 (cmp negato) 13 = tutte le attese; fuori dominio INTATTE
  ⇒ rilievo 1 CHIUSO. Corsa 1 rc=1 = difetto di DISEGNO della fixture S-171 (`unset($r)` lascia
  `$s`/`$i` Ref: metà presidio vuoto), curato; fixture estesa (typed-prop-ref → TypeError, shift su
  l negativo) bilaterale; dump ops archiviato (assign-form = BinarySCSC+BinaryDst, non coperta).
2·**Census forme prop** (dump): 8 op/iter; bigramma fuso PropGetSlotRecv+BinaryTCPropSetPop (4 Zval
  temp, funnel, write generico, gc_note) e PropGetSlot+BinarySTDst (push/pop, read_slot, funnel,
  reg_store_slot) = STESSA forma di BinarySCSCDst pre-S-171.
3·**A/B R=5 tre bracci** (A=pin s171, B=P1 260fbc3b, C=P1+P2 e396498b): **prop-dq 72,60→57,53
  (D_B +15,07) →52,87 (D_C +19,73)**, attese [8;20]/[12;30] centrate, rumore ≤0,20; C−B +4,67 =
  direzione (C mai alternato); guardia arith-dq −0,3/−0,5; kill-1 no; disasm bl 5970→5984→5974.
4·**Promozione** rc=0 (tentativo 2 + ripresa): build ricetta 5f2dff7d (≠ candidato solo
  LC_UUID+firma), batteria 1748/0, corpus 1412×2 zero flip, fixture chain 10/10 + 17 gate byte-id
  (fx-sl2 NUOVA con forme P1 in loop = IC calda; fx-sl2-div §3.30 pin==stash s171), micro R=5,
  conferma post-pin prop-dq +20,07 5/5 (73,47→53,40), ORM 16 nomi == baseline, hk 1665 0E/0F.
5·Revisione (lente PROCESSO): wp172-harness/revisione.md — REGGE CON RILIEVI (C−B non alternato:
  solo direzione; fx-sl2 a IC fredda → riscritta PRIMA del tentativo 2; manifest build stantio →
  rigenerato; PIN_REGISTRY dei bracci con HEAD invece del commit sorgente: az.rev. S-173).
## ⭐ Lezioni (max 3)
- ⭐⭐ un presidio di fixture si prova col MUTANTE: la fixture S-171 era per metà vuota (slot Ref
  residui) e fx-sl2 single-shot non entrava nel probe (IC fredda) — il mutante lo misura.
- ⭐⭐ `git archive` dà l'mtime del commit: in un target cargo condiviso il braccio successivo NON si
  ricompila (C==B al byte) — la guardia «hash coincidenti» vale più della fiducia nel copione.
- ⭐ durante una catena che fa push (pin-*.sh) NON si pusha a mano: collisione sul ref, rc=1, ripresa.
