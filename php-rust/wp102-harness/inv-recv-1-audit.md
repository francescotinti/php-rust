# INV-RECV-1 — audit degli osservatori assoluti di `strong_count` (S-102 punto 2a)

**Mandato**: A-MA-103-1 (Concilio WP-103, refutazione capitale RC-MA-103-1).
La motivazione registrata del criterio H-C1b («il valore assoluto non è
osservabile da PHP») è FALSA: il motore stesso osserva conteggi assoluti.
Ciò che rende sound la leva è l'invariante qui nominata e auditata.

**Grade**: AUDIT (lettura del codice sul pin phpr 48a5d4384970d8ff @ HEAD;
nessuna misura). Verificato in-sessione S-102 sui sorgenti:
`vm/run.rs` (PropGet 3413-3454, PropSet 3577+, prop_get_fallback 376+),
`vm/oop.rs`, `vm/mod.rs`.

## L'invariante (nominata)

**INV-RECV-1**: in ogni punto di un braccio Prop-op (`Op::PropGet`,
`Op::PropSet`) raggiungibile da PHP sincrono (hook get/set, `__get`/`__set`,
lazy-init 8.4, `__destruct` del vecchio valore, `include` dentro uno di
questi) o da un osservatore di conteggio del VM, vive **≥1 handle OWNED del
ricevitore**: prima `target` (l'handle MOSSO dal pop — H-C1b), poi — nei
sentieri magic/hook — il clone `target.clone()` incapsulato nel frame
(`push_hook`/`push_magic_prop`) PRIMA che il fallback ritorni; nel sentiero
lazy, `target` è mosso DENTRO `lazy_prop_access`, che lo possiede per tutta
l'inizializzazione sincrona.

**Conseguenza contabile**: rispetto a pre-H-C1b il MOVE abbassa il conteggio
mid-arm di ESATTAMENTE 1 (rimuove il *secondo* handle del braccio — la
coppia deref_clone — mai l'ultimo). Per ogni osservatore assoluto sotto, il
conteggio mid-arm resta separato dalla soglia da ≥1 in ENTRAMBI gli schemi
⇒ **verdetto dell'osservatore INVARIANTE**. Il verdetto potrebbe flippare
solo se il braccio tenesse 0 handle — escluso da INV-RECV-1.

## Tavola di audit (osservatori × raggiungibilità mid-arm)

Base = holders fuori-braccio (per un oggetto tracked in uno slot: `created`
+ slot = 2). Mid-arm: pre-move = base+2 (pop-handle + deref_clone),
post-move = base+1 (solo handle mosso).

### Classe A — nel vivo del runtime, raggiungibili mid-arm

| # | sito | soglia | cosa decide | canale mid-arm | verdetto |
|---|---|---|---|---|---|
| 1 | oop.rs ~1084 `last_user_ref` | `== 2 + extra` | release dello slot globale al `Sweep` | hook/`__get`/`__destruct` → `include` → Sweep top-level dell'unità inclusa | **INVARIANTE**: mid-arm ≥ 3+extra (post) / ≥ 4+extra (pre), mai `== 2+extra`; fuori-braccio i due schemi coincidono |
| 2 | oop.rs ~1091/1100 | `== 1` (Closure) | release closure da slot | come #1; il ricevitore mosso può essere una Closure | **INVARIANTE**: slot + handle braccio ⇒ ≥ 2 in entrambi |
| 3 | oop.rs ~1096 | `== 1` (cella Ref) | release cella condivisa | come #1 | **FUORI PERIMETRO**: osserva la CELLA, non l'handle; il braccio `Ref` conserva `deref_clone` (il wrapper non viaggia mai) |
| 4 | mod.rs ~4126 | `== 2` | candidato del buffer collectable-ADESSO (created + clone buffer) | Sweep raggiunto come #1 con ricevitore candidato | **INVARIANTE**: ≥ 3 (post) / ≥ 4 (pre) ⇒ demoted in entrambi; la demozione ri-nota, non perde |
| 5 | mod.rs ~4160 | `− extra == 1` | id massimo releasable in sweep | come #4 | **INVARIANTE**: ≥ 2 in entrambi |
| 6 | mod.rs ~4458 `exclusive` | `== 2 && !lazy && !buffered && created-match` | cascade release del figlio | gc_release mid-arm via `__destruct` sincrono del vecchio valore (PropSet old) | **INVARIANTE**: l'handle del braccio tiene il conteggio ≥ 3 ⇒ `exclusive=false` in entrambi (resta buffered: sentiero `&$foo->bar`) |
| 7 | mod.rs ~4554 | `== 1` | gc_cascade su oggetto destructed non-created | pass distruttori raggiunto mid-arm | **INVARIANTE**: ≥ 2 in entrambi |
| 8 | mod.rs ~4914-4918 collector | `− 2 > in_edges` (Obj), `− 1 >` (Arr/Ref/Clo) | external-holder ⇒ nodo VIVO | `gc_collect_cycles()` chiamato da PHP dentro hook/`__get`/`__destruct` | **INVARIANTE**: l'handle del braccio è un holder ESTERNO (+1 post, +2 pre): il ricevitore in volo è LIVE in entrambi; mai collezione di un ricevitore in volo |
| 9 | mod.rs ~5252 | `== 1` | dtor-dead (solo `created`) a fine pass distruttori | come #7 | **INVARIANTE**: ≥ 2 in entrambi |
| 10 | run.rs ~742 | `== 1` (vecchia cella static) | gc_note del contenuto della cella rimpiazzata | StaticSlot, non Prop-op | **FUORI PERIMETRO**: osserva una cella Ref static, mai l'handle mosso |
| 11 | mod.rs ~14155-14157 typed_refs | `> 0` (weak vivo) e `obj > 1` | raccolta typed-ref per coercion | assegnamenti mid-arm (typed_ref_assign) | **INVARIANTE**: soglie `>`; l'handle del braccio può solo alzare il conteggio; post-move mid-arm ≥ 2 > 1 come pre |
| 12 | mod.rs ~3946/3974/3989 gc_note descend | `== 1` (Ref/Array/Closure) | descend del valore NOTATO | `gc_note(old)` mid-arm in PropSet | **FUORI PERIMETRO**: osserva il VALORE notato (old), non il ricevitore; i conteggi di old non sono toccati dal move |

### Classe B — fuori dal run_loop (mai mid-arm)

Teardown/census/reporting: oop.rs ~1242 (weaks a teardown), mod.rs ~1147,
~1884/1894, ~2007, ~2429, ~17650/17658, ~18042/18050, gc_census.rs.
Girano a request_end/shutdown o in build census: nessun braccio in volo
per costruzione. **NON ARBITRANO** la leva.

## Esito dell'audit

**INV-RECV-1 REGGE su tutti gli osservatori nominati** (12 di classe A — più
dei ≥6 di Matsakis — e la classe B fuori perimetro). Nessun osservatore
cambia verdetto tra pre e post H-C1b. I punti d'appoggio verificati sul
codice: (i) `target` owned dal pop alla fine del braccio; (ii)
`push_hook(func, target.clone(), …)` e `push_magic_prop(…, target.clone(), …)`
incapsulano il clone nel frame PRIMA del return del fallback; (iii)
`lazy_prop_access(target, …)` possiede l'handle per l'intera init sincrona;
(iv) il braccio `Ref` è escluso dal move per costruzione.

## Vincoli attivi che questo audit NON scioglie

- **KS-MA-103-2**: estensione del move a Silent/Dynamic/OpSet/IncDec = reject
  senza (a) guardia `matches!(obj, Zval::Ref(_))`, (b) occorrenze contate
  per-forma, (c) RI-audit di questa tavola sul nuovo perimetro.
- **KS-MA-103-3**: qualunque cambio che ribalti il verdetto di un sito
  exact-count (`==1/==2/−2`) nelle fixture = reject senza appello.
- La motivazione del criterio H-C1b è CORRETTA in
  `wp101-harness/hc1b-criterio.out` §ADDENDUM (l'originale resta, non si
  riscrive la storia).
