# WP_SESSION_174 — LEVA «Sweep-in-op» (B: Sweep inerte scavalcato a fine op senza dispatch; C: B + back-edge IncDecSlotJmp→CmpJmpSC fuso) AMMESSA a criterio-bis: prop-dq 41,3→36,1/36,5 (2,58-2,60× l'oracle, era 2,95×), arith-dq 24,0→20,4/20,9 (2,35-2,41×, solo direzione); promozione DIFFERITA (Data 4G); az.rev. S-173 chiuse; pin INVARIATO s173
**In una frase**: dopo un'istruzione che scrive un intero «al posto» il motore non passa più dal dispatcher per
scoprire che la pulizia di fine istruzione non ha nulla da fare, e al ritorno del `for` decide subito il confronto:
prop-dq scende da 2,95× a 2,6× l'oracle, arith-dq da 2,8× a 2,4×; la promozione a pin aspetta spazio su disco.
**SCOREBOARD** (pin INVARIATO **s173 phpr da4921a52eba0187 + server 2d2adc4549a820e0**): **arith 2,8 = · prop 3,0 = ·
calls 4,8 = · str 4,1 = · arr 3,1 = · re 2,5 =** (micro non rimisurate) · **candidato C (stash phpr-s174-sw-C b6c4b5876971d76d):
arith-dq 24,04→20,40 / 24,44→20,92 (2,77×→2,35-2,41×) · prop-dq 41,33→36,13 / 42,07→36,47 (2,95×→2,58-2,60×)** · WP t19 1,772 /
ORM net [6,988;7,012] (pin invariato) · **leve spedite: 1** (A/B ×2, AMMESSA, promozione differita) · incidenti: **1** (#1 copione
bis caduto dopo ESITO: RC inglobato da un commento; sw2.rc da ri-analisi dichiarata) · Data 4-5G (<10G) ⇒ promozione differita.
## Esiti secchi (criteri PRE-registrati s174-azrev-criterio.md · s174-criterio.md · s174-criterio-bis.md; verdetti s174-*.out)
1·**gh-status-sync** (pin s173): corpus 2655/1412/1238 e fns 1017/2143 IDENTICI; docs a data/pin S-173 + micro/ORM ~7,0× (235ae860, 7c01a999); nessuna build (pin verificato per hash).
2·**Az.rev. S-173** TUTTE chiuse: (a) banda tra run |A−PREV| nei .out (arith 0,36/0,76, prop 0,20/0,94), 0,94 mai
  più importato (soglia direzione = max(rumore, |A−Z|)); Z = sorgente del pin ricostruita = 19c0540e (GEMELLO a
  contenuto: ricetta deterministica anche cambiando path) ⇒ |A−Z| = controllo NULLO (0,16-0,40), non banda-layout
  (emenda dichiarata prima dell'A/B); (b) mediane appaiate + cifra a intervallo, P2 = [4,87;5,60] retroattivo;
  (c) decomposizioni già IPOTESI (verificato); (d) i 5 blocchi `-each` INTATTI nominati nel verdetto S-173.
3·**Leva** (zero unsafe; B: `sweep_idle` = predicato del handler estratto UNA volta, `sweep_skip_next` a fine
  op dai SOLI sentieri in place su Long di BinarySCSCDst/BinarySTDst/P3/P1-P4; C: back-edge fuso con
  `long_cmp_i64` verbatim; spente sotto census): build Z/B/C su target esterno; parità + dump {main}
  IDENTICI sui due giudici (peephole runtime); fx-sw1 NUOVA bilaterale (88 righe: ordine dtor/statement,
  fuori dominio, IN_DESTRUCTOR, pressione GC, 20 forme `for`) + fx-sl1..3 byte-id; --braccio s174-sw-B/C col
  commit; disasm bl 5986→5988→5993 (≤ +8). **Corsa 1 rc=4**: arith-dq C +3,64/+3,56 (5/5, attesa [2,5;6]
  centrata, KILL-B no) < pavimento 4; prop-dq (guardia) C [5,13;5,20] nominata ma non giudicata.
  **Criterio-bis** (prop-dq co-bersaglio, attesa [2;6] già dichiarata; commit 22:56:47 < lancio 22:56:59) ⇒
  **corsa 2 rc=0: prop-dq C = CIFRA [5,07;5,60] (entrambi gli stimatori), arith-dq C +3,52/+3,48 direzione,
  C−B +1,96 direzione, nessuna regressione ⇒ AMMESSA braccio C**. B (Sweep-in-op) da solo: +1,3/+1,6 arith,
  +2,7/+3,1 prop = direzione, magnitudine non ripartita.
4·**Mutante MS/MC** (s174-mutante-sw.sh, intatti per NOME): corsa 1 rc=1 (attese a LOOP non discriminanti: lo Sweep del `for`
  drena prima di «post»; p1-loop idempotente) → v3/v4 (echo nel corpo, coppie per sito, `+=` = forma vera di BinarySCSCDst,
  classe PLAIN P2) → **corsa 4 rc=0: MS 11/11 ROTTE = i 5 siti provati uno a uno, controlli INTATTI; MC 29/29; nessuna presa fuori dominio**.
5·**Promozione DIFFERITA**: Data 4G (<10G): build canonica su target potato (~5G) + batteria non ci stanno; candidato
  stashato phpr-s174-sw-C; dente run.rs cap 7274→7361 DICHIARATO (9f349b0f); s174-promozione.sh pronto; coppia dovuta col pin nuovo.
6·Revisione (lente SEMANTICA, wp174-harness/revisione.md): REGGE CON RILIEVI (wp174-harness/revisione.md): il codice regge per costruzione (un solo predicato letto DOPO l'effetto, back-edge verbatim); rilievi (1) artefatti fx-sw1 in ab-out erano v1, (2) MS non provava forme a loop né i siti P1/P4/BinarySCSCDst, (3) criterio-bis: regola scelta a dati visti, arith-dq resta SOLO direzione. Replica S-174: (1)(2) CHIUSE in sessione — fixture v4 (loop con echo nel corpo, coppie per SITO, forma esatta di arith-dq `$l += …`, classe PLAIN P2: la P tipizzata è fuori perimetro) bilaterale oracle==pin==B==C in ab-out; mutante corsa 4 rc=0 (MS 11/11 ROTTE coi 5 siti, controlli p3-site-ctl e p1-site-typed INTATTI; MC 29/29); (3) accolto.
## ⭐ Lezioni (max 3; apparato: skill con `model:` → memoria)
- ⭐⭐ il pavimento 4 ns/iter su un loop da 4 op nasconde un −15%: il giudice va scelto dove la leva incide
  di più (2 Sweep/iter su prop-dq); una guardia si emenda SOLO rieseguendo il criterio emendato (REGOLE §3).
- ⭐⭐ una copia dichiarata si collauda INTERA (shell + analisi): il dry-run del solo python non vede un
  commento che ingloba un'assegnazione sulla stessa riga (incidente #1).
- ⭐ ricostruire la stessa sorgente NON dà un layout diverso (hash identico anche cambiando path): la
  banda-layout non si ri-deriva così; |A−Z| resta utile come controllo nullo in-run.
