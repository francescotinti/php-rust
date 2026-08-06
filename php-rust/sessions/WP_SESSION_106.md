# WP_SESSION_106 — S-106: il grado pieno del server (dopo la caduta del pin fuori-ricetta), il concilio consumato, l'igiene completa e la leva arith RMW-su-slot SPEDITA

**In una frase**: abbiamo collaudato a fondo il server web del nostro
motore PHP (scoprendo che il binario messo da parte la sera prima era
stato costruito male e rifacendolo con la ricetta giusta — ora tutte le
3.921 verifiche di compatibilità passano in entrambe le modalità), e poi
abbiamo reso più veloci le operazioni aritmetiche fondendo tre passi
interni del motore in uno solo: i calcoli su variabili semplici ora
costano il 7% in meno e l'intero pacchetto è stato promosso dopo tutti i
collaudi.

**SCOREBOARD** (micro R=5 SUL PIN DI CHIUSURA eb555106, N emessi):

| giudice | S-105 | S-106 | trend |
|---|---|---|---|
| **aritmetica** | **12,4** | **11,6** | **↓ −0,8 (leva H-A1)** |
| **proprietà** | **11,5** | **10,6** | **↓ −0,9 (H-A1, 2° beneficiario NOMINATO: `$s += $o->x` nel loop, dump BinarySTDst)** |
| chiamate | 6,3 | 6,3 | = |
| stringhe | 6,7 | 6,6 | = (−0,1) |
| array | 4,5 | 4,2 | = (−0,3, in banda) |
| regex | 3,6 | 3,5 | = (−0,1) |

WordPress (riferimento WP-102, confermato dalla coppia S-105 letta):
full CPU **1,894×** (citabile: retro-verifica D-6 SALDA con eccezione
nominata wp_is_stream) · media CPU 2,64× (riferimento; la coppia S-105
ha dato 2,697/2,734 fuori banda = voce aperta) · peak ~1942-1990 MiB.
**Leve perf spedite in questa sessione: 1** (H-A1 arith RMW-su-slot).
Contatore sessioni-senza-Δ-rapporti: **0** (due categorie mosse).

**Data**: 2026-08-07 (00:3x–02:0x). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: ratificato WP-107 punti 1, 2, 3, 4, 5
INTERI; punti 6, 7 → rotazione per NOME. **Commit**: c7b6eb2 → 6019890
su main, tutti pushati.

## Ordine eseguito

| # | Esito |
|---|---|
| **1 · Retro-verifica coppia S-105 (D-6)** | SALDO CONFERMATO con UNA eccezione NOMINATA: rc=0×2, media failnames VUOTI ×2, identity phpr d4d0fa52 ✓, oracle inferito da versione+build (hash corrente 07b0df8d, stringa -v byte-identica); full failnames = SOLO `wp_is_stream` (pre-esistente, identico nei 2 modi, presente ANCHE nella baseline WP-102: leggere «VUOTI» alla lettera voiderebbe il riferimento stesso). **1,894 CITABILE**. `wp106-harness/retro-verifica-coppia-s105.out` |
| **2 · Grado PIENO server** | 🔴→🏆 **Il pin de67cb64 è DECADUTO al tentativo di grado** (primo atto, proroga A-PE-107-1 SPESA e onorata): la sentinella è morta con UNA riga nel srv.log — `--axum requires 'axum-server' feature` ⇒ lo stash S-105 era una build workspace FUORI RICETTA (taglia 17,2M contro 18,5-18,6 dei pin veri; stessa specie del 49a91e4d; mai registrato a PIN_REGISTRY in S-105). Cura S-103 in-sessione: build con ricetta a HEAD (55s; **pin phpr INVARIATO verificato**), smoke --axum, stash `php-server-s106` = **dde2a64dcc2eb32b**, chain v2 RILANCIATO con le 4 sanature del team processo (watchdog anche su oracle; hash oracle FAIL-CLOSED; conteggi 413/3508 pinnati; rc separa VOID/DIVERSO). **GRADO PIENO VERDE: rc=0 voids=0 ×2 modi** — sentinella+mode-probe ✓, option 413 IDENTICO ✓×2, restapi 3508 IDENTICO ✓×2. Primo grado pieno dal S-100. `wp106-harness/grado-verdetto-{de67cb64,dde2a64d}.out`, PIN_REGISTRY aggiornato (riga DECADUTO + riga GRADATO PIENO) |
| **3 · Concilio WP-107 fase 2 + SYNTHESIS** | CONSUMATO: 3 team tematici (metodo-misura, VM-semantica, processo-pin-server) + **SYNTHESIS RATIFICATA**: 21 direttive vincolanti S-106-D-1..21 + 6 raccomandazioni, ordine S-106 ratificato coi fatti nuovi recepiti. `wp107-harness/COUNCIL_WP107_SYNTHESIS.md` (vincolante), indice aggiornato |
| **4 · Igiene pin** | `cargo check --workspace --all-targets` a HEAD: **exit 0** (A-KL-107-1 SALDATO). **Sigillo Copy RISCRITTO sui costruttori di variante** (D-11: `const fn _seal<T: Copy>(_: fn(T)->Zval)`) e PROVATO dal controesempio (mutante `_seal(Zval::Str)` → E0277 esatto, revert pulito). **Commento drop-order DECLASSATO** (D-9: «pure-read» era claim non provato; dente a backlog). **Backstop ArgPlace RUMOROSO** (D-12: debug_assert + contatore census). **fx21 → GATE fail-closed** (D-14: doppio golden riga 5 in repo — phpr=divergenza §3.15 pinnata ROSSA alla cura, oracle=arbitro; mutation-check 66/1/0). Unificazione A-HO-107-4 fuori timebox (vige D-8) |
| **5 · LEVA H-A1 arith RMW-su-slot** | Scelta D-18 VERBALIZZATA (contatori L1I non eseguiti ⇒ leva=arith). **Istruttoria dal dump**: corpo arith 11 op/iter col tris `LoadSlot;Swap;BinaryDst` del compound assign; il doc del pass dichiarava «LoadSlot never folded» per FREDDEZZA — refutata dal dump (lettura silenziosa ⇒ fold diagnostic-safe). **Forma**: finestra nuova nel pass → op monomorfa `BinarySTDst{op,l,dst}` (sola forma Dst; STESSI read_slot/binary_value_ab/reg_store_slot del braccio BinaryDst: zero biforcazione). Criterio PRE-registrato e committato PRIMA (60448b3). Co-primario strutturale: **corpo 11→9 per NOME**, OFF invariato, valore=oracle. Admission DICHIARATA (run_loop **−128 B**, bl 5416→5418, ±1-2 su sei target = rimescolo inliner nominato). Smoke 2/2 → **R=5 ABAB: Δ=+7,0 ns/iter, 5/5 concorde** (5,34→4,99) ⇒ **PROMOSSA**. **PIN-107 SALDO su eb555106a3c7b718**: batteria **1740/0 rc=0 stessa run** (terza run: due lettere-gate emendate DICHIARATAMENTE — reg_lower_funnel attesa {main} sostituita con BinaryDst ri-collocato nella probe; census op_index riposizionato col blocco registri, N_OPS=187) · 5 gate fixture ×5 rc=0 · **corpus 1417×2 per NOME IDENTICO a wp82** · micro sul pin (scoreboard) · taglia run_loop 257.828 B a verbale. `wp106-harness/ha1-{criterio,ab-verdetto}.out`, `pin107-gate-verdetto.out` |
| **6-7 · Denti + fedeltà** | NON entrati (ore 02:00, la leva ha consumato la finestra) → rotazione per NOME: terza mutazione OBS-8 · mutante leak-parziale fx20 · contatore hit/miss (D-5) · dente VM direct-bind (D-10) · dente drop-order (D-9) · **§3.15 cura D-13 (attesa 1417→1415)** > get_gc > §3.13 > §3.12-i > §3.14 |

