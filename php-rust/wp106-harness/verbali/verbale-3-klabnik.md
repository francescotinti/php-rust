# Verbale sedia 3 — KLABNIK (spec, testabilità, matrici/gate) — Concilio WP-106 su S-104

## VERDETTO: CON EMENDAMENTI (nessun MI OPPONGO). Una refutazione CAPITALE, sulla REGOLA PIN-105, non sulla condotta di S-104.

### R-KL-106-1 (CAPITALE — perimetro a): PIN-105 è insoddisfacibile per costruzione
La sequenza «build→hash→STASH→batteria→fixture→corpus» obbliga la propria
violazione: la batteria relinka il binario release (fenomeno NOTO e a
registro dal S-99/S-100: «l'hash churna col relink: fa fede HEAD»), quindi
lo stash contestuale pre-batteria è invalidato dal passo immediatamente
successivo, SEMPRE. La clausola «"gradato" senza stash contestuale =
retroattivamente NON-gradato», letta alla lettera, declasserebbe ogni pin
futuro — incluso 86a50d1c, che invece è il primo pin misurato secondo lo
spirito della regola. S-104 ha fatto la cosa giusta (ri-stash + churn
66681884→86a50d1c documentato) violando la lettera. Una sequenza che
obbliga la propria violazione non è una sequenza: si emenda o morde
l'autore a ogni chiusura.

**A-KL-106-1 (PIN-106, vincolante da S-105)**:
build→hash₁→**BATTERIA**→re-hash₂→**STASH(hash₂)**→fixture→corpus→micro.
(i) Dopo lo STASH nessun passo che possa relinkare; (ii) ogni gate che
ESEGUE il binario pinnato (fixture, corpus, micro) gira su hash₂ — la
batteria certifica il SORGENTE, non il binario, e il registro distingue le
due specie di evidenza; (iii) il churn hash₁→hash₂ si registra con un
invariante byte del codegen (es. taglia run_loop, come fatto in S-104).
**KS-KL-106-3: build/relink dopo lo stash ⇒ gate VOID, si riparte da
re-hash₂.**

### R-KL-106-2 (perimetro b): A-KL-105-3 soddisfatta alla lettera — ma il claim non nomina il suo perimetro
Prima volta a verbale: micro sul PIN DI CHIUSURA (±0,2) + funzionale
(1740/0 · fixture con fx20-RSS · 1417×2 per NOME) — la definizione
funzionale∧strumentale è ONORATA per phpr CLI. Due buchi restano: (1) il
claim «runtime parity-null» copre di fatto DUE binari e ne misura UNO —
il server 31aa7c2e è @ 37312e8, N commit dietro il HEAD di chiusura
090e2eb: il suo parity-null è INFERITO dal sorgente, non misurato. È lo
stesso vizio che A-KL-105-3 curava sul CLI, spostato sul server. (2) La
banda strumentale ±0,2 cita la banda tra-sere che per la regola di Gregg
è ancora a 2 punti stesso-giorno = 1 punto: banda PROVVISORIA, citabile
solo se dichiarata tale.

**A-KL-106-2**: ogni claim parity-null porta OBBLIGATORIAMENTE (a) il
perimetro nominato — binari coperti da MISURA vs per INFERENZA, con
distanza in commit; (b) la banda citata col suo N e stato
(chiusa/provvisoria). **KS-KL-106-1: «parity-null» senza perimetro
nominato = dichiarato, non provato ⇒ declassato nella riga di registro.**

### R-KL-106-3 (perimetro c): «la salda la prima leva vera» è un timeout senza scadenza
TERZA sessione col debito nominato; la leva H-C2 è CADUTA, H-D non è
ancora leva: l'evento saldante può non arrivare MAI. Intanto il rischio
che la coppia sorveglia CRESCE a ogni commit «parity-null» che aggiunge
codice — e la scoperta capitale di S-104 (run_loop ICACHE-BOUND) dice
esattamente che micro ±0,2 NON copre la pressione icache del full. La
regola era sana come trigger d'evento; senza scadenza è una scappatoia.

**A-KL-106-3**: regola evento-O-timeout: coppia WP dovuta alla prima leva
spedita **O entro S-105, quale viene prima**; contatore
«sessioni-senza-coppia = N» nei FONDAMENTALI accanto al contatore leve.
**KS-KL-106-2: S-105 che chiude senza coppia WP (leva o no) ⇒ anomalia
dichiarata in testa al report.**

### Priorità S-105 (questa sedia)
1. **Coppia WP** (timeout scattato; serve comunque baseline fresca al
   design per-fase A-LE-105-5). 2. **PIN-106 nel registro** (costo ~zero).
3. Claim parity-null sempre col perimetro. Il resto (SiteTag H-D,
per-fase) è materia d'altre sedie.
