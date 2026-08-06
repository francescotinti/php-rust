# Team «misura» — Concilio WP-104, fase 2
Relatore: sedia 5 (Bak) · membri: 5 Bak, 7 Leijen, 9 Gregg
Fonti: verbale-5-bak.md · verbale-7-leijen.md · verbale-9-gregg.md

## 1) CONVERGENZE (per NOME)

- **C-1 «Canale mai contato ⇒ nessun criterio pre-registrato»** — Bak
  KS-BA-104-3 (generalizza KS-GR-103-1 di Gregg dai ns ai conteggi) ∥ Bak
  A-BA-104-3 (drop-census sulla guardia H-C1a PRIMA della banda H-C2) ∥
  Leijen RC-LE-104-1 (la cifra «2 alloc/~35 B» non è verdict-grade). Tre
  sedie, stessa legge: la banda si deriva dal canale CONTATO, mai da stime
  statiche o medie lorde.
- **C-2 «Attribuzione H-D senza sampling»** — Bak A-BA-104-4 ≡ Leijen
  A-LE-104-3: istogramma per size-class in CountingMi + tag thread-local
  RAII di sito, chiusura contabile (somma per-sito = 2/iter, residuo≡0);
  vietati backtrace e sampling (20M eventi). Indiziato primo condiviso:
  `ret_cell` Rc fresco per chiamata (run.rs ~3459). Gregg concorre con
  A-GR-104-4: riconciliare l'unità «per chiamata» (10 vs 5 gc_note/iter)
  PRIMA di ogni leva H-D.
- **C-3 «Denominatori per specie, mai aggregati»** — Bak KS-BA-104-1
  (÷23 = tariffa media inesistente; il Δ si divide per il sotto-insieme
  sito×primitiva che la leva NOMINA, A-BA-104-1) ∥ Leijen p.2 (media lorda
  35,25 B include l'avvio; il netto è 32,0 B; il fingerprint si fa sul
  netto) ∥ Leijen p.1 (alloc fresca vs realloc = specie diverse, leve
  diverse). Stesso principio applicato a pila e alloc.
- **C-4 «Audit della finestra A/B peak, tetto 1,5×»** — Gregg A-GR-104-2 +
  KS-GR-104-2 ≡ Leijen KS-LE-104-2: spread di fase 2 > 1,5× la banda quieta
  di fase 1 (34,64 MiB) ⇒ il verdetto «RUMORE» è VOID, si ripete in finestra
  quieta; il buco è asimmetrico (banda gonfia premia la chiusura). Coppie
  adiacenti a segno opposto ≥3/5 ⇒ ripetere.
- **C-5 «Le bande sono numeri locali, non etichette»** — Gregg A-GR-104-1
  (banda tra-sere del giudice = NUMERO: stesso pin, R=5, ≥3 sere, per
  categoria) ∥ Leijen KS-LE-104-3 (la banda R=5 vale solo per QUESTO bisect,
  mai riusata senza rimisura stessa-sera) ∥ Gregg RC-GR-104-1 (i trasversali
  S-101 arith/calls erano TRA-SERE con movimenti ~±0,4 della stessa scala ⇒
  declassati a INDIZIO; verdict-grade solo i Δ A/B stessa-sera di H-C1a+b).
- **C-6 «Igiene contabile del census»** — Bak A-BA-104-5 (tag `Other` sui
  push fallback; contatore grow per certificare zero-realloc del Vec pila) ∥
  Leijen A-LE-104-1 (disaggregare realloc: GA_REALLOC_N + delta byte) +
  A-LE-104-2 (due-punti calls_small↔calls: ogni giudice il SUO controllo di
  linearità) + A-LE-104-4 (dump atexit anche su empty.php).

## 2) CONFLITTI (posizione di ciascuna sedia)

- **K-1 Leva-nulla: A CHI serve.** Bak (A-BA-104-2): prefisso di H-C2 come
  taratura del bias di LAYOUT tra binari diversi (swing 1-5% da alignment;
  verdetti dentro la banda layout = VOID). Gregg (b): lo slittamento della
  leva-nulla era IMPROPRIO perché è il calibro del PROFILO (unico strumento
  che nomina il 21,2% e il 26,6%), non di una leva. Leijen: non si esprime.
  **Composizione del relatore**: nessuna incompatibilità di merito — la
  STESSA build inerte serve entrambi gli scopi: ABAB R=5 dà la banda layout
  (Bak), il profilo della stessa build dà l'errore di attribuzione a
  campioni (Gregg). UNA leva-nulla, due letture; si esegue come prefisso di
  H-C2 e i suoi numeri si pubblicano per entrambi gli usi.
