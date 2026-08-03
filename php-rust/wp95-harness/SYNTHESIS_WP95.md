## ⚖️ SINTESI DI CONVERGENZA — Concilio WP-95 su S-93.0 (compilata dalle ricevute fase 1+2 + estrazioni mirate; verbali = fonte VINCOLANTE)

**Verdetto complessivo: 9 sedie, 8 CON EMENDAMENTI + 1 REFUTATO (Klabnik).
Nessuna opposizione al lavoro dell'OGGETTO; DUE refutazioni capitali di
apparato (autorità gate + canale stat) e una retrocessione di grado
convergente 4/4 sul probe. Tre team di fase 2 (cifre, misura, leva).**

### §FONDAMENTALI (in testa, per direttiva utente 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: POSITIVO, per la prima
volta dopo due sessioni di sola ri-analisi. Sappiamo oggi di phpr tre cose
che ieri non sapevamo: (1) i «40 MB mai liberati per worker» sono l'arena
di parse del PRELUDIO stdlib (sei chunk bumpalo in raddoppio), **spike
transiente liberato alla prima richiesta**, non una riserva né un leak;
(2) `malloc_huge` di m90 è **cumulativo per costruzione** (MI_STAT=0 elide
il decremento C) — l'attribuzione huge-worker.out di WP-92 andava
superseduta; (3) il CLI di parità paga **4,42× l'oracle su hello** per lo
stesso preludio → candidato n.1 del roadmap footprint, ma la priorità è
dichiarabile **solo dopo una coppia full fresca**.

**(b) Contatore sessioni-senza-misura**: ultima full/media = **WP-85, OTTO
sessioni fa**; ultima campagna sull'oggetto = m90 (WP-90). S-93.0 ha
prodotto misure di PROBE (R=1), non verdict-grade. Il contatore NON si è
azzerato: serve la coppia full.

**(c) Rischio d'oggetto più trascurato ORA**: la leva footprint viene
governata dalla regola WP-48 (predizione-misurata) ma **manca il «prima»
fresco** — otto sessioni senza full. Rischio: attuare la leva per-file
contro una baseline stantia e non poter firmare il guadagno.

**Regola di ammissione all'ordine S-94.0**: l'unico apparato ammesso è
quello che BLOCCA il prossimo passo sull'oggetto — la riparazione
dell'autorità del gate (A-SK-82 è AGGIRATA, PASS forgiati verificati a
macchina: ogni cifra futura nascerebbe senza autorità) e il deadlock/panic
latente nel canale di misura. Tutto il resto → BACKLOG PER NOME. Timebox:
mezza sessione d'apparato, tetto duro.

### Refutazioni capitali

1. **🔴 A-SK-82 è AGGIRATA — tre canali producono un `PASS --all` rc=0
   firmato col judge_sha pristino su codice NON pristino (Klabnik, 7/9
   forge; RIPRODOTTO dal team-cifre a macchina, HEAD a9a1b364)**: (a) env
   `BASH_SOURCE` iniettato; (b) symlink logico/fisico; (c) `BASH_ENV` +
   funzioni esportate. La classe comune: **il giudice autentica stringhe
   scelte dal chiamante, mai gli artefatti che il kernel legge**. Cura
   minima UNICA (un commit): re-exec sanificante `exec env -u BASH_ENV -u
   ENV -u SHELLOPTS bash -p "$SELF_PHYS"` su path FISICO (`cd -P`/`pwd -P`)
   come PRIMO atto, marker anti-loop VALIDATO (non una env che salta il
   re-exec), + falsificatore che pretende rc esatto sui tre canali. →
   A-SK-88/89/90/91, KS-SK-95-1..4.
