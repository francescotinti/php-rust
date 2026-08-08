# Verbale sedia Gregg — lente: metodologia di misura e attribuzione (S-116, concilio di rotta)

## COSA SAPPIAMO OGGI DI PHPR CHE PRIMA NON SAPEVAMO (S-113..116, fatti secchi)
1. **La banda-layout esiste ed è misurata**: leve NULLE spostano le micro fino a 10 ns/iter per categoria (re 0→10 tra N=1 e N=2); una nulla fa 5/5 segni concordi su 3 categorie. Una banda a N=1 mente.
2. **Le leve singole (3-30 ns) affogano nel layout**: H-P1 (+3,33) è indistinguibile dal nulla; solo L-A (+26/29/30, tre campioni, spread_A depurato 2-4) emerge.
3. **La tassa calls è SISTEMATICA, non banda**: L-A −6,50/−7,00/−6,50 tutti oltre le due nulle −5,50 identiche → ~1-1,5 ns/iter reali su un sentiero non toccato nei dump. L'attribuzione «layout/icache del probe» è ipotesi nominata, non firmata.
4. **Il gate held-out a soglia fissa è REFUTATO come diagnostico**: la nulla-2 lo sfonderebbe (9,80>9,71) a semantica zero.
5. **Costo/op ~9-10 ns quasi invariante tra categorie (S-103)**: il collo è il ciclo di vita degli Zval, non i singoli opcode.
6. **Le famiglie 1,3×min con esclusione per NOME recuperano il metro senza toccare la leva** (spread 47→2); i binari CONSERVATI (zero rebuild) rendono le bande riusabili.

## VERDETTO: CONCORDO CON EMENDAMENTI
(su: A subito / B regime / D metodo / C riserva)

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)
Ordine: **A (S-117) → B con gate netto pesato (S-118) → istruttoria C in parallelo (da S-118/119) → D come cava di vagoni**. Due refutazioni alla raccomandazione:
- **A non «ripara» il metro per fede**: PGO cambia il layout in funzione del profilo — se il profilo non è versionato e deterministico, A AGGIUNGE una sorgente di rumore. E **BOLT non supporta Mach-O** (target ELF): su Darwin la via è PGO (llvm-profdata) + **ld64 `-order_file`** per l'ordinamento deterministico delle funzioni. A va trattata come ipotesi con banda pre/post, non come cura.
- **C non è «riserva»: l'aritmetica è già computabile OGGI**. Prop pin ~107 ns/iter vs obiettivo 3× ≈ 42: servono ~65 ns; la migliore leva mai vista ne vale 29; A promette 5-15%. Fatto 5 (invarianza) dice che il pavimento è lo Zval. Aspettare 2-3 sessioni per «scoprire» che A+B non bastano è spreco pre-registrabile: l'istruttoria C (design + strumenti, niente codice sul path caldo) parte comunque.

**Mossa concreta S-117**: (i) feasibility spike PGO+order_file (mezza sessione, timebox REGOLE §1): build passa batteria 1742/0 + corpus 1415 per NOME ×2 + parità output; (ii) su pipeline nuova: micro R=5 + **2 leve nulle** (patch s114/s115 riusate) → banda_new; (iii) test mirato: L-A ricompilata sotto PGO — se la tassa calls rientra nella banda nulla nuova, l'ipotesi «probe/layout» è firmata e il metro è riparato nei fatti.

## EMENDAMENTI
- **R1 (feasibility)**: niente BOLT su Darwin; A = PGO + order_file. Misura: build riproducibile (2 build stesso sorgente → hash identico = definizione operativa di «layout deterministico»).
- **R2 (doppio criterio per A, pre-registrato)**: A promossa come RIPARA-METRO solo se banda_new ≤ ½ banda_old (globale 10→≤5) su ≥2 nulle; altrimenti promuovibile come sola leva velocità (Δ micro/WP sopra banda_new). Mai fondere i due claim.
- **R3 (reset dichiarato)**: A invalida TUTTE le bande e baseline: nuovo pin, micro R=5, held-out e bande rifatte sulla pipeline nuova. Riusare bande vecchie su binari PGO = vietato.
- **R4 (gate del treno B)**: giudizio sul NETTO pesato tra categorie con regressioni cap = banda_new; vagoni ammessi solo con direzione firmata su binari conservati; protocollo di espulsione leave-one-out pre-registrato, MAX 1 giro, altrimenti il treno cade intero.
- **R5 (tassa calls)**: primo lavoro-vagone = collocazione cold/outlining del probe con dente disasm bl-count (già nominato nel verdetto S-116) — oppure assolto gratis da A (test R5 = punto iii della mossa).
- **R6 (C non condizionale)**: pre-registrare la proiezione aritmetica; design doc C entro S-119 comunque.

## KILL-SWITCH (pre-registrati)
- **KS1**: build PGO fallisce parità (batteria/corpus per NOME/output) → A abbandonata in sessione, pipeline revertata.
- **KS2**: banda_new ≥ banda_old E Δ velocità sotto banda → A revertata intera; non tenere una pipeline che complica il build senza pagare.
- **KS3**: treno B fallisce il netto dopo 1 giro di espulsione → smontato, vagoni in coda singola; niente ricomposizione sugli stessi binari.
- **KS4**: 2 sessioni di solo apparato-A senza un A/B misurato → anomalia dichiarata, stop rotta.

## APPARATO minimo (blocca l'oggetto)
Script pipeline in `scripts/` (variante pin-phpr.sh con passo profilo: input = 6 micro + WP script, profdata hashato e committato); kit leva-nulla riusabile (patch zavorra versionate: apply/misura/revert in un atto); rc dei gate da FILE, mai da pipe.
