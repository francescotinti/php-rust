# Verbale sedia 1 — Hoare (design linguaggio/runtime Rust, safe-only) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: CON EMENDAMENTI

## 1. Handler `Op::BinaryAdd` (run.rs ~944-975) — forma Rust SANA

Il pattern è corretto: `pop` di rhs (owned), `last()` immutabile per la
guardia con estrazione per copia dei due `i64` (il borrow muore prima di
`last_mut()`), scrittura in-place che sovrascrive un `Zval::Long` — drop
banale, nessun Rc, nessun aliasing possibile. L'overflow promuove sul
valore ORIGINALE (`l as f64 + r as f64`), fedele a `binary_fast` e a Zend.
Il MISS rifà `pop lhs` e rientra INTEGRALE in `binary_value_ab` con lo
stesso ordine di pop del braccio generico («pop rhs then lhs»): diags,
overload GMP/Number ed errori identici per costruzione. Gli `expect` sono
invarianti di compilazione, coerenti con lo stile di casa (CmpJmpConst).
Nessuna refutazione qui.

## 2. Sonda M1 e verdetto «preambolo gratis» — verdetto LEGITTIMO, con due debiti

Il ragionamento cammino-critico batte il conteggio di istruzioni: giusto.
MA:

- **A-HO-100-1 «Fedeltà assoluta della sonda dichiarata»**: main.rs
  promette che la forma A «si valida contro i 6,27 ns/op»; la sonda ne fa
  3,94 — un buco di ~2,3 ns/op che vive esattamente nelle parti NON
  replicate (Zval enum vs i64, drop glue, spill dei caldi). Il verdetto
  regge SOLO grazie al tetto statico anti-hiding (0,55 ⇒ P≤6,7%<10%):
  questo va scritto nel .out — la fedeltà è STRUTTURALE (ASM), non
  assoluta.
- **A-HO-100-2 «Nominare la catena ip come cammino critico»**: la KS «no
  ip locale» preserva lo store→load forwarding di `frames[top].ip` (4-5
  cicli, loop-carried): è il candidato più plausibile a LIMITARE il loop,
  ed è ciò che rende gratis il resto del preambolo. Il verdetto refuta
  H-B1-sotto-KS, non ogni ottimizzazione del preambolo: la clausola di
  riapertura («dato nuovo di cammino critico») deve nominare QUESTA catena.

## 3. REFUTAZIONE CAPITALE — punto 3 dell'ordine S-99.0

**«Criterio pre-registrato in ns/occorrenza derivato da D=6,07» è
illegittimo per le forme registro.** D=6,07 misura il plumbing del
percorso PILA: call con due Zval owned da 24B per valore + pop/push +
match del payload. `BinarySS/SC/Dst` quel plumbing in gran parte NON ce
l'hanno già (binary_fast su BORROW dei slot, zero pop/push, zero
marshalling owned): trasportare 6,07 come àncora produce o una soglia
irraggiungibile o un tetto finto. Il criterio va ri-derivato da un
controfattuale statico NUOVO del corpo di BinarySS (che cosa resta da
togliere: il match su BinOp + la call binary_fast), PRIMA del codice —
**A-HO-100-3**.

## 4. Rollout H-B2 nelle forme registro — rischi di forma

- **A-HO-100-4 «Census dei Binary(Add) residui flag-off»**: `emit_binary`
  specializza, ma ogni sito che chiama `emit(Op::Binary(Add))` DIRETTO
  resta generico in silenzio (buco di perf, non di parità). Il controllo
  positivo copre solo due micro: serve un dente dump-based su fixture più
  larghe (zero `Binary(Add)` flag-off nel {main} e nelle funzioni).
- Ogni specializzazione dentro le forme registro scrive su SLOT, non su
  pila: l'in-place attraverso un `Zval::Ref` corromperebbe l'alias.

## Kill-switch

- **KS-HO-100-1**: il hit-path specializzato nelle forme registro DEVE
  conservare la guardia `Undef|Ref` esistente ed essere equivalente a
  `binary_fast` sugli stessi tag; MISS → funnel owned integrale; VIETATA
  la scrittura in-place su slot senza il deref-funnel.
- **KS-HO-100-2**: al primo cambio di emissione flag-on, il claim
  «flag-on bit-identico» DECADE e non è più citabile; corpus flag-ON per
  NOME + census flag-on ri-pinnati NELLA STESSA sessione, pena VOID della
  misura.

**Firmato**: sedia 1 (Hoare), 2026-08-05.
