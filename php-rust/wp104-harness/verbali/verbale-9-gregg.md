# Verbale sedia 9 — Gregg (metodologia di misura, attribuzione) — Concilio WP-104 (su S-102), MANDATO INVERSO

**VERDETTO: S-102 AMMESSA come sessione d'oggetto — con UNA refutazione
capitale RETROATTIVA e due contratti di misura nuovi.**

## (a) L'oggetto è avanzato?

Sì, in CONOSCENZA e fedeltà, non in velocità. I rapporti fermi (prop 11,5)
sono il prezzo ORDINATO dal WP-103, non l'apparato che si autogiudica: la
prova è che gli esiti principali sono fatti SUL MOTORE (il call-path alloca;
23 transiti/iter; §3.13 corretto con un phpt guadagnato; parità
capture-boundary provata). Verità scomoda: se S-103 non produce un Δ-oggetto
misurato (H-C2 promossa O refutata dal suo criterio), due sessioni a rapporti
fermi diventano deriva d'apparato — serve il limite nominato (A-GR-104-3).

## (b) I due rischi nominati in WP-103

- Rumore full-peak phpr: MISURATO ✓ — 34,64 MiB su 1896,91 (~1,8%), R=5,
  mediana (statistica giusta per una coda). Rischio chiuso.
- Il **21,2% di run_loop resta senza nome E senza strumento**: la leva-nulla
  di taratura (A-BA-103-4) — unico calibro dell'errore di attribuzione a
  campioni — è slittata a condizione degli slot-diretti. Slittamento
  IMPROPRIO: era il calibro del PROFILO (l'unico strumento che nomina il
  21,2% e il 26,6%), non di una leva. Sì: è oggi il rischio più trascurato.

## (c) Banda tra-sere del giudice micro

Serve come NUMERO. Oggi «tra-sere» è un'etichetta senza contratto: n=1
coppia di sere (±0,4 osservato) non è una banda. Senza il numero, ogni
lettura cross-sessione dei rapporti è NON FALSIFICABILE. → A-GR-104-1.

## (d) A/B peak in volo

Il verdetto meccanico si accetta solo DOPO l'audit della finestra: (i) coppie
adiacenti a segno opposto ≥3/5 ⇒ si ripete (già pre-registrata, bene);
(ii) DIFETTO nella regola: banda = max(spread_A, spread_B) della fase 2
stessa ⇒ una finestra rumorosa GONFIA la banda e premia la chiusura «RUMORE».
Tetto: spread di fase 2 > ~1,5× i 34,64 MiB di fase 1 ⇒ ripetere, non
chiudere. → A-GR-104-2 / KS-GR-104-2.

## (e) Contatore sessioni-senza-misura

Corretto: 0 (coppia bimodale + giudice R=5 stessa sera in S-102). ✓

## REFUTAZIONE CAPITALE

**RC-GR-104-1** — I «trasversali» di S-101 (arith 12,7→12,2; calls 7,9→7,3)
sono movimenti TRA-SERE mai passati da un A/B stessa sera; S-102 mostra ±0,4
bidirezionali sulla stessa finestra e calls 7,3→7,7 ha RIMANGIATO l'intero
«guadagno». Si declassano a INDIZIO nel record (anche in SYNTHESIS WP-103
§FONDAMENTALI dove sono citati come esito). Verdict-grade restano SOLO i
Δ ns/iter A/B di H-C1a+b su prop. (Narrazione refutata, non il guadagno
prop — feedback-keep-partial-wins.)

## Emendamenti

- **A-GR-104-1**: misurare la banda tra-sere del giudice — stesso pin, R=5,
  ≥3 sere distinte, banda PER CATEGORIA pubblicata (~15 min in coda ai
  pre-flight; nessuno slot dedicato).
- **A-GR-104-2**: audit finestra dell'A/B peak PRIMA del verdetto: segni
  delle coppie + tetto spread 1,5× fase 1.
- **A-GR-104-3**: contatore gemello «sessioni-senza-Δ-oggetto-misurato»
  nella riga ⏱ (leva promossa O refutata dal suo criterio; oggi = 1).
- **A-GR-104-4**: riconciliare la contabilità di calls PRIMA di ogni leva
  H-D: 10 gc_note/iter (S-102) vs 5 nominate (S-101); out=2×iter — definire
  UNA volta l'unità «per chiamata».

## KS (kill-switch)

- **KS-GR-104-1**: movimento di rapporto tra sessioni narrato come effetto
  SENZA A/B stessa sera = reject della narrazione.
- **KS-GR-104-2**: chiusura «RUMORE» dell'A/B peak con spread di finestra
  oltre il tetto = VOID; si ripete, non si interpreta.

## Cosa sappiamo oggi che ieri non sapevamo (lista secca)

1. Il call-path di phpr ALLOCA: ~2 alloc + ~2 free/chiamata (~35 B), churn
   bilanciato invisibile alle pagine.
2. alloc/iter≡0 su prop è VERDICT-grade: +2 eventi su 29,9M iter.
3. Il denominatore della pila è 23 transiti-sorgente/iter — statico
   confermato dal dinamico, lineare 300:1.
4. Il rumore full-peak phpr è ~1,8%: la gamba È bisecabile (il ~10%
   dell'oracle non era trasferibile).
5. §3.13: il warning si timbra alla riga dell'ACCODAMENTO; il fix fa
   passare `nullsafe_operator/015.phpt` (corpus 1417).
6. Il residuo `Binary(Add)` fuori-funnel esiste ed è pinnato per nome.
7. La parità capture-boundary (`__destruct`/shutdown → request_end) regge:
   3 richieste stesso worker, byte-id all'oracle, nei 2 modi.
8. I movimenti tra-sere del giudice sono ~±0,4 — grandi quanto i
   «trasversali» narrati in S-101: da qui RC-GR-104-1.
