# MEASURE86_RESULTS.md — misure S-86.0 nelle FORME ordinate dal Concilio WP-87

Campagna di ATTRIBUZIONE (§Sintesi WP-87 punti 5-7): contro-prova A-BB45,
burst-control A-DL33, terzo punto VW A-BB48, overlap A-BB47, metà fisica
ridotta A-DL31, ABBA purge A-BB46. Cifre BYTES-FIRST con companion
VERIFICATO (A-DL26/A-SK40/A-SK43). Verdetto macchina:
`wp86-harness/verdict86.out` (VERDICT86 PASS, fail-closed, blocchi
raccogli-poi-emetti A-SK44/KS-SK-87-3).

## Identità

- git campagna: c259bc6 (head_unmoved per-run E nel supplement-path;
  nessun commit mid-campaign)
- battery-86pre: **PASS (15/15 CONTATO, A-SK42) a c259bc6 per NOME**,
  riga PASS terminale ANCORATA, `.done` con `matrix=`+`matrix_sha256=`
  (A-AH40), stamp LEDGERATO committed (A-SK41, KS-SK-87-1) — campagna
  alla STESSA rev: nessuna equivalenza consumata
- binari: mem-census 874e744ede57b4ca · union b2074e451cbc7fc3 — ognuno
  ENFORCE contro la matrix del `.done` (KS-AH-87-1) e contro
  gate-binary-noprobe (nm+strings SUFFICIENTE dal marker A-TH37; hash
  belt)
- driver_sha=699db00a9808489e — DIVERSO da measure85 (54717a9afe6ccb96):
  measure78.sh ha guadagnato il parametro purge (A-BB46); ogni confronto
  cross-campagna nomina entrambi i lati e gli R (A-BG41/KG-87-3)
- dispatch-count per-thread IN-BAND su ogni raw mem-census (A-PP36) ·
  attempt-ledger OVL (KG-87-1) · reqns solo via reqns-guard.pl (A-PP35)

## Verdetti (da verdict86.out)

- **VCAL — riproduzione IDENTICA**: NET_H = 7.349.977 B = 7,01 MiB ·
  NET_P = 7.803.281 B = 7,44 MiB (W=1, binario di questa campagna) —
  terza campagna consecutiva al byte sui record WP-83/84/85.
- **VINV — A-BB45 CONTRO-PROVA SUPERATA (ordine invertito)**: pad come
  ord=1 netta NET_P ESATTO (7.803.281 B) e hello come ord=2 netta
  6.842 B [derivata: NET_H−NET_P+460.146] — la predizione nominata
  EX-ANTE della scomposizione additiva è riprodotta AL BYTE con l'ordine
  invertito; il confound dei nodi condivisi (996.838 B [derivata: v.
  MEASURE85]) NON ha morso su questa coppia. La scomposizione resta
  MODEL-GRADE fino a delibera (KB-87-1); la promozione è materia del
  Concilio WP-88 con questa contro-prova agli atti.
- **VBURST — A-DL33 PIPELINE FAIL-CLOSED PROVATA**: il burst da
  1.048.576 B = 1,00 MiB DENTRO il bracket di produzione muove la riga a
  8.398.553 B [derivata: NET_H+1.048.576] — delta ESATTO al byte — e il
  dente di calibrazione la DECLASSIFICA (net ≠ NET_H): la pipeline
  rifiuta una riga polluta, non solo il contatore la conta. Run mai
  usabile come misura (dichiarato in-band, burst=1).
- **VOVL — ⚠️ FALSO-DAI-RAW, CORRETTO a WP-88 (v. §Correzioni)**: il
  verdetto originale («10 tentativi, nessuna coppia intersecante, hello
  in ~µs») era prodotto da un qualificatore awk che confrontava STRINGHE
  (A-BB49); il ri-giudizio dai raw dà overlap 10/10 e «per-thread sotto
  concorrenza» è REFUTATO, non candidato.
