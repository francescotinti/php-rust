# MEASURE85_RESULTS.md — misure S-85.0 nelle FORME ordinate dal Concilio WP-86

Campagna di DISCRIMINAZIONE (§Sintesi WP-86 punto 5): il canary monolaterale
A-DL28≡A-BB41 e il VP a R=9. Cifre di memoria BYTES-FIRST con companion
VERIFICATO (A-DL26/A-SK40). Verdetto macchina: `wp85-harness/verdict85.out`
(VERDICT85 PASS, fail-closed, blocchi A-SK38).

## Identità

- git campagna: 368c91d (nessun commit mid-campaign; head_unmoved per-run
  nel campaign script). **CORREZIONE S-86.0 (A-BG40, Concilio WP-87)**: la
  dichiarazione originale «A-BG37 anche in supplement» era un OVERCLAIM —
  al momento del run dl28-supplement.sh aveva ZERO occorrenze di
  head_unmoved (GIT_REV letto una volta, poi una build intera senza
  ricontrollo). Mitigazione a posteriori: il filesystem mostra un solo
  tentativo e la catena rev è confermata da git=368c91d in-band + enforce
  mem_hash==CAL. head_unmoved (post-build, pre-launch) e l'attempt-ledger
  A-BG39 sono ora NELLO script per ogni run futuro (KG-87-1).
- battery-85pre: PASS 15/15 a 368c91d per NOME, riga terminale ANCORATA +
  `.done` rev+sha256 (A-SK36) — **con `gate-measure-cifre --all` DENTRO il
  15/15 (A-SK40: il buco GRAVE KS-SK-86-3 è sanato NEL perimetro)**;
  corpus Zend 1418 IDENTICO + refl 290 IDENTICO per NOME sul binario di
  parità nuovo (phpr bf278d55fd5efb0a)
- binari: union axum a8b65c0578c42fb7 · mem-census a3c901dfddd474c0 —
  ognuno ENFORCE contro la riga matrix per-run E contro
  gate-binary-noprobe (KH86-1: nm + hash, entrambe le metà)
- driver_sha=54717a9afe6ccb96 (measure78 invariato; campaign script
  measure85-campaign.sh + dl28-supplement.sh, sha in-band A-AH30)
- NESSUNA fase slope (VC binario-bound a 7a610457, A-BB43; il prossimo
  verdict slope DEVE passare `gate-slope-verdict.sh`, KG-86-1)
- A-DS32: N/A dichiarata — nessuna finestra census-instrumentation in
  questa campagna

## Verdetti (da verdict85.out)

- **VDL28 — PER-THREAD CONFERMATO dal canary monolaterale (A-DL28≡A-BB41,
  KL-86-2/KB-86-2 soddisfatte)**: calibrazione a W=1 sullo stesso binario:
  NET_H = 7.349.977 B = 7,01 MiB (hello) · NET_P = 7.803.281 B = 7,44 MiB
  (hello_pad85, sorgente 9.276 B [derivata: wc -c fixture]); run W=2 una-richiesta-per-worker:
  hello→thr0 net 7.349.977 B = 7,01 MiB E pad→thr1 net 7.803.281 B = 7,44 MiB —
  **ogni thread netta ESATTAMENTE la cifra calibrata della PROPRIA
  fixture, al byte**. La finestra ATTRIBUISCE (né specchio né
  congelata): **il budget ×W è PROMOSSO da IPOTESI CANDIDATA a
  verdict-grade**. NET_H identico al byte al record WP-83/84. Bande
  A-BB42 (ri-ancorate al residuo: 734.998 B / 3.674.989 B [derivata:
  0,1×/0,5× di 7.349.977 B]) SUPERATE dal match esatto.
