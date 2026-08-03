# team-engine — relazione fase 2, Concilio WP-94

Relatore: sedia-engine. Fonti VINCOLANTI: verbale-8-stogov (checker LSP), verbale-7-leijen (canale), verbale-1-hoare (sigilli). Capitali: 2 (Stogov) + 2 (Leijen) + 0 (Hoare).

## CONVERGENZE

1. **Un solo filone è PRONTO a produrre: i sigilli.** Gate a HEAD e52a634 PASS, controesempi ESEGUITI, nessuna cifra cade. A-TH-68..72 sono lavoro interamente locale allo script, senza dipendenze dagli altri due.
2. **Il canale è BLOCCATO da due cose distinte, non una.** (a) A-DL-57 è refutato *com'è scritto*: heaps_total=1 rende il dente REFUSE potenzialmente scopato ⇒ barriera bloccata da PT-1 (A-DL-61). (b) KS-DL-94-1 blocca la LETTURA di malloc_huge current come massa viva finché non c'è il controllo di decremento. Sono blocchi su oggetti diversi: la barriera e la cifra.
3. **Il checker è BLOCCATO da sé stesso, non da altri.** q3b (benedice dove Zend fatala) e iterable-desugar (byte-divergenza su ogni messaggio con iterable) sono difetti del codice esistente, non lacune del wiring. Un wiring montato sopra installerebbe falsi-negativi di massa (ORM/hk) e messaggi byte-divergenti.
4. **I sigilli sono prerequisito del CANALE, non del checker.** KS-TH-94-1: nessuna cifra m9x con probe può citare i pin arm/dot-name come verdict-grade finché A-TH-68/69 non mordono ⇒ il join INV(W) è a valle dei sigilli. Il giudice del checker è l'ORACLE per fixture (+ gate corpus/ORM per NOME): le reti lessicali non entrano nella sua catena di prova. Il checker corre in PARALLELO.
5. **A-TH-70 precede ogni dente nuovo.** check_pin è fail-open su ~25 denti: ogni riga nominata nuova del canale (A-DL-62) nasce con una cattura numerica. Aggiungere denti prima della guardia moltiplica la vacuità presunta (KS-TH-94-2).
6. **Ordine INTERNO del canale: prima la MISURA sui raw esistenti, dopo gli strumenti** — ma la misura è ammessa solo appaiata al controllo di decremento (one-off, binario esistente, zero campagne). I termini huge sono già nei 30 raw dentro mi_arena_json: servono un parse section-aware e il controllo, non una campagna. Gli strumenti (righe nominate, mappa byte chunk-bin, jbuf) servono al JOIN, non al primo indiziato.
7. **Ordine INTERNO di A-DS51 fase 2: si corregge il checker PRIMA del wiring.** E ogni correzione nasce con fixture oracle-morsa + pin length-prefixed PRIMA del codice (KS-DS-94-1). A-DS65 (registry seed builtin+tentative) è prerequisito del gate ORM/hk (KS-DS-94-2), quindi del wiring, non delle correzioni.

## CONFLITTI / CONFINI DA DICHIARARE

- **C1 — retroattività di KS-TH-94-2.** Se le righe OK dei denti non guardati sono VOID, cadono anche i PASS storici del gate-lever (WP-79..92)? Posizione del team: vacuità presunta **prospettiva** (nessuna cifra NUOVA li cita), verdetti d'epoca ancorati al loro epoch — precedente M84/M85 (WP-91). NON decidibile qui: al plenum / team-cifre via ledger.
- **C2 — proprietà del probe on-thread.** PT-1 (A-DL-61) e A-DL-52 census on-thread (P1 team-misura) insistono sullo stesso territorio. Confine proposto: PT-1 è del CANALE (è un pre-test di pluralità, non un censimento); il suo esito decide il piano di split di team-misura. Da concordare con team-misura prima di S-93.0.
- **C3 — riverifica VDL24/canary.** Leijen la chiede per NOME solo se la sorgente era il piano tls-dump. È atto di LEDGER (supersessione), non di engine: al team-cifre.
- **C4 — il one-off huge non è una cifra.** Il controllo di decremento produce meccanismo, non verdetto: ammesso prima che i sigilli mordano, purché nessun numero esca etichettato verdict-grade.

## ORDINE PROPOSTO PER S-93.0 (prerequisiti per NOME)

- **E1** A-TH-70 (guardia numericità universale + meta-dente su `-ne`/`-gt`) + A-TH-72 (emendare «num_or_void su OGNI cattura» → «12 catture per NOME», supersessione da ledger). Prereq: nessuno. *Sblocca ogni dente successivo.*
- **E2** (parallelo a E1) Controllo positivo decremento `malloc_huge` current, binario esistente. Prereq: nessuno; vincolo C4.
- **E3** A-TH-68 + A-TH-69 con decoy ESEGUITI stesso-commit; A-TH-71 in coda. Prereq: E1. *Estingue KS-TH-94-1.*
- **E4** Termine T-HUGE misurato sui 30 raw m90, parse SECTION-AWARE (KS-DL-94-3). Prereq: E2 (altrimenti VOID).
- **E5** A-DL-61 PT-1 in-band; poi A-DL-62 righe nominate + mappa byte chunk-bin, A-DL-63 controllo page_committed, A-DL-64 jbuf. Prereq: E1 (guardia), E4 (il termine nominato). Barriera A-DL-57/58 solo dopo PT-1 (KS-DL-94-2).
- **E6** Join INV(W)=T-HUGE+T-CHUNK+T-EAGER+residuo su W∈{4,8,12,16}, grammar v2. Prereq: E3, E5.
- **D1** (traccia parallela, giudice = oracle) A-DS61 iterable-desugar + A-DS62 ctor_proto da ctor abstract. Prereq: fixture oracle-morsa + pin length-prefixed PRIMA del codice.
- **D2** A-DS63 (set-hook ereditato, set-param untyped x2) + A-DS64 (final-const da iface, abstract-count sing/plur). Prereq: D1.
- **D3** A-DS65 registry seed (grafo builtin + tentative). Prereq: D2.
- **D4** Wiring fase 2 + gate ORM 3E/13F e hk 1665. Prereq: D3 (KS-DS-94-2) — mai prima.
