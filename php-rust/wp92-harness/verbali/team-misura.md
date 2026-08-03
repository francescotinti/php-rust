# Verbale TEAM-MISURA — Concilio WP-92 (fase 2)

**Composizione**: sedia 5 Bak (statistica/stimatori) · sedia 6 Pedersen
(lifecycle/quiescenza) · sedia 7 Leijen (mimalloc/contatori). Relatore: sedia 5.
**Fonti**: `verbale-5-bak.md`, `verbale-6-pedersen.md`, `verbale-7-leijen.md`
(integrali). I verbali individuali restano VINCOLANTI: questo documento
riconcilia, non sostituisce né emenda.

**Verdetti individuali**: tutte e tre le sedie CON EMENDAMENTI. Refutazioni
capitali dichiarate: Bak 2 (label del residuo; b_peak min-based), Pedersen 1
(quiescenza al census), Leijen 0.

---

## CONVERGENZE

**CV-1 — Le CIFRE del g2 non sono in discussione; gli STIMATORI e le ETICHETTE
sì.** Bak ha ricomputato tutte e 20 le raw committate
(`m90.slope.w{4,8,12,16}.r{1..5}.a1.memcensus`) e ogni numero del g2 riproduce
al byte (b_boot=2.252.800 se=6.345; b_work=17.276.928 se=501.348;
b_peak=19.723.059,2 se≈447.215; residuo 193.331; marginali 9/9). Nessuna sedia
contesta la fedeltà doc↔g2 (Q4 Bak: sì). L'attacco è a monte (selezione del
punto) e a valle (che cosa il numero significhi), mai sull'aritmetica.

**CV-2 — Il canale è PROCESS-WIDE mentre l'oggetto misurato è PER-WORKER.**
Tre attacchi indipendenti nominano la stessa malattia: Bak Q3 (i punti per-lane
vengono da RUN DIVERSE ⇒ il residuo cross-lane misura mismatch di selezione),
Leijen Q2 (il bracket su contatori di processo sottrae il traffico concorrente
dell'altro lato: `df=89.955.518 > da=86.862.318` eppure net=0, KB-88-1),
Pedersen Q3 (nessuna esclusività client asserita su 127.0.0.1:8297: un client
locale ruba uno slot senza testimone). Corollario condiviso: finché la misura
non è scopata all'ATTORE (thread/worker) e alla RUN, ogni Δ è contaminabile.

**CV-3 — Il residuo 193.331 non è il segmento post-work.** Coincide
esattamente con b_peak − (b_boot+b_work), cioè è per costruzione un artefatto
cross-lane. Bak lo refuta dai raw: Δcommit post-work = 0 su **17/20**. Leijen
spiega perché il canale non potrebbe comunque vederlo: le allocazioni di
self+census atterrano su spazio già committato in attesa di purge (purged
cumulativo 844.627.968 al picco) o su `malloc_huge` (96 chiamate, total
638.582.784) — invisibili al commit e al visit. La label del giudice va
ritirata come **non falsificabile su questo canale**, non solo come falsa.

**CV-4 — Principio epistemico comune: un check che non può fallire è VOID.**
KS-BB-92-2 (ratio min-of-R ≡ 1,000 per costruzione con tie=4 ⇒ controllo
vacuo) e KS-DL-92-1 (heap ptr duplicato ⇒ REFUSE, mai media) sono lo stesso
dente applicato a due oggetti. Pedersen lo applica al lifecycle: un ramo che
esce senza riga di ledger (`head_unmoved`, campaign.sh:82-85) è un
non-testimone.

**CV-5 — b_boot è la lane più solida delle tre.** se relativo 0,28%; modi
dominanti reali (non tie); quiescenza triviale (pboot precede ogni richiesta,
quindi KS-PP-92-1 non morde); semantica del contatore pulita — Leijen refuta
l'ipotesi "stack 2 MiB/thread" per SEMANTICA (il commit mimalloc non contiene
gli mmap di sistema degli stack pthread), senza bisogno di misura.

