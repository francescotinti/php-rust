# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-168 = FETTA 0-bis MOCK (sola misura, sanzionata ⚖️):
Σ mock nominati 9,92 < 10 ⇒ KILL regola 4 SCATTA A FILO (margine 0,08 con
rumore 0,2-0,6: dichiarato) ⇒ DELIBERA R4 DOVUTA AL CONCILIO/UTENTE; chiusura
52,7% ⇒ nessun codice di leva · REPERTO E2: controllo-loop 2 op = 14,7 ns/iter
vs 3,56 oracle (7,3 vs 1,8 ns/op) — il costo PER-OP è 4× e vale il 31% del dq;
statement 32,0 vs 5,1 con ~22 ns residui DENTRO il handler (per esclusione) ·
sanature S-167 chiuse: stride REFUTATO (layout verificato + controllo
positivo), xctrace c1=backend/c3=discarded/c0=useful (etichetta S-167
corretta; refutazione mispredict regge su c3), branchmut S-167 PREDICIBILE** ·
leve: 0 (sanzionato) · incidenti: 2 (catena sotto timeout tool → daemonizer;
output di run in un commit → untracked) · revisione (semantica): REGGE CON
RILIEVI — kill «A FILO, non deliberabile da solo» (Σ grezza 12,9 >10 a
registro), portata del kill = UN handler (E2: 11,1 ns nei 2 op di controllo
mai toccati), m3 «confuso», Sweep ≈3 nel residuo (≈19), mispredict «NON
FIRMATO» (c3 circolare) · QUESITI UTENTE: (a) delibera R4 (concilio?);
(b) T2/A2; (c) census server (18° slitt.); (d) ratifiche §3.

## Scoreboard (pin INVARIATO s166 phpr 092dcff431bef876 + server caa4e4b2638686a9)
**arith 5,4 · prop 5,5 · calls 4,8 · str 4,2 · arr 3,2 · re 2,5** · giudici
propri mc2 ~155 / mc3 181 · arith-dq 46,6-47,1 vs oracle 8,64 (gap ~38) ·
E2 loop nudo 14,7 vs 3,56 · WP 1,746-1,749 · ORM [7,023;7,053] (RIF) · corpus
1412×2 · batteria 1748 · denti: run.rs 6917 · mod.rs 25909 · host.rs 7726.

## §S-169 — ordine (⚖️ dopo il kill: DELIBERA prima di ogni codice)
1. **DELIBERA R4** (regola 4 ⚖️ S-167): il kill è scattato meccanicamente a
   filo. Opzioni da mettere all'utente (o al concilio se lo convoca): (i) R4 =
   ridefinizione della campagna sul reperto E2 — costo PER-OP (dispatch+handler
   banale 7,3 vs 1,8 ns/op: CmpJmpSC/IncDecSlotJmp/Sweep) come bersaglio
   nominato, con decomposizione del singolo op (mock «handler vuoto») PRIMA di
   qualunque leva; (ii) leggere il kill «a filo» come non-decisivo e rieseguire
   i mock con soglia/banda del kill pre-registrate (R alto, tick 0,04) — solo se
   l'utente lo ratifica; (iii) R2/R3 restano chiuse (veti confermati).
   Nessun codice F1/F2 senza delibera. Az.rev.1: la delibera riceve ENTRAMBE
   le cifre (9,92 nominata / 12,9 grezza) — il conto a pavimento 4 azzera
   componenti a direzione firmata: soglia = rumore SEPARATA dal pavimento.
2. **Az.rev. S-168 PRIMA della delibera** (misura, ½ sessione): mock sui DUE
   handler di controllo (CmpJmpSC/IncDecSlotJmp: «handler vuoto») con giudice
   E2 · ri-misura m4b e m3-puro (solo hoist, forma invariata) a R esteso ·
   m2 senza guardia (tupla via flag di build) · rbranchmut (LCG bit 30) +
   mutante proprio di c0 su xctrace PRIMA di ogni lettura di c3.
3. Sanature/quesiti (a)-(d) se l'utente ratifica; residuo ≈19 ns dentro
   BinarySCSCDst (guardie Undef/Ref, read_slot clone, store+gc_note) SOLO se
   la delibera lo chiede.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
delibera R4 · mock handler-vuoto per-op · rbranchmut · residuo-handler (guardie ·
read_slot clone · store+gc_note) · Sweep/iter ~3 ns (sotto pavimento) · F1
predecode (SOSPESA dal kill) · F2 BinOp-3ind (SOSPESA) · tetto-fuso (e) ·
autoload statiche · sonda strmap · ri-fondazione banda sentinella ORM · gamba
server census (18°) · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 ·
slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC ·
evalcls 316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano)
**S-168: un mock senza dump dell'op-sequence del loop PRIMA della misura (m4
nullo) · lanciare una catena di misura sotto il timeout del tool (daemonizer,
sempre) · output di run (`ab-out/`) nel repo · soglia di controllo positivo che
scala col rumore dell'effetto · kill senza banda pre-registrata · patch mock
promosse (m4b NON preserva la semantica GC/dtor).**
S-167: chiusura da quote-che-sommano-a-1 · strumento nuovo senza mutante · copia
d'albero senza pulizia · commit concatenato a collaudo. S-166/165: copione senza
grep dei nomi · RIF fuori criterio · guardie senza banda-layout · fast path
inline in run_loop. Trasversali: NaN-boxing/fn-table/arena (⚖️) · BOLT/PGO ·
pin senza collaudo · rc da pipe · promozione sotto banda · denominatori a memoria.
**Riscritto** 2026-09-01 sera (chiusura S-168; storia in `sessions/` · `gaps/`).
Pre-flight S-169: pin phpr **s166 092dcff4**31bef876 + server **caa4e4b2**
638686a9 (SOLO via pin-*.sh; stash bracci `phpr-s168-m0..m4b` NON pin) · Data
≥10G (target dedicato rimosso; canonica potata) · MySQL wp8 con l'elenco ·
uploads sotto guardia · corpus 1412 · lock misura da CREARE · **NESSUNA coppia
dovuta** · lettura: REGOLE.md → QUI → **wp168-harness/s168-mock-verdetto.out +
revisione.md** → wp167-harness/COUNCIL_WP167_REVIEWS.md (⚖️ regola 4) →
sessions/WP_SESSION_168.md → PERF_MAP.md.
