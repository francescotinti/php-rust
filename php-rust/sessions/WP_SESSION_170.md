# WP_SESSION_170 — DELIBERA R4 = (i) «corpo del handler» ESEGUITA come sola misura: mock magri nominati su ENTRAMBI i giudici (m9 e2 −5,64; m13 dq −22,20 ⇒ arith-dq 2,84× l'oracle contro 5,4×); la forma sigillata Long è la prossima leva DA MISURARE COME CODICE
**In una frase**: abbiamo costruito versioni «magre» delle istruzioni calde (senza
controlli ridondanti, senza valori temporanei, senza scritture superflue) e misurato
quanto tempo tornerebbe: sul ciclo aritmetico phpr passa da 5,4 a 2,8 volte l'oracle
senza toccare il dispatcher né il compilatore — il tempo perso è DENTRO le istruzioni,
e ora sappiamo in quale parte; il prossimo passo è scriverlo come codice vero e misurarlo.
**SCOREBOARD** (pin INVARIATO **s166 phpr 092dcff431bef876 + server caa4e4b2638686a9**):
arith 5,4 = · prop 5,5 = · calls 4,8 = · str 4,2 = · arr 3,2 = · re 2,5 = · WP
1,746-1,749 (rif.) · ORM [7,023;7,053] (rif.) · **leve spedite: 0 — sanzionato ⚖️
(sola misura per delibera R4)** · incidenti: **2** (#1 pavimenti S-167..169 misurati su
`wp164-harness/empty.php` INESISTENTE: ≤0,02 ns/iter, D invariato, ereditato e
dichiarato; #2 operativo: target Cargo di gads-mcp cancellata intera invece di potata
⇒ regola utente «tenere i binari») + 4 difetti di copione curati in corsa PRIMA di
ogni numero (path patch dal git root; `[ -s ]` sul pavimento; wrapper con path non
quotati; let-else su blocco unsafe) · coda CI: potata a HEAD (delibera), runner rilanciato in chiusura.

## Esiti secchi (criteri s170-criterio.md + s170-criterio-b.md pre-registrati; A=m0 36d73812 riproducibile al byte)
1·**Handler banali** (giudice arith-e2, 2 op/iter, R=5): **m8** magro safe (no guardia
  Undef/Ref, no to_zval, bool diretto, no Result) **D=+4,08 NOMINATO a filo** (rumore
  0,04); **m9** = m8 + bounds elisi **D=+5,64** ⇒ bounds +1,56 firmati; per-op 7,32 →
  4,50 vs oracle 1,76; kill (D_m9<4) NON scatta ⇒ ipotesi «corpo del handler» REGGE;
  residuo 2,75/op oltre dispatch 1,75 = accesso slot + resto (NON nominato).
2·**BinarySCSCDst** (giudice arith-dq, R=5): m10 guardie +1,44 e m11 store +1,20 sotto
  pavimento (direzione firmata, 0 nel conto); **m12 funnel i64 +17,16 NOMINATO**;
  **m13 corpo magro totale +22,20 NOMINATO** (dq 46,8 → 24,6; statement 32,1 → 9,7 vs
  oracle 5,1; corpo fuso 27,5 → 5,1); differenze dichiarate: funnel+read_slot+guardie
  = m12−m123 +9,16 · store+bounds = m13−m12 +5,04; kill-b non scatta, soglia ≥20
  superata ⇒ **forma sigillata Long (regola 8) = leva da misurare come CODICE in S-171**.
3·**xctrace-3**: mutante c0 positivo (array_sum L1) FALLITO bilaterale (phpr→c1,
  oracle→c2): una catena seriale non è «retiring»; c0 resta per esclusione; quote dq
  replicate N=2; c2 delivery 15,8 vs 0,84 ns/iter INDIZIATA (front-end).
4·Apparato: copia-gate **v3 per TOKEN** (riga mista morde v3, passa v2); parità vs
  ATTESO oracle; lock per token. 5·Estrapolazione DICHIARATA: m13+m9 ⇒ dq ≈18,9 ≈2,2×.
## ⭐ Lezioni (max 3)
- ⭐⭐ il pavimento va collaudato contro l'ATTESO come ogni driver: tre sessioni hanno
  misurato il floor su un file inesistente senza che nessun gate lo vedesse.
- ⭐⭐ i controlli «piccoli» (guardie, store) non pesano da soli: pesa la FORMA del
  cammino (Zval temporanei, funnel Option, clone/drop) — misurare a forme, non a righe.
- ⭐ una potatura di target Rust conserva i binari; su progetti altrui si propone
  la potatura selettiva, mai il rm intero (regola utente 2026-09-07).
