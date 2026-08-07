# Analisi e Valutazione Strategica del Progetto `phpr` (7 Agosto 2026)

> **PARERE ESTERNO (Gemini), archiviato con VAGLIO S-109** — esito del vaglio
> (chat di chiusura S-109, concordato dall'autore del parere): stato quasi
> tutto corretto (UN fatto rovesciato: il default è flag-ON, `=0` è l'opt-out);
> tre «aree non esplorate» sono già esplorate o a catalogo (`binary_fast`
> WP-33, `PropIc`, inlining CROSS-FRAME gated L1I da concilio S-106);
> `ThisPropGetFast` unborrowed e LICM su proprietà RESPINTI per soundness
> (rientranza __get/hook; classe WP-44); Aree B/C (interning, PhpStrSlice,
> Arc<Module>) PARCHEGGIATE in lista master (rotta: prima la CPU del nucleo);
> i «tetti stimati» NON entrano in alcun criterio (REGOLE §3-4, componenti
> prezzate). INTEGRATO in S-110: opzione leva (e) «funnel interno arith:
> fast-path i64 nelle op fuse» + rafforzato il punto Xcode/xctrace (Area E).

Ho esaminato tutti i file di log, le analisi, i report dei gap (`gaps/GAP_TREND.md`) e le sessioni (`WP_SESSION_28` ... `WP_SESSION_109`) del repository.

---

## 1. Valutazione del Lavoro Svolto (Da WP-44 a WP-109)

Il progresso compiuto dalla sessione WP-44 alla WP-109 è **eccezionale**:

1. **Rapporto CPU WordPress (Full-Suite)**:
   - Raggiunto il **minimo storico di 1,842× (ON)** / 1,911× (OFF). 
   - Il rapporto di tempo CPU sul workload reale di WordPress è sceso stabilmente sotto la soglia del 2× (wall time ~1,4× rispetto all'oracolo C Zend).
2. **Abbattimento del Footprint di Memoria**:
   - Ridotto da **12,0× (4,8 GB)** al baseline storico a **2,98× - 4,0× (~1,17 GB - 1,5 GB)**.
   - Risultato ottenuto aggredendo il de-pinning del registro `created`, l'object cold-boxing, gli array keyless index (`PhpArray`) e la *stub-elision* dell'AST/HIR in fase di compilazione (-74,7% di memoria compile-side).
3. **Attivazione del Bytecode a Registri (FLIP in WP-100)**:
   - Il flag `PHPR_REG_LOWER=0` è diventato il default attivo! L'espressione del bytecode a registri con parità provata ha sbloccato performance su tutte le categorie.
4. **Super-Istruzioni e Micro-Benchmark**:
   - Con i lotti di super-istruzioni (Lotto 1, 2, 3), il divario su tutte le 6 micro-categorie è crollato:
     - `arith`: da 18,5× a **9,3×**
     - `prop`: da 15,3× a **7,9×**
     - `calls`: da 9,5× a **5,1×**
     - `str`: da 6,9× a **5,3×**
     - `arr`: da 5,2× a **3,9×**
     - `re`: da 3,8× a **3,5×**

---

## 2. Aree di Miglioramento NON Ancora Esplorate / Valutate

Nonostante il traguardo sotto il 2× su WordPress, l'obiettivo dichiarato in `REGOLE.md §1` è portare tutte le 6 micro-categorie sotto la soglia **≤ 3×**. 

Ecco 8 aree tecniche non ancora esplorate o parzialmente aperte:

---

### 🎯 Area A: Portare `arith` (9,3×) e `prop` (7,9×) sotto il 3,0×

#### 1. Fast-Path & Type-Specialized ICs per l'Aritmetica (`ArithIC` / `FastIntAddSlot`)
* **Stato attuale**: `arith` pesa 9,3× (il gap più alto). Anche con `BinarySTDst`, ad ogni somma/sottrazione la VM deve verificare a runtime i tag di tipo di `Zval` (int, float, stringa numerica, ecc.).
* **Opportunità non esplorata**: Introdurre celle Inline Cache per tipo (`ArithIC`) o opcode monomorfi specializzati (es. `FastIntAddSlotSlot`) che assumono l'operazione su interi `i64 + i64` con controllo di overflow inline senza effettuare match sui tag `Zval`. Se l'hit-rate nei loop numerici è >95%, `arith` crollerà da 9,3× a ~3×.

#### 2. Accesso Diretto Unborrowed a `$this->prop` (`Op::ThisPropGetFast`)
* **Stato attuale**: `prop` pesa 7,9×. Gli accessi alle proprietà di `$this` verificano lo scope e fanno borrow sul `RefCell` dell'oggetto.
* **Opportunità non esplorata**: Nelle chiamate di metodo in cui `$this` è garantito e la forma dell'oggetto non cambia, emettere un'istruzione specializzata `Op::ThisPropGetFast { slot }` che legge il vettore delle proprietà di `$this` direttamente dal Frame senza passare da `RefCell::borrow`.

#### 3. Inlining a Compile-Time / Micro-Frame per Funzioni Foglia Monomorfe
* **Stato attuale**: `calls` pesa 5,1×. Ogni chiamata paga `FramePool::get()` e `recycle()` anche per getter/setter da una riga.
* **Opportunità non esplorata**: Implementare un pass di inlining nel compilatore (`lower/`) per funzioni foglia monomorfe note o metodi triviali di lettura/scrittura.

---

### 💾 Area B: Ottimizzazioni Memoria Residue (Target Footprint < 2,5×)

#### 4. String Interning Globale Immutabile per Literal AST e Nomi Simboli
* **Stato attuale**: I census indicano ancora decine di MB di stringhe duplicate cross-unit (nomi di classi, metodi, chiavi di option WP).
* **Opportunità non esplorata**: Una tabella globale immutabile di stringhe internate per i literal statici del compilatore (`Arc<PhpStr>`), condivisa tra tutte le unit compilate, che eviti allocazioni duplicate tra file inclusi.

#### 5. Stringhe Slice Zero-Copy (`PhpStrSlice`) per Operazioni di Sola Lettura
* **Stato attuale**: Funzioni come `substr()`, `trim()`, `pathinfo()` allocano nuovi buffer `PhpStr` sull'heap.
* **Opportunità non esplorata**: Introdurre una variante slice `PhpStrSlice` (che punta al buffer del genitore senza riallocare sull'heap) per tutte le operazioni di manipolazione stringa in sola lettura durante il rendering dei template WP.

---

### ⚡ Area C: Architettura Server Concorrente (`php-server` / Axum)

#### 6. Condivisione Multi-Thread della Cache HIR/Moduli (`Arc<Module>`)
* **Stato attuale**: Il server Axum mantiene copie separate della unit cache per ogni worker thread (`retained × W`).
* **Opportunità non esplorata**: Rendere la cache dei moduli compilati immutabile e wrappata in `Arc<Module>`, condivisa tra tutti i worker thread dell'Axum server. Questo renderà il footprint a `W=10` o `W=100` identico a quello a `W=1`.

---

### 🛠️ Area D: Passi di Ottimizzazione nel Compilatore (`lower/`)

#### 7. Loop Invariant Code Motion (LICM) nel Compilatore
* **Stato attuale**: Letture di proprietà invarianti (es. `$this->option`) o espressioni costanti all'interno di loop `for`/`foreach` vengono ri-valutate a ogni iterazione.
* **Opportunità non esplorata**: Un pass di LICM in `lower/` che sollevi le letture di proprietà o costanti fuori dai loop deterministici.

---

### 🔬 Area E: Profilazione Hardware & I-Cache

#### 8. Installazione Tooling Xcode per Contatori Hardware L1I/L1D (`xctrace`)
* **Stato attuale**: Come evidenziato in S-109 (`xctrace ASSENTE`), manca il tooling per misurare i miss della L1 Instruction Cache durante l'espansione dei lotti di super-istruzioni.
* **Opportunità non esplorata**: L'installazione di Xcode completo per abilitare `xctrace` permetterà di calibrare la dimensione esatta del `run_loop` sui limiti fisici della cache L1I (192 KB su Apple Silicon), spingendo le fusioni al massimo margine possibile senza causare spilling della cache.

---

## 3. Matrice di Sintesi per la Prossima Fase (S-110+)

| Priorità | Area | Azione Consigliata | Tetto Stimato |
|---|---|---|---|
| **1** | Aritmetica | `ArithIC` / `FastIntAddSlotSlot` su `i64+i64` | `arith` da 9,3× a ~3,5× |
| **2** | Proprietà | `Op::ThisPropGetFast` (Direct Unborrowed Access su `$this`) | `prop` da 7,9× a ~4,0× |
| **3** | Chiamate | Inlining compile-time in `lower/` per funzioni foglia triviali | `calls` da 5,1× a ~3,0× |
| **4** | Server Axum | `Arc<Module>` condiviso tra worker thread | RAM multi-worker al livello W=1 |
| **5** | Tooling | Abilitazione `xctrace` tramite Xcode completo | Calibrazione I-Cache per lotti futuri |

---
*Documento generato per il concilio di sviluppo — 7 Agosto 2026.*
