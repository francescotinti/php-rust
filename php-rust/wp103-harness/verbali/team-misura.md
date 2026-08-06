# Team «misura» — Concilio WP-103, fase 2
Relatore: sedia 9 area (sintesi). Sedie: 5 Bak, 7 Leijen, 9 Gregg.
Fonti: verbale-5-bak.md · verbale-7-leijen.md · verbale-9-gregg.md.

## FONDAMENTALI (in testa, per la sintesi)
- Contatore sessioni-senza-misura: 0 (Gregg). Le promozioni H-C1a/b reggono
  per tutte e tre le sedie (contatori esatti, criteri pre-registrati, A/B
  interleaved). Nessuna sedia tocca le leve promosse.
- Refutazioni capitali del team: **RC-LE-103-1** (la gamba alloc: le stats a
  pagine non vedono il churn — «alloc/iter≈0» declassato a «retained≈0 su
  f29883eb», vietato come premessa in S-102) e **R-GR-103-1** (la lezione
  «gli inlined sovracontano» refutata come legge: segno IGNOTO).

## CONVERGENZE

1. **Il 26,6% «pila operandi» è POROSO e non genera attesi.** Bak (§1) e
   Gregg (§3) coincidono su meccanismo (skid del PC, righe di confine,
   scheduling/CSE, bounds-check fusi: il confine col 21,2% dispatch perde
   nei due sensi; Gregg: banda forse ±10 punti) e su rimedio: il census
   push/pop di S-102.2 è l'unico arbitro; il costo/accesso fa fede SOLO da
   Δ_A/B ÷ transiti contati. Convergenza operativa: A-BA-103-1 (census per
   SITO-OPCODE e per PRIMITIVA: push/pop/len/as_slice/expect distinti, con
   assert conteggi↔dump) + A-GR-103-3 (conteggi per NOME + ns/evento
   dall'A/B; se conteggio×ns ≪ 26,6%×T la quota si CORREGGE nel file di
   profilo, non si tramanda). Kill-switch congiunto: **KS-GR-103-1 ≡
   KS-BA-103-3** — qualunque atteso/criterio costruito citando il 26,6% (o
   la tariffa ns/op) invece del canale contato ⇒ VOID / non si scrive.
2. **Attesi solo a BANDA, mai puntuali da quota%.** Bak A-BA-103-2 e Gregg
   A-GR-103-2: il costo/evento da campioni è banda larga; l'atteso puntuale
   derivato da una quota di profilo è vietato.
3. **Punto attribuzione peak (+95 MiB): banda phpr PRIMA, altrimenti si
   biseca un fantasma.** Leijen A-LE-103-3 e Gregg A-GR-103-4 sono lo
   stesso emendamento da due lati: il ~10% noto è rumore dell'ORACLE
   (motore sbagliato); prima del punto 1/4 si misura il rumore full-peak
   PHPR su UN pin, R≥5, intra-sera, interleaved; statistica NOMINATA =
   mediana per arm + spread max−min (mai la media: il peak è coda
   unilaterale); regola di chiusura pre-registrata: |Δ| entro banda ⇒ voce
   chiusa come RUMORE, senza bisect. KS congiunti: KS-GR-103-2 (punto
   eseguito senza banda stessa-sera = VOID) + **KS-LE-103-3 (spread
   intra-arm ≥ metà dell'effetto, ≥~48 MiB ⇒ bisect VIETATO)**.
4. **Modo FISSATO nell'A/B S-99↔S-100** (Leijen A-LE-103-2, nessuna
   obiezione dalle altre sedie): off su entrambi i pin (unico modo del pin
   S-99); l'effetto-modo (~116 MiB, R=1, PISTA non fatto) è più grande
   dell'oggetto (+95 MiB) — default-contro-default confonde pin e modo.
   KS-LE-103-2: modi diversi ⇒ VOID, bisect vietato. La pista modo→peak
   diventa voce PROPRIA (R≥3).
5. **ABAB/interleaved obbligatorio.** KS-GR-103-3 promuove a kill-switch la
   regola S-101 (verdetti micro con burst remoto senza ABAB = VOID); Bak
   la assume in ogni A/B; Gregg A-GR-103-1 la raffina: pubblicare i delta
   per coppia adiacente accanto alle mediane; ≥3/7 coppie a segno opposto
   ⇒ finestra sospetta, si ripete.
6. **Gamba alloc a mem-census DIRETTO** (Leijen A-LE-103-1, coerente con la
   dottrina «canale contato» di Bak/Gregg): wrapper `#[global_allocator]`
   che conta alloc/dealloc/bytes (o mimalloc MI_STAT≥2), linearità 300:1,
   assert conteggi↔nomi; SHOW_STATS ammesso SOLO per footprint ritenuto.
   KS-LE-103-1: «alloc/iter≈0» da stats a pagine ⇒ gamba VOID.
7. **Forme slot-dirette dei Prop-op: onere della prova ALTO.** Bak §3
   (precedente Add [0, 0,5] già refutò la tesi round-trip; branch mai-preso
   +2,9%): dump-diff PRIMA, pavimento pre-registrato, leva-nulla di taratura
   (A-BA-103-4), collaudo categorie NON toccate (KS-BA-103-1/2). Gregg
   convergente via KS-GR-103-1 (criterio solo dal controfattuale contato).

