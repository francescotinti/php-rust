# Consulenza esterna — Lars Bak — sul dispatch di `Vm::run_loop`

**Grado**: parere su dati SCREEN (samply R=1). Non è un verdetto. La decisione resta al team.

## 0. Il tetto, prima di tutto

`run_loop` = 19,16% della CPU userland (7466 campioni su 38965). Le sei zone calde
valgono 22,6% di `run_loop` = **4,33% della CPU userland**. Con userland al 50,6%
del wall, un dispatch *a costo zero* varrebbe **~2,2% di wall**. Nello stesso
profilo: `Zval` clone+drop 8,2%, ciclo di vita `Frame` 6,7% — insieme 14,9%, cioè
**3,4× l'intero budget del dispatch**. Se la sessione ha come scopo la CPU, il
dispatch non è dove stanno i soldi. Se lo scopo è il dispatch, sotto c'è l'ordine.

## 1. Diagnosi: combinazione, ma decomponibile

La finestra `+0x250..+0x2bc` è 108 byte ≈ 27 istruzioni; il disassemblato ne mostra
7 (ldrb/adrp/add/adr/ldrh/add/br). Le altre ~20 sono il **prologo per-tick**:
`frames.len()`, guardia `MAX_CALL_DEPTH`, indicizzazione bound-checked di
`frames[top]` (fatta *tre volte*: `.ip`, `.func`, poi lo store `ip+1`), bound-check
su `ops[ip]`. Quindi il 22,6% è **prologo + dispatch**, e i dati temporali soli non
li separano.

Sul singolo dato che abbiamo: 6,66% su una `ldrh` che legge una tabella di
185×2 = 370 byte (6 linee, praticamente sempre L1-hit, 4 cicli). Quattro cicli di
latenza non producono il 6,66% del tempo di una funzione **a meno che non blocchino
il ritiro** — ed è esattamente ciò che succede quando il `br` indiretto ha
sbagliato predizione: la catena `ldrb→ldrh→br` viene rieseguita sul percorso di
recupero e il costo si attribuisce lì. **Il picco sulla `ldrh` è l'impronta della
misprediction, non della load.** La i-cache è terza: 241,7 KiB > L1i (192 KiB sul
P-core Apple), ma con top-10 = 27,8% e top-50 = 51% il *working set caldo* è molto
più piccolo del testo.

**Misura che decide** (Instruments / `xctrace record --template 'CPU Counters'`,
stesso workload, contatori normalizzati sugli op dispatchati presi dalla feature
`op-census` già esistente). Scrivere PRIMA la regola:

- mispredict indiretti / op dispatchato **> 0,25** → predizione dominante
  (a ~14 cicli sono ~3,5 cicli/op ≈ tutto il 4,33%);
- **< 0,10** con L1I-miss/op **> 0,02** → i-cache dominante;
- entrambi bassi → il 22,6% è **il prologo**, e la leva è la dieta della testa,
  non il dispatch. (È il ramo su cui punterei una parte della scommessa.)

## 2. Opzioni in Rust stabile, con stima e rischio

| # | Leva | Guadagno atteso (CPU user) | Rischio |
|---|---|---|---|
| O1 | **Outlining dei bracci freddi**: i ~140 opcode rari in `fn` `#[inline(never)]`; restano inline i ~40 caldi | 0,3–1,5% | BASSO |
| O2 | **Dieta della testa**: guardia `MAX_CALL_DEPTH` fuori dal tick (solo dove `frames` cresce), un solo bound-check via `ops.get(ip)`, una sola indicizzazione di `frames[top]` | 0,4–1,0% | BASSO-MEDIO (tocca solo la testa, non i 185 bracci) |
| O3 | **Superistruzioni**: fusione dei 6–10 bigrammi più frequenti misurati con `op-census` | 1,0–2,5% — ma solo ~0,5% è dispatch: il resto è push/pop + clone/drop risparmiati, cioè aggredisce l'8,2% di `Zval` | MEDIO: **aggiunge corpi caldi**, il modo esatto in cui è fallito WP-39..44. Ammissibile solo DOPO O1 e con tetto: un corpo outlineato per ogni corpo aggiunto |
| O4 | Tabella di puntatori a funzione | **regressione prevista 5–15%** (prologo/epilogo per handler, stato non più residente in registri, indiretta comunque impredicibile) | — non farla, se non *come implementazione* del percorso freddo di O1 |
| O5 | Dispatch replicato / tail-duplicato | non esprimibile in Rust stabile (no computed goto, no tail call garantite). L'unico avvicinamento onesto è **rimpicciolire la testa finché LLVM la duplica**, verificabile contando i `br` nel disassemblato (oggi: 1). Se atterra: ≤0,8%, con testo raddoppiato | ALTO, guadagno BASSO → ultimo |
| O6 | Fuori asse ma vicino: **`Op` da ~48 B a ≤24 B** (boxare le 5 varianti grasse, internare i nomi `Rc<[u8]>`→`u32`) | poco CPU (il prefetcher copre lo stream sequenziale), ma è **footprint per worker** | BASSO — serve la roadmap footprint |

**Se avessi una sola sessione: O1.** È l'unica leva che è simultaneamente (a) il
controllo decisivo sull'ipotesi i-cache, (b) a rischio quasi nullo perché
meccanica e parity-preserving, (c) l'abilitatore di O3, e (d) l'unica coerente con
la vostra stessa lezione WP-39..44 («il costo è il numero di corpi handler caldi»):
è la prima leva che quel numero lo *abbassa*.

## 3. Controllo positivo che pretendo

1. **Il contatore del meccanismo prima dell'orologio**: byte del simbolo `run_loop`
   (`nm -S`), predizione scritta prima — es. 247452 → **< 90000**. Se la taglia non
   si muove, la leva non ha agito: qualunque Δ tempo viene da altro.
2. **Numero di `br` indiretti** nel disassemblato di `run_loop` (per O5: 1 → N).
   `size_of::<Op>()` stampato per O6.
3. **Op dispatchati** da `op-census`: **invariante** per O1/O2 (prova che non è
   cambiato il lavoro); per O3 deve **calare della percentuale predetta prima**.
4. **Ri-profilo con la stessa ricetta samply**: la quota delle sei zone deve
   scendere dal 22,6% al valore predetto. Se la CPU totale cala ma la quota no,
   la leva **non è provata**.
5. **Coppia A/A**: ricompilazione della sorgente immutata, R≥3. Ogni Δ sotto lo
   spread inter-build è **SCREEN, non verdetto** (la lezione di LEVER-2 a WP-93).
6. **Bite test**: una variante che outlinea bracci **mai eseguiti** su questo
   workload. Predizione: taglia che cambia tutta, CPU che non cambia. Se la CPU si
   muove lo stesso, il canale è fortuna di layout, non i-cache.
7. Parità: gate corpus **per nome**, coppia oracle↔phpr la stessa sera.

Un'ultima cosa, da chi ha già sbagliato così: non riscrivete il dispatcher perché è
in cima al profilo. È in cima perché *ogni* opcode ci passa, non perché costi.
