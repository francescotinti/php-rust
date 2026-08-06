# Team PROCESSO-PIN-SERVER — Fase 2 Concilio WP-107
Membri: Klabnik (relatore), Pedersen, Bak · Data: 2026-08-07

## 1. Composizione delle posizioni

### Convergenze (ID canonico dichiarato)
- **Chain v2**: A-KL-107-4 ≡ A-PE-107-2 nella sostanza (pre-flight, restore-on-fail,
  watchdog, marker il cui rc È il verdetto del gate, assert bilaterale). Canonico: **T-PS-107-9**.
- **Lettura coppia S-105 con precondizioni**: A-PE-107-3 ≡ A-KL-107-5 (le identity
  vanno ASSERITE, oracle incluso, prima di citare i rapporti). Canonico: **T-PS-107-5**;
  KS-PE-107-2 e KS-KL-107-4 ne sono i denti (nessun saldo da marker il cui rc non è il verdetto).
- **Igiene pin**: A-KL-107-1 + KS-KL-107-1 (già recepiti in ordine punto 4) — nessuna
  sovrapposizione interna al team, Pedersen concorda via KS-PE-107-1(ii) (pin firmatario).
- **Copertura ≠ istogramma**: R-BA-107-1/KS-BA-107-1 non contestati da Klabnik/Pedersen;
  il contatore hit/miss è census-gated, quindi coerente con A-PE-107-4 (mai nel binario pinnato,
  mai process-global citato da contesto server). Canonico: **T-PS-107-8**.

### Conflitti risolti
- **C1 — «primo atto» conteso** (A-KL-107-1 cargo check vs A-PE-107-1 grado prima di ogni
  build): risolto per orologio di decadenza — il grado ha un KS a scadenza (KS-PE-107-3,
  seconda proroga = pin decade), il cargo check no. Ordine: grado lanciato per primo (fatto);
  cargo check = **primo atto CPU dopo `grado-chain.done`**, prima di ogni misura. Il cargo
  check non tocca il binario release ma consuma CPU: resta comunque dietro il chain.
- **C2 — coppia S-105: VOID o saldata?** R-PE-107-2 (l'autore risolve: coppia CLI gradata
  dal pin phpr, non dal grado server) vs R-KL-107-4 (il `.done` non può arbitrare il saldo).
  Composizione: la coppia NON è VOID (KS-PE-107-1 riscritto fa fede), ma il «SALDATO» del
  punto 1 esige la retro-verifica del protocollo completo (T-PS-107-5). I due verbali sono
  complementari, non in conflitto: Pedersen fissa il perimetro del grado, Klabnik il rito di lettura.
- **C3 — timing del contatore hit/miss** (A-BA-107-1 «prima di nominare il prossimo sito»
  vs ordine punto 6 dopo la leva): nessun conflitto reale — il vincolo morde la NOMINA del
  prossimo sito calls (S-107), non la leva prop/arith del punto 5, che ha il SUO prerequisito
  (braccio contatori prop, checkout 4ea2cff). Punto 6 resta dov'è.

### Aperto (fuori mandato del team, da passare alla SYNTHESIS)
- fx21: la MECCANICA del gate è nostra (T-PS-107-6); il CONTENUTO dell'attesa interseca
  KS-ST-107-2 (doppia copia della semantica di bind) — coordinare con team-vm.
- Se la lettura della coppia S-105 fu eseguita col protocollo completo (identity oracle
  verificata?) non risulta dai file letti: da accertare in S-106, esito aperto (T-PS-107-5).

## 2. Giudizio sull'attuazione del grado (s106-grado-server.sh + s106-grado-chain.sh)

**VERDETTO: CONFORME NELLA SOSTANZA alla lettera A-PE-107-1/2, con 4 difformità puntuali
— nessuna richiede di uccidere il run in volo; tutte si sanano ALLA LETTURA.**

Conforme: binario stashato `php-server-s105` con hash da file FAIL-CLOSED (PIN_SRV_ATTESO
= de67cb6466acb030, verificato) ✓ · pin phpr FAIL-CLOSED (d4d0fa5217515dd9, verificato) ✓
· option+restapi per NOME con assert conteggi↔nomi (KS-KL-101-3) ✓ · env -i costruito con
lista chiusa e re-exec guard ✓ · 2 modi ESPLICITI (exit 64 senza argomento) ✓ · mode-probe
REALE nella sentinella (dump unità, FAIL se il modo non corrisponde) ✓ · pre-flight chain:
ping MySQL con elenco database (wp, wptests), STATE guardia assente, ≥10G ✓ · restore
tentato su rc gamba ≠0 (belt oltre il trap del launcher) ✓ · lanciato PRIMA di cifre e build ✓.

Difformità:
1. **Watchdog solo sulla gamba phpr** (s106-grado-server.sh r.100-101 vs r.105): la gamba
   phpunit ORACLE gira nuda — la lettera A-PE-107-2 dice «gambe phpunit sotto watchdog»,
   e R-PE-107-3 nasceva proprio da lì. Un hang oracle = niente `.done` = notte bruciata.
2. **Hash oracle mai asserito né registrato** (A-KL-107-4 chiede assert per-gamba di phpr
   E oracle): nessuna identity dell'oracle brew nel grado. Sanatoria: hash dell'oracle
   registrato SUBITO alla lettura, con la riserva che un churn brew notturno resta non escluso.
3. **Conteggi attesi non pinnati**: la lettera nomina «option 413 + restapi 3508», il gate
   asserisce solo dichiarati↔estratti — un oracle che corresse un gruppo dimezzato (es.
   reinstall wptests) passerebbe. Alla lettura: n nomi = 413 e 3508, pena lettura VOID.
