# WP_SESSION_112 — S-112: leva H-A2 fast-path Long RMW PROMOSSA — arith 9,3 → 5,5

**In una frase**: una singola operazione (lo shift di interi) cadeva fuori
dalla corsia veloce e trascinava la catena lenta 50 milioni di volte per run;
con tre ritocchi minimi e collaudati il divario dall'originale PHP sul
giudice peggiore è sceso del 41%, senza peggiorare nessun altro giudice
(compresi i tre fuori-concorso).

**SCOREBOARD** (micro R=5 sul pin s112 f71abd2a, N emessi; frecce vs S-109):
**arith 5,5 ↓↓ (da 9,3) · prop 7,6 ↓ (7,9) · calls 5,2 = (5,1) · str 5,3 = ·
arr 4,2 ~↑ (3,9, tra-sere; A/B fermo) · re 3,4 = (3,5)** · held-out poly 6,4 ↓
(6,7) · err 2,5 = · wploop 5,6 =. WP NON rimisurato (rif 1,867/1,869 S-110);
**coppia DOVUTA in S-113** (icache +3.372 B). **Leve perf spedite: 1 (H-A2).**
**Data**: 2026-08-08 (01:0x–02:1x) · Fable 5 · commit d8d7c68→5502f67 pushati.

## Esiti secchi
1·istruttoria PRIMA del criterio (s112-istruttoria.md): (b) RMW-su-dim
DECLASSATA (finestra legale max [ConcatN,FetchDim], attesa ~3,5 < pavimento 4;
FetchDim/StringifySlot sospendono); (c) istruita fino alla forma: Shr fuori
da binary_fast + code RMW senza guardia inline → 2·criterio PRE committato
PRIMA (d8d7c68, con soglia held-out PRE az.5) → 3·leva 3 siti (A2a Shl/Shr
VERBATIM, y<0 al generico; A2b/A2c guardia inline su code BinarySCSCDst/
BinarySTDst = hoisting della prima riga di binary_value_ab) → 4·admission:
emissione INVARIATA (dump ON {main}×6 + OFF×6 al byte), run_loop
287.944→291.316 B dichiarati, batteria 1742/0 rc=0 (relink gemella
b70e049a dichiarato) → 5·A/B R=5 ABAB:
**arith Δ=+32,80 ns/iter 5/5 (soglia 4)**; str/re reggono; calls +1,50 in
MIGLIORAMENTO su sentiero non toccato = guardia bilaterale emendata
DICHIARANDO a solo-regressione (banda-layout N=2); prop +4,33 5/5 →
6·held-out 3/3 su soglia PRE (poly 10,22→9,59 MIGLIORA) → 7·pin s112 da
pin-phpr.sh; corpus **1415×2 IDENTICO** rc=0; fixture rc=0; micro R=5.

## ⭐ Lezioni (max 3)
- ⭐⭐ **L'istruttoria dai dump prima del criterio ribalta la scelta**: la
  candidata in agenda era sotto-pavimento; il collo vero era UNA op.
- ⭐⭐ **Una guardia bilaterale |Δ| si sfonda anche MIGLIORANDO**: le guardie
  anti-tassa si scrivono a solo-regressione; l'emendamento in corsa si
  dichiara nel verdetto, non si tace.
- ⭐ **Prima di nuove superop, censire i BinOp assenti da binary_fast sui
  corpi caldi**: il −41% era un hoisting di poche righe, non una fusione.
