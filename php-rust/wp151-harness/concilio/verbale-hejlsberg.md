# VERBALE — sedia HEJLSBERG (S-151, fase 1 indipendente)
Lente: ingegneria dei compilatori — build, LTO/CGU, inlining, layout del binario.

**VERDETTO: CONCORDO CON EMENDAMENTI** (impianto A1..A4 ratificabile; la
giustificazione-build di Gemini per A2 è refutata; perimetro e gate da emendare).

## Fatti verificati sul repo (non da memoria)
- `Cargo.toml` righe 42–44: `[profile.release] lto = "fat"` + `codegen-units = 1`
  (mandato A' S-117: i 16 CGU default erano essi stessi lotteria di layout ~10 ns/iter).
- `crates/php-runtime/src/vm/run.rs`: `#[inline(always)]` ×6 e `#[inline(never)]`
  ×3 DELIBERATI, con commento di misura (righe 50–54: il never tiene la funzione
  FUORI dai sentieri, |Δ| misurato). `vm/mod.rs`: 31 attributi inline (4 always).
- mod.rs / host.rs / run.rs vivono tutti nello STESSO crate `php-runtime`;
  il piano Gemini §4.3 resta same-crate (sottomoduli, non crate nuovi).

## Refutazioni dalla lente
1. **«Tempi di build dimezzati» (Gemini Fase 1): NON FONDATA sotto il profilo
   vigente.** Con CGU=1 l'intero crate è UNA unità di codegen qualunque sia il
   numero di file; il fat LTO serializza comunque l'ottimizzazione whole-program
   a valle. Spaccare file nello stesso crate non parallelizza né il front-end
   (single-thread per crate) né il codegen (CGU=1). Anche §4.1-1 di Gemini
   («rustc non parallelizza dentro lo stesso mod.rs») è vera solo con CGU>1.
   Il beneficio residuo di A2 è manutenibilità/contesto-AI: reale, ma è un'ALTRA
   giustificazione e va messa a verbale come tale.
2. **«Rischio parità zero (puro refactoring)»: falso al livello del BINARIO.**
   Lo spostamento di item cambia l'ordine dei simboli → layout .text →
   I-cache/BTB. Prova interna: FR1 = +3,00 ns/iter da delta STRUTTURALE
   +3180 B/+26 bl senza toccare il loop giudicato (S-150). Ogni tranche A2
   cambia il binario PER COSTRUZIONE: prima/dopo non confrontabili a
   risoluzione ns se non via banda-layout e mediana.
3. **Inlining**: sotto same-crate+CGU=1+fat-LTO il code-motion tra moduli non
   cambia di per sé le decisioni di inlining (LLVM vede tutto). Il rischio vero
   sono gli ATTRIBUTI misurati (`inline(never)`/`always` di run.rs): un refactor
   che li perde o «ripulisce» cambia il binario in modo firmato.

## Q1 — sequenza
Census PRIMA del refactor è giusto: i CONTEGGI per canale sono proprietà
semantiche del programma, sopravvivono al code-motion. MA: (a) il census deve
chiavare per CANALE/nome logico, mai per file:riga, o A2 invalida la mappa dei
SITI; (b) emendo il perimetro di A2: interleaving — refactor SOLO dei moduli che
A3 toccherà (exec/, memory/, dispatch hostcall). Spaccare tutte le 25,7k righe
per poi riscriverne il cuore è doppio churn di layout e allunga l'anomalia
senza-leve oltre il necessario.

## Q2 — gate per tranche (lente principale)
- Batteria + corpus 1412×2 per NOME + fixture bilaterali: OGNI tranche.
- **Censimento simboli `nm`** (set nomi + size, simboli caldi NOMINATI) pre/post
  tranche: Δsize dichiarato per nome. È il sostituto meccanico legittimo della
  byte-identità vietata (strumento diverso: inventario simboli, non identità).
- **disasm run_loop size+bl-count (stile S-150)** per ogni tranche che tocca
  exec/run/mod; per tranche host-only: disasm del dispatcher hostcall.
- **micro R=5 SOLO-REGRESSIONE**, soglia max(4, rumore, banda-layout); attesa
  pre-registrata: Δ entro banda-layout. Oltre banda = STOP (KS-H1).
- **Inventario attributi inline** contato pre/post (invariato o delta dichiarato).
- **Coppia WP**: dovuta a OGNI pin nuovo (regola utente, non negoziabile) ⇒
  impacchettare 2–3 tranche per sessione in UN pin: 4–6 coppie totali, non 12.
- Partizione a rischio minimo: prima i moduli FREDDI (dom, reflect, constants —
  foglie senza attributi caldi), per ultime le tranche exec/run; ogni tranche =
  pure code-motion, ZERO rinomini di simboli/firme (il rinomino cambia il
  mangling e acceca il confronto nm).

## Q3 — A3 dalla lente
- `vm.objects[id]` NON è «puramente matematico» (Gemini §2.2): bounds-check +
  generation-check = 1–2 branch per accesso. Il confronto onesto è branch vs
  flag RefCell; il guadagno vero sta in alloc/dealloc, clone/drop del contatore
  e località, NON nell'azzeramento dei check.
- **Decidibilità**: il census DEVE produrre tre conteggi per workload —
  borrow/borrow_mut · inc/dec refcount · alloc Props — ciascuno × prezzo per-op
  misurato da giudice micro ⇒ TETTO componibile per canale. Senza questi tre
  numeri A3 non è decidibile, è solo plausibile.
- **Tensione TETTO movimenti 1,27 s ≈ 3,4% (pronunciamento dovuto)**: il tetto
  REFUTA la componente «azzerare il traffico di movimento» della promessa
  Gemini sul gap ORM. Se A3 ripaga, ripaga dai canali borrow/refcount/alloc —
  la scommessa va pre-registrata su QUEI canali con pavimento dichiarato
  (lezione BT1).
- `&mut Vm` esclusivo vs RE-ENTRANCY (call_method_sync, __destruct che
  rientrano nella VM): il RefCell oggi risolve un aliasing REALE; senza, ogni
  rientro obbliga a ri-derivare l'accesso allo store (l'arena può riallocare)
  — costo nuovo non prezzato da Gemini. Il vincolo semantico del MANDATO
  (mai ObjectId Copy senza refcount) è giusto e lo sottoscrivo.

