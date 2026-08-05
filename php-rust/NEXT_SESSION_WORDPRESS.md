# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **WP-100 (QUESTA
sessione — coppia nei DUE modi, contatore 0)** · ultima campagna
sull'OGGETTO = **S-100 (questa: FLIP eseguito + H-C prima misura)**.

**Ultima sessione (S-100, 2026-08-05)**: l'ordine del Concilio WP-101
eseguito 6/6, condizionale compreso. **LA PROMOZIONE È FATTA: flag-on è il
DEFAULT** (`DEFAULT_ON=true` @ fb861e4; opt-out `PHPR_REG_LOWER=0`,
grammatica VALUE-PARSED a lista chiusa — prima `=0` ACCENDEVA). Gate tutti
verdi: corpus 1418 per NOME nei 2 modi SUL PIN + diff per-test ZERO fuori
carve-out nominata (3 test `random_bytes`, nondeterminismo provato
intra-modo); server bimodale fails=0 sul pin (sentinella ESTESA 16
interleaved + 4 concorrenti, workers=2; option 413 + restapi 3508 per NOME
— prima esecuzione assoluta del server flag-on); coppia WP nei due modi con
bande pre-registrate (CPU on/off 1,008; peak on 0,965 = FAVOREVOLE);
batteria **1735/0** senza premesse ambientali (modo = INPUT del funnel,
`lowered()` eliminato). **H-B2 sotto flip decisa CON MISURA**: estensione
BinaryAdd ai siti stack residui (L=12,9 ns/occ → [−1,0]); giudice add on
3,25 netto (−31% vs off). **H-C prima misura**: prop on 12,4× = conteggio
2,0× × costo/op 6,2×; profilo co-equale: ~27% del tempo phpr nel CICLO DI
VITA Zval (drop 12,6 + clone 7,6 + gc_note 5,3 + deref_clone 2,1) vs
handler TAILCALL specializzati dell'oracle; **candidata H-C1 nominata**
(prestito/refcount al posto del clone su PropGet), nessuna riga scritta.
Dettaglio: `sessions/WP_SESSION_100.md` + `wp100-harness/*.out`.

**Rotta (utente 2026-08-04)**: dritti al PHP; WordPress = collaudo di
PARITÀ. OBIETTIVO **X = nucleo interprete ≤ 3× l'oracle sulle categorie
pure**; giudice = le sei micro-categorie di `wp97-harness/micro/`.

## Baseline CORRENTE del giudice (S-100, post-flip: il DEFAULT è flag-on)

| categoria | rapporto (default on) | ipotesi attiva |
|---|---|---|
| proprietà | **12,4** (misurata S-100; decomposta: 2,0 conteggio × 6,2 costo/op) | **H-C ATTIVA** — candidata H-C1 (clone→prestito su PropGet) da iscrivere col SUO controfattuale |
| aritmetica | ~12,7 flag-on (S-99; add on −31% post-estensione; da ri-baseline completa) | residuo ~9,9 ns/op nei corpi non-Binary e slot |
| chiamate | 8,6 (S-99 flag-off; da rimisurare in modo on) | **H-D ATTIVA**, in coda dopo H-C |
| stringhe | 6,9 · array 4,9 · regex 3,8 (S-99) | — (regex = parte sana) |

NB: la ri-baseline completa delle sei categorie IN MODO DEFAULT (on) non è
stata fatta in S-100 (misurate solo prop e add): è il primo passo di misura
di S-101.

## LE IPOTESI — stato dopo S-100

- ~~H1, H-A1, H-B1~~ — chiuse (rotazioni precedenti). ~~**H-B2**~~ —
  **CHIUSA COL FLIP**: specializzazione Add sul percorso pila SPEDITA e
  PROMOSSA (emissione flag-off + estensione post-finestre flag-on); la coda
  Sub/Mul/cmp sul percorso pila resta AVVICINABILE per occorrenza col SUO
  controfattuale (KS-BA-100-1); il perimetro compare esige la dichiarazione
  scritta PRIMA (A-ST-101-2/KS-ST-101-2). Il rollout nelle forme registro
  resta chiuso salvo misura ≥ pavimento (KS-ST-101-3).
