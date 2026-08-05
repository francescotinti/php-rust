# WP_SESSION_100 — S-100: la promozione di flag-on a default, eseguita per intero con l'ordine del Concilio WP-101, più la prima misura di H-C

**In una frase**: da oggi il nostro motore PHP usa di serie la modalità
veloce che finora era solo sperimentale — prima di accendere l'interruttore
abbiamo verificato che con la modalità nuova migliaia di test, il sito
WordPress completo e il server web producono esattamente gli stessi
risultati di prima, e abbiamo misurato dove il motore perde ancora tempo
sull'accesso alle proprietà degli oggetti.

**Data**: 2026-08-05 (19:5x–23:1x). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: Concilio WP-101 per intero, 6/6 punti (il
condizionale H-C compreso). **Commit**: b9a5750 → 5861f81 → 838edbf →
aecaf32 → 6c05775 → 01ec1e4 → 9d0d001 → fb861e4 (FLIP) → c176347, tutti
su main, pushati.

## Ordine eseguito

| # | Esito |
|---|---|
| **1 · Contratto di modo** | `PHPR_REG_LOWER` VALUE-PARSED a lista chiusa (assente⇒`DEFAULT_ON`, `=1`⇒on, `=0`⇒off, altro⇒default+warning forte) — prima `=0` ACCENDEVA (`is_some()`). Costante `DEFAULT_ON` nominata; `REG_LOWER_DEFAULT_ON` esportato (i denti derivano dal CONTRATTO); anti-putenv 2→5 bracci su valori ESPLICITI (nuovi: `=0`+putenv(set); set→putenv(`=0`); fuori-grammatica⇒default+warning asserito); funnel col braccio off `=0`; tre launcher BIMODALI (`s100-parity-server.sh <off|on>`, `s100-corpus-gate.sh`, `pair100.sh <off|on>`) con assert conteggi↔nomi |
| **2 · Strumenti sanati** | A-HE-100-4: dump copre gli HOOK (RC-1 chiuso; ordinati per nome — prop_info è HashMap); prop_init dichiarato FUORI (RC-2); `all_funcs` enumera dal Module per DESTRUCTURING ESAUSTIVO. A-HE-100-2/A-MA-101-2: `visit_addrs`+`bin_op_of` esaustivi (variante nuova di Op ⇒ NON COMPILA in 2 punti). Funnel: probe oltre `{main}`, dump-hash dei 2 bracci + emissione DIVERSA asserita, stdout DERIVATO pinnato + exit (A-KL-100-1), zero `Binary(Add)` flag-on. A-HE-100-3: differenziale permanente BinaryAdd≡Binary(Add) (overflow±, coercizioni, union, throw, warning, `+=`). A-KL-101-4: `s100-corpus-diff.sh` = definizione operativa del diff per-test |
| **3 · Trappole + H-B2** | Sette trappole A-ST-99-3 (a–g) in-tree, BYTE-IDENTICHE nei due modi (KS-ST-101-1 ✓); gamba oracle: 5/7 anche ==oracle, 2 divergenze PRE-ESISTENTI trovate e CATALOGATE (§3.11 undef-lhs AssignOp senza warning; §3.12 Zend AZZERA il typed-ref dopo AssignOp fallito — phpr conserva). **H-B2-sotto-flip DECISO CON MISURA**: micro isolante (i due `{main}` differiscono di UN op, dump-verificato) → L=12,9 ns/occ ≥ pavimento ⇒ ESTENSIONE: ogni `Binary(Add)` sopravvissuto alle finestre diventa `BinaryAdd` in `lower_func`; contro-misura L'∈[−1,0]; giudice add on 3,55→3,27 (−31% vs off) |
| **4 · Collaudo pre-flip** | Corpus: insiemi 1418 IDENTICI a wp82 nei 2 modi + **diff per-test ZERO** fuori carve-out nominata (3 `settype_*_nan_with_error_handler3`: `random_bytes` nell'output, nondeterminismo provato INTRA-modo: 8bdbc200≠70cfe091). Server: parità bimodale fails=0 (sentinella ESTESA A-PE-101-3: 16 interleaved su 3 endpoint + 4 concorrenti, workers=2; option 413 + restapi 3508 per NOME) — **prima esecuzione assoluta del server flag-on**. Coppia WP con BANDE PRE-REGISTRATE: G1/G2/G3/G4_media PASS; G4_full fuori banda in direzione FAVOREVOLE (on 1929,0 MiB < off 1998,5), decomposto con R=2 (off stabile: movimento CROSS-albero da attribuire, voce aperta); G5: anomalia oracle +28,7% = varianza TRA-sere (intra-sera −0,2%), resta aperta per nome; oracle full peak balla +14,6% intra-sera |
| **5 · FLIP + rotazione** | `DEFAULT_ON=true` @ fb861e4. Ri-derivazione batteria SENZA premesse ambientali: il modo è un INPUT del funnel (`ProgramCtx.reg_lower`, `compile_program_with_mode`); `lowered()` (mirror a mano, fonte RC-2) ELIMINATO — il braccio on dei test è il funnel VERO; nuovo dente `production_entry_follows_process_mode` (vale in ogni modo). Batteria post-flip **1735/0 rc=0 vero**. Rotazione COLLAUDATA sul pin: phpr 725a2ffad763bbc4 (corpus 1418×2 per NOME sul pin, smoke default), php-server f2ab06369239f389 (parità bimodale fails=0 ×2 modi sul pin); braccio OFF collaudato (KS-HE-101-3); PIN_REGISTRY graduato; stash `phpr-s100-flip`/`php-server-s100-flip` |
| **6 · H-C prima misura** | Census (opcache 0x20000 vs dump): oracle 9 op/iter, phpr-on 18 (2,0×), off 22. Cronometri R=5: prop on 12,4× / off 14,0× (S-99 13,8 in banda). **Decomposizione: 12,4 = conteggio 2,0 × costo/op 6,2** (9,67 vs 1,56 ns/op; il costo/op phpr ~9-10 ns è quasi invariante di categoria). Profilo CO-EQUALE: phpr ~27% nel CICLO DI VITA Zval (drop 12,6% + clone 7,6% + gc_note 5,3% + deref_clone 2,1%) + write_property_at 4,3%; oracle = handler TAILCALL SPECIALIZZATI per operando, zero simboli alloc. **Candidata H-C1 nominata** (prestito/refcount al posto del clone su PropGet), da iscrivere col SUO controfattuale. Nessuna riga scritta |

## 🔵 Scoperte

1. **`=0` accendeva il pass** (presence-based): il contratto value-parsed
   l'ha chiuso e il braccio anti-putenv `=0` ora lo PROVA sul binario.
2. **BinaryDst fonde anche gli add a operandi stack con store**: la classe
   che il flip ritirava da H-B2 è SOLO "risultato che resta in pila" — e il
   GIUDICE stesso ne aveva 1/iterazione (add.php, catena a 3).
3. **Due divergenze oracle pre-esistenti** dalle trappole (b)/(e), identiche
   nei due modi: catalogate §3.11/§3.12.
4. **3 fail del corpus sono nondeterministici per costruzione**
   (`random_bytes` nell'output): il diff per-test esige la carve-out per
   NOME, provata intra-modo.
5. **Il full-peak è rumoroso anche sull'oracle** (+14,6% intra-sera): una
   banda ±2% sul full-peak siede SOTTO il rumore dello strumento.
6. **Il costo per opcode phpr è ~9-10 ns quasi ovunque** (prop come arith
   residuo): non è un problema di quanti opcode, ma di quanto costa
   ciascuno — e ~27% è ciclo di vita Zval (clone/drop/gc_note).

## ⭐ Lezioni

- ⭐⭐ **Un gate il cui rc è quello di `grep` in coda a una pipe non è un
  gate**: la prima batteria era `cargo test | tail | grep` — rc del grep.
  Ri-lanciata con log integrale e rc vero. (Classe forgia-silenziosa.)
- ⭐⭐ **`wait` nudo dopo un server in background aspetta anche il server**:
  la sentinella estesa si è impiccata al burst concorrente; si aspettano i
  PID dei soli curl.
- ⭐⭐ **Il diff byte-wise non giudica un test che stampa entropia**: la
  carve-out va provata INTRA-modo (stesso modo, due run diversi) prima di
  nominarla — altrimenti è un tappeto.
- ⭐⭐ **Il modo come INPUT del funnel** (`ProgramCtx.reg_lower`) elimina
  in un colpo le premesse ambientali della batteria E il mirror a mano
  `lowered()`: la stessa mossa ha chiuso M5 e RC-2.
- ⭐ La banda pre-registrata simmetrica su una metrica rumorosa morde in
  direzione favorevole: fermarsi e decomporre (R=2) è costato 17 minuti e
  ha trovato sia la stabilità della gamba sia la voce aperta vera
  (movimento cross-albero).

## Stato binari e processi

- phpr: **725a2ffad763bbc4** @ HEAD fb861e4 (hash churna col relink: fa
  fede HEAD) — DEFAULT flag-ON, opt-out `PHPR_REG_LOWER=0`. Stash
  `phpr-s100-flip`. Batteria 1735/0.
- php-server: **f2ab06369239f389** (ricetta axum-server) — parità bimodale
  fails=0 sul pin. Stash `php-server-s100-flip`. Registro graduato in
  `PIN_REGISTRY.md`.
- Nessun processo orfano; uploads ripristinata dalla guardia a ogni run
  (count=3, backup tar conservati); MySQL wp8 su.
- Harness di sessione: `wp100-harness/` (launcher bimodali, sentinella
  estesa, corpus-diff, trappole, `.out` di pre-misura/bande/H-C, evidence).

## Post-scriptum di chiusura (Concilio WP-102 in sessione)

Il concilio ha morso TRE volte a macchina e i morsi sono stati SALDATI
prima della chiusura: (1) `emit_binary` leggeva `enabled()` invece di
`ctx.reg_lower` (Hoare+Hejlsberg, indipendenti) — fix + batteria 1735/0;
(2) l'evidenza del diff per-test andava giudicata SUL pin (Klabnik) —
ri-eseguita: ZERO fuori carve-out + 1418×2; (3) la parità server non
provava il modo effettivo (Pedersen) — mode-probe nella sentinella, OK
nei due modi. **Pin FINALI: phpr f29883eb432806ce · php-server
62b978c51c62e108 @ HEAD b618e3a** (stash `*-s100-fix`), registro
graduato. H-C1 RIFORMULATA A STADI dal team hc-canale (il churn è il
RICEVITORE, non il valore); ordine S-101 definitivo in NEXT_SESSION.
