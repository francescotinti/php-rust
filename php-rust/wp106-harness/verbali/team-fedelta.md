# Team-FEDELTÀ (Stogov + Pedersen) — Concilio WP-106, fase 2

Relatore: sedia 8. Fonti: verbale-8-stogov.md, verbale-6-pedersen.md.
Nessuna refutazione capitale in nessuno dei due; entrambi CON EMENDAMENTI.

## (a) memory_get_usage — forma finale a catalogo + slot S-105

Forma finale (A-ST-106-1): voce 🔴 SUBITO a catalogo, insieme a
memory_get_peak_usage (stesso stub) e memory_reset_peak_usage (no-op),
con semantica DICHIARATA: functional-parity (monotono, ordine di
grandezza, peak vero, reset funzionante), MAI byte-parity — galloc−gfree
conta il processo intero mentre AG conta solo emalloc per-richiesta
(R-ST-106-2). Refutata la promozione degli atomics process-global:
conflaziona i worker (AG è per-thread) e mette 2 atomiche sul canale
calls appena inchiodato da H-D. Gradino release: contatore NETTO
per-thread TLS oppure query mi_* on-demand. **Slot S-105: catalogo +
gradino (i)** (priorità Stogov n.2). Promozione a release SOLO con A/B
sui sei giudici + disasm bl-count, criterio pre-registrato (KS-ST-106-1);
senza, resta census-only e la voce resta 🔴.

## (b) ordine di fedeltà — quanto entra in S-105

Ordine confermato: **generator get_gc** (container + descend catture;
arbitro = fixture rossa esistente) > **§3.13** marca (unit,line) con
fixture include+eval > **§3.12 regime (i)** weak+op-throws (bracci
strict e `.=` nel gate, KS-ST-105-1 resta). In S-105: generator +
§3.13 (priorità Stogov 3–4). §3.12-regime-i NON come voce autonoma:
viaggia DENTRO la fusione prop RMW, che vive nello stesso sito
(§3.11/§3.12) — la leva perf è anche il veicolo di fedeltà. Resto §3.12
a backlog. Risposta: FEDELTÀ, non assenza (A-ST-106-2).

## (c) server: pin fermo + scadenza operativa della coppia WP

Pin server fermo = sano SOLO con clausola negativa: nessuna osservazione
su 31aa7c2e citata come se parlasse di HEAD (R-PE-106-1). Cifre
server/WP citabili SOLO da pin same-HEAD del phpr firmatario E gradato
PIENO (option 413 + restapi 3508 per NOME, env -i, 2 modi, mode-probe)
— altrimenti VOID senza appello (KS-PE-106-1/A-PE-106-1).
**Scadenza composta**: la leva H-C3 di S-105 È il trigger (A-PE-106-2)
⇒ se spedita, la STESSA sessione salda il debito: rebuild server @ HEAD
di chiusura → grado PIENO → coppia WP sul pin stesso; il budget di
sessione della leva DEVE includere questo costo. Ramo fallback: se
nessuna leva entro chiusura S-106, PIENO comunque sul rebuild @ HEAD di
chiusura e il riferimento WP-102 (1,89×/2,64×) decade da baseline a
STORICO (KS-PE-106-2) — la crescita peak B>A firmata (7/7, p=0,0078)
resta non attribuita finché la coppia non gira. In più: retention per
NOME del backup uploads (A-PE-106-3).

## (d) fusioni op — posizione di H-C3 nella coda

**Prop RMW fuso (omologo ZEND_ASSIGN_OBJ_OP/PRE_INC_OBJ) DAVANTI alla
leva calls**: doppio rendimento (elimina alla fonte gran parte degli 11
DropS/iter + porta §3.11/§3.12-i nello stesso op) e il canale calls è
già inchiodato a 1 alloc×32 B — toccarlo ora con atomics è refutato.
Seconda: forme registro arith (ARCO REGISTRI, 23 transiti/iter S-102).
Vincolo di forma: fusione che SOSTITUISCE sequenze, mai specializzazione
che aggiunge varianti al match monolitico di run_loop (budget: taglia
pin 257.632 B). Ogni fusione col protocollo S-104: criterio prima,
disasm bl/byte prima-dopo, A/B da sola (KS-ST-106-2).

## Convergenze / conflitti / priorità

Convergenza forte: la leva H-C3 prop-RMW soddisfa contemporaneamente il
ritmo-leva, la fedeltà §3.11/§3.12-i e il trigger del debito server —
un solo atto scioglie tre vincoli. Nessun conflitto; unica tensione:
il costo rebuild+grado PIENO+coppia WP comprime la finestra della leva
in S-105 — si dichiara nel budget, non si rinvia.
Priorità S-105: 1) fusione prop RMW (protocollo S-104) → salda debito
server; 2) catalogo memory_get_usage + gradino (i); 3) generator
get_gc; 4) §3.13.