## Q4 — dente A4
Sede: BATTERIA (cargo test), come il precedente rczval_funnel — morde a ogni
promozione; la CI (backlog ~3 giorni) non morde in tempo; il pre-commit hook è
bypassabile. Perimetro: RATCHET — file esistenti: cap = righe attuali +50 (una
soglia fissa 2000 sui monoliti non morderebbe MAI, sono già a 25k); file nuovi:
≤2000; il cap scende a valle di ogni tranche promossa. Anti-auto-morso: contare
righe con `wc -l`, niente pattern testuali componibili (lezione bea7ea3).

## Q5 — cosa manca (Gregg)
1. **La misura del costo di build PRIMA di A2**: `cargo build --release
   --timings` (clean + touch a un file) pre-registrato prima della tranche 1 e
   ripetuto a fine A2. Previsione dalla lente: Δ≈0 sotto CGU=1+fat-LTO; senza
   questa misura il beneficio-build resta un claim non falsificabile.
2. **Decisione esplicita SAME-CRATE**: vietare in A2 la creazione di crate
   nuovi (churn Cargo/LTO-graph senza beneficio sotto fat LTO); sottomoduli e
   basta.
3. Cosa sappiamo oggi che ieri no: FR1 ha DIMOSTRATO che un delta strutturale
   +3180 B costa +3,00 ns/iter reali su un giudice — quindi A2 non è gratis a
   risoluzione micro e i suoi gate devono aspettarsi derive entro banda, non zero.

## Kill-switch pre-registrabili
- **KS-H1**: micro R=5 post-tranche con |Δ| > banda-layout su qualunque
  categoria → STOP tranche, revert al byte, tranche ri-partizionata.
- **KS-H2**: nm-census post-tranche con simbolo caldo nominato Δsize >2%
  (run_loop in testa) o attributo inline perso → tranche RESPINTA.
- **KS-H3**: se dal census A1 (borrow+refcount+alloc-props)×prezzo < 15% del
  gap ORM netto → A3 si FERMA prima della chirurgia; rotta all'utente.
- **KS-H4**: build-timing post-A2 entro ±10% del pre → la giustificazione
  «build» di Gemini è refutata AGLI ATTI (non blocca, vincola il verbale finale).
