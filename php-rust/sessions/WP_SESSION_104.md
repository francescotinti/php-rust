# WP_SESSION_104 — S-104: la leva H-C2 messa alla prova e CADUTA con canale refutato; l'A/B peak chiuso per sempre; il free di calls inchiodato

**In una frase**: abbiamo finalmente messo alla prova, con misure ripetute,
la modifica su cui contavamo per velocizzare il motore: non rende (anzi
rallenta un po'), quindi l'abbiamo ritirata — ma ora sappiamo perché e dove
NON cercare (il collo vero è la cache delle istruzioni, non il costo delle
chiamate di pulizia), abbiamo chiuso il confronto di memoria fra le due
versioni del motore e inchiodato la contabilità esatta della memoria nelle
chiamate di funzione.

**SCOREBOARD** (micro R=5 sul PIN DI CHIUSURA 86a50d1c — prima volta che la
riga strumentale è misurata sul pin che chiude, A-KL-105-3):

| giudice | S-103 | S-104 | trend |
|---|---|---|---|
| aritmetica | 12,3 | 12,4 | ↑ +0,1 (banda) |
| proprietà | 11,5 | 11,5 | = |
| chiamate | 7,4 | 7,6 | ↑ +0,2 (banda) |
| stringhe | 6,6 | 6,7 | ↑ +0,1 (banda) |
| array | 4,4 | 4,5 | ↑ +0,1 (banda) |
| regex | 3,7 | 3,6 | ↓ −0,1 (banda) |

WordPress (riferimento WP-102, non rimisurato): full CPU **1,89×** · media
CPU **2,64×** · peak phpr ~1942-1990 MiB. **Leve perf spedite in questa
sessione: 0** — ma la regola di ritmo è SODDISFATTA: la leva H-C2 è stata
TENTATA col suo A/B ESEGUITO DUE VOLTE (KS-GR-105-1 saldato); il verdetto è
CADUTA CON CANALE REFUTATO, che è conoscenza d'oggetto, non apparato.

**Data**: 2026-08-06 (19:1x–21:4x). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: WP-105 §S-104 punti 1-4 (5 = backlog non
aperto). **Commit**: 8dbb16d → (chiusura) su main, tutti pushati.

## Ordine eseguito

| # | Esito |
|---|---|
| **1 · Verdetto A/B peak R=7** | Il verdetto meccanico «VOID (spread 99,16 > tetto 51,96)» cade per la regola PRE-registrata (tetto R=5 vs range R=7 = auto-VOID); i CO-PRIMARI: **sign test 7/7 B>A ⇒ p=0,0078 — DIREZIONE FIRMATA** (s100-fix cresce su s99-sigillo), mediana dei Δ ACCOPPIATI **+22,47 MiB** sotto la banda fase-1 34,64 ⇒ magnitudine NON firmata ⇒ niente bisect. R=7 era l'ULTIMO full-peak: metrica esaurita, attribuzione futura solo via design per-fase (A-LE-105-5, backlog). Regime-shift infra-run annotato (coppie 5-7; +120 della coppia 7 = regime, non leva). `wp104-harness/ab-r7-lettura.out` |
| **2a · Atto zero criterio** | `hc2-criterio-v2.out`: predicato `is_trivial_drop` (5 scalari; WeakHandle/ArgPlace ESPLICITI a false — glue Weak/Rc), `dispose` unico, align==8 + fingerprint della definizione di Zval (sha256 blocco enum 789812e5, 14 varianti in ordine), banda [8,22] declassata a NOMINALE. |
| **2b · Disasm prefisso** | `drop_in_place::<Zval>` **OUTLINED**: 1101 callsite `bl` in run_loop (1077 su una copia), glue = ldrb discriminante + albero cmp (~6 istr ramo scalare). R5-Bak(«se inlined, 2-4 ns») refutata. A/B AMMESSO. `hc2-disasm-verdetto.out` |
| **2c · DENTI-105** | Unit tooth `trivial_drop` VERDE (batteria→1740) · absent_eq_one: **per-CORPO ancorato + controllo positivo `=0` — nato ROSSO** (contava `Binary(Add)` generico; la pila emette `BinaryAdd` dedicato), rosso archiviato, attesa corretta con causa nominata, VERDE · 🔵 **mutation-check 19a/19b: DUE perturbazioni mirate (nota saltata nel dispose; soglia GC 50.000→50.001) NON le fanno fallire ⇒ RC-MA-104 si RIAPRE per 19a/19b** (restano regressioni byte-parity, NON arbitrano il meccanismo) · 🔵 **fx20: il mutation-check (Str→forget) ha scoperto che `memory_get_usage` di phpr è uno STUB costante ⇒ verdetto in-script VACUO**; fixture ridisegnata (1M iter) + **braccio RSS nel gate** (cap 150 MiB: clean 50, mutante 301) — ROSSO ARCHIVIATO, l'arbitro ora morde il difetto capitale. `wp104-harness/denti-rossi/` |
| **2d · LEVA H-C2: A/B ×2 ⇒ CADUTA con canale REFUTATO** | Implementata parity-esatta (6 siti + funnel; 9 degli 11 DropS/iter). Forma 1: **Δ=−10,33 ns/iter** (5/5 B>A, rumore ~2,3). Forma 2 (inline-always + cold split): **Δ=−11,33** (5/5). Meccanismo NOMINATO dal disasm: l'inliner LLVM ha FLIPPATO — `bl` 1101→**0**, run_loop **+8.000 B**: B realizzava un fast-out PIÙ aggressivo del progettato (zero chiamate ovunque) ed era PIÙ LENTO ⇒ **le 1101 chiamate erano quasi gratis; run_loop è ICACHE-BOUND**. La banda [8,22] è refutata al numeratore dalla misura. **Revert verificato AL BYTE** (run_loop 257.632 B esatti = pin S-103; prop 4,87s = baseline). Restano (parity-null): predicato+tooth, align-assert, doc emendata, assert nested-Ref issato. `hc2-ab-verdetto.out` |
| **2e · Gate chiusura PIN-105** | Stash NEL grading (⚠️ churn di relink documentato: 66681884 stashato pre-batteria → la batteria relinka → ri-stash **86a50d1c**, fa fede HEAD). Batteria **1740/0** · fixture **13+5+19a/b+fx20** (pin BILATERALE anche oracle 07b0df8d) · corpus **1417 per NOME ×2 modi, insieme IDENTICO a wp82** · micro sul pin (sopra) ⇒ **parity-null STRUMENTALE misurato**, non dichiarato. Coppia WP: debito RESTA nominato (nessuna leva spedita — la regola «la salda la prima leva vera» non è scattata). |
| **3 · H-D free-hist** | Attese PRE-registrate (b106d3a) → freehist NUOVO in memcensus (census-gated) → **4/4 CONFERMATE**: free/chiamata **1,0000** tutto in (16,32], **32,0000 B** simmetrico all'alloc, realloc ≡ 0, NESSUNA massa a 40 B ⇒ **ret_cell escluso per layout E per misura; indiziato = args-Vec (2×16=32)**. Il canale calls è inchiodato: 1 alloc × 32 B + 1 free × 32 B/chiamata. SiteTag residuo≡0 = prossimo passo (S-105). `hd-free-hist-verdetto.out` |
| **4 · Igiene (in finestra)** | Catalogo §3.12 tre-regimi/§3.13 unit già emendato pre-sessione (0439be5) · N EMESSO dal giudice in run-micro.sh (KS-GR-105-2) · assert nested-Ref issato (A-HO-105-2) · doc is_gc_container (A-HO-105-3) · terzo punto banda su GIORNO distinto: IMPOSSIBILE oggi (stesso giorno), resta aperto per NOME. |

## 🔵 Scoperte

1. **Run_loop è ICACHE-BOUND**: eliminare TUTTE le 1101 chiamate al glue
   (inlining totale) costa ~11 ns/iter invece di guadagnare — le leve
   devono ridurre VOLUME di codice/lavoro (meno op, meno byte), non
   micro-costi di chiamata. Raffina S-100 («costo/op quasi invariante»).
2. **Un A/B su run_loop misura anche il TILT dell'inliner**: il confronto
   bl-count prima/dopo (2 minuti di disasm) ha nominato il meccanismo —
   protocollo obbligatorio per ogni leva futura sul loop.
3. **19a/19b non arbitrano il meccanismo**: due perturbazioni mirate non
   le fanno fallire (la sweep di fine statement compensa la nota saltata;
   la soglia±1 non sposta il loro osservabile) — «un arbitro mai visto
   fallire non arbitra» ha ora due prove sperimentali.
4. **`memory_get_usage` di phpr è uno stub costante (2.000.000)**: ogni
   fixture il cui verdetto vi poggia è VACUA su phpr — scoperto dal
   mutation-check, non dalla review. Il verdetto di leak vive nel gate
   (RSS), finché lo stub non diventa contatore vero (backlog per NOME).
5. **La crescita peak s100-fix>s99-sigillo è REALE** (7/7, p=0,0078) ma
   sotto banda di magnitudine: si attribuirà per-fase, mai più full-peak.
6. **Il free di calls era assunto: ora è misurato** — simmetria esatta
   32 B/32 B, e l'istogramma free esclude ret_cell dalla scena.

## ⭐ Lezioni

- ⭐⭐ **Una caduta con meccanismo nominato vale più di una promozione
  cieca**: l'A/B eseguito (regola di ritmo) ha comprato la mappa vera del
  collo di bottiglia (icache) al prezzo di una sera.
- ⭐⭐ **Il mutation-check si fa sull'arbitro del RISCHIO della leva**: 
  19a/19b (ereditati) non mordevano il difetto capitale; fx20 sì — ma solo
  dopo che il SUO mutante ha rivelato il canale vacuo (stub). Un dente
  nuovo si prova col SUO rosso, non con quello di un dente vicino.
- ⭐⭐ **Il revert si verifica al byte** (taglia di run_loop + timing
  baseline), non a sensazione.
- ⭐ **Lo stash pre-batteria può essere invalidato dal relink della
  batteria stessa**: l'ordine giusto è build→hash→batteria→(re-hash se
  churn)→STASH→fixture→corpus, col churn documentato.
- ⭐ **La finestra d'attesa di una run pesante si riempie di scrittura**
  (criterio, disasm read-only, denti, attese): zero build fino al `.done`
  — l'ultimo tentativo di una metrica non si contamina.

## Stato binari e processi

- **phpr pin chiusura: 86a50d1c01c6f45a** @ HEAD di chiusura (fa fede
  HEAD; churn 66681884→86a50d1c documentato) — flag-ON default; runtime
  parity-null PROVATO (funzionale: batteria/fixture/corpus ×2; strumentale:
  micro sul pin entro ±0,2). Stash ADDITIVO `phpr-s104`.
- **php-server pin: 31aa7c2eef899cce** @ 37312e8 INVARIATO (nessuna leva
  spedita; è indietro rispetto a HEAD di N commit parity-null — si
  rigrada col pin che porterà cifre, A-PE-105-1/3/4 backlog).
- Census build in `phpr-census-target` (freehist incluso). Nessun processo
  residuo; MySQL wp8 su; uploads: restore verificato dal launcher A/B
  (backup conservato in `/Volumes/Extreme Pro/Claude/uploads-backups/`).
- Harness di sessione: `wp104-harness/`.
