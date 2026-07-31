# design78.md — WP-78 protocollo di misura (A-BG3/A-DL3, Council WP-78; emendato S-78.1.7 per Council WP-79)

**Stato**: protocollo scritto in S-78.0 (sanatoria), PRIMA di qualunque run di
misura; EMENDATO in S-78.1.7 (A-DL8/A-BG8/A-BG9/A-BG10/A-BB9 + KG-79.B/C/D).
Nessuna cifra è valida se prodotta fuori da questo protocollo
(KG-78.D: cifra senza run tracciato = respinta d'ufficio).

## Identità del binario (KS-AH-78-1)

Ogni run cita: hash sha256[0:16] di **php-server** (mai phpr — A-SK5), feature
set, e il log `gate-axum/out/feature-matrix.log` della stessa build. Prima di
ogni batch di misura: `gate-feature-matrix.sh` PASS + `gate-axum/run-gate.sh`
PASS sulla build che verrà misurata.

## Configurazione di misura (A-BG3, KB-78-3, KG-78.A)

- **`--workers 1`** per ogni misura alloc/footprint (il default num_cpus
  moltiplica il footprint retained per N e spalma le richieste ≈N/W per worker:
  steady-state per-RetainSet mai raggiunto).
- Carico **chiuso-sequenziale** — MECCANISMO (A-BG9, KH78-2 riformulato): il
  client emette la richiesta N+1 SOLO dopo aver ricevuto e letto per intero la
  risposta N (curl sequenziale in un unico loop; mai `&`, mai keep-alive
  pipelining). OSSERVABILE: contatore di profondità del canale mpsc
  (strumentazione §Contatori, build census-only) — un campione con depth >1
  smentisce il meccanismo ⇒ run VOID (KH78-2/KB-78-2).
- **Warm-up ridefinito** (A-BG8, KG-79.C): le prime 10 richieste servono a
  raggiungere lo steady-state e sono escluse SOLO dalle metriche PER-RICHIESTA
  (alloc/req, delta RSS per-richiesta, delta contatori). Le metriche di PICCO
  processo-lifetime (`/usr/bin/time -l` peak footprint, mimalloc peak) coprono
  l'INTERO processo warm-up incluso e si riportano così: «picco = intero
  processo» — un "warm-up escluso" su una metrica di picco è una cifra
  respinta (KG-79.C).
- **R≥3** run; spread = (max−min)/media, accettato **≤2%** (KB-78-4/KG-78.B:
  oltre ⇒ si indaga la variance, MAI si media).
- Ogni A/B in **coppia build-adiacente stessa-sera** (WP-57/65), stessa
  versione mimalloc nel Cargo.lock.

## Metriche (A-DL3, KL-78-3)

- **Peak fisico**: `/usr/bin/time -l` (richiede uscita PULITA: SIGTERM al
  server — implementato in S-78.0.7; kill -9 = zero stats = run nulla).
  S-78.1.2: l'uscita pulita ora JOINA i worker (`shutdown()`); una cifra a
  exit senza la riga «all N workers joined» nel log è ADVISORY (KS-MS-5/
  KL-78-4). Un panic worker durante il run ⇒ run VOID (KS-PP-6, il processo
  abortisce — gate-worker-panic.sh).
- **Residente**: `vmmap <pid>` Physical footprint, con `MIMALLOC_PURGE_DELAY=0`.
- **Snapshot vmmap DICHIARATI** (A-DL8) — tre punti fissi, sempre gli stessi:
  (V1) post-warm-up (dopo la richiesta 10, coda vuota); (V2) steady-state
  (dopo l'ultima richiesta misurata, coda vuota, PRIMA del SIGTERM);
  (V3) assente per definizione dopo SIGTERM (il processo esce — il residuo
  post-teardown si legge dalle stats mimalloc a uscita pulita, non da vmmap).
  Ogni snapshot cita richiesta-indice e timestamp; transiente vs residente =
  V2 − V1 vs V1.
- **Stats allocatore**: `MIMALLOC_SHOW_STATS=1` a uscita pulita (post-join).
- **VIETATO**: RSS di `ps` (mente su macOS — WP-20/52).

## Attribuzione per-fase (A-BB2, KB-78-1, A-AH4/KS-AH-78-3)

Il census riporta contatori alloc SEPARATI per (a) lower+compile,
(b) run, (c) shutdown/end — predizione-misurata: dichiarare il contatore del
canale PRIMA della run (WP-45..48). Il verdetto A-BB1 (≤+2% alloc) si emette
su (b)+(c) E sul totale, riportati separatamente; un numero unico = VOID.
Se compile > 50% dell'alloc/req e il report non è per-fase ⇒ A-BB1 VOID
(KB-78-1); compile > 30% ⇒ A-BB1 NON GIUDICABILE finché l'attribuzione non
isola il canale (KS-AH-78-3). La cache Module è POST-censimento (A-BB6:
frequenza×taglia prima di progettare — WP-57).

## Denominatore A-BB1 (A-BG4, emendato A-BG10/KG-79.B)

Confronto onesto: **stesso binario** (union build), stessa fixture, stesso N,
`--cli-server` vs `--axum`. Il "≤+1.5% misurato" del verbale .3 è una
PREDIZIONE da verificare, non una baseline.

**Verbalizzazione A/B (A-BG10)**: il delta A/B è la SOMMA di
{runtime tokio + canale mpsc + pool/worker + registry-once-per-worker +
retention RetainSet}, mai etichettato «overhead del pool» da solo; il
verbale riporta la decomposizione per contatori (§Attribuzione) o dichiara
«delta aggregato non decomposto».

**Lifecycle del braccio cli-server (KG-79.B)**: il ramo `--cli-server` NON ha
oggi un percorso di uscita pulita equivalente (niente SIGTERM drain+join) ⇒
le sue stats a exit non sono verdict-grade. Una coppia A/B in cui un braccio
non ha stats a uscita pulita è VOID per le metriche exit-based; per quel
braccio si usano SOLO snapshot vmmap in-run (V1/V2) e si dichiara
esplicitamente «exit-stats: non disponibili sul braccio cli-server» finché il
drain pulito non viene implementato e gateato.

## Probe di amplificazione (A-BG5, KG-78.C, KL-78-1/2)

- RSS(N) vs RSS(2N): delta per-richiesta intero-esatto ≈0 (prova regina
  WP-71/72). Delta strutturalmente >0 a steady-state ⇒ HALT, riapertura C-leak.
- footprint(W): misurare W=1 e W=num_cpus con ≥100 richieste PER worker;
  atteso ≈ base + W·k (non-linearità ⇒ HALT, KL-78-2).
- Firma intera-esatta per-worker (A-DL4): contatore entries RetainSet per
  richiesta, R≥3, con controllo POSITIVO (fixture-leak che il contatore DEVE
  vedere — WP-71/72).

## Gate concorrente prima del census (KH78-1)

G-APERTURA-2 è sequenziale. Prima del census: gate con richieste CONCORRENTI
su N worker (fixture stateful); panic o divergenza ⇒ HALT.

## Contatori census (S-78.1.6 — A-BB7/A-BB8/A-BG7/A-DL7, KB-78-5/6, KL-78-5, KG-79.A)

Strumentazione SOLO dietro feature `census-instrumentation` (build separata):
- contatori alloc per-fase (a=lower+compile, b=vm+run+render, c=shutdown/end) — A-BB7;
- per-worker: richieste servite, `RetainSet::len()` (S-78.1.5: il RetainSet è
  il PIN della SINGOLA richiesta — atteso = n° unit distinte incluse dalla
  richiesta, costante; >n = parking duplicato ⇒ KS-DS-78-4), `live_objects()`
  post-request_end (l'osservabile used_n WP-72, atteso 0 ⇒ KS-DS-78-2) — A-BG7/A-DL7;
- profondità canale mpsc campionata a ogni dispatch — A-BB8 (osservabile di KH78-2).
OGNI contatore nasce con un CONTROLLO POSITIVO (fixture che DEVE muoverlo:
contatore visto muoversi almeno una volta nel run o census VOID — KG-79.A).
**I verdetti footprint/peak escono SOLO dal gemello NON strumentato**
(KB-78-5/KL-78-5: cifra da build strumentata = NULLA); la build strumentata
produce solo contatori.

## Tier-0 (KG-79.D — configurazione ESEGUIBILE tracciata)

Tier-0 = pavimento Axum: binario UNION, flag `--tier0` (S-78.1.6): il server
risponde `200 "tier0\n"` dal handler stesso — NESSUN pool, NESSUN worker,
NESSUNA Vm istanziata. Comando tracciato:
`php-server --axum --tier0 --port P` + hash binario + feature-matrix.log
della stessa build. Qualunque «Tier-0» privo di questo comando = baseline
nulla (KG-79.D).

## Micro-igiene path caldo (A-BB9 — eseguita in S-78.1.6)

- `file_s` (lossy String del path) calcolata SOLO nel branch d'errore, mai
  nel fast path (prima: alloc per-richiesta contata come overhead Axum).
- `meta.path` non viene clonata nel fast path: `lower_source` riceve il
  borrow dei byte del path.

## Ordine della fase misura

1. Gate: feature-matrix + run-gate + gate concorrente (KH78-1) sulla build da
   misurare.
2. Tier-0 baseline (`--tier0`, §Tier-0).
3. Census per-fase `--workers 1` (A-BB1 si giudica qui).
4. A-DL1 frammentazione (vmmap + mimalloc stats, transiente vs residente).
5. Probe amplificazione (A-BG5) + linearità in W (A-DL2).
6. R-G4 (CPU attribution) SOLO se commissionato, con contatori ns/evento
   owner-level su coppia strumentata/non-strumentata (A-BG6; mai solo `sample`).