2. **🔴 La statistica `malloc_huge` è cumulativa PER COSTRUZIONE, non
   «monca per assenza di free» (Leijen, meccanismo per NOME)**: MI_STAT=0
   in release (libmimalloc-sys 0.1.49 build.rs → types.h) compila VIA
   `mi_stat_free` (free.c:612, stub 635-637) mentre l'increase huge è
   incondizionato (page.c:935); theap.c:362 è dead code. Corollario
   refutato: «committed invariata ⇒ nessun decommit» NON segue — in
   release committed non scende mai su purge (prim.c:504, os.c:590-591).
   La conseguenza-2 di huge-sites.out era scritta come «non morde»: va
   «non è compilato». → A-DL-65..68, KS-DL-95-1/2.

### Retrocessione di grado — probe B3 (convergenza 4/4: Bak, Pedersen, Gregg, Leijen)

Il probe slope di B3 è uno **SCREEN, non una misura**: R=1 per punto, due
soli W, delta 21837 SOTTO lo spread inter-build 38229 B/worker dello stesso
slope. «REFUTATA CON MISURA» è **RITIRATA** → «effetto non rilevato sotto la
risoluzione»; l'ADVISORY sulla gamba m90 è ritirato (poggiava sull'inferenza
committed refutata da Leijen). L'arm senza readback in banda è muto
(Pedersen): un delta nullo è indistinguibile da un arm mai scattato.

### Refutazione della CIFRA della leva (Matsakis + Stogov, team-leva)

«peak arena ~74 KB» confondeva SORGENTE e ARENA. Cifra difendibile
ricostruita a residuo zero dal team-leva: `allocated_bytes=39534144` è la
CAPACITÀ; il **touched reale è 25.795.552 B** (11 chunk; `chunk_capacity`
= residuo libero dell'ultimo, non «coda mai toccata»). Predizione ex-ante
WP-48: N = 25.795.552 − T_max; D = 44.630.520; peak_post = D − α·N con
α∈[0,8;1,0] → **2,3-2,7× oracle** (falsificata se >40MB o <21MB). Il
~500× è refutato, il ~5,5× è capacità. Audit vivo di Stogov: **zero
forward-reference cross-file** ⇒ la leva per-file è semanticamente
percorribile; il rischio vero è la sentinella `b"prelude"` osservabile.

### Convergenze forti (dai team)

- **team-cifre**: una cura di classe chiude tutti e tre i canali (re-exec
  sanificante + path fisico); l'apparato ENTRA in S-94.0 perché non è un
  gate nuovo ma la riparazione di uno rotto e verificato aggirabile.
  Ledger battery: `writer=` va autenticato contro lo sha del battery a HEAD
  (A-AH-71), `.done` per-RIGA (A-AH-69) — percorso di consumo di
  battery61/91pre.
- **team-misura**: 13 atti di sanatoria di carta in UN commit (già
  applicati: somma 39423200→39223200, residuo 488224→688224, B3→SCREEN,
  coerenza→REFUTATA, «non morde»→«non compilato», ripristino pin→dichiarato,
  huge-worker.out→SUPERSEDED-IN-PART). Ordine: sanatorie → **coppia full
  stessa-sera (prima misura, nasce verdict-grade)** → battery61 → probe
  slope v2 FUSO (MI_STAT=1 + coppia alloc/free in-band + eco d'arm fired==W
  + R≥5 interleaved + doppia metrica peak+residency). Leva per-file
  ESCLUSA da S-94.0 (nessuna leva prima del giudice).
- **team-leva**: rank unico 1) per-file+`reset()` (1b pre-size separato),
  2) precompilato embedded, 3) condiviso Arc, 4) lazy; tie-break 2↔3
  deciso da due misure (residuo CLI post-leva-1 ≥2× oracle → 2; m91 nomina
  ≥50% dello slope come live PRELUDE_CACHE → 3). 16 obblighi di prova
  ordinati in 3 fasi, ciascuno col giudice; controllo positivo del
  contatore per-unità `Σ T_i ≈ 25,8 MB ±10%` (lezione WP-72).

### Delibera di consumabilità