**CV-6 — La composizione di b resta ignota anche dove la magnitudine è
solida.** b_boot è un numero senza scomposizione (Leijen: serve il braccio
bare-thread, A-DL-56); la copertura del visit è 0,576–0,634 e il 42% mancante
è nominato ma non contabilizzato (huge current, `pages_abandoned=2481`,
free-committed, i 25 theap non enumerabili). Nessuna sedia considera
l'attribuzione chiusa.

**CV-7 — Gli strumenti per il prossimo passo esistono già.**
`census::OUTSTANDING` (worker_pool.rs:630), pid-echo, `mi_heap_get_default` +
`mi_heap_visit_blocks` on-thread (zero API nuove), i 20 raw committati. Nessuna
priorità dell'iterazione 3 richiede infrastruttura nuova.

---

## CONFLITTI

### CF-1 — Grado della cifra di testata (l'unico conflitto vero)

- **Bak**: la testata onesta è **b_boot+b_work = 19.529.728 B** (lane FORTI),
  a 45.875 B da b m89 = 19.575.603; b_peak min-based non può essere testata
  (KS-BB-92-1). Bak non assegna un grado alla somma: la propone come forma
  migliore.
- **Pedersen**: **b_work è ADVISORY** finché la riga pwork non porta
  `outstanding=0` (KS-PP-92-1). HTTP-complete ≠ request_end-complete: per la
  Binding Rule il body parte PRIMA del teardown, quindi la Δ pboot→pwork può
  assorbire il teardown della richiesta n-esima. Aggravante che il team
  registra: il residuo di teardown è **per-worker**, quindi aliasa nella
  PENDENZA, non nell'intercetta — è esattamente il parametro citato.
- **Leijen**: non si pronuncia sul grado; convalida la semantica del contatore
  per b_boot e non porta obiezioni alla lane WORK come magnitudine.

**Composizione del team**: le due posizioni sono su assi diversi (Bak = quale
stimatore; Pedersen = quale grado) e si compongono a strati. Vedi NODO.

### CF-2 — Tensione con l'ORDINE WP-91 ("b = pendenza del PICCO")

- **Bak**: la lane PEAK ha `modes==R=5` (tie=4) su TUTTI e 4 i punti; il
  tiebreak=min la rende una statistica d'ESTREMO (−2,8% vs mediana) e a W=12
  SELEZIONA la run anomala r3 (303,4M contro sorelle 325–334M, −7…−10% su tutti
  i contatori). Quindi: non citabile come testata.
- **Leijen**: usa senza riserve i raw di picco (w16.r1 peak: commit 399.638.528,
  `heaps.total=1` vs `theaps.total=26`) per coverage e diagnosi.
- **Pedersen**: con commit monotono il PICCO è salvo; è la Δ di FASE a essere
  esposta.

**Non è un conflitto tra sedie**: il PUNTO di picco resta uno stato osservato
valido (Leijen, Pedersen); è la PENDENZA della lane PEAK a non essere
stimabile **con l'attuale disegno**. L'ordine WP-91 non va revocato, va reso
eseguibile: la lane PEAK diventa stimabile solo se il punto di picco è preso
per-worker on-thread a una barriera (P1) e stimato con mediana/punto additivo
per-run (P2). Il team registra questo come vincolo di progetto, non come
dissenso dal Concilio WP-91.

### CF-3 — Nessun conflitto su clamp/purge

La refutazione P-CLAMP-PD regge ma va SCOPATA (Leijen): pd=1000 non rende
purge-free il RUN (`purge_calls=274`, purged 85.721.088 già a exit_mi con
mono_ms=1064) — difende solo la FINESTRA (span clamped [6.059, 20.236] µs). Il
discriminatore reale è **OVERLAP⇔clamp: 6/6 clamped hanno spans=OVERLAP, 4/4
NO-OVERLAP hanno clamped=0**, con la coppia dt5.pd1000 come esperimento
naturale. Debolezza dichiarata: `purged≡0` in-window è ARITMETICA, non lettura
(A-DL50 unavailable) — chiude A-DL-53. Le altre due sedie non obiettano.

---

## NODO CENTRALE — la scomposizione b_boot/b_work è citabile? A che grado?

