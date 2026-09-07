# WP_SESSION_171 — LEVA L-SL1 «forma sigillata Long» fetta 1 PROMOSSA: arith-dq 46,8→23,4 ns/iter (2,71× l'oracle), micro arith 5,4→2,7 = TAPPA ≤3× RAGGIUNTA sulla prima categoria; pin s171
**In una frase**: la forma «magra» che in S-170 era solo un mock è diventata codice vero
e sicuro (niente unsafe, ogni caso fuori dal dominio intero ricalcola il corpo originale):
il ciclo aritmetico passa da 5,4 a 2,7 volte l'oracle, e il pin nuovo ha superato tutti i
gate (batteria, corpus, 15 fixture, ORM, http-kernel) al primo giro utile.
**SCOREBOARD** (pin NUOVO **s171 phpr b360b2933eddfe18 + server b3ddaede545ba894**):
**arith 2,7 ↓↓ (5,4) · prop 5,2 ↓ (5,5) · calls 4,8 = · str 4,1 ↓ (4,2) · arr 3,2 = · re 2,5 =**
· WP/ORM: coppia t17 DOVUTA lanciata 19:27 (esiti in wp171-harness/s171-pair-verdetto-t17.out
e s171-orm-coppia-verdetto.out; rif. precedenti WP 1,746-1,749 · ORM [7,023;7,053]) ·
**leve spedite: 1 (L-SL1 promossa)** · incidenti: **1** (#1 xctrace crash sul braccio B
lascia 8,4G di ktrace: Data 21G→2G, purgato; classe S-169) + 3 difetti di copione curati in
corsa PRIMA di ogni numero (lock senza token `s171`; quiescenza a falso positivo per argv
del tool-shell/lanciatore ×2 → symlink neutri) · dente loc run.rs 6917→7091 DICHIARATO
(+174: helper i64 + 3 corpi esatti #[cold]) dopo un morso in promozione (tentativo 1
rc=101) · coda CI: 11 job accodati dai push, trattenuti dal lock (potare a HEAD in chiusura).

## Esiti secchi (criterio s171-criterio.md pre-registrato; verdetti s171-leva4-verdetto.out, s171-verdetto.out)
1·**A/B R=5 a tre bracci** (A=pin s166, B=candidato 34afdc54, C=tetto m13): **dq
  46,76→23,44 D=+23,32 NOMINATO** (attesa [17;26] centrata, rumore 0,32); **e2
  14,72→10,64 D=+4,08 NOMINATO** a filo (= mock m8); **B−C = −1,04 su dq**: la forma
  GENERICA (un match per op) riproduce il tetto driver-shaped — l'attesa «≈17» della
  revisione S-170 cade in direzione favorevole; kill-1 non scatta; promozione ammessa.
2·**Forma**: `long_arith_i64`/`long_cmp_i64` = arm Long di `binary_fast` verbatim; store
  in place su dst Long; corpi originali in 3 metodi `#[cold]` (nm: 3 simboli nuovi, 4 bl);
  disasm bl run_loop 6036→5970 (Δ −66 = chiamate uscite coi corpi; attesa +2..+4 EMENDATA).
3·**Promozione** rc=0 (tentativo 2): build ricetta b360b293 (≠ candidato solo LC_UUID+
  firma, identità a contenuto), batteria 1748/0/2, corpus 1412×2 zero flip, fixture chain
  10/10 + 15 gate byte-id (fx-sl1 NUOVA oracle==pin; fx-sl1-div pin==stash s166), micro
  R=5, conferma post-pin dq +23,36 5/5, ORM 16 nomi == baseline, hk 1665 0E/0F.
4·**Az.rev. S-170**: #4 m8 a N=1G (N dal driver) D=+4,43 oltre il tick → CHIUSA; #2
  xctrace pin/B/m13 su dq: `xctrace record` crasha su B (rc=7) → APERTA (Data ≥20G).
5·Revisione (lente SEMANTICA): wp171-harness/revisione.md.
## ⭐ Lezioni (max 3)
- ⭐⭐ un mock nominato si converte in leva se la FORMA è la stessa: qui la forma generica
  (match per op) non ha pagato nulla rispetto alla tupla cotta — misurare a forme paga due volte.
- ⭐⭐ `pgrep -f` nel gate di quiete morde l'argv di chi attende: i binari si passano per
  symlink neutri e il lock porta il TOKEN che il copione cerca.
- ⭐ ogni `xctrace record` costa ~8G di ktrace anche se crasha: purge in `trap EXIT`, mai
  solo dopo l'export; pre-flight Data ≥20G per una sessione con xctrace.
