# WP_SESSION_55 — PhpStr growable + append-in-place `.=` (canale O(n²) chiuso: probe 250×, full −2,56%) + checkpoint Fase 3 (pilota = hashed-array)

> ⚡ **WP-55 (2026-07-25, `a8bc9aa`+`50e2fb2`)** — Sessione di REDESIGN a
> leva singola. **Ob.1**: `PhpStr { hash: Cell<u64>, bytes: Vec<u8> }`
> (24→32B; RcBox 40→48B = +8B/stringa, quotato PRIMA in size-class:
> ~713k vive al picco media = +5,7MB = +0,36%) + op fuso
> `ConcatAssignSlot` (mirror ASSIGN_OP Zend) per `$s .= rhs` su locale
> nudo: slot-Str-UNICO + rhs Str ⇒ `PhpStr::append` via `Rc::get_mut`
> (extend ammortizzato, hash azzerato); ogni altra forma ⇒ fallback
> byte-identico alla vecchia sequenza (binary_value_ab condiviso,
> typed-ref, gc_note sul displaced). **Mechanism-check: probe 5k×1KB =
> old 499ms → new 2ms (250×), ORACLE 2ms**. **Giudici**: full run44
> **716,9s vs old (phpr-wp54) 735,7s stessa-sera = −2,56% (−18,8s,
> bordo basso della banda quotata 15-90s) → 716,9/339 = 2,11× nuovo
> minimo**; ab55 media CPU **FLAT** (mediane +0,16%; r2-6 −0,04% —
> il media non è append-bound, atteso); footprint peak **+1,27%**
> (1546→1566MB: 5,7MB di +8B + ~14MB slack di crescita dei buffer
> append-grown — DENTRO guardia ≤+2%, tenuto e verbalizzato).
> **Ob.2 checkpoint Fase 3** (census fresco, binario census dal tree
> wp54 in target separato): al picco master media **arr vive ~421MB
> (713k, 39% proxy) vs str ~45MB (712k, 4%)** ⇒ **pilota heap-a-handle
> = HASHED-ARRAY** (10× di margine); il design stringhe NON si fonde.
> Parità: corpus **1421 IDENTICO** per nome · refl **290 IDENTICO** ·
> cargo **1639/0** · ORM 3484 **3E/13F IDENTICO** (16 nomi) · hk 1665
> **0E/0F** · fail-set full **BYTE-ID a run33 (88 nomi) su run44 E
> run44-old** · probe55-sem/dtor BYTE-ID new vs old. Stash:
> `phpr-wp55` (sha256 ecc04817…); old = `phpr-wp54` (e4341e77…).

## Ob.1 — design e pin rispettati

- **Pin (a) size-class PRIMA del layout**: RcBox 40→48B (bin mimalloc a
  passo 8 sotto i 64B) = +8B/stringa VIVA; census fresco: master media
  ~713k vive al picco ⇒ +5,7MB su 1564MB = +0,36% (guardia ≤2%, margine
  5×). Actual A/B: +1,27% — il delta oltre la quota è lo slack
  `capacity>len` delle stringhe costruite per append e ancora vive
  (crescita ammortizzata ≤2×): fisiologico del design, non un canale.
- **Pin (b) hash-cache**: `PhpStr::append` è l'UNICO sito di mutazione
  in-place del codebase (audit `Rc::get_mut`: solo Func/Class reloc) e
  azzera `hash` a ogni chiamata; raggiungibile solo via `Rc::get_mut`.
- **Pin (c) exact-size**: `new` fa `shrink_to_fit` sul Vec in ingresso =
  stessa disciplina della vecchia conversione `Into<Box<[u8]>>` (che
  riallocava se capacity>len): le stringhe non-append restano exact.
- **Pin (d) sentinelle PRIMA del layout**: probe55-sem (alias-COW,
  letterali const-pool, ref, expression-value, coercizioni, loop+zhash,
  interpolazione) e probe55-dtor (timing `__destruct` del displaced,
  `__toString` che throwa ⇒ slot intatto) pinnati su old → new BYTE-ID.
