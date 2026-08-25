# S-159 p.3 — ISTRUTTORIA di forma leva (sola lettura, nessun edit/build; finestra coppia in volo)

## Reperto (a)+(b): i due candidati CONVERGONO sullo stesso collo
`ho_array_map` (host.rs:1943-2000), caso dominante 1-array+callback:
- per CHIAMATA: args-Vec host (array_map NON è nei 6 nomi HD2) + `entries:
  Vec<(Key,Zval)>` raccolta upfront (1 alloc + clone di TUTTE le chiavi) +
  `arrays: Vec` (1 alloc);
- per ELEMENTO: `cb.clone()` (Rc bump, non alloc) + **`vec![v]` → 1 alloc+free**
  → `call_callable` (calls.rs:640) → `invoke_value` (calls.rs:509) → dispatch
  PIENO rifatto A OGNI elemento; per callable STRINGA anche `bytes.to_vec()`
  (1-3 alloc/elemento: nome, e per "C::m" PhpStr+metodo);
- destino della Vec: `push_closure_frame` (mod.rs:11394) → `bind_params(frame,
  args: Vec)` (calls.rs:346) MUOVE i valori negli slot del frame POOLED e la
  Vec muore ⇒ la `vec![v]` è burocrazia pura, stessa classe di L-AL1/HD2/RF2.

## Siti `vec![…]` per-iterazione su call_callable (per NOME)
array_map host.rs:1975 (k=1) · array_filter 2081 (k=2) · array_reduce 2129
(k=2) · array_walk mod.rs:8695 (k=2) · usort 11018 (k=2) · **autoload loader
mod.rs:12093 (k=1) = residuo L-AL1 (candidato b)** · preg_replace_callback
1732/3298 (k=1) · session/xslt/pdo (freddi).

## Forma candidata (stile FD1/AP1: fast path + pieno come fallback per costruzione)
1. array_map 1-array+closure-anonima: HOIST della risoluzione callable FUORI
   dal loop (oggi dispatch invoke_value per elemento) + chiamata per-elemento
   SENZA Vec (variante arità-1 che binda l'argomento diretto nel frame pooled);
   ogni altro caso (string/array-callable, multi-array, ArgPlace) resta sul
   cammino pieno INVARIATO. Rischio nominato: duplicazione della logica di
   binding ⇒ la variante deve RIUSARE bind_params/push_closure_frame a meno
   del solo intake degli argomenti (come field_write_walk riusato in FD1).
2. Estensione naturale (tranche): loader autoload 12093 k=1 (residuo b) sulla
   STESSA variante arità-1.
3. Semantica da vigilare: risoluzione hoistata = stessa per tutti gli elementi
   (in PHP funzioni/metodi non ridefinibili nel loop; autoload scatta alla
   prima risoluzione) · ArgPlace/by-ref mask (SEND_VAR_EX) esclusi dal fast ·
   eccezioni nel callback (drive nested run_loop invariato).

## Vincoli di sequenza (REGOLE §3-§4 + revisione S-158)
- La SONDA p.2 tara il coefficiente ns/alloc del cammino Vec (lezione S-158:
  6,9 sottostima alloc+free+doppio match): l'UB falsificabile della leva si
  scrive col coefficiente TARATO, quindi **sonda PRIMA del criterio leva**.
- Giudice NUOVO m-arrmap da scrivere (N ~10M elementi, parità con oracle);
  criterio con BANDA SMOKE VINCOLANTE (az.rev. S-158 #4: fuori banda ⇒
  arbitrato dedicato PRIMA del R=5, o stop).
- Quota suite: census "array_map 7,68M" (lista aperture) da riconfermare come
  DENOMINATORE dal verbale census (che cosa conta: chiamate callback o alloc)
  prima di ogni attesa pre-registrata.
- Scelta finale del bersaglio (a niente-vec per-elemento vs b solo-loader):
  dai numeri della sonda; l'opzione 1 copre entrambe le fette con un solo
  meccanismo.