- **K-2 Lo status di «2 alloc/chiamata ~35 B».** Gregg (lista secca p.1) lo
  timbra tra le cose «che oggi sappiamo»; Leijen RC-LE-104-1 lo declassa:
  ESISTENZA sì (>0, RC-LE-103-1 vendicata), CIFRA no (realloc conta doppio
  anche in-place; lordo≠netto; linearità di calls presa in prestito da
  prop). **Composizione**: si adotta la formulazione Leijen nel record —
  «il call-path ALLOCA» è conoscenza; «2» e «35» restano sub iudice fino a
  realloc disaggregato + due-punti calls.
- **K-3 Prefissi vs pressione al Δ-oggetto.** Bak antepone a H-C2 due
  prefissi (leva-nulla, drop-census); Gregg (A-GR-104-3) mette il contatore
  «sessioni-senza-Δ-oggetto» a 1 e pretende che S-103 produca una leva
  promossa O refutata dal suo criterio, pena deriva d'apparato.
  **Composizione**: i prefissi sono corti (Bak stima mezz'ora la
  leva-nulla; il sito di conteggio drop esiste già nella guardia H-C1a) e
  NON sono alternative al verdetto: la sequenza S-103 deve arrivare
  comunque all'A/B H-C2 nella stessa sessione. Se i prefissi mangiano la
  sessione, il contatore sale a 2 e la riga ⏱ lo dice.
- **K-4 Zona marginale vs tetto finestra** (conflitto apparente, in realtà
  ortogonali): Leijen A-LE-104-5 governa il caso |Δ| ∈ (banda, 2×banda] ⇒
  R≥7 prima di dichiarare crescita reale; Gregg A-GR-104-2 governa il caso
  banda-di-fase-2 gonfia ⇒ ripetere in finestra quieta. Si adottano
  ENTRAMBI: il primo protegge dal falso-positivo, il secondo dal
  falso-negativo («RUMORE» a potenza degradata).

## 3) PRIORITÀ PROPOSTE per l'ordine S-103

### Sequenza H-C2 (vincolante, in quest'ordine)
1. **Leva-nulla** (prefisso, A-BA-104-2 + Gregg b): build con modifica
   inerte, ABAB R=5; pubblicare PRIMA (i) la banda layout per l'A/B,
   (ii) il profilo della build inerte come calibro dell'errore di
   attribuzione a campioni (21,2%/26,6%).
2. **Drop-census** (A-BA-104-3): contare le esecuzioni reali del canale
   («~11 drop scalari/iter» oggi è un numero statico MAI contato) sul sito
   già esistente = guardia scalare di H-C1a. KS-BA-104-3: senza questo, la
   banda [8,22] non entra in nessun criterio.
3. **Criterio pre-registrato**: banda RI-DERIVATA dal canale contato;
   denominatore = il sotto-insieme sito×primitiva che la leva rimuove
   (A-BA-104-1), MAI ÷23 (la voce in NEXT si emenda).
4. **A/B ABAB stessa-sera**: verdetti dentro la banda layout = VOID
   (KS-BA-104-2, si ripete non si interpreta); |Δ| ∈ (banda, 2×banda] ⇒
   R≥7 (A-LE-104-5). Esito = promozione O refutazione dal criterio: azzera
   il contatore di Gregg (A-GR-104-3).

### Sequenza H-D (vincolante, in quest'ordine)
1. **Strumentazione contabile**: disaggregare realloc in CountingMi
   (A-LE-104-1) + istogramma size-class (bucket 0-16/17-32/33-48/49-64/65+,
   atomics) + tag TL RAII per-sito nei costruttori del call-path
   (A-BA-104-4 ≡ A-LE-104-3). Indiziati per nome: ret_cell (primo), Vec
   args, pooled_frame su pool-miss, diag/String. Chiusura contabile:
   somma per-sito = eventi/iter, residuo≡0.
2. **Cifra netta verdict-grade**: due-punti calls_small↔calls
   (A-LE-104-2), unità «per chiamata» definita UNA volta (A-GR-104-4:
   10 vs 5 gc_note, out=2×iter), specie separate (fresca vs realloc),
   byte al NETTO (32,0, non 35,25).
3. **Leva H-D**: solo DOPO 1+2 (KS-LE-104-1: nessuna leva finché la somma
   per-sito non chiude con realloc disaggregato), con criterio
   pre-registrato sul canale ora contato (C-1).

### Trasversali (in coda ai pre-flight, nessuno slot dedicato)
- Banda tra-sere del giudice micro: stesso pin, R=5, ≥3 sere, per
  categoria (A-GR-104-1, ~15 min/sera).
- Record: trasversali S-101 declassati a INDIZIO anche in SYNTHESIS WP-103
  §FONDAMENTALI (RC-GR-104-1); KS-GR-104-1 attivo sui movimenti tra-sere
  narrati senza A/B.
- Voce peak WP-102 p.4: eseguire con audit finestra (segni coppie + tetto
  spread 1,5× fase 1; A-GR-104-2/KS-GR-104-2 ≡ KS-LE-104-2).
- Igiene census: tag `Other` sui push fallback + contatore grow
  (A-BA-104-5); dump atexit su empty.php (A-LE-104-4).
