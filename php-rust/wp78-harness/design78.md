# design78.md — WP-78 protocollo di misura (A-BG3/A-DL3, Council WP-78)

**Stato**: protocollo scritto in S-78.0 (sanatoria), PRIMA di qualunque run di
misura. Nessuna cifra è valida se prodotta fuori da questo protocollo
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
- Carico **chiuso-sequenziale** (depth coda mpsc ≤1; KH78-2/KB-78-2: depth >1
  in un campione ⇒ run VOID).
- **Warm-up escluso**: prime 10 richieste fuori dalle metriche.
- **R≥3** run; spread accettato **±2%** (KB-78-4/KG-78.B: oltre ⇒ si indaga la
  variance, MAI si media).
- Ogni A/B in **coppia build-adiacente stessa-sera** (WP-57/65), stessa
  versione mimalloc nel Cargo.lock.

## Metriche (A-DL3, KL-78-3)

- **Peak fisico**: `/usr/bin/time -l` (richiede uscita PULITA: SIGTERM al
  server — implementato in S-78.0.7; kill -9 = zero stats = run nulla).
- **Residente**: `vmmap <pid>` Physical footprint, con `MIMALLOC_PURGE_DELAY=0`.
- **Stats allocatore**: `MIMALLOC_SHOW_STATS=1` a uscita pulita.
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

## Denominatore A-BB1 (A-BG4)

Confronto onesto: **stesso binario** (union build), stessa fixture, stesso N,
`--cli-server` vs `--axum`. Il "≤+1.5% misurato" del verbale .3 è una
PREDIZIONE da verificare, non una baseline.

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

## Ordine della fase misura

1. Gate: feature-matrix + run-gate + gate concorrente (KH78-1) sulla build da
   misurare.
2. Tier-0 baseline (Axum runtime senza pool — riferimento WP-77.1).
3. Census per-fase `--workers 1` (A-BB1 si giudica qui).
4. A-DL1 frammentazione (vmmap + mimalloc stats, transiente vs residente).
5. Probe amplificazione (A-BG5) + linearità in W (A-DL2).
6. R-G4 (CPU attribution) SOLO se commissionato, con contatori ns/evento
   owner-level su coppia strumentata/non-strumentata (A-BG6; mai solo `sample`).