## 🔵 Scoperte

1. **L'hash pinna l'identità, non la RICETTA** (⭐⭐ a veto in SYNTHESIS):
   lo stash S-105 del server aveva congelato un binario MAI collaudato e
   costruito senza la feature — il difetto era stashato insieme al
   binario. Un pin server esiste solo con ricetta a verbale + collaudo
   del binario stashato (D-19).
2. **La finestra RMW-su-slot è TRASVERSALE alle categorie**: promossa
   sul giudice arith, ha mosso anche prop (11,5→10,6) perché prop.php
   accumula in una locale (`$s += $o->x`) — beneficiario nominato e
   provato dal dump, non anomalia. Il giudice per categoria misura il
   MECCANISMO, e lo stesso meccanismo vive in più categorie.
3. **La leva ha ACCORCIATO run_loop** (−128 B con un braccio in più):
   l'inliner ripaga la rimozione di due dispatch; il text-budget R-2
   resta lontano (+196 B cumulativi vs S-104).
4. **«LoadSlot è freddo» era un'assunzione di design mai rimisurata**:
   il compound assign la refutava da sempre nel corpo caldo del giudice
   peggiore. Le regole di design dei pass invecchiano come i verdetti.

## ⭐ Lezioni

- ⭐⭐ **Il primo atto di un grado è eseguirlo, anche quando cade**: il
  tentativo su de67cb64 è durato 6 minuti e ha comprato la diagnosi
  esatta (una riga di srv.log); rimandare il grado avrebbe fatto
  decadere il pin SENZA la diagnosi.
- ⭐⭐ **Una lettera-gate che morde una leva promossa si emenda
  DICHIARANDO, mai zittendo**: reg_lower_funnel e census op_index sono
  stati riscritti col controllo positivo RI-COLLOCATO dove il sito
  sopravvive (probe) e l'invariante documentale preservato (BinaryAdd
  chiude la tabella).
- ⭐ **«rc di gate mai da pipe» morde anche chi la conosce**: la prima
  batteria è partita con `cargo test | tail` — beccata PRIMA della
  lettura e rifatta; la regola va applicata alla FORMA del comando, non
  ricordata dopo.
- ⭐ **Il daemonizer non crea la directory del log**: due lanci morti in
  silenzio (`grado-chain/`, `corpus-gate/`) — mkdir PRIMA del daemonize,
  e il «lanciato» si verifica col pgrep, mai col solo exit 0 del padre.

## Stato binari e processi

- **phpr pin chiusura: eb555106a3c7b718** @ HEAD (churn hash₁ fcda5e4f →
  pin via relink batteria + reindex census inerte, documentato; fa fede
  HEAD) — DEFAULT flag-ON; contiene la leva H-A1 + igiene D-9/D-11/D-12.
  Stash ADDITIVO `phpr-s106`. Batteria 1740/0. Corpus 1417×2 per NOME.
  Micro sul pin (scoreboard). run_loop 257.828 B.
- **php-server: dde2a64dcc2eb32b** @ HEAD c7b6eb2, stash
  `php-server-s106` — **GRADATO PIENO** (rc=0 voids=0 ×2, rito D-16):
  le cifre server sono ATTRIBUIBILI. de67cb64 DECADUTO a registro.
- Nessuna run detached in volo a fine sessione. MySQL wp8 su; uploads
  ripristinata e verificata (count=3) dopo ogni gamba.
- Harness di sessione: `wp106-harness/`; concilio WP-107 consumato in
  `wp107-harness/`; concilio WP-108 in `wp108-harness/`.
