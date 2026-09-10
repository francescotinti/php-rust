# WP_SESSION_173 — LEVA L-SL2 fetta 3 (prop, residuo del corpo) PROMOSSA: prop-dq 52,5→41,1 ns/iter (2,95× l'oracle, era 3,78×); az.rev. S-172 tutte chiuse (mutante P1/P2 rc=0, P2 = +4,87 cifra, --braccio col commit); pin s173
**In una frase**: le due istruzioni che leggono una proprietà e la sommano a una variabile ora si
parlano direttamente senza passare dalla pila, e quando un oggetto legge e scrive su sé stesso lo
fa con un solo «prestito»: il ciclo di prova scende sotto 3× l'oracle sul giudice della leva; le
cure promesse dalla revisione S-172 sono state eseguite e misurate PRIMA di questa leva.
**SCOREBOARD** (pin NUOVO **s173 phpr da4921a52eba0187 + server 2d2adc4549a820e0**):
**arith 2,8 (2,7: tick) · prop 3,0 ↓↓ (3,8) · calls 4,8 (4,7: tick) · str 4,1 = · arr 3,1 (3,0: tick, guardia di P3 senza regressione) · re 2,5 =** · **prop-dq 41,13 vs oracle 13,93 =
2,95× (pin s172: 52,53 = 3,77×)** · **coppia t19: __** · **leve spedite: 1 (fetta 3, bracci P3 e
P3+P4)** · incidenti: **1** (#1 log della build C cancellato durante la build, scambiato per residuo
S-172: esito da .rc/hash, log perso) · dente run.rs __ · §3.32 catalogata (riga del Deprecated per
prop dinamica) · perimetro dichiarato: il probe sigillato prop (P1/P4) copre SOLO classi plain (IC set
non riempita su typed/private) — leva futura «prop typed» a TODO.
## Esiti secchi (criteri PRE-registrati s173-azrev-criterio.md + s173-criterio.md; verdetti s173-*.out)
1·**Az.rev. S-172 (a)** mutante abortivo P1/P2 (s173-mutante-sl2.sh, fx-sl2 estesa a 118 righe):
  corsa 1 rc=1 = 3 attese MAL POSTE (loop-overflow satura in float, p2-and coincidenza 85&7+1==70&7)
  + 1 di dominio (p1-private-inscope: IC set non riempita su classi non-plain ⇒ probe non preso,
  corretto); fixture resa discriminante (righe -step, reset); corsa 2 rc=0 (MP1 26/26 rotte, MP2 17/17,
  fuori dominio intatte, p1-shl-63 preso).
2·**(b) A/B alternato B↔C** (s173-ab-bc.sh, BC/CB, R=5, stash 260fbc3b/e396498b): B−C=+4,87 NOMINATA
  ⇒ **cifra a P2 = +4,87 ns/iter** (attesa [2;8] centrata; in S-172 solo direzione +4,67).
3·**(c)** pin-phpr.sh `--braccio <tag> <bin> <commit>` (3° arg obbligatorio, `git cat-file -e`), righe
  s172-sl2-B/C corrette (c419f29a/59ca87fb); collaudato sui bracci f3-B/C. **(d)** typed/private/dynamic
  = fuori perimetro del probe (mutante), TODO.
4·**Fetta 3 A/B R=5** (A=pin s172, B=P3 8a58fb19 @2006d11d, C=P3+P4 19c0540e @64c55cc2;
  ordine dei 3 bracci RUOTATO per coppia): **prop-dq 52,53→44,93 (D_B +7,60) →41,13 (D_C +11,40)**,
  attese [2;8]/[2,5;12] CENTRATE, rumore 0,07, KILL-3 no; **C−B=+3,80 firmato NON nominato ⇒ P4 solo
  direzione**; guardia arith-dq +0,12 (piatta); disasm bl 5974→5983→5986 (+12, sopra il +10 atteso,
  dichiarato); parità A==B==C==oracle, dump {main} identico (peephole runtime); fx-sl3 NUOVA 86 righe
  + fx-sl2 + fx-sl1 byte-id su B/C; mutante P3/P4 corsa 1 rc=5 = dst che DIVENTA Long dopo la 1ª
  iterazione (non fuori dominio) ⇒ forme -each, corsa 2 rc=0.
5·**Promozione** (s173-promozione.sh, braccio C): rc=0 al tentativo 2 (t1 rc=101 = dente run.rs 7274 > cap 7200, alzato DICHIARANDO): build ricetta = candidato a contenuto (da4921a5 vs 19c0540e: soli LC_UUID/firma), batteria 1748/0 (+2 denti dichiarati), churn neutralizzato al byte, corpus 1412×2 ZERO flip, fixture chain 10/10, 19 gate byte-id (fx-sl3 NUOVA + fx-sl3-div), micro R=5, conferma post-pin prop-dq +11,67 rumore 0,93 segni 5/5 (52,93→41,27), ORM 16 nomi == baseline (3484: 3E/13F), hk 1665 0E/0F
6·Revisione (lente MISURA): wp172-harness/revisione-s173.md — REGGE CON RILIEVI (drop-1 0,07 = tick di
  quantizzazione, banda tra run 1,14 ns/iter; 0,94 non ri-derivato su prop-dq; mediane appaiate: P2 = [4,87;5,60],
  P4 +3,87 sotto soglia; decomposizioni del residuo = ipotesi; pre-registrazione e igiene REGGONO; replica: i blocchi
  `-each` intatti sono la prova del dominio) — az.rev. S-174.
## ⭐ Lezioni (max 3)
- ⭐⭐ un'attesa «intatta» va provata iterazione per iterazione: dopo la 1ª il dst è già Long e la 2ª
  è in dominio (corsa 1 di ENTRAMBI i mutanti) — il presidio del tag si scrive col reset DENTRO il loop.
- ⭐⭐ la rotazione a 3 bracci trasforma un contrasto «solo direzione» in un A/B proprio: P2 ha preso la sua cifra (+4,87) e P4 no (+3,80 < 4): la soglia morde anche quando la direzione è 5/5.
- ⭐ una IC «per lato»: la get si riempie anche su typed/__get-class, la set solo su plain — il probe sigillato copre meno di quel che sembra; il mutante lo dice, la fixture bilaterale no.