- **Pin (e) sito value**: solo l'emissione `ExprKind::AssignOp(Concat)`
  su locale nudo cambia (op fuso); tutti gli altri path (AssignOpPlace,
  PropOpSet, StaticPropOpSet, globals) INTATTI. Arm run_loop = una call
  `#[inline(never)]` (WP-33); Sweep-elision: op non in whitelist ⇒
  sweep conservato per costruzione (safepoint dtor intatto).
- COW per costruzione: alias/const-pool ⇒ rc>1 ⇒ `get_mut` fallisce ⇒
  path copia (il primo `.=` su un letterale COPIA sempre, poi la copia
  unica cresce in place).

## Divergenze PRE-esistenti trovate (non introdotte, preservate)

1. `$undef .= "x"` NON emette "Undefined variable" (l'emissione usa
   `LoadSlot` silente, non `LoadVar`) — anche il fuso replica.
2. Operandi OGGETTO in `.=` non passano da `__toString` (il path
   AssignOp non emette `Stringify`; `to_zstr` warna + placeholder, e un
   `__toString` che throwa non throwa affatto in phpr) — famiglia da
   catalogare in PHPR_DIVERGENCES se non si chiude.

## Ob.2 — checkpoint d'ingresso Fase 3 (ri-attribuzione BYTE)

Census media (phpr-memgc55 = tree wp54 + mem-census, target separato;
run 75,6s, peak esterno 1612MB, proxy 1076MB master):

| canale | live fine-run (≈picco, crescita monotona) | n | quota proxy |
|---|---|---|---|
| arr | **~421MB** (stima death-avg ~619B/arr) | 713k | **39%** |
| str | ~45MB (payload medio ~26B + 40B overhead) | 712k | 4% |
| cum churn | str 2,4GB/29,1M · arr 2,7GB/4,8M | | |

**Verdetto: il pilota heap-a-handle di WP-56 è l'HASHED-ARRAY** (tabella
singola Zend-style, arena-compatibile, come demansionato da Fase 1) —
10× il canale stringhe. Il canale str ha appena avuto il suo redesign
(growable) e l'overhead fisso (48B/str ≈ 34MB) non giustifica un'arena
prima degli array. Fusione single-alloc resta chiusa (incompatibile con
growable + 0,3% CPU, WP-54).

## ⭐ Lezioni

- ⭐⭐ **Il giudice full va letto SOLO in coppia stessa-sera**: run44
  716,9s contro il run43 del pomeriggio (717,3s, binario old!) leggeva
  "flat"; la coppia interleaved serale (old 735,7) rivela −2,56%. La
  deriva d'ambiente pomeriggio→sera (~2,6%) è della stessa taglia delle
  leve che misuriamo.
- ⭐⭐ **La banda di quota si realizza al bordo che il SITO copre**: la
  leva copre solo i `.=` su locale nudo e ha reso −18,8s (banda 15-90s):
  il resto della banda (str-copy residuo + madvise) vive sui siti
  prop/element (`$this->buf .= …`, `$a[k] .= …`) — da QUOTARE per sito
  (op-census su AssignOpPath/PropOpSet-Concat) prima di estendere il
  fuso (direttiva ns/evento).
- ⭐ Il probe di complessità (5k append) è il mechanism-check perfetto
  per le leve algoritmiche: 499ms→2ms=oracle non è interpretabile come
  rumore, e costa 3 secondi di misura.
- ⭐ Un layout growable paga un footprint DOPPIO: la struct (+8B,
  quotabile ex-ante) e lo slack ammortizzato dei vivi cresciuti
  (quotabile solo ex-post): predire col solo primo termine sotto-stima
  (qui 5,7 vs 19,7MB reali — guardia comunque rispettata).

## Prossimo (WP-56)

1. **Fase 3, pilota HASHED-ARRAY** (mandato dal checkpoint): redesign a
   tabella singola Zend-style (entries + indice senza Key duplicata),
   arena-compatibile, end-to-end su UN tipo. Parità enorme: ordine di
   iterazione, tombstone, chiavi numeriche-stringa, dtor-order.
2. **Residuo append non-locale**: quotare per sito (op-census:
   AssignOpPath/PropOpSet con op=Concat) PRIMA di estendere il fuso.
3. Residuo full vs WP-40 ≈ 14s: gc-walk fasi mock (classify del full),
   drop/rc churn — servono finestre sample/census sulle fasi mock.
4. Ritorno cap reflect 16384→8192: resta decisione utente (evictions 1).
