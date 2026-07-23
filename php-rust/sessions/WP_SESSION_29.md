# WP_SESSION_29 — archivio storico della sessione WP-29

> Estratto da NEXT_SESSION_WORDPRESS.md alla rotazione WP-40 (convenzione: un file per sessione; il handoff tiene solo l'ultima).
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-29 (2026-07-20 sera, gated `4297fe5`→`f375bc9`, 6 commit)**: punti 1+2
> del piano perf. **(A) Proprietà**: `PropInfo.slot` precalcolato (allineato a
> `PropsLayout`; virtual-hooked = None) + `Props::get_slot/replace_slot` +
> `PropAccess::Slot { key, slot }` + gemelli slot-aware `read/write_property_at`
> (write-through-Ref identico) + de-dup PropOpSet/PropIncDec (era 2 resolve +
> 2 slot_of) + Cow::Borrowed nei FieldScope (via i to_vec per Prop-step) +
> **PropIc**: inline-cache monomorfica per-op-site su PropGet/PropSet/PropIsset
> (`Rc<Cell<(epoch, class_id+1, slot)>>` — il dispatch CLONA l'op ⇒ cella
> condivisa; PartialEq sempre-true per la unit-cache; epoch per-run perché gli
> id classe cambiano tra run sui moduli riusati). Fill SOLO scope-indipendente
> (public hook-free; SET solo plain_set_props — le closure sono ri-bindabili)
> e ⭐ ANCHE dai fast-path WP-25 (senza, la cache resta fredda per sempre sulle
> classi all-public). **(B) Dispatch**: `methods_ci` per classe (ci-hash
> ordinata, binary search in resolve_method_runtime, ⚠️ soglia ≥12 metodi —
> sotto, lo scan early-exit vince; ⭐ PRIMO-vince dentro la stessa classe:
> alias di trait duplicano i nomi, bug61998) + `Module.fn_ci` (via lo scan
> O(n) di invoke_named col prelude) + registry builtin SipHash→FxHash + LcKey
> stack-buffer (via il to_ascii_lowercase allocante da class_index/linked) +
> Hash di PhpStr = zhash cached (zend_string->h). **Esito misurato**: media
> group **−0,4% (rumore)** — è dominato da gd/webp/mysql; **full-suite
> master-CPU 16:43→15:27 = −7,6%** (run20; il carico dispatch-heavy
> beneficia). run20 = run19 per nome; gate22 TUTTO verde.

## Lezioni operative della sessione

- ⭐⭐ **Le inline-cache vanno riempite da OGNI percorso che risolve** — se un
  fast-path esistente intercetta il traffico prima del percorso generale (il
  solo che riempiva), la cache resta fredda per sempre e paghi solo il guard.
- ⭐⭐ **Il media group NON misura il dispatch**: è dominato da gd/webp/mysql
  (le stesse C lib dell'oracle). Le ottimizzazioni VM si vedono sulla
  FULL-suite (−7,6% qui) — scegliere il benchmark in base a cosa si ottimizza.
- ⭐ **hash-then-bsearch perde contro lo scan early-exit sotto ~12 voci**
  (stessa soglia di HASH_SCAN_MIN); e FxHasher `write_u8` per byte = un round
  per byte, SEMPRE lowercase su stack buffer + un `write(slice)`.
- ⭐ **Op payload con stato runtime**: il dispatch CLONA l'op → la cella va
  `Rc`-condivisa; `PartialEq` sempre-true per non rompere la unit-cache;
  epoch per-run perché gli id classe non sono stabili tra run sui moduli
  riusati.
- ⭐ Le closure sono ri-bindabili (`Closure::bind`): MAI cachare per-sito un
  esito di visibilità non-public.
- Il worktree per il binario old: se l'HD interno è pieno, `rm -rf` del
  profilo debug di php-rust-output (2,2GB ricreabili) prima di cambiare
  target dir.