## CONFLITTI (posizione per sedia, per NOME)

1. **Il segno della distorsione dei simboli inlined — Bak vs Gregg.**
   - **Bak (A-BA-103-2, e banda H-C2)**: fissa il pavimento a METÀ del
     contabile citando la «sovrastima 2× registrata su H-C1b» — assume
     direzione: sovraconteggio.
   - **Gregg (R-GR-103-1, refutazione capitale)**: quella generalizzazione
     è refutata come legge (n=1, segno non garantito: skid e code motion
     possono sovra- O sotto-attribuire); forma ammessa: banda larga a
     SEGNO IGNOTO.
   - **Composizione del relatore**: il conflitto è sulla MOTIVAZIONE, non
     sull'atto. Il pavimento ½ di Bak resta ammesso come CAUTELA
     unilaterale su canale contato (più stringente del necessario se il
     segno fosse favorevole), ma la sua giustificazione va riscritta in
     forma Gregg: «floor prudenziale», NON «perché gli inlined
     sovracontano». La lezione in memoria si riscrive secondo A-GR-103-2.
2. **Terzo strumento (perf counter / xctrace) — tensione lieve.**
   - **Gregg**: utile ma NON dovuto finché nessun go/no-go dipende dalla
     tariffa; il criterio resta il pavimento.
   - **Bak**: non lo chiede; chiede invece la leva-nulla (A-BA-103-4) come
     taratura del rumore dispatch↔pila.
   - Composizione: la leva-nulla di Bak È il controllo positivo a costo
     minore; il terzo strumento resta fuori dall'ordine S-102 (regola di
     ammissione: non blocca l'oggetto).
3. **Il 21,2% «corpo proprio» senza nome — priorità contesa.**
   - **Gregg (FONDAMENTALI)**: seconda quota del profilo senza ipotesi
     iscritta = rischio d'oggetto trascurato; chiede che una ipotesi venga
     NOMINATA.
   - **Bak**: lo tratta solo come l'altro lato del confine poroso; nessuna
     voce dedicata.
   - Composizione: il census per sito-opcode/primitiva (conv. 1) e la
     leva-nulla stringono il confine da entrambi i lati; l'iscrizione di
     un'ipotesi sul 21,2% si propone come voce di BACKLOG nominata, non
     come punto d'ordine S-102 (nessuna leva pronta).
4. **Nessun conflitto** su: promozioni S-101 (unanimi), RC-LE-103-1
   (terreno solo di Leijen, nessuna sedia lo contesta), H-C2 (proposta solo
   da Bak; Leijen e Gregg silenti — l'ammissione passa dal canale contato,
   che c'è: ~11 drop/iter).

## PRIORITÀ PROPOSTE PER L'ORDINE S-102
(regola di ammissione: apparato SOLO se blocca l'oggetto; qui l'unico
apparato ammesso è la banda-rumore phpr, che blocca il punto peak)

1. **Rumore full-peak PHPR intra-sera** (apparato ammesso perché blocca il
   punto attribuzione): UN pin, R≥5, ABAB, mediana+spread, regola di
   chiusura scritta PRIMA (A-LE-103-3, A-GR-103-4). Poi **A/B S-99↔S-100 a
   modo FISSATO off/off** (A-LE-103-2). Vincoli: KS-GR-103-2,
   KS-LE-103-2, KS-LE-103-3 (spread ≥~48 MiB ⇒ bisect vietato).
2. **Census pila push/pop** per SITO-OPCODE e PRIMITIVA con assert
   conteggi↔dump (A-BA-103-1, A-GR-103-3); NIENTE attesi dal 26,6%
   (KS-GR-103-1/KS-BA-103-3); quota corretta nel file di profilo se il
   contato la smentisce. Leva-nulla di taratura prima di ogni forma
   slot-diretta (A-BA-103-4).
3. **H-C2 — drop fast-out scalare** (A-BA-103-3): canale contato ~11
   drop/iter, banda [8, 22] ns/iter (pavimento ½ come cautela, motivazione
   riscritta a segno ignoto), misurata DA SOLA prima del cumulo; sotto
   pavimento = si registra, non si spedisce.
4. **Gamba alloc rifatta a mem-census diretto** (A-LE-103-1); declassamento
   della frase «alloc/iter≈0» eseguito nei file che la citano
   (RC-LE-103-1, KS-LE-103-1).
5. **Manutenzione lezioni** (contestuale, costo ~0): riscrivere la lezione
   inlined a segno ignoto (A-GR-103-2); promuovere ABAB a KS permanente
   (KS-GR-103-3); delta per coppia negli A/B (A-GR-103-1).
6. **Eventuali forme Prop-op slot-dirette**: SOLO dopo il punto 2, con
   dump-diff prima, pavimento pre-registrato, collaudo categorie non
   toccate (KS-BA-103-1/2). Backlog nominato: ipotesi sul 21,2% corpo
   proprio (Gregg); pista modo→peak come voce propria R≥3 (Leijen).