- **VW123 — A-DL31 (forma ridotta) NAMED-DEVIATION — ⚠️ tag KS-PP-88-2 +
  metrica RSS, v. §Correzioni**: Δpeak per worker
  aggiunto = 23.625.728 B = 22,53 MiB (W1→2) e 15.040.512 B = 14,34 MiB
  (W2→3) — FUORI dalla predizione 3.605.572 B ±5%. Il protocollo qui
  (peak d'avvio, una richiesta per worker) NON è quello della derivazione
  KL-85-2 (steady W=10 N=100): la deviazione è un fatto del protocollo,
  nominata a WP-88. 🔵 **Split per-thread REFUTATO DALLO STRUMENTO**: su
  questo mimalloc v3 il default heap è CONDIVISO fra i thread (dente
  `heap=` in-band: puntatore IDENTICO sui due worker) — la chiusura
  contata-vs-fisica per-thread non è misurabile via heap-visit; la metà
  fisica piena resta aperta (WP-88).
- **VW500 — A-BB48 ESEGUITA, terzo punto SUL modello**: len=500 ⇒
  a_calls=4, a_bytes=2.002 B [derivata: 2·500+2·501] ESATTI sulla
  predizione ex-ante; controllo hello len=98 ⇒ 2 call,
  196 B [derivata: 2·98] sul floor. Il modello piecewise VW (std 384)
  regge sul terzo punto.
- **VABBA — A-BB46 eseguita — ⚠️ l'INCONCLUSIVE era ARTEFATTO DELLA
  METRICA max RSS, CORRETTO a WP-88 su peak footprint (v. §Correzioni)**:
  R=9 per braccio, interleave ABBA, effetto purge in-band per run
  (option table verbose: 0 vs 1000). Spread r2..r9: braccio A (purge=0)
  = 21.315.584 B = 20,33 MiB · braccio B (default) =
  14.139.392 B = 13,48 MiB [derivata: max−min r2..r9 per braccio].
  Nessun braccio dimezza l'altro (soglia Bak): **il purge timing è
  REFUTATO come driver principale dello spread** — anzi purge=0 (il
  protocollo storico) mostra lo spread MAGGIORE. Il pin identità resta
  RITIRATO, citazione legale SOLO envelope (KL-87-1/KB-87-2); candidati
  residui (first-touch, ordine spawn heap) a WP-88.

## Aperture dichiarate (per NOME — mai chiusure in silenzio)

1. **A-BB47**: overlap mai qualificato in 10 tentativi — serve fixture a
   lowering ~ms su ENTRAMBI i lati (pad-vs-pad calibrati distinti).
2. **A-DL31 metà fisica piena**: chiusura contata-vs-fisica NON
   misurabile per-thread su questo allocatore (heap condiviso, dente
   heap=); forma alternativa (Δcommitted di processo fra W, win=0 exit
   snapshot) da disegnare a WP-88.
3. **Attribuzione spread VP**: purge timing refutato; il pin resta
   envelope-only. Envelope di QUESTA campagna (braccio A, r2..r9 max):
   252.772.352 B = 241,06 MiB [derivata: max r2..r9 braccio A] · r1
   NOMINATO 240.320.512 B = 229,19 MiB [derivata: r1 braccio A] — R=9,
   driver_sha 699db00a9808489e; il confronto con l'envelope WP-85
   (252.526.592 B = 240,8 MiB, driver_sha 54717a9afe6ccb96, R=9) è
   legale coi due lati così nominati (A-BG41).
4. **Promozione scomposizione additiva**: contro-prova A-BB45 SUPERATA —
   delibera di promozione (da MODEL-GRADE a verdict-grade) al Concilio
   WP-88 (KB-87-1 resta vincolante fino ad allora).
5. **A-AH38 + dry-run KS-AH-86-1** (ereditata, A-AH42): nessuna fase
   slope/base-arm anche in questa campagna — l'apertura resta nominata.
6. **A-MS27**: precondizione della via registry (KS-MS-86-2), backlog.

## ⚖️ Delibere eseguibili QUI

- Nessun pin nuovo (KL-87-1/KB-87-2 rispettate: l'ABBA non ha attribuito).
- La delibera ×W di WP-85 RESTA nella sua forma riformulata (solo
  m85.dl28s, sequenziale; VOVL OPEN non la tocca — KL-87-2 già nel
  perimetro dichiarato).

## ⚖️ CORREZIONI WP-88 (S-87.0 p1 — ri-giudizio DAI RAW ESISTENTI, nessun run nuovo)

Ordine vincolante Concilio WP-88 §Sintesi p1. Macchina:
`wp87-harness/rejudge86.sh` → `wp87-harness/rejudge86.out` (raw invariati;
campagna git c259bc6, driver_sha 699db00a9808489e).

- **VOVL → REFUTATO-DAI-RAW (A-BB49)**: il qualificatore awk della
  campagna confrontava STRINGHE (dopo `sub()` i campi perdono lo status
  strnum: `"4"<"13300"` è falso) — riparato con coercizione `+0` nel
  tool. Ri-giudizio sui 10 raw esistenti: **overlap 10/10** (hello si
  abbassa in ~13,3–13,9 ms, NON in µs — i µs erano di inv ord2), tid
  distinti in tutti i tentativi. Firma dell'inghiottimento in 10/10:
  pad_net = 15.153.408 B = 14,45 MiB [derivata: NET_H+NET_P+150] ESATTO
  — la finestra process-counters ha assorbito il lowering altrui INTERO.
  Per KB-88-1 TUTTI i net= di questi 10 raw sono VOID come cifre
  per-thread. **«Per-thread sotto concorrenza» = REFUTATO, non OPEN**;
  la delibera ×W resta SEQUENZIALE-ONLY (KL-87-2 in direzione dura).
  Il ledger `m86.ovl.attempts.ledger` (0/10 QUALIFIED) resta agli atti
  come output dello strumento difettoso — non è più citabile come
  verdetto.
- **VABBA → su peak footprint SEPARA 8/8 (A-BB51)**: l'INCONCLUSIVE era
  un artefatto della metrica max RSS. Sulla metrica vincolante
  peak_memory_footprint (time -l, nei .log archiviati), r2..r9:
  braccio A (purge=0) media 209.670.819 B = 199,96 MiB [derivata: media
  r2..r9], max 213.828.232 B **< min braccio B** (default) 226.132.640 B;
  media B 230.597.283 B = 219,91 MiB [derivata: media r2..r9] —
  **separazione completa 8/8**, Δmedie 20.926.464 B = 19,96 MiB
  [derivata: mediaB−mediaA]: **purge=0 ABBASSA il fisico**. Lo spread
  resta ADVISORY (KB-88-3: max−min con R=8<16 mai giudice; attribuzione
  dello SPREAD ancora aperta — protocollo futuro R≥16 + Levene). Diff
  per-regione dai vmmap V2 archiviati: il delta inter-braccio vive
  INTERAMENTE nella regione arena anonima (vmmap la tagga
  `IOAccelerator` su questo sistema), dirty V2 +44.420.319 B = 42,36 MiB
  [derivata: mediaB−mediaA V2 r2..r9] su B; Stack e MALLOC_* byte-identici
  fra i bracci. Il «purge REFUTATO come driver» vale SOLO su metrica RSS
  (NON-riproporre); su footprint il purge ha effetto di LIVELLO provato.
- **VW123 → tag KS-PP-88-2 + doppia metrica (A-BB52 contesto)**: l'arm
  union non emette la riga dispatch ⇒ «one hello per worker» era
  un'ASSUNZIONE di protocollo, non attribuzione: i Δpeak/worker citati
  portano il tag **envelope**. In più i peak citati erano max RSS; su
  peak footprint i tre run danno 38.552.056 / 55.263.784 / 78.561.808 B
  [fonte: m86.w123.w*.log] → Δ 16.711.728 B = 15,94 MiB e
  Δ 23.298.024 B = 22,22 MiB [derivata: differenze consecutive] — le due metriche
  INVERTONO il trend fra loro: il Δpeak d'avvio non misura una quantità
  per-worker stabile. La forma che sostituisce questo protocollo è
  **A-DL38≡A-BB52** (slope di committed steady-state).
- **A-BG44-forma su verdict86**: verdict86.out r.18 enunciava A-BG41
  violandola (un solo driver_sha). Il .out è un raw e resta INTATTO; la
  forma sanata è QUI: il confronto VCAL «terza campagna consecutiva» ha
  i lati — measure86 driver_sha 699db00a9808489e (R=1 cal) vs measure85
  driver_sha 54717a9afe6ccb96 (R=1 cal); i lati WP-83/84 NON hanno sha
  registrati nei documenti di campagna (il lato-84 fe6983d8 vive solo
  nel verbale Gregg WP-87) — il claim «terza consecutiva» è quindi
  legale al byte solo sulla coppia 85↔86; per 83/84 resta
  claim-di-cronaca senza ancora sha (dichiarato, KG-88-2 rispettata).