**POSIZIONE UNICA DEL TEAM (motivata, senza dissenso registrato):**

1. **b_boot = 2.252.800 B/worker (se=6.345) — VERDICT-GRADE come
   MAGNITUDINE.** Regge a tutti e tre gli attacchi: modi dominanti reali (non
   min-statistic ⇒ KS-BB-92-1 non morde), quiescenza triviale al pboot
   (KS-PP-92-1 non morde), semantica del contatore convalidata (Leijen Q3).
   **Non** è verdict-grade come COMPOSIZIONE: "bootstrap theap + stato phpr"
   resta ipotesi finché non gira il braccio bare-thread (A-DL-56). Da citare
   come *quanto*, mai come *di che cosa*.

2. **b_work = 17.276.928 B/worker (se=501.348) — ADVISORY.** Due ragioni
   indipendenti, entrambe sufficienti: (a) KS-PP-92-1, manca il testimone
   `outstanding==0` e il residuo di teardown è per-worker ⇒ aliasa nella
   pendenza; (b) Bak, i marginali "9/9 IN fascia" sono estimator-dipendenti —
   con la mediana W8→12 = 22.183.936 (OUT alto) e W12→16 = 13.271.040 (OUT
   basso), e la dispersione a W12 è ~11% del livello. Citabile con l'etichetta
   ADVISORY esplicita e la banda; mai come cifra di verdetto.

3. **b_peak = 19.723.059 B/worker — DA RIFARE (declassata da testata).**
   Stimatore d'estremo spacciato per modo dominante; il controllo di robustezza
   min-of-R è VOID per costruzione (KS-BB-92-2). Ricomputabile subito sulle raw
   esistenti come mediana (~20,29M attesi) o punto additivo per-run, con la run
   W12 r3 flaggata per NOME (A-BB67).

4. **b_boot+b_work = 19.529.728 B — miglior FORMA di testata (Bak), ma eredita
   il grado della lane più debole: ADVISORY.** La continuità con b m89
   (19.575.603, Δ 45.875 B) è **suggestiva, non verdict-grade**: due numeri
   ADVISORY che si somigliano non fanno una conferma. Va citata come coerenza
   di ordine, mai come conferma di P-RET0 o di alcunché.

5. **Il residuo 193.331 non è citabile con la label attuale** (CV-3): va
   rietichettato "selection mismatch cross-lane" (A-BB65) e l'additività va
   giudicata **within-run**, dove è esatta, con il residuo cross-lane riportato
   a parte.

6. **Correzione formale trasversale**: con 4 punti (df=2) "2σ⇒95%" è falso
   (t(0,975;2)=4,30 ⇒ ~82%). Tutte le bande vanno ripubblicate come ±2se
   dichiarato o ±t(df)·se (A-BB68). Questo tocca ogni figura del doc, incluse
   quelle che restano.

**Condizione di promozione a verdict-grade per b_work**: riga pwork con
`outstanding=0` da `census::OUTSTANDING` (A-PP-63) **e** punto di lane preso
con stimatore non-tautologico (A-BB64/66) **e** censimento per-worker
on-thread senza collisione di heap ptr (A-DL-52/KS-DL-92-1). Le tre condizioni
sono congiunte: due su tre lasciano b_work ADVISORY.

---

## PRIORITÀ ITERAZIONE 3 (attribuzione di b) — ordinate per POTERE DISCRIMINANTE

Criterio: quanto l'intervento cambia la CLASSE di conclusioni ottenibili, non
il suo costo. Il costo è annotato a parte perché l'ordine di ESECUZIONE
temporale differisce (vedi P0).

**P0 (da eseguire per primo, costo ≈ 3 righe, potere alto per unità di costo)**
— testimone di quiescenza `outstanding=0` sulla riga pwork (A-PP-63) + `-f` e
pid-echo su tutti i census (A-PP-62) + `LC_ALL=C` (A-PP-64). Decide da solo se
b_work è ADVISORY o promuovibile. Contatore e header esistono già.

