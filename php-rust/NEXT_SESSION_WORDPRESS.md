# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **WP-99 (QUESTA
sessione — contatore azzerato, era 4)** · ultima campagna sull'OGGETTO =
**S-99.0 (questa: sole misure — collaudo per NOME chiuso, sei categorie
ri-baseline sui due motori, pre-misura del rollout, tre pin collaudati)**.

**Ultima sessione (S-99.0, 2026-08-05)**: l'ordine del Concilio WP-100
eseguito PER INTERO più il condizionale del timebox. (1) **Parità server**:
il pin 365f4d40 era **REFUTATO per costruzione** (compilato senza la
feature `axum-server` — recidiva WP-77.6.5.2.3, pin nato come effetto
collaterale); ricostruito CON RICETTA e collaudato rc=0 (sentinella
output-capture + option 413 + restapi 3508 per NOME sotto env -i);
`PIN_REGISTRY.md` creato. (2) **Collaudo WP full+media CHIUSO PER NOME**
(unica divergenza = catalogo `wp_is_stream` ftp): il debito regola-n.2 di
H-A2+H-B2 è SALDATO; coppia peak: full CPU 1,893× · full peak 2,374×
(phpr 1892,56 MiB, PIATTA vs WP-94). (3) **Ri-baseline sei categorie sui
DUE motori**: arith 17,5 · prop 13,8 · calls 8,6 · str 6,9 · arr 4,9 ·
re 3,8 — gamba oracle sanata, **H-C e H-D rianimate**. (4) **Pre-misura
del rollout**: controfattuale statico → **il rollout Add nelle forme
registro ha predizione ZERO (banda [0, 0,5] ns/occ)** — cadrebbe a
tavolino; build INT1 → **D = 6,27 ns/occ = 3,60 call/marshalling (57%) +
2,67 traffico Vec (43%)**; baseline flag-on in finestra (arith on 5,44,
INVARIATA da S-97.1 ⇒ emissione flag-on davvero bit-identica; vantaggio
flag-on −26,9%). (Timebox) **Sigillo eager** di `reg_lower::enabled()`
nei due main + dente anti-putenv a 2 bracci; batteria 1726/0; rotazione
pin COLLAUDATA in sessione (corpus 1418 per NOME off E on; ri-parità
server rc=0). Dettaglio: `sessions/WP_SESSION_99.md` + i tre `.out` di
`wp99-harness/`.

**Cambio di rotta deciso dall'utente (2026-08-04)**: si va dritti al PHP;
WordPress = collaudo di PARITÀ. OBIETTIVO **X = nucleo interprete ≤ 3×
l'oracle sulle categorie pure**; giudice = le sei micro-categorie di
`wp97-harness/micro/`.

## Il fatto che ha cambiato tutto (S-97.0, confermato da S-99.0)

Misurando lo STESSO sorgente su entrambi i motori per categoria, al netto
dei pavimenti di avvio (che sono DIVERSI — non sottrarli = fattore 3 di
errore): il nucleo interprete è oltre un ordine di grandezza più lento;
le estensioni riscritte (regex 3,8×) sono la parte sana; l'aggregato
WordPress (1,9×) è diluito da I/O, DB e builtin. Baseline CORRENTE
(S-99.0, `wp99-harness/micro-rebaseline99.out`, flag-off):

| categoria | rapporto | ipotesi attiva |
|---|---|---|
| aritmetica | **17,5** | percorso pila: rollout Sub/Mul/cmp; percorso registro: PROMOZIONE flag-on |
| proprietà | **13,8** | **H-C ATTIVA** (≫ 5×) |
| chiamate | **8,6** | **H-D ATTIVA** (≫ 5×) |
| stringhe | 6,9 | — |
| array | 4,9 | — |
| regex | 3,8 | la parte sana |

## LE IPOTESI — stato dopo S-99.0

- ~~H1 lettura operandi~~ · ~~H-A1 (−30,7% TENUTO flag-on, criterio
  −40% non raggiunto)~~ · ~~H-B1 preambolo (P=0%, a tavolino)~~ —
  refutate/chiuse, dettagli nelle rotazioni precedenti.
