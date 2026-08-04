# Concilio WP-99 — SINTESI DI CONVERGENZA (su S-97.0+S-97.1 e programma S-98.0)

## §FONDAMENTALI (prima di tutto, per regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: NON zero — è la sessione
più densa di misure dell'oggetto da molte settimane. Nuovo per NOME: il
divario `arith` è decomposto in fattori esatti; H-A1 eseguita fino in fondo
con verdetto numerico (19→11 opcode/iter, −30,7%); il costo per opcode
residuo è pinnato (9,87 ns contro 1,23 dell'oracle) e la sua NON-uniformità
confermata tre volte; due ipotesi (H1, H-A1) sono cadute su criterio, una
(H-A2) è spedita. Il metodo nuovo (criterio scritto prima) ha morso due
volte in due sessioni: funziona.

**(b) Contatore sessioni-senza-misura**: ultima full/media WordPress =
WP-94, 3 sessioni fa (per REGOLA della spina: WordPress è collaudo di
parità, non cronometro — l'emissione flag-off non è cambiata). Il micro,
che ORA è il cronometro dell'oggetto, ha girato due volte in coppia R=3.

**(c) Rischio d'oggetto più trascurato**: la parità del SERVER — il binario
php-server è stato ricostruito con H-A2 dentro (incondizionata) e nessun
gate server è girato (Pedersen: parità DOVUTA, non facoltativa; pin ruotato
832568a72b925dd1 senza verifica). Secondo: la roadmap footprint resta ferma
(Leijen: coppia peak dovuta al prossimo collaudo WP).

## Verdetti di fase 1 (9/9 CON EMENDAMENTI, nessun MI OPPONGO)

Verbali integrali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali, per convergenza:

1. **La forma letterale di H-B1 è FALSA** (Hoare+Matsakis, indipendenti):
   «frame in registro, ricaricato solo a call/ret/throw» ignora che gli
   archi di ri-entrata (gc_note, flush_diags, __toString, dtor) attraversano
   quasi ogni handler e che ogni diag legge `frames[top].ip`. Forma safe
   possibile: loop interno su split-borrow di campo, confine = ogni opcode
   che chiama metodi `&mut self`, `ip` come campo vero mai copiato.
2. **Il tetto di H-B1 è ~1,4 ns/op (−17%, banda 8–27%)** (Bak+Gregg,
   indipendenti, dallo stesso dato ha2-sweep: il dispatch noop costa ~1,4
   ns). Il fattore ~8 residuo vive nei CORPI degli handler ⇒ **H-B1
   declassata a sotto-passo, H-B2 (specializzazione per tipo) promossa ad
   asse**. Bak: il criterio −40% di H-A1 era irraggiungibile a tavolino con
   dati già posseduti (la scala uniforme dava −42%) — il criterio del
   prossimo passo si deriva dal TETTO, non dal desiderio.
3. **Il PASS del gate era parzialmente vacuo** (Klabnik): WP_SESSION_97
   registrato judge=no (= escluso) mentre il report ne vantava il PASS; i
   tre .out di S-97.0 senza riga. RIPARATO in chiusura (WP_SESSION_97
   judge=yes → il gate ha subito morso una cifra irrisolta, corretta;
   esclusioni dichiarate per i .out). Resta a S-98.0: delibera su 94/95 e
   ri-collaudo dello strumento census (il pin 9,87 ns ne dipende).
4. **Il harness di test applica il pass in un punto di pipeline diverso
   dalla produzione** (Hejlsberg): `lowered()` gira post-cessione WP-65 —
   batteria potenzialmente vacua sui 13 snippet top-level. La vacuità è di
   COPERTURA TEST, non di produzione (il census CLI ha foldato il {main}:
   19→11 col dump che mostra le forme). Chiusura: assert positivo di forme
   registro nel {main} lowered + batteria spostata sul funnel vero.
5. **Il confine del flag è attaccabile da PHP** (Pedersen): putenv→set_var
   può decidere il modo del worker prima della prima lettura del OnceLock.
   Cura: lettura EAGER all'avvio + dente anti-putenv + fail-loud su
   unwrap_or(b"").
6. **«I confronti non lanciano» è falso in generale** (Stogov): opCoerce
   branda Left/Right; il mirror Lt↔Gt vive su tre contingenze non
   dichiarate — vanno pinnate in property-test di antisimmetria + fixture
   GMP/Number PRIMA di estendere i fold. Le sette fixture-trappola
   dell'eventuale fold AssignOp si scrivono PRIMA del fold.
7. **Igiene di misura** (Gregg): str/re non si dichiarano «rumore» senza
   banda (Δ0,04 < spread 0,08 va scritto così); deriva inter-build
   dichiarata; claim < 3× spread = nulli.

## Ordine proposto per S-98.0 (regola di ammissione applicata)

1. **M1 (misura, ~mezz'ora, PRIMA di ogni codice)**: micro noop 200M +
   census noop/prop/calls + ASM del preambolo corrente; predizione P
   scritta nel .out (P = 19·D/156,6 per arith). Se P < 10%: H-B1 cade a
   tavolino e si passa a H-B2 senza scriverla.
2. **Decisione H-B1 dal numero**: se sopravvive, forma = split-borrow del
   team forma-hb1, criterio = max(P/2, 0,7 ns/op) con caduta se arith
   flag-off > 7,2 s; KS: no unsafe/raw ptr, no mem::take, no ip locale.
3. **H-B2 (asse)**: specializzare UN opcode (Binary Add int-int deciso a
   compilazione) e misurare `arith`; guardia contata (KS-ST-99-3).
4. **Debiti bloccanti ammessi** (apparato che blocca, timebox ½ sessione
   in tutto): parità server restapi+option per NOME sotto env -i (blocca
   OGNI uso del server col binario nuovo — pin attuale NON verificato);
   smoke flag-ON con controllo positivo dump nel CI di sessione; assert
   {main} della batteria (chiude la vacuità di copertura).
5. **A BACKLOG per NOME** (non bloccano l'oggetto): delibera manifest
   94/95; property-test antisimmetria mirror + fixture GMP/Number; flag
   eager + dente anti-putenv; dente N_OPS<256; coppia peak al prossimo
   collaudo WP; bande str/re nel prossimo run-micro; fold coda AssignOp
   (dopo le sette trappole di Stogov).

L'ordine NON è composto di solo apparato: i punti 1-3 sono l'oggetto.

## Conflitti registrati (nessuno di sostanza)

- forma-hb1: sonda-per-addizione (Matsakis) vs sola predizione (Hoare) —
  risolto dall'ordine: M1 è la sonda, la predizione la accompagna.
- team-giudice: B1-prima-di-B2 vs tetto-misura: B2-asse — riconciliato al
  punto 2: B1 sopravvive SOLO se M1 le dà un numero sopra soglia.
