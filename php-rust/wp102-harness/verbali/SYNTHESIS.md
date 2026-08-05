# Concilio WP-102 — SINTESI DI CONVERGENZA (su S-100 e programma S-101)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: Gregg (mandato
inverso) dà PASS pieno — «sessione d'oggetto piena». Nuovo per NOME: il
FLIP è ESEGUITO E MISURATO (flag-on default, corpus 1418×2 per NOME sul
pin, diff per-test ZERO, server bimodale, coppia WP nei 2 modi con bande);
H-B2 chiusa con misura (estensione, L=12,9→[−1,0]; add on −31%); H-C
DECOMPOSTA (12,4 = conteggio 2,0 × costo/op 6,2) con profilo co-equale e
simboli per NOME; due divergenze semantiche nuove a catalogo.

**(b) Contatore sessioni-senza-misura**: full/media = **WP-100 = QUESTA
sessione (0)**; giudice: prop e add freschi post-flip (le altre quattro
categorie da ri-baseline in modo default — primo atto S-101).

**(c) Rischio d'oggetto più trascurato**: la crescita d'albero del peak
(voce RINOMINATA dal concilio: **OFF +95 / ON +36,5 MiB** vs WP-99, «cross-
albero» era una conclusione non ancora provata contro l'ambiente) è senza
attribuzione; e il costo/op quasi-invariante ~9-10 ns resta senza
decomposizione DENTRO run_loop (il 50% del profilo è un simbolo solo).

## Verdetti di fase 1 (9/9: nessun MI OPPONGO; capitali per convergenza)

Verbali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali:

1. **`emit_binary` leggeva `enabled()` globale, non `ctx.reg_lower`**
   (Hoare A-HO-102-1 + Hejlsberg, convergenti indipendenti; mod.rs:779):
   «il modo è un INPUT del funnel» era falsificato da un sito residuo — il
   braccio OFF in-process dei test emetteva `Binary(Add)` sotto default ON
   invece dell'emissione di produzione OFF. **SALDATA IN SESSIONE**: fix a
   `self.ctx.reg_lower`, denti mirati verdi, batteria 1735/0, pin ruotati
   (b618e3a).
2. **«Il flip cambia solo la costante» è falso** (Klabnik): fb861e4 ricabla
   l'entry (ProgramCtx) — l'evidenza del diff per-test va giudicata SUL
   PIN (A-KL-102-1). **IN SALDO IN SESSIONE**: corpus-diff ri-eseguito sul
   pin post-fix (copre anche il collaudo del fix n.1).
3. **La parità bimodale server non provava il modo effettivo** (Pedersen
   A-PE-102-1): fails=0 nei due bracci è indistinguibile da env mai
   propagato. **SALDATA IN SESSIONE**: mode-probe nella sentinella (dump
   dell'unità nel log del server, atteso dal contratto), esercitata nel
   ri-collaudo del pin.
4. **H-C1 «prestito al posto del clone» punta al canale sbagliato**
   (Bak + Matsakis + Stogov, convergenti): su prop.php i valori sono Long
   (copy senza alloc) — il churn nominato dal profilo è il RICEVITORE
   (Rc dell'oggetto + gc_note), prior art `ThisPropGet`; Zend fa
   copy+addref CONDIZIONALE, mai borrow nudo. **RIFORMULATA dal team
   hc-canale** (sotto), il borrow nudo dello slot valore è VIETATO
   all'unanimità.
5. **«Cross-albero» non provato + peak pubblicato per un binario mai
   misurato** (Leijen): l'attribuzione esige A/B dei pin stessa-sera PRIMA
   del bisect; ogni cifra peak nomina l'HASH del binario che l'ha
   prodotta. Voce rinominata: «crescita d'albero OFF+95 / ON+36,5».
6. Metodo (Gregg, non capitali): la «tariffa» 9-10 ns/op viene da 2 punti
   (arith, prop) — banda, mai coefficiente; il 27% Zval è un PAVIMENTO
   (run_loop inlinea altro ciclo-vita); la sanity 2,0×6,2=12,4 è esatta
   per costruzione — serve il census dinamico a validare lo statico.

## Riformulazione H-C1 (team hc-canale, VINCOLANTE per S-101)

H-C1 sostituita da FORMA A STADI, ognuno col suo controfattuale e criterio:
- **H-C1a**: bypass del bookkeeping per SCALARI (niente gc_note/clone-
  contabile per Long/Double/Bool/Null) — misurabile da sola.
- **H-C1b**: prestito/addref del RICEVITORE à la `ThisPropGet` (sigillo di
  tipo sul borrow; fixture aliasing/hook/__get/ref/readonly PRIMA).
- **H-C1c**: copy+addref condizionale stile Zend per i valori refcounted.
Prerequisiti di misura PRIMA di ogni riga: ri-baseline sei categorie in
modo default; census dinamico specie×sito×canale su prop.php nei DUE motori
con TRE predizioni pre-registrate; TETTO scritto nell'ordine: il successo
pieno di H-C1a-c lascia ~9× (KS-BA-102-1/KS-KL-102-2) — H-C1 NON chiude
H-C da sola, chi presenta il contrario è refutato in anticipo.

## Ordine DEFINITIVO S-101 (regola di ammissione applicata)

1. **Ri-baseline sei categorie IN MODO DEFAULT** sui due motori, stessa
   finestra (i numeri che giudicano tutto il resto).
2. **Census dinamico specie×sito×canale** su prop.php (due motori,
   predizioni pre-registrate) + decomposizione inline-aware del 50%
   run_loop (A-BA-102-2/A-GR-102-2): il census DINAMICO valida lo statico.
3. **H-C1a → b → c** nella forma a stadi (fixture semantiche PRIMA,
   criterio per stadio, tetto dichiarato); gate cumulativi ad ogni stadio:
   batteria + corpus 2 modi + diff per-test; WP pair di parità NON
   derogabile se cambia il runtime (KS-KL-102-3/KS-MA-102-4).
4. **Attribuzione crescita d'albero**: A/B pin S-99↔S-100 stessa-sera
   (prima del bisect) + census allocatore; bande peak UNILATERALI
   calibrate sul rumore misurato per-motore (R≥5, mediana, spread
   pubblicato; banda < rumore ⇒ VOID, KS-GR-102-2/KS-LE).
5. **Denti residui di rotazione** (piccoli, bloccano la fiducia nei gate):
   dente in-process BinaryAdd sul braccio OFF (A-HE-102-1); dente
   absent≡`=1` (A-KL-102-3); smoke peak sul pin quando una cifra peak
   entra nel registro (A-LE-102-4).
6. (timebox) H-D prima misura: stessa tavola su calls.php col census
   bi-regime (A-ST-102-5).

**BACKLOG per NOME** (non slot di sessione): A-HO-102-2 (sigillo ZST),
A-HO-102-4 (corpo condiviso BinaryAdd≡Binary(Add)), A-HE-102-2 (batteria
nei 2 modi espliciti), A-HE-102-3 (destructuring CompiledClass),
A-HE-102-5 (modo nel Module — chiude A-HE-101-3), A-HE-102-6 (dump
ereditarietà), A-KL-102-2 (carve-out per token + prova entropia flag-on),
A-PE-102-2/3/4 (sigillo STUB_ELISION/UNIT_CACHE; endurance N≥100; WP vero
via HTTP), A-ST-102-1/2 (riscrivere §3.12 mode/typed-prop col meccanismo C
verificato; §3.11 famiglia fetch-undef), A-BA-102-3 (fixture per specie),
A-GR-102-3 (bande asimmetriche), --build-info (A-HO-101-4/A-PE-101-5).

## Conflitti registrati

- forma del fix emit_binary (Hoare: variante forte incondizionata; il
  fix applicato usa ctx — registrato, non bloccante).
- priorità del sigillo ZST (Hoare: S-101; relatore team flip-residuo:
  backlog — resta BACKLOG per regola di ammissione).
- prestito del ricevitore (Matsakis: borrow col sigillo di tipo; Stogov:
  addref; team: si decide con le fixture — chi morde vince).
- verdetti divergenti su S-100 (Gregg 0 capitali, Leijen 2, Pedersen 1):
  domini disgiunti, nessuna contraddizione di merito.
