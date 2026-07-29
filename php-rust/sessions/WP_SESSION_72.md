# WP_SESSION_72 — 🔴 LEVA MASS-TEARDOWN SPEDITA E VALIDATA: il leak per-richiesta è CHIUSO (used_n 20,000→0,000) + debiti d'apertura del concilio chiusi + S-72.6 trait-FQN (wrong-result smascherato dal gate) + ⚡ AXUM SBLOCCATO

> ⚡ **WP-72 (2026-07-29, `e0094c6`…→`730fa4a`)** — sintesi a 9 recepita
> INTEGRALE in `wp72-harness/design72.md` PRIMA del codice; pre-registro
> predictions72 a catena ADDITIVA (lock1 9b9391fc → lock2 376147c2
> post-B-72.1 → lock3 U-72 strutturale), harness wp72 SOTTO GIT
> dall'apertura (K-72.2: hash pre-letto per costruzione). **GATE72 PASS
> fails=0 sul binario finale** (attempts=4, onesto: 1 rigettato per
> rebuild + 1 FAIL con 2 morsi — §S-72.6 e fase-A — pagati con fix a
> meccanismo; correzione K-73.3 recepita). Stash **phpr-wp72 (4fd7b2d5…, 28°)**; census memgc72.

## 🔴 PUNTO 0 — debiti d'apertura delta-zero CHIUSI (d3edc18 + 2333039)

- **M-72.1** (opposizione Matsakis): UnsetWalk col canale AA (enum
  `UnsetAa` Unset/Descend parcheggiato da field_unset_walk E
  unset_into_walk; `drain_unset_aa` al VM: offsetUnset leaf,
  offsetGet+resume mid-path, Notice indirect-modification su
  non-object, Fatal su non-AA) + leaf-op-parity in
  field_unset_prop_step (enum-case → visibilità → asym → readonly con
  clone-writable → mark_typed_unset). **f1/f6 BYTE-ID** + 4 forme
  adiacenti (f7 Notice, f8b catena OG→OU, f9 Fatal as-array, f10 dim
  puro) ⇒ gate-h72; K-M72.1 SCIOLTO. Corpus IDENTICO dopo il fix.
- **S-72.1/2/3**: dedup register per identità + `callable_eq` per
  `[$obj,'m']`/`['Cls','m']` (b1/b7); cursore SOSPESO muore con
  l'elemento (b5b; riposizionamento solo per il cursore ATTIVO);
  prepend-durante-lookup = deviazione DELIBERATA (Zend livelocka col
  cursore posizionale: BUG(port) + PHPR_DIVERGENCES + pin phpr
  PROVENANCE). gate-s72 11/11 (con a8 final-vs-readonly BYTE-ID).
- **H-72.3** drain-fail conta e SKIPPA (mai op(Null,·) fabbricato);
  **H-72.5** chiavi Ref try-routed; **H-72.1/M-72.2** gate-walkborrow72
  esteso (path_op, drain_aa_pending, drain_unset_aa, field_set_mode/
  in_root + blocco FieldUnset run.rs) con **BORROW-OK=13 PINNATO**;
  **K-72.1** k72-stamps emesso PRIMA di ogni letto (C4 =
  ASSORBITA-NON-MISURATA, decisa poi da T-72.c).

## 🔴 LEVA S-72.4 — MASS-TEARDOWN Zend-fedele (d5fd1c9 + e30b955 + 730fa4a)

Scoperta d'audit: `Vm::created` È GIÀ l'object-store per-richiesta
(BTreeMap id→Rc = store order) e run_shutdown_destructors lo iterava
in `.rev()`. Forma spedita (ordine P-72.3):
1. **Fase A** = `zend_hash_reverse_apply` a fixpoint: slot globali in
   REVERSE, muore solo l'ultimo riferimento utente; estesa a celle
   CONDIVISE (post-include i globali sono Zval::Ref) e CLOSURE (oggetti
   nel symtab Zend) — frame globale da frames[0] o retired_main.
2. **Dtor-walk in ORDINE DI CREAZIONE** a round ripetuti (oggetti nati
   nei dtor vengono distrutti, d4 = Zend).
