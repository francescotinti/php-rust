# WP_SESSION_168 — MOCK sottrattivi F0 (fetta 0-bis): kill ⚖️ scatta A FILO (Σ 9,92 < 10), chiusura 52,7%; reperto E2 = controllo-loop 7,3 ns/op (4× oracle); stride REFUTATO con controllo positivo; legenda xctrace CORRETTA
**In una frase**: quattro versioni «finte» dell'interprete, ognuna con un solo
costo tolto dal cuore dell'istruzione aritmetica, cronometrate: insieme valgono
~10 ns su 47 — per la regola del concilio questa strada (alleggerire il gestore
via compilatore) è esaurita e va deliberata una rotta nuova; in compenso ora
sappiamo che anche le due istruzioni banali del ciclo costano 4× l'oracle.
**SCOREBOARD** (pin INVARIATO **s166 phpr 092dcff431bef876 + server
caa4e4b2638686a9**): arith 5,4 = · prop 5,5 = · calls 4,8 = · str 4,2 = ·
arr 3,2 = · re 2,5 = · WP 1,746-1,749 (rif.) · ORM [7,023;7,053] (rif.) ·
**leve spedite: 0 — sanzionato ⚖️ (fetta 0-bis = sola misura, gate mock
dovuto PRIMA di F1)** · incidenti: **2** (catena di misura lanciata sotto il
timeout del tool, uccisa e rilanciata col daemonizer senza cifre perse; output
di run `ab-out/build-m0.out` finito in un commit, poi untracked + .gitignore).

## Esiti secchi (criterio s168-criterio-mock.md pre-registrato, emende p.7-9 dichiarate)
1·MOCK (A=m0 braccio nullo 36d73812, D=−0,24 vs pin, bl 6036=6036; giudice
  arith-dq N=250M R=5): m1 consts-predecode **+4,72** · m2 BinOp cotto **+5,20**
  · m3 hoist frames[top] **−2,28** (0) · m123 +8,00 (additivo: Σ grezza 7,64) ·
  m4 NULLO (Sweep del loop riemesso dal Block: D=0,00) · m4b no-Sweep **+2,96**
  (sotto pavimento 4 ⇒ 0; direzione: Sweep/iter ≈3 ns). **Σ nominati 9,92 <10 ⇒
  KILL regola 4 SCATTA (per 0,08, rumore 0,2-0,6: margine DICHIARATO) ⇒
  delibera R4 al concilio**; chiusura (e2+Σ)/dq = 52,7% ⇒ nessun codice di leva.
2·REPERTO E2 (loop nudo bilaterale): 2 op = 14,7 ns/iter phpr vs 3,56 oracle
  (7,3 vs 1,8 ns/op); statement 32,0 vs 5,1; residuo ~22 ns DENTRO il handler
  nominato solo per esclusione.
3·SANATURE (az.rev. S-167 2-4 chiuse): stride S-167 aveva $s/$i sulla STESSA
  linea (dump slot 14/15) — stride2 (slot 0/15, linee distinte) D2=−0,04 con
  controllo positivo dcnear/dcfar +270 ns ⇒ pila REFUTATA a 0,5 · xctrace:
  memstall alza SOLO c1 ⇒ c1=backend-stall (S-167 la chiamava mispredict:
  ETICHETTA CORRETTA; refutazione mispredict REGGE su c3 0,006 vs 0,037) ·
  repliche p-dq spread ≤0,014, oracle ±0,1 · anomalia o-mut spiegata · branchmut
  S-167 PREDICIBILE (periodo ~10): rbranchmut (LCG bit 30) scritto per S-169.
4·Apparato: --braccio ×7 · build da copia in target dedicato (rimosso) · copia-gate rc=0 ×3 · raw fuori repo.

## ⭐ Lezioni (max 3)
- ⭐⭐ un mock si collauda col DUMP dell'op-sequence del loop PRIMA di misurarlo
  (m4 ha speso un A/B intero per un braccio nullo; m4b l'ha dimostrato in 1 s).
- ⭐⭐ un kill pre-registrato che scatta per 0,08 ns si ESEGUE (meccanico) ma si
  DICHIARA a filo: la banda del kill andava pre-registrata come la soglia.
- ⭐ la soglia di un controllo positivo non scala col rumore dell'effetto (10×rumore su far=40 ns nascondeva un morso 3,7×): emenda + riesecuzione.