- **H-B2 specializzazione per tipo**: meccanismo PROVATO e ora DECOMPOSTO
  (57% call/marshalling + 43% traffico Vec, `premisura-rollout99.out`).
  La coda ha DUE gambe con destini diversi:
  - **percorso PILA (flag-off)**: Sub/Mul/compare int-int — ogni
    occorrenza col SUO controfattuale e criterio pre-registrato
    (KS-BA-100-1), frequenza dell'op nel giudice a scalare l'atteso.
  - **percorso REGISTRO**: predizione D_registro = 0, banda [0, 0,5]
    ns/occ (le forme inlineano già `binary_fast` su prestiti) ⇒ un
    rollout lì **cade a tavolino** sul criterio pre-registrato in
    `premisura-rollout99.out`. NON è più un asse.
- **⭐ PROMOZIONE flag-on a DEFAULT = OBIETTIVO NOMINATO** (decisione
  utente 2026-08-05). Gate già SODDISFATTI in S-99: corpus flag-ON per
  NOME (due volte, stesso albero) · parità server collaudata · sigillo
  eager + anti-putenv (gate di promozione KS-HE-100-1/KS-PE-100-2) ·
  smoke con controllo positivo (funnel {main}) · coppia peak stessa-sera.
  Gate RESIDUI per NOME: **diff riga-per-riga** dei corpus (A-KL-100-2:
  il gate per NOME è un bit/fail — la promozione esige il confronto
  degli output) · sanatoria dump/`lowered()` sugli hook riscritti
  (A-HE-100-4) · stdout/exit pinnati nel funnel (A-KL-100-1) · braccio
  flag-OFF nel funnel con dump-assert zero forme registro (A-PE-100-4) ·
  collaudo WP full+media + coppia peak IN MODO flag-on stessa-sera.
- **H-C (proprietà, 13,8×)**: si attiva ORA. Primo passo = misura, non
  codice: decomposizione conteggio × costo/opcode su `prop.php` (stessa
  tavola di arith-decomposition) + census degli opcode del ciclo prop.
- **H-D (chiamate, 8,6×)**: attiva, in coda dopo H-C.

### ⚖️ Concilio WP-100 (su S-98.0/S-99.0) — ESEGUITO PER INTERO in S-99.0

Verbali in `wp100-harness/`. Ordine eseguito: 4/4 punti + condizionale.
Emendamenti SALDATI in S-99: A-PE-100-1/2/3 (parità-prima, sigillo eager
+ anti-putenv, registro pin), A-BA-100-1 (build decomposizione),
A-ST-100-1 (controfattuale registro), A-GR-100-1/4 (misure insieme,
ri-baseline due motori), A-LE-100-1 (peak con time -l + env WP-94).
BACKLOG per NOME che resta (non slot di sessione se non bloccante):
A-KL-100-1/2/3, A-HE-100-1/2/3/4, A-ST-100-2/3 (sette trappole AssignOp),
A-HO-100-4, A-LE-100-3 (N_OPS≤255: ora 186/256), A-BA-100-2/3,
A-KL-100-4/5, A-PE-100-4, A-GR-100-3.

### ⚖️ Concilio WP-101 (su S-99.0 e programma S-100) — vedi blocco in coda

## Regole di metodo (invariate)

1. Il giudice è la micro-categoria. 2. WordPress è un collaudo di PARITÀ
(si esegue quando cambia l'emissione). 3. Ogni ipotesi porta il criterio
di caduta scritto PRIMA. 4. L'apparato non entra nell'ordine se non
blocca; timebox mezza sessione.

## Stato gate

- **phpr (parità release)**: **52330330873f0132** — HEAD 4c34f61, col
  sigillo eager A-PE-100-2 (l'hash churna col relink del test-funnel: fa
  fede HEAD). Corpus Zend **1418 per NOME IDENTICO flag-OFF E flag-ON**
  (`wp99-harness/evidence/corpus-s99-{off,on}.fails`). Batteria 1726/0.
  Stash additivo `phpr-s99-sigillo`.
- **php-server**: **a838866e134b6a20** — **collaudato: sì** (ri-parità
  rc=0: sentinella output-capture + option 413 + restapi 3508 per NOME
  sotto env -i). Ricetta OBBLIGATORIA: `cargo build --release -p
  php-server --features axum-server` (i default NON la includono —
  recidiva WP-77). Stash `php-server-s99-sigillo`. **Registro con campo
  `collaudato:` = `PIN_REGISTRY.md` (repo root)** — vietato il pin
  effetto-collaterale, vietata la rotazione non collaudata.
- **Launcher riusabili S-99**: `wp99-harness/s99-parity-server.sh`
  (parità server sotto env -i; aggiornare PIN_SRV_ATTESO a ogni
  rotazione) · `s99-corpus-gate.sh` (corpus off+on per NOME; i fail del
  runner sono INDENTATI di 2 spazi) · `pair99.sh` (coppia full+media
  con failure per NOME).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).
- ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA (rust-analyzer):
  rimuoverla nel pre-flight. ⚠️ `/Volumes/Extreme Pro/Claude/
  phpr-int1-target/` = target-dir della build di misura INT1,
  cancellabile in qualunque momento.

## Che cosa è SOSPESO (non abbandonato)

- **A-ZV2** (liveness+TakeSlot): invariata (guadagno ~1% dove serve un
  fattore; buchi di soundness nominati dal Concilio WP-98 ancora aperti).
- **Rollout Add nelle forme registro**: REFUTATO in pre-misura
  (predizione 0, banda [0,0,5] ns/occ) — si riapre SOLO con una misura
  che mostri D_registro ≥ 0,7.
- **Roadmap footprint**: ferma (peak ora fotografato a 1892,56 MiB full).

## NON riproporre

Tutti i NON-riproporre WP-83..98 restano. Nuovi da S-99.0:

- **pin costruito come effetto collaterale di una build** (365f4d40:
  refutato senza eseguire nulla — mancava la feature; il pin si
  costruisce con la SUA ricetta e si registra in PIN_REGISTRY.md).
- **criterio del rollout ereditato da D del percorso pila** — ora è
  MISURATO: 57% call/marshalling + 43% traffico Vec, entrambi ASSENTI
  nelle forme registro.
- **estrarre i fail del phpt-runner senza l'indent** (2 spazi) e senza
  `tr -d '\0'`; **`while (<$h>)` dentro `map { t($_) }`** in perl
  ($_ read-only ⇒ die silenzioso dentro `{ } > file`).
- ereditati e ribaditi: aggregato WordPress come cronometro; profilo a
  un lato solo; pavimenti non sottratti; misurare con build concorrente;
  uccidere processi su una sola evidenza; criteri dal conteggio di
  istruzioni senza cammino critico.

---
**Riscritto**: rotazione S-99.0 il 2026-08-05. Apertura/chiusura =
skill `apri-sessione`/`chiudi-sessione`. Harness di sessione:
`wp99-harness/`.

## §S-100 — BOZZA d'ordine (da sottoporre al Concilio WP-101)

Oggetto: **la PROMOZIONE di flag-on a default** (obiettivo nominato) con
i suoi gate residui, e l'apertura di H-C con una misura.

1. **Gate residui della promozione, per NOME**: diff riga-per-riga dei
   corpus off/on (A-KL-100-2) · stdout/exit pinnati nel funnel
   (A-KL-100-1) · braccio flag-OFF nel funnel (A-PE-100-4) · sanatoria
   dump/`lowered()` (A-HE-100-4).
2. **Flip del default** (emissione: pass registro ON senza env) +
   collaudo COMPLETO della nuova emissione: corpus per NOME nei DUE modi
   + parità server (script 1a) + collaudo WP full+media + coppia peak
   stessa-sera IN MODO nuovo (regola n.2).
3. **H-C, prima misura** (solo misura): decomposizione conteggio ×
   costo/opcode su `prop` + census del ciclo + confronto col profilo
   oracle (dov'è il fattore 13,8: risoluzione slot, PropIc, hash?).
4. Se il timebox regge: controfattuale + criterio della PRIMA occorrenza
   stack-path della coda H-B2 (Sub o cmp int-int, scelta dal census).