3. dopo TUTTI i flush osservabili (session/stream/ob): **fase BREAK**
   esplicita pre-Drop (M-72.3): drena i root VM (superglobals,
   retired_main, autoloaders, gc_buf coi suoi cloni forti), droppa lo
   store forte, e per ogni sopravvissuto via WEAK taglia gli edge
   uscenti (props take FUORI dal guard + proxy_instance=None).
   Contatori `tag=teardown reg/broken/busy/alive_after` SEMPRE
   compilati; skip sotto FAST_SHUTDOWN (CLI one-shot a costo zero).
⚠️ FIX META (730fa4a): il walk consuma lo store wholesale — senza i
Weak catturati PRIMA del drop il break trovava lo store vuoto e la
metà memoria era un NO-OP (istogramma tutto-zero l'ha svelato).
Batteria **P-72.4 d72 6/6 BYTE-ID** + c1 (KS-S72.2 SCIOLTA) + c2 +
droporder BYTE-ID all'ORACLE; 3 sentinelle RI-PINNATE con attestazione
oracle (gc_id_reuse: coda ORA Zend-order — la vecchia era la
divergenza prezzata; frame_drop generator; probe58-dtor 29/29,
.pre-wp72 + PROVENANCE). **Corpus 1420→1418 = −2 FIX REALI**
(fibers/destructors_011 + bug74053, dtor-order) — RI-PIN PROVENANCE.

## 🔴 IL GATE HA MORSO DUE VOLTE — e ha pagato (e30b955)

Il fix callable_eq ha ATTIVATO DebugClassLoader di Symfony (prima il
suo unregister falliva in silenzio: la via checkClass non era MAI
girata) ⇒ hk 5E/2F da 4 difetti PRE-esistenti smascherati, tutti
chiusi: **S-72.6 wrong-result: tabella trait keyed per NOME BARE** —
due trait omonimi in namespace diversi collidevano (il secondo mai
registrato, `use` bindava il PRIMO in silenzio; fixture g12) ⇒ re-key
FQN uniforme (registrazione join_ns + riferimenti via resolve_class +
conditional-trait + autoload gate); ReflectionClassConstant::
getType/hasType (tipo non ritenuto ⇒ null, catalogato);
STREAM_CRYPTO_METHOD_*/PROTO_*; declared-id RISTRETTO a
seed_conditional (l'allineamento seed↔runtime non è universale con
l'elisione). hk 1665 0E/0F RIPRISTINATO. Scoperto → backlog:
get_included_files() assente.

## E-71.H1/H2 — AUDIT ESEGUITO + BUG REALE (675cbeb, ristretto e30b955)

Censite TUTTE le letture class_index nel lower: 2 gated, 3 span-gated,
check_readonly_extends GATED-BY-CONSTRUCTION; 3 NON gated → flag
impure. **La fixture E-71.H2 ha scovato un wrong-result**: il bind
deferito su nome CONDIZIONALE usava la PRIMA immagine compilata (h2a/
h2c: Fatal readonly sul ramo MAI eseguito) — fix: il callback
`declared` restituisce l'ID runtime e l'indice binda il ramo
DICHIARATO (solo per nomi seed_conditional). probe72-h2 (flip
readonly del parent tra 2 richieste, stesso server): PASS 3/3.

## 📊 LETTI POST-FIX — TUTTI PASS (runner committati, catena lock additiva)

- **B-72.1**: reg/req=680 (O(roots)) · **broken/req=21 COSTANTE
  199/199** · busy=0 → lock2.
- **TRIPLA72 PASS**: **used_n 0,000 obj/req (spread 0,000) — era
  20,000**; used_b 0,0010 — era 2,1109; i 5 canali per-bin → ~0;
  **160|192 = 0,008 ⇒ C4 CHIUSA (≤0,2)**; cellpark/drainfails/busy 0; U-72
  steady in forma STRUTTURALE lock3 (pin 512 STANTIO: fp/req=505, 7
  load-unit in meno dai fix di fedeltà — zero re-cold).
- **AMP72 PASS**: K=10 post-fix delta **0,005** ∈ [−0,5,+5] (pre-fix
  +80,000) — controllo negativo chiuso.
- **LADDER P-71.3**: fp 191,0/189,0/176,4 — **ESISTENZA-ONLY, non
  attestato a macchina** (correzione concilio K-73.3: token verdict
  vuoto per bug `perl --`, classificazione in nota umana, banda
  unilaterale cieca al drift purge d23=−12,6 ≈5σ). La prova flat VERA:
  tripla 0,000 + **peak vmmap 221,5|221,1|221,5 MiB (spread 0,4 su
  1000→9000 req)** — L-73.1: il peak diventa l'invariante primaria.
- **C-72 CPU PASS-PARZIALE** (correzione concilio: il lock aveva DUE
  metri — media-group MAI letta; e Bak: il path CLI non esercita il
  walk per-request ⇒ il costo O(680)/req si prezza in WP-73 su -S):
  coppia full stessa-sera old(wp71)=821,4s /
  new=818,4s ⇒ **−0,36%** ≤ +1: leva CPU-NEUTRA; fail-set full 88
  IDENTICO (== run33 baseline); sagoma 30.472/2F/86W/73S conservata.
- **P-71.4/P-72.6 PASS** (probe72-axumgate): attempted-retry
  cross-request 3/3.

**⚡ SBLOCCO AXUM DICHIARATO** (E-72.H1/KS72-H1): P-71.3 ✓ AND
E-71.H1/H2 ✓ AND P-71.4/P-72.6 ✓ — KG70-1 DECADE.

## GATE72 — PASS fails=0 sul binario FINALE (attempts=4, onesto)

cargo 1651/0 · sentinelle 5 assi (dtor RI-PIN) · fixture WP-67..72
TUTTE (h72 13 casi, s72 11, d72 6, walkborrow72 BORROW-OK=13) ·
corpus **1418 IDENTICO** · refl 290 · ORM 3E/13F · hk 1665 0E/0F ·
reverse 2F per nome. Attempt 1 = contaminato da rebuild (rigettato,
lezione rispettata); attempt 2 = FAIL fails=3 con 2 morsi (droporder + hk);
attempt 3 = PASS su 72b76f80; attempt 4 = PASS sul finale 4fd7b2d5.

## ⭐ Lezioni

- ⭐⭐ **Un gate che morde vale più di dieci che benedicono**: il FAIL
  di hk ha trasformato un fix di fedeltà (callable_eq) nella scoperta
  di un wrong-result dormiente da sempre (trait bare-name) — la via
  appena ATTIVATA da un fix va trattata come superficie NUOVA da
  gateare, non come regressione da zittire.
- ⭐⭐ **Un contatore tutto-zero è un controllo positivo fallito, non
  un successo**: l'istogramma B-72.1 a zero ha svelato che la metà
  memoria della leva era un no-op (walk consuma lo store → break senza
  radici); senza il letto strumentale la tripla avrebbe attribuito il
  merito a un meccanismo MAI eseguito.
- ⭐⭐ **La sagoma Zend del teardown è a TRE fasi asimmetriche**:
  reverse-symtab-apply a fixpoint (refcount-1 only, Closure incluse,
  celle condivise incluse) → store-walk in creazione (spawn compresi)
  → free wholesale; ogni fase ha la sua fixture e nessuna si deduce
  dalle altre (d2 ha richiesto la prima, d4 la seconda, l'istogramma
  la terza).
- ⭐⭐ **I pin numerici cross-sessione invecchiano**: il 512 di U-72
  era vero in WP-66..71 e STANTIO stasera (505 da 7 load in meno,
  effetto benigno dei fix); la forma giusta del lock è STRUTTURALE
  (miss==unit/req ∧ zero-after), non il numero assoluto.
- ⭐ Gli argomenti negativi a `perl -e` vogliono `--` (il classificatore
  del ladder ha emesso token vuoto su d12=−2.0).
- ⭐ L'attempt-counter onesto è la storia del gate: 4 tentativi con 2
  FAIL che hanno prodotto fix a meccanismo valgono più di un PASS al
  1° run su una matrice che non morde.

## Parità e stash

Release **phpr-wp72 (4fd7b2d5…, tree `730fa4a`)**, stash ADDITIVO —
28° in archivio (motivo: fix engine M-72.1/S-72.x/leva S-72.4/
E-71.H1-H2/S-72.6). Census phpr-memgc72 (c6cf3da9) in phpr-mem-target.
Delta engine vs wp71: UnsetAa+drain_unset_aa + leaf-parity unset +
callable_eq/dedup/cursore + mass-teardown (fase A + store-order +
break + teardown_weaks + contatori tag=teardown) + trait table FQN +
declared-id per seed_conditional + note_seed_super ×3 + getType/
hasType + STREAM_CRYPTO_* + 2 sentinelle interne ri-pinnate.

## Prossimo (WP-73) — vedi NEXT_SESSION §WP-73