- **H-C (proprietà, 12,4× nel modo default)**: PRIMA MISURA FATTA (S-100):
  la gamba dominante è il COSTO PER OPCODE (6,2×), non il conteggio (2,0×);
  ~27% del tempo nel ciclo di vita Zval (clone/drop/gc_note su
  PropGet/Pop), simboli per NOME in `wp100-harness/hc-premisura100.out`.
  **H-C1 candidata**: lettura proprietà in prestito/refcount senza clone.
  Si iscrive SOLO con criterio pre-registrato dal SUO controfattuale.
- **H-D (chiamate)**: attiva, in coda dopo H-C; prima mossa = stessa tavola
  (census × costo + profilo co-equale su calls.php).

### ⚖️ Concilio WP-101 — ESEGUITO PER INTERO in S-100 (6/6)

Verbali in `wp101-harness/`. KS del flip TUTTI soddisfatti: KS-MA-101-1
(contratto value-parsed), KS-HO-101-1/2 (dump-diff, mai cronometro),
KS-KL-101-1/KS-PE-101-1 (server bimodale sulla stessa rotazione),
KS-ST-101-1 (sette trappole byte-id nei 2 modi), KS-HE-101-2 (match
esaustivi), KS-HE-101-3 (braccio OFF collaudato alla rotazione),
KS-GR-101-1/KS-LE-101-1 (bande pre-registrate, gate su phpr-off vs on).
BACKLOG per NOME che resta (non slot di sessione se non bloccante):
A-HO-101-1 (sigillo di tipo), A-HO-101-4+A-PE-101-5 (`--build-info`),
A-PE-101-2 (census PHPR_* lazy), A-MA-101-1 (C0' same-tree), A-BA-101-3
(census post-flip CmpJmp*), A-LE-101-2/3 (sonda taglia-unità, N_OPS
const-assert), A-ST-101-2/4 (perimetro compare; mappa divergenze
solo-flag-on), A-HE-101-3 (controllo positivo chiave unit-cache),
A-LE-100-3 (N_OPS≤255: 186/256), A-KL-100-4/5, A-GR-100-3.

## Regole di metodo (invariate)

1. Il giudice è la micro-categoria. 2. WordPress è un collaudo di PARITÀ
(si esegue quando cambia l'emissione). 3. Ogni ipotesi porta il criterio
di caduta scritto PRIMA. 4. L'apparato non entra nell'ordine se non
blocca; timebox mezza sessione.

## Stato gate

- **phpr (pin release)**: **725a2ffad763bbc4** @ HEAD fb861e4 (l'hash
  churna col relink del test-funnel: fa fede HEAD). **DEFAULT = flag-ON**;
  opt-out `PHPR_REG_LOWER=0` (value-parsed; altro valore ⇒ default +
  warning). Corpus Zend **1418 per NOME IDENTICO con `=0` E `=1` SUL PIN**
  (`wp100-harness/corpus-gate/`); diff per-test off↔on ZERO fuori carve-out
  (`wp100-harness/evidence/corpus-diff-verdetto.txt`). Batteria **1735/0**.
  Stash additivo `phpr-s100-flip`.
- **php-server**: **f2ab06369239f389** — ricetta OBBLIGATORIA
  `cargo build --release -p php-server --features axum-server`.
  **collaudato: sì, GRADUATO** (emissione-CLI 2 modi · sentinella estesa 2
  modi · option+restapi per NOME 2 modi, `wp100-harness/parity-out-*`).
  Stash `php-server-s100-flip`. Registro = `PIN_REGISTRY.md`.
- **Launcher bimodali S-100** (`wp100-harness/`): `s100-parity-server.sh
  <off|on>` (aggiornare PIN_SRV_ATTESO a ogni rotazione) ·
  `s100-corpus-gate.sh` (2 modi espliciti) · `s100-corpus-diff.sh`
  (diff per-test, carve-out per NOME, S100_COMPARE_ONLY per ri-giudicare) ·
  `pair100.sh <off|on>` (assert conteggi↔nomi) · `s100-sentinella-estesa.sh`
  · `s100-assignop-oracle.sh` (divergenze attese per NOME).
- GATE72 CLI resta baseline trasversale (corpus 1418 · refl 290 · ORM
  3E/13F · hk 1665).
- ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA (rust-analyzer):
  rimuoverla nel pre-flight. Disco locale stretto (~13G).

## Voci APERTE per NOME (misura/attribuzione dovuta)

- **full peak gamba phpr-OFF +~95 MiB CROSS-ALBERO** (WP-99 1892,56 →
  S-100 1988,6/1998,5 MiB, R=2 intra-sera stabile 0,5%): attribuzione
  dovuta (bisect o census; la gamba ON promossa è a +1,9%). NON è del flip.
- **anomalia oracle media peak** (S-99 ~446 MB vs S-100 394 MB, −11,6%
  tra-sere; intra-sera stabile): varianza della gamba oracle, aperta per
  nome; il full peak oracle balla +14,6% INTRA-sera ⇒ ogni banda futura
  sul full-peak va calibrata sul rumore MISURATO dello strumento.
- divergenze §3.11 (AssignOp undef-lhs senza warning) e §3.12 (typed-ref
  azzerato da Zend dopo AssignOp fallito) — catalogate, non urgenti.

## Che cosa è SOSPESO (non abbandonato)

- **A-ZV2** (liveness+TakeSlot): invariata.
- **Rollout Add nelle forme registro**: chiuso salvo misura ≥ pavimento
  (KS-ST-101-3); la banda [0, 0,5] resta pre-registrata.
- **Roadmap footprint**: ferma (full peak ora 1929,0 MiB nel modo default).

## NON riproporre

Tutti i NON-riproporre WP-83..99 restano. Nuovi da S-100:

- **gate il cui rc è quello di `grep`/`tail` in coda a una pipe** (la
  prima batteria S-100: rc del grep, non di cargo — log integrale + rc
  vero, sempre).
- **`wait` nudo con un server in background** (aspetta anche il server:
  si aspettano i PID espliciti dei client).
- **diff byte-wise su test che stampano entropia** senza carve-out per
  NOME provata INTRA-modo (`random_bytes` nei settype_*_handler3).
- **bande simmetriche strette su metriche col rumore non misurato** (il
  full-peak balla +14,6% intra-sera SULL'ORACLE: prima si misura il
  rumore dello strumento, poi si scrive la banda).
- **premesse ambientali nella batteria** (M5-style): il modo è un INPUT
  del funnel; un test di emissione che legge l'ambiente del processo è
  un falso verde in attesa.
- ereditati e ribaditi: pin effetto-collaterale; bit-identità dal
  cronometro; criteri sotto il pavimento della sonda; 57/43 come tariffa;
  misurare con build concorrente; aggregato WordPress come cronometro.

---
**Riscritto**: rotazione S-100 il 2026-08-05. Apertura/chiusura = skill
`apri-sessione`/`chiudi-sessione`. Harness di sessione: `wp100-harness/`.

## §S-101 — BOZZA (da giudicare dal Concilio WP-102, `wp102-harness/`)

Oggetto: **H-C — il costo per opcode del percorso proprietà** (12,4× di
cui 6,2× è costo/op; ~27% ciclo di vita Zval). Bozza d'ordine:

1. **Ri-baseline sei categorie IN MODO DEFAULT (on)** sui due motori,
   stessa finestra (S-100 ha misurato solo prop e add): i numeri che
   giudicheranno H-C nascono qui.
2. **H-C1 iscritta col SUO controfattuale**: che cosa rimuove il
   prestito/refcount sul PropGet (clone+drop+gc_note del valore letto);
   criterio di caduta pre-registrato PRIMA di ogni riga (soglia ≥
   pavimento sonda), fixture di semantica (hook, __get, ref, readonly,
   visibilità) PRIMA del codice.
3. Se H-C1 si scrive: corpus per NOME nei 2 modi + diff per-test + WP
   collaudo di parità (l'emissione non cambia, cambia il runtime: batteria
   + corpus bastano salvo cambi d'emissione).
4. **Attribuzione voce aperta full-peak OFF +95 MiB** (bisect S-98→S-100 o
   census): mezza sessione max, è d'apparato ma la gamba di rollback deve
   restare sana.
5. (timebox) H-D prima misura: stessa tavola su calls.php.