**P1 — CENSUS ON-THREAD PER-WORKER ALLA BARRIERA DI PICCO (A-DL-52, dente
KS-DL-92-1).** Potere massimo: è l'unico intervento che cambia la NATURA della
stima. Oggi b è una regressione cross-lane su 4 punti scelti da run diverse;
con lo snapshot preso dal thread del worker su `mi_heap_get_default()` +
`mi_heap_visit_blocks`, b diventa una misura **per-worker, per-run**. In un
colpo: (a) chiude la collisione `mi_heap_of` — l'indirizzo identico per tutti i
thr (0x105c3ae80 a w16.r1, 0x10562ee80 a w4.r1, diverso per RUN non per thread)
è `_mi_heap_main` letto dalla TLS del chiamante, non pagine condivise, provato
dal jitter non monotono ±20 KB degli `used_b`; (b) elimina il mismatch di
selezione (CV-2, CV-3) perché l'additività torna within-run; (c) rende il
worker quiescente rispetto alla propria richiesta *per costruzione* alla
barriera, sussumendo in gran parte P0; (d) rende finalmente eseguibile
l'ordine WP-91 sulla pendenza del PICCO. Fallback in ordine: heap espliciti per
worker (`mi_heap_new` + allocazioni VM heap-scoped), poi merge stats tld
at-exit (richiede patch al crate).

**P2 — RIPARAZIONE DEGLI STIMATORI SUI 20 RAW GIÀ COMMITTATI (A-BB64, A-BB65,
A-BB66, A-BB67, A-BB68).** Potere alto e retroattivo, zero run nuove:
ripubblica b_peak con mediana/punto additivo, flagga W12 r3 per NOME,
sostituisce il controllo di robustezza tautologico con min/mediana/media
(misurato: 0,972), rietichetta il residuo, corregge le bande. Discrimina se le
conclusioni di WP-90 **sopravvivono al cambio di stimatore** — e i marginali
già dicono di no in due punti su nove. Da fare PRIMA di qualunque run nuova:
se le conclusioni non reggono sui dati esistenti, la campagna nuova
misurerebbe la cosa sbagliata meglio.

**P3 — CHIUSURA DEL BILANCIO DI COPERTURA E SCOMPOSIZIONE DI b_boot (A-DL-55,
A-DL-56).** Potere sulla COMPOSIZIONE, non sulla magnitudine: `malloc_huge`
CURRENT (non total), `pages_abandoned`, free-committed in arena, classi
per-chunk in `mi_arena_chunk_bins` — l'arena si cammina senza TLS, quindi
questo braccio è indipendente da P1 e può girare in parallelo. Il braccio
bare-thread (W thread che toccano mimalloc, zero init phpr) separa
thread-runtime da stato phpr dentro b_boot. Senza P3 l'attribuzione di b
resta un numero senza composizione (CV-6).

**P4 (sotto la linea, ma da non perdere)** — overlap forzato a barriera
(A-DL-54) per rendere l'overlap controllato invece che emergente; lettura
purged/purge_calls ai bordi dello span sul worker path (A-DL-53, chiude A-DL50
e toglie l'aritmetica dal posto della lettura); disciplina failpath (A-PP-60/61)
e supersessione del giudice in ledger (A-PP-65, KS-PP-92-2 — g1→g2 è avvenuto
con judge_sha 1b1e9e96→97a8eff0 e `supersede_of=g1` **senza riga che autorizzi
la sostituzione**: catena gG fuori banda).

---

## KILL-SWITCH ADOTTATI DAL TEAM (unione, nessuno ritirato)

KS-BB-92-1 (lane con modes==R su tutti i punti ⇒ mai cifra di testata) ·
KS-BB-92-2 (check di robustezza che non può fallire ⇒ VOID, verdict declassato)
· KS-PP-92-1 (Δ di fase senza testimone outstanding==0 ⇒ mai verdict-grade) ·
KS-PP-92-2 (judge_sha ≠ pinned a phase=start senza autorizzazione ⇒
generazione VOID) · KS-DL-92-1 (heap ptr duplicato tra worker ⇒ REFUSE, mai
media) · KS-DL-92-2 (claim sul clamp senza riga overlap-status per span ⇒ VOID).