**S-93.0 è consumabile in ADVISORY dopo le sanatorie (APPLICATE in questa
chiusura).** B1 regge in ADVISORY (catena di raddoppio VERDICT come
identità aritmetica); B2 regge riformulato; B3 è SCREEN (nessun pin). La
falla A-SK-82 è would-have-allowed a HEAD ma il forge è VIVO: la
riparazione è la prima voce di S-94.0.

### Ordine vincolante di apertura S-94.0 (FONDAMENTALI-first)

**P0 (precondizione, già fatta in chiusura WP-93)**: sanatorie §5 team-misura
in un commit — nessuna cifra nuova mentre agli atti resta una somma sbagliata.

**Mezza sessione d'apparato (tetto duro, ordine di taglio A3→A2, A1 mai)**:
1. **A1 = A-SK-89 + A-SK-90 + A-SK-88 + A-SK-91** (blocco unico): path
   fisici, re-exec sanificante come primo atto, marker validato,
   falsificatore rc-esatto sui tre canali. Assorbe la sotto-portata di T23.
2. **A2 = A-AH-71** (writer autenticato contro sha del battery a HEAD).
3. **A3 = A-AH-69** (`.done` per-RIGA: 4 campi dalla riga che porta rev=$BREV).
4. **A4 = A-TH-73 + A-TH-74** (env-read fuori dall'allocatore, nessun
   panic-path nel GlobalAlloc — non è apparato, è UB latente nel canale
   di misura).

**L'OGGETTO (non conta nel timebox, è il corpo della sessione)**:
1. **COPPIA FULL stessa-sera** (media + peak footprint + CPU): prima
   misura, verdict-grade, precondizione della leva (contatore fermo a WP-85).
2. **battery61 riproducibile in modo nativo** (criterio 5, debito 31
   sessioni). *Dissenso ordinale registrato: Bak/Pedersen la vogliono al
   posto 1; il team-misura al 2. Se il tempo basta per una sola, la scelta
   è del plenario — qui: coppia full prima, è precondizione della leva.*
3. **Probe slope v2 FUSO** = canale unico di m91 (i quattro emendamenti in
   un solo strumento, non tre probe): MI_STAT=1 dichiarato, coppia
   alloc/free in-band, eco d'arm `fired==W`, R≥5 interleaved W∈{1,2,4}
   mediana±2se, doppia metrica peak+residency, `huge_note` simmetrico su
   realloc.

**S-95.0 (NON S-94.0)**: leva arene per-file del preludio, con i 16
obblighi del team-leva, predizione ex-ante firmata (2,3-2,7× oracle) e
gate parità COMPLETI + ricert. baseline phpr nello stesso commit.

**BACKLOG PER NOME** (non «più avanti»): A-SK-92-PROBE (grado rc=65),
A-AH-70/74/75 (ancore ledger), A-AH-73 (HIR plain-data, precond. leva #2),
audit A-BG-72 (derivate m90 che consumarono malloc_huge come retained).

### Kill-switch nuovi consolidati (attivi da subito)

KS-TH-95-1/2/3 · KS-MS-95-1/2/3 · KS-SK-95-1..4 · KS-AH-95-1/2/3 ·
KS-BB-95-1/2 · KS-PP-95-1/2/3 · KS-DL-95-1/2 · KS-DS-95-1/2/3 · KS-BG-95-1/2
— tabella nei verbali. Ereditati ATTIVI: WP-94 (22) + WP-93 (23) + WP-92
(22) + WP-91 (27) + WP-88..90. **KS-SK-91-1 resta NON sollevabile.**

**NON riproporre (nuovi)**: «tether su una stringa che il chiamante
sceglie» (A-SK-82 su `$0`/`BASH_SOURCE` da solo — KS-SK-95-1); «un
contatore stat come prova di retention senza leggere il #if di build»
(MI_STAT, KS-DL-95-1); «committed invariata ⇒ nessun decommit» (REFUTATA);
«refutare una leva con un probe R=1» (SCREEN, mai refutazione — KS-BB-95-1);
«peak arena = taglia sorgente» (confonde sorgente e arena); «leva prima del
giudice» (nessuna leva footprint prima di una coppia full fresca).
