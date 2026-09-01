# CONCILIO S-167 — «rotta strutturale per il nucleo» (convocato dall'utente 2026-09-01)
Fase 1: 9 verbali INDIPENDENTI in `verbali/` (fonte VINCOLANTE) · Fase 2: 3 team
(`team-sonda`, `team-meccanismo`, `team-governo`) · Questa sintesi recepisce; i
dissensi restano a registro col nome della sedia.

## §FONDAMENTALI (apertura obbligata)
Sessioni-senza-misura: **0** (S-166: 2 coppie + micro + 2 A/B R=5). Oggetto:
2 promozioni in 2 giorni (mc2 −8,5%, mc3 −10,6%), WP 1,761→1,746. Rischio più
trascurato: **arith/prop FERMI da ~15 sessioni** — oggetto di questo concilio.

## VERDETTO: GO-CONDIZIONATO (9/9 — nessun NO-GO, nessun GO secco)
**Campagna R1 «interno-handler prima, dispatch poi»**. Rotte chiuse dal sapere
accumulato: R2-Zval (taglia GIÀ 16 B = Zend; borrow ≤1; alloc ≈0; **veto
NaN-boxing CONFERMATO** — Stogov: Zend fa 8,6 ns senza) · R3-arena
(movente caduto con L-AL3; rischio Drop-order/RetainSet; resta SOLO la forma
sigillata della regola 8 — **dissenso Hoare: fuori in toto**) · fn-table
(indiretta opaca, icache peggiora — H-C2/MC1) · BOLT/PGO (veti confermati).
**Reperto capitale (Hejlsberg, dal sorgente)**: phpr dispatcha ~3 op/iter
contro ~6-7 di Zend ed è comunque 5,4× ⇒ il divario vive DENTRO il handler
(fetch operandi, match BinOp a runtime, bounds, guardie, gc_note).

## FETTA 0 (prossimo atto, SOLA MISURA — nessun codice di leva prima)
Sonda arith a 5 bracci (team-sonda, criterio da pre-registrare PRIMA):
(a) A/B `PHPR_REG_LOWER` on/off · (b) gemello data-stride (pila/D-cache:
|D|≤0,5 ⇒ refutata) · (c) CPU counters BILATERALI via `xctrace` (branch-miss/op,
L1I-miss, IPC; phpr ≥2× oracle ⇒ front-end firmato) · (d) drop-census dcn!/iter
(interno-handler) · (e) tetto-fuso dietro flag SOLO se (c) firma front-end.
**Gate**: strumento collaudato con MUTANTE che deve mordere · chiusura
sottrattiva bilaterale ≥85% (≥90% = gate di Gregg PRIMA della prima leva di
trasformazione) · firma: ≥60% del gap 38,2 ns nominato ai tre canali
(obiettivo 70%) · **GO-leva: canale dominante ≥15 ns/iter**.

## SEQUENZA FETTE (team-meccanismo; ognuna promovibile DA SOLA sul pin)
F0 sonda (sopra) → **F1 predecode consts** (Vec<Zval> per-Func; D≥5) →
**F2 BinOp 3-indirizzi slot-diretti Long-guarded** (+,−,*,>>; superop
GENERICA, mai cucita sul driver — Matsakis; D≥8, str/calls/mc2 in
banda-layout) → **F3 guardie/slow-path outlined** (dipende da F2) →
F4 cmp+jmp (SOLO sotto clausola nomina-il-costo di Hejlsberg) → F5 threaded
(in coda, resa attesa bassa). Furti dichiarati alle leve promosse: slot-diretti
H-D/HD2 · IC monomorfo · cura outline MC1.

## LA FORMA (team-governo, 8 regole — VINCOLANTI)
1 safe-only: dentro = varianti di `Op`/pre-lowering; fuori = fn-table,
NaN-boxing, arena raw-ptr, transmute. 2 fette promovibili o revert al byte,
MAI branch lunghi-viventi. 3 fetta 0 = prezzatura con mutante. 4 kill di
campagna PRE-registrato in `.out` prima di F1: aggredibile <21 ns/iter O
residuo dispatch <10 O mock F0 <10 nominati ⇒ R1 morto, si delibera R4;
2 fette di trasformazione cadute di fila ⇒ vaglio R4 (**dissenso Klabnik: 3 +
concilio**). 5 gate two-request parity byte-identica su OGNI fetta (Pedersen).
6 disasm bl-count in ogni criterio; flip oltre banda-layout ⇒ revert.
7 timebox apparato mezza sessione, PERMANENTE. 8 R3 solo sigillata (v. sopra).

## KILL-SWITCH AGGIUNTIVI A REGISTRO
KS-BAK-167-1 (tetto-fuso: se nemmeno il fuso ≤3× ⇒ R1 muore) ·
KS-LE-167-1 (stride muto E counters ≈ oracle ⇒ campagna non parte) ·
Gregg: dispatch+decode <40% del gap o chiusura <90% dopo 2 sessioni ⇒ R4 ·
Stogov (soglia di campagna, più dura): 46,8→≤35 dopo F1-F3, dissenso a registro.

## EFFETTO SU ROTAZIONE
Il blocco ⚖️ in NEXT_SESSION dichiara questi verbali VINCOLANTI; la «leva di
ritmo» delle prossime sessioni È la fetta corrente di campagna (la fetta 0,
di sola misura, è l'oggetto sanzionato — non un'anomalia di ritmo).