4. **rc=FAILS conflaziona VOID e DIVERSO** (r.114 e r.119 incrementano lo stesso contatore)
   e il `.done` non porta l'esito failnames-diff per gamba (lettera A-PE-107-2); inoltre il
   chain esce SEMPRE 0 (r.53): il `.done` è l'unico arbitro. La distinzione VOID/rosso si
   recupera solo da progress.txt: la lettura deve leggerlo, mai il solo rc.

A favore del chain, contro il difetto di pair105 (R-KL-107-4): qui RC_OFF/RC_ON catturano
`exit $FAILS` dello script gamba, cioè IL verdetto del gate — KS-KL-107-4 è rispettato.

## 3. Direttive composte del team

### (a) VINCOLANTI per l'ordine S-106
1. **T-PS-107-1** — Primo atto CPU dopo `grado-chain.done`: cargo check a HEAD con esito a
   verbale; «parity-null» dicibile solo per commit senza file compilati (`git show --stat`
   a verbale). [assorbe A-KL-107-1, KS-KL-107-1, R-KL-107-1]
2. **T-PS-107-2** — Batteria PIN-106: rc E conteggio dalla STESSA run; ripetizione ammessa
   solo con hash dopo OGNI run. Vale per il PIN-106 di chiusura del punto 5. [KS-KL-107-2, R-KL-107-3]
3. **T-PS-107-3** — Nessuna build (né cargo check) prima di `grado-chain.done`; nessuna
   cifra server prima della lettura VERDE del grado; seconda proroga ⇒ il pin server DECADE
   e i report tornano a «server NON misurato». [A-PE-107-1, KS-PE-107-3, KS-PE-107-1]
4. **T-PS-107-4** — Rito di lettura del grado (sana le 4 difformità): (i) n nomi = 413/3508
   pena VOID; (ii) hash oracle registrato subito; (iii) VOID vs DIVERSO distinti da
   progress.txt, mai dal solo rc del `.done`. [KS-KL-107-4, R-PE-107-4, A-KL-107-4 in parte]
5. **T-PS-107-5** — Il «SALDATO» del punto 1 vale SOLO con retro-verifica del protocollo:
   rc_off/on=0, 8 run rc=0, failnames VUOTI, identity phpr d4d0fa52 E oracle 07b0df8d in
   ENTRAMBE le identity; altrimenti WP-102 decade a storico. [A-PE-107-3 ≡ A-KL-107-5, KS-PE-107-2]
6. **T-PS-107-6** — fx21 promosso a `s105-fx21-gate.sh` modello fx20: pin bilaterale
   fail-closed, rc 0/1/66, golden riga 5 IN repo; attesa = 7 righe oracle-identiche ∧ riga 5
   ≡ golden pinnato (va ROSSO quando §3.15 sarà curata). [A-KL-107-3, R-KL-107-6; coord. KS-ST-107-2]
7. **T-PS-107-7** — Leva punto 5: prop SOLO se il braccio contatori (checkout 4ea2cff, ~30′)
   è ESEGUITO nella stessa finestra e PRIMA del criterio; altrimenti la leva è ARITH 12,4.
   Nessuna terza via. [A-BA-107-3, R-BA-107-4]
8. **T-PS-107-8** — Nessun claim di copertura del fast path senza contatore hit/miss al
   branch + per-chiamante + volume `pop_keys` builtin; il prossimo sito calls (S-107) si
   nomina SOLO da quei numeri. [A-BA-107-1, KS-BA-107-1, R-BA-107-1]
9. **T-PS-107-9** — Ogni chain futuro è v2: pre-flight, restore-on-fail per gamba, watchdog
   su TUTTE le gambe phpunit (oracle inclusa), `.done` con esito failnames-diff per gamba e
   rc = verdetto del gate. Il chain in volo NON si rilancia: si sana con T-PS-107-4. [A-PE-107-2 ≡ A-KL-107-4, R-PE-107-3]

### (b) Raccomandazioni
10. **T-PS-107-10** — Census lato server solo per-request bracketed; la static GA_ARITY
    process-global va rimossa o feature-gated alla prima occasione che non churna il pin. [A-PE-107-4]
11. **T-PS-107-11** — Investigare la gobba a4=15,6% > a3=7,7% (candidato: filiera hook WP)
    col contatore per-chiamante, prima di nominare siti in S-107. [R-BA-107-2]
12. **T-PS-107-12** — Verdetti sui contenitori registrati per forma-e-confine; vietato
    estenderli per categoria («mai Vec») senza misura del sito. [KS-BA-107-3, R-BA-107-3]

## 4. Modifiche chieste all'ordine provvisorio §S-106

- **Punto 1**: EMENDARE il testo — «debito SALDATO» diventa «SALDATO CONDIZIONATO alla
  retro-verifica T-PS-107-5»; prima della verifica, 1,894 non si cita nei report.
- **Punto 2**: ratificato COME ATTUATO, con il rito di lettura T-PS-107-4 reso parte
  dell'ordine (le 4 difformità si sanano alla lettura, non rilanciando).
- **Punto 4**: ratificato; aggiungere la clausola di sequenza «dopo grado-chain.done» (C1).
- **Punto 5 (leva prop vs arith)**: ratificato NELLA FORMA CONDIZIONALE con default
  esplicitato — se il braccio contatori non entra per primo nella finestra, la leva È arith
  12,4; la scelta si verbalizza citando l'esecuzione (o la non-esecuzione) del braccio.
- **Punti 3, 6, 7**: ratificati senza modifiche (punto 6 confermato dietro la leva: C3).
