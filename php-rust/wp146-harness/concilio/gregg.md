# Verbale GREGG — Concilio S-146 (quesito B3/filone conteggi) — bozza indipendente

## §MANDATO-INVERSO — che cosa sappiamo OGGI di phpr che ieri non sapevamo
1. Il ciclo per-movimento è PAVIMENTO-dominato: memcpy 69,5% (1,06 s) vs inc-dec 0,21 s vs nota 0,25 s (SONDA, regola p.3 pre-registrata). Rendere il movimento più economico (B1/B2) è morto per misura; può pagare solo muovere MENO.
2. Abbiamo PREZZI per-movimento firmati (2,88–3,85 ns/coppia, per tipo) e CONTEGGI ORM (367,6M): qualunque conteggio futuro si converte in SECONDI — il moltiplicatore SCREEN 4,5–6,5% è pensionabile.
3. Il perimetro modellato ha un TETTO assoluto: 1,52 s su 37,6 s di gap (memcpy 1,06 s). Coerenza esterna: gcnote 238,6M == dossier S-141 ESATTO.
4. churn_zval è IN budget vs oracle (whole-stack 0,24–0,26pp ≪ 5,15pp); memops FUORI (62–66% ≥50%): metà del bersaglio storico di B è fuori budget in attesa di attribuzione Zval-move.
Che cosa NON sappiamo né oggi né ieri: la liveness su ORM (tutti i numeri liveness sono media group WP) e il PONTE tra le due convenzioni di conteggio (slot_reads vs movimenti).

## VERDETTO
**CONCORDO CON EMENDAMENTI** sul quesito. Ordine d'istruttoria dalla mia lente: (1) census F1-su-ORM + provenienza per sito, con gate aritmetico pre-registrato (R3); (2) solo se la banda sopravvive si istruiscono forma (a) e perimetro (b); le estensioni borrow-first (d) procedono in parallelo: non richiedono liveness.

- **a) CON EMENDAMENTI.** «Il flag take non è un corpo caldo in più» è un'ASSERZIONE, non una misura: si istruisce solo con taglia `nm -S` PREDETTA + disasm bl-count prima/dopo pre-registrati (criterio-B p.4, A-LB-97-1). Non valutabile prima del census.
- **b) CON EMENDAMENTI.** Il nucleo-stringhe è un perimetro derivato dai numeri WP (str 18,7% di slot_reads_rc); trasferirlo a ORM è un denominatore a memoria (veto vigente). Il perimetro si sceglie DOPO i conteggi ORM — la composizione differisce (str = 27,6% del galloc ORM; mv_str 104,1M vs mv_scalar 91,1M nella sonda).
- **c) CONCORDO: è il numero che manca.** Con tre obblighi (R1–R3).
- **d) CON EMENDAMENTI:** borrow-first di pari grado, ordinata dal census di provenienza (R5); **arena-conteggi si ARCHIVIA** se nessuno la definisce con criterio proprio ≤10 righe — una leva senza giudice nominabile non si istruisce (az.rev. S-145 #4).
- **e) CONCORDO** (Matsakis R4 confermato): tetto modellato 1,52 s = contributo di tappa, mai parità; anche al tetto (irraggiungibile) B3 compra ≤2,8% del gap. Nessun claim oltre la risoluzione; i ~4,4 s di glue restano fuori modello.

## Emendamenti R1..R5 (cosa/perché/misura)
**R1 — Ponte di convenzione.** slot_reads (census: letture di slot) e «movimenti» (sonda: coppie clone+drop) NON sono la stessa grandezza — i 367,6M includono nascite/args/prop fuori dagli slot. Il census F1-ORM emette NELLA STESSA RUN slot_reads_rc(ORM), would_take_safe(ORM) E un contatore-ponte (movimenti con origine slot-read / totale confrontabile col 367,6M), denominatori dal sorgente; VIETATO dividere would_take (convenzione census) per 367,6M (convenzione sonda). Monobinario census, ×2 repliche, r1==r2 per chiave.
**R2 — Pensionare lo SCREEN.** Banda d'attesa B3 = would_take_safe per-tipo × prezzo pair per-tipo della sonda = SECONDI (grade derivato CENSUS×SONDA, dichiarato). Mai più il 4,5–6,5% R=1 del media group WP su ORM.
**R3 — Kill aritmetico di visibilità (pre-registrato PRIMA del census).** Se l'estremo ALTO della banda R2 < risoluzione del giudice della scommessa (±0,7% della coppia ORM net ≈ 0,26–0,30 s), B3-TakeSlot NON si apre: ucciso a tavolino, zero codice.
**R4 — Memops resta VOCE PROPRIA.** B3 non chiude l'attribuzione Zval-move: un calo della famiglia memops post-B3 tra binari diversi è solo DIREZIONE (veto «differenze tra A/B distinti come cifra»). Giudici di B3 = micro churn + coppia ORM, MAI la quota memops.
**R5 — Census di provenienza per SITO dei 367,6M**: ranking misurato dei bersagli borrow-first (famiglia FR1-ext), stesso principio del «moltiplicatore» interno di B1.

## Kill-switch
**KS-G1** = R3 (banda sotto risoluzione ⇒ B3 chiuso senza codice). **KS-G2**: contatore-ponte non definibile dal sorgente, o r1≠r2 >1% ⇒ nessuna banda derivata, riconvoca. **KS-G3**: qualunque fetta B3 giudicata sulla quota memops ⇒ STOP (giudice sbagliato).
