# s123-istruttoria-phpstr.md — PhpStr single-alloc: PERIMETRO misurato (NEXT p.3; SOLO istruttoria, nessuna patch senza criterio col metro sanato)

## Oggi (zstr.rs:26-35)
`ZStr = Rc<PhpStr{hash: Cell<u64>, bytes: Vec<u8>}>` (PhpStr 32 B, RcBox 48 B) ⇒ **2 malloc + 2 free + 2 hop per stringa** (RcBox e buffer del Vec). Bersaglio: UNA allocazione stile `zend_string` — header {refcount, hash, len, cap} + bytes in coda, refcount custom. NON è l'SSO bocciato WP-38 (+1,5-2,5%: branch inline su ogni lettura): qui la lettura resta un deref, sparisce un'allocazione e l'header condivide la cache line coi primi byte.

## Perimetro MISURATO (Explore S-123, verbale integrale nel task output)
- **Funnel integro**: unica `Rc::new(PhpStr…)` del workspace = zstr.rs:54 dentro `PhpStr::new`; tutti i costruttori (concat2/from_i64/from_str/empty, increment_string ops.rs:1136, concat_n_join run.rs:276) delegano lì. Una riga da riscrivere.
- **API Rc su stringhe: 7 siti totali** — 1 load-bearing: `Rc::get_mut` run.rs:433 (fused `.=`, append-in-place WP-55); 6 banali (ptr_eq: ops.rs:632 fast-path compare, zval.rs:376 test; as_ptr per dedup census: memcensus.rs:358/377, vm/mod.rs:1261/1844) → rimpiazzabili da `ptr_eq()/as_ptr()` sul tipo custom.
- **Zero**: make_mut (PhpStr è !Clone, DELIBERATO), strong_count, Weak, try_unwrap, into_raw/from_raw su stringhe. php-cli/php-server/phpt-runner: 0 siti.
- **15 siti** con `Rc<PhpStr>` scritto per esteso (zval.rs:22 `Str(Rc<PhpStr>)` è il centrale; elenco nel verbale) + ~809 riferimenti PhpStr di superficie che l'alias assorbe.

## 3 rischi NON testuali (il compilatore non li segnala)
1. **RcEqIdent**: `Key::Str == Key::Str` oggi eredita dalla libstd `ptr_eq || byte-eq` — il tipo custom lo PERDE in silenzio ⇒ PartialEq manuale con fast-path identità, o regressione sui lookup di chiave.
2. **Hash cached**: `Key::khash` (array.rs:171) riusa la `Cell<u64>` dentro PhpStr — l'header custom DEVE tenere l'hash con interior mutability via `&self`.
3. **!Clone strutturale**: oggi nessuno può fare COW implicito su una stringa; l'invariante «chiave condivisa ⇒ mai mutabile» (append solo via get_mut) va preservata dal refcount custom.

## Vincoli di meccanica
- Append `.=`: il Vec growable (WP-55, canale O(n²) 244× ucciso) diventa `cap` nell'header + regrow con realloc/copy (mirror esatto di `zend_string_extend`); disciplina exact-size nel funnel INVARIATA, slack solo sul path append.
- memcensus: Drop/alloc CH_STR (zstr.rs:44-47,150-158) e i 4 as_ptr census si riscrivono col nuovo tipo (stesso patch census).

## Attesa (da confermare con classifica-v2 fusa PRIMA del criterio)
str.php: 2 ZStr/iter (concat2+substr) = 4 GA-alloc → 2 (**−2 alloc/iter**); re: 3 gruppi Caps = −3; arr: chiavi `"k$i"` −1/insert sul path from_bytes. Prezzo alloc→latenza ~14 ns (L-RE1): attesa str ≈ −28 ns/iter lordi, DA GIUDICARE col costo sostitutivo (3 cadute recenti insegnano: il modello del costo SOSTITUTIVO va scritto nel criterio).
Gate promozione: batteria + corpus 1415×2 + fixture + ricetta ORM/http-kernel (tocca php-types) + micro col METRO S-123.