- **🔵 SCOMPOSIZIONE ADDITIVA — MODEL-GRADE(src=m85.dl28) (A-BG42,
  Concilio WP-87)**: il run a protocollo rotto (hello dispatchato due
  volte) ha prodotto il pad come SECONDO main di thr0: net(ord2) =
  460.146 B — e l'algebra chiude AL BYTE: residuo one-time del THREAD =
  7.343.135 B [derivata: 7.803.281−460.146, MODEL-GRADE(src=m85.dl28)]
  · hello-own = 6.842 B [derivata: 7.349.977−7.343.135, MODEL-GRADE(src=m85.dl28)]
  · pad-own = 460.146 B [MODEL-GRADE(src=m85.dl28)] · verifica:
  7.343.135+460.146 = 7.803.281 B = NET_P ESATTO [derivata: somma dei due
  addendi]. **Classe**: cifre da raw VOID — legali SOLO come MODELLO con
  tag di provenienza, MAI load-bearing in una delibera (KS-SK-87-4/
  KG-87-2). Confound noto (Bak WP-87): il walk a 2 entry mostra
  996.838 B [derivata: standalone 4.283.308+4.633.332=8.916.640 −
  retained_walk(2 entry) 7.919.802] condivisi ⇒ pad-own a ord=2 può
  sottostimare lo standalone; contro-prova A-BB45 (ordine invertito,
  predizione hello-own(ord2)=6.842 B [derivata: v. algebra sopra]) prima
  di ogni promozione. La domanda A-DL27 («di chi è il residuo») ha ora la
  metà CONTATA scomposta per costruzione; resta la metà FISICA (A-DL31).
