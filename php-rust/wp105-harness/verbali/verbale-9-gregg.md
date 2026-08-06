# Verbale sedia 9 — GREGG (metodologia di misura, attribuzione) — Concilio WP-105, mandato INVERSO

## §FONDAMENTALI

**(a) Δ-oggetto.** S-103 produce conoscenza VERA dell'oggetto: drop-census
11 DropS+3 DropC/iter esatti (linearità 300:1), cifra H-D 1 alloc×32,0 B
+1 free/chiamata realloc≡0, generator-in-cycle che perde per TUTTA la
richiesta (fixture rossa), §3.12 typed-LVALUE 4/4 specie. Questo È sapere
su phpr che ieri non c'era. Ma i sei rapporti sono fermi da DUE sessioni
(prop 11,5 · calls 7,4) e il contatore segna 2. Il prezzo è giusto UNA
volta sola, e solo perché la bozza S-104 apre finalmente la leva. Audit
prefissi della bozza: il punto 2 non ha prefissi residui (predicato,
criterio, fixture: tutti consumati) — ma il punto 1 è un prefisso
POTENZIALE mascherato: se il R=7 esce VOID, il «ridisegno della metrica»
può mangiarsi la sessione prima della leva. Va sequenziato (A-GR-105-4).
La «coppia WP bimodale» nel gate del punto 2 non è prefisso: è il debito.

**(b) Il denominatore 2×.** L'errore è sopravvissuto a una sessione E a
un concilio perché la cifra «2 alloc/chiamata» citava un denominatore
tenuto A MEMORIA (10M), mai emesso da nessun run. La regola che l'avrebbe
preso: **il giudice emette il proprio N a runtime, e ogni cifra per-iter
si firma SOLO dividendo per l'N stampato nell'output del run stesso** —
non dal sorgente riletto (fallibile), non dalla memoria (provato
fallace). «Rileggere il sorgente» (lezione S-103) è il rimedio; l'N
auto-emesso è la prevenzione: rende l'errore impossibile, non solo
scoperto.

**(c) Audit A/B.** L'audit pre-registrato che ribalta «RUMORE» in VOID è
metodo corretto: una banda scelta dal run giudicato si assolve da sola.
Il 5/5 stesso segno B>A (p≈0,03 al sign-test sotto nulla) è indizio
legittimo, correttamente NON promosso. Il rerun R=7+warmup aggredisce la
sola sorgente di rumore DIAGNOSTICATA (coppia 1 fredda, ~85 MiB su
entrambi i bracci). Criterio di STOP: **il R=7 è l'ultimo tentativo della
metrica full-peak su questo quesito**. Un secondo VOID sotto protocollo
valido, con la sorgente diagnosticata già rimossa, significa che il
rumore residuo non ha nome ⇒ la metrica è INADATTA e si passa alla misura
per-fase (peak per finestra di richiesta), design pre-registrato. Nessun
terzo rerun, nemmeno «non cieco».

## VERDETTO

Sessione metodologicamente ONESTA (banda pre-registrata, attese v2 prima
del dinamico, correzione ammessa in pubblico) ma seconda consecutiva a
rapporti fermi: **soglia dell'anomalia raggiunta, non superata**. S-104
si giudica su UN fatto: l'A/B della leva H-C2 contro `hc2-criterio.out`.

## Refutazioni

- R-GR-105-1: la banda «tra-sere» ha 2 punti dello STESSO giorno di
  calendario (mattina/pomeriggio 2026-08-06): misura varianza intra-day,
  non tra-sere. Come requisito «≥3 sere» i punti 1-2 valgono UNO.
- R-GR-105-2: «i rapporti NON cambiano per costruzione» è un claim di
  costruzione, non una misura — accettabile stavolta (corpus/batteria/
  server ×2), ma non crea precedente citabile.
- R-GR-105-3: gli spread «caleREBBERO a ~13,8/47,9» escludendo coppia 1 è
  correttamente marcato esercizio; resta NON citabile in ogni sede.

## Emendamenti

- **A-GR-105-1**: ogni micro-giudice emette N iterazioni a runtime; ogni
  cifra per-iter cita l'N emesso dal run, pena VOID di citazione.
  Retroattivo: tavola bi-regime S-102 corretta ×½ con N emesso.
- **A-GR-105-2**: STOP full-peak = R=7 ultimo tentativo; VOID ⇒ metrica
  dichiarata inadatta, misura per-fase con design pre-registrato.
- **A-GR-105-3**: banda tra-sere nominabile solo con ≥3 punti su ≥2
  giorni di calendario distinti; fino ad allora KS-GR-104-1 pieno.
- **A-GR-105-4**: in S-104 la lettura del verdetto R=7 è atto breve; se
  VOID, il ridisegno per-fase va DOPO la leva H-C2, mai prima.

## Kill-switch

- **KS-GR-105-1**: se S-104 chiude senza l'A/B della leva H-C2 eseguito
  (qualunque verdetto), contatore = 3 ⇒ anomalia DICHIARATA; WP-106 apre
  con riallocazione obbligatoria fondamentali-first.
- **KS-GR-105-2**: cifra per-iter senza N emesso dal run = mai citabile.

## Priorità S-104 (ordine di Gregg)

1. Leva H-C2: implementazione → A/B vs criterio → gate pieno + coppia WP
   (salda il debito). 2. Lettura verdetto R=7 (breve; VOID ⇒ A-GR-105-2).
3. H-D SiteTag (residuo≡0). 4. Igiene: tavola ×½ con N emesso · terzo
punto banda su GIORNO distinto · decisione generator-in-cycle.