- **VP — R=9, pin 232±1 MOSSO confermato; NESSUN pin nuovo deliberabile
  (KL-86-1)**: peaks 232.079.360 / 243.433.472 / 252.526.592 /
  238.157.824 / 241.844.224 / 242.384.896 / 243.384.320 / 235.634.688 /
  250.085.376 B (W=10 R=9 arm=union fixture=hello.php
  driver_sha=54717a9a — scope A-BG38 completo); r1 NOMINATO:
  232.079.360 B = 221,3 MiB con wall 4,94s (r2..r9: 3,28–3,40s) — stavolta r1 è il
  MINIMO; spread r2..r9 = 16.891.904 B = 16,11 MiB [derivata: max−min di
  r2..r9] NON attribuito ⇒ per KL-86-1 la delibera di un pin identità è
  NULLA. Citazione legale: **envelope max = 252.526.592 B = 240,8 MiB**
  (sostituisce l'envelope 240.287.744 B = 229,2 MiB della campagna-84).
  **EMENDATA S-86.0 (A-BG41/KG-87-3)**: il confronto cross-campagna è
  legale solo coi lati NOMINATI — lato-85: driver_sha=54717a9afe6ccb96,
  R=9; lato-84: driver_sha=fe6983d8fee7d0c2, R=3 (delta driver = solo il
  wrapper di campagna, measure78.sh invariato — sanatoria Gregg a
  ricomputo). «SALITO» NON è un regress dimostrato: max su R=9 domina
  stocasticamente max su R=3 e la
  differenza 12.238.848 B = 11,67 MiB [derivata: 252.526.592−240.287.744]
  è DENTRO lo spread non attribuito
  16.891.904 B = 16,11 MiB [derivata: max−min di r2..r9]. KB-86-1
  soddisfatta nella forma (R=9, primo
  run nominato); l'attribuzione dello spread è materia A-DL30/A-DL31
  (purge timing, first-touch — candidati nominati dal Concilio).
- **Guardia KS-PP-85-1/A-PP31**: 0 righe reqns senza `w=1` nei raw.

## ⚖️ Delibera peak (eseguita QUI, nelle forme del Concilio WP-86 punto 5)

1. **×W ACCETTATO, verdict-grade — RIFORMULATA S-86.0 (A-BG42): la
   delibera si appoggia SOLO a m85.dl28s (VERDICT-GRADE)**: nel run W=2
   una-richiesta-per-worker ogni thread netta ESATTAMENTE la cifra
   calibrata della PROPRIA fixture (hello→thr0 = NET_H 7.349.977 B, pad→
   thr1 = NET_P 7.803.281 B, byte-exact) ⇒ il costo retained del main
   cache è PER-THREAD e il budget upper 20.648.477 B = 19,69 MiB × W
   (record WP-83) REGGE. La scomposizione «residuo 7.343.135 B [derivata:
   7.803.281−460.146] + own» è SOLO illustrazione
   MODEL-GRADE(src=m85.dl28) — mai addendo di questa delibera
   (KB-87-1/KS-SK-87-4). Perimetro (KL-87-2/A-BB47): PER-THREAD
   è provato per il protocollo SEQUENZIALE — l'estensione a dispatch
   concorrente, W>2 o fixture con distruttori cross-richiesta esige
   re-canary sotto overlap. Il trade resta quello di WP-82 (costo ×W
   contro il churn per-richiesta risparmiato dal HIT — record WP-81; VC
   ~55–75× su 7a610457).
2. **Pin identità peak: RITIRATO** — non rifondabile oggi (KL-86-1:
   spread 16.891.904 B = 16,11 MiB a R=9 non attribuito; il vecchio
   232/232/232 era con ogni probabilità quantizzazione d'arena, Bak
   WP-86). Fino ad attribuzione: SOLO envelope
   max 252.526.592 B = 240,8 MiB (W=10).
3. **Registry condivisa: RESTA BLOCCATA** — KB-86-2 sbloccata dal canary
   MA KS-MS-86-2 vincola alla partizione-per-TIPO A-MS27 (backlog
   nominato) + A-BB35 + riapertura esplicita KH81-3.

## Aperture dichiarate (per NOME — mai chiusure in silenzio)

1. **A-DL27/A-DL31 (p7)**: metà contata SCOMPOSTA (sopra); la metà fisica
   (mi_bin thr=, vmmap purge=0, chiusura additiva ±5%) NON eseguita in
   S-85.0 — prossima campagna; A-DL30 (phys_peak<phys da spiegare, righe
   mi_proc escluse dal corpus fino ad allora, KL-86-3) vive lì.
2. **Attribuzione spread VP**: senza, il pin peak resta RITIRATO
   (envelope-only). Candidati dal Concilio: purge timing in-band,
   first-touch, ordine spawn heap.
3. **A-MS27**: precondizione della via registry (KS-MS-86-2), backlog.
4. **m85.dl28 (campagna-1)**: raw VOID per protocollo rotto, SUPERSEDED
   da m85.dl28s (dl28-supplement.sh), tenuto in place mai rm
   (KS-AH-83-2) — ha fruttato la scomposizione additiva.
5. **KH86-1 refutazione parziale del meccanismo**: il binario contaminato
   (feature accesa) ha hash DIVERSO ma NESSUN simbolo vm_gate_probe
   (dead-strip, nessun caller nel bin) — il dente nm è necessario ma non
   sufficiente; la metà che morde è hash==matrix (cablata nel gate).
   → Concilio WP-87.
6. **ROADMAP (p8)**: non ripresa in S-85.0 (campagna+sigilli hanno
   esaurito la sessione, dichiarato).
7. **A-AH42 (retro-dichiarazione S-86.0, Concilio WP-87)**: A-AH38
   (scope-file su split_lock) + dry-run KS-AH-86-1 legittimamente NON
   eseguiti in S-85.0 (nessuna fase slope/base-arm, A-BB43) ma NON
   dichiarati — la disciplina «mai chiusure in silenzio» vale anche per
   le NON-esecuzioni ereditate. Vincolo: il dry-run resta OBBLIGATORIO
   prima del prossimo uso base-arm, e A-AH38 deve atterrare PRIMA del
   dry-run (un dry-run sulla v2 vecchia certifica l'oggetto sbagliato).
8. **A-BB48 (retro-dichiarazione S-86.0, Concilio WP-87)**: A-BB44
   (terzo punto VW len≥500) NON eseguito in S-85.0 NÉ dichiarato —
   chiusura silente, sanata qui. Esecuzione ordinata in S-86.0 con
   predizione nominata ex-ante: len=500 ⇒ a_calls=4, a_bytes =
   2.002 B [derivata: 2·500+2·501].
