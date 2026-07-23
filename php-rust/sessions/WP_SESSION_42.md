# WP_SESSION_42 — archivio storico della sessione WP-42

> Convenzione: un file per sessione; il handoff tiene solo l'ultima sintesi.
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-42 (2026-07-23, commit `c6e82c2` warm-up + `19b4d27` piano)** —
> **punto 1 (warm-up silent_get_path by-borrow) ESEGUITO: FLAT su A/B 6
> round → KEEP (precedente WP-36), mini-leva chiusa. Punto 2 (apertura
> formale Leva B) ESEGUITO: census op WP-33 misurato (743,9M op, 30,8%
> data-movement) + piano d'arco in `REGISTER_BYTECODE_PLAN.md`.**

## Punto 1 — warm-up: walk isset/empty by-borrow (`silent_walk`)

Implementazione (design WP_SESSION_41 §verdetti, verificato ok):
- `silent_walk` in vm/arrays.rs: segue il path per riferimento (ricorsione
  = borrow-guard per livello per le catene `Ref`; sicuro: nel walk non gira
  MAI codice utente — un `Object` baila subito in `SilentOut::Object` con
  un solo bump di `Rc`) e piega il leaf raw nel verdetto `IsMode`
  (`Exists` per isset: presente e non-null; `Truthy` per empty, il
  chiamante nega) calcolato sul borrow. Zero cloni di base/intermedi/leaf.
- `DimIsLeaf::Raw(Option<Zval>) → Verdict(bool)`; `dim_is_walk` +
  `is_walk_resume` (driver condiviso): protocollo ArrayAccess INVARIATO
  (offsetExists→offsetGet sugli intermedi, leaf `Aa` al chiamante);
  `field_aa_walk` riusa il driver. `dim_aa_leaf` (unset) resta su
  `silent_get_path`, che sopravvive per quell'unico uso.
- ⭐ La biforcazione exists/truthy vive nei 4 SITI CONSUMER
  (IssetPath/EmptyPath/FieldIsset/FieldEmpty); `??` NON passa di qui
  (CoalesceFetchDim è già inline) — il ramo "value" previsto dal design
  non serviva.
- ⭐⭐ **Trappola trovata dalla probe**: il walk vecchio era per-CHIAVE
  (slice singola a ogni passo), quindi la restrizione "string offset solo
  al passo finale" di silent_get_path non si applicava MAI in dim_is_walk:
  `isset($s[0][0])` chaina attraverso stringhe a un byte. Il walk
  full-slice deve riprodurlo (arm `Str` che continua nel byte estratto) —
  la prima stesura lo perdeva e la probe old-vs-new l'ha preso subito.

Parità provata PRIMA della misura (metodo standard):
- probe battery `wp42-harness/probe-isset.php` (~70 casi: array annidati,
  buchi packed, Ref base/mid/leaf, string offset ±/strkey, AA
  base/mid/leaf/annidato, log ordine protocollo, magic __get/__isset,
  short-circuit offsetExists=false su Throwy, fused-field su prop
  dichiarata/privata, closure): **oracle==new e old==new byte-id**.
- cargo test 1636/0; output media old==new nei 6 round A/B.

Esito A/B interleaved stesso-giorno (media group, user CPU), old =
`phpr-0a03772` (= tree WP-40/41, riusato: f462126 è byte-id sui crates):
| round | old | new | Δ |
|---|---|---|---|
| 1 | 56,64 | 56,40 | −0,42% |
| 2 | 56,26 | 56,29 | +0,05% |
| 3 | 58,74 | 58,23 | −0,87% |
| 4 | 56,90 | 59,40 | +4,4% ⚠️ outlier (wall 79,9s, sys alto) |
| 5 | 57,95 | 58,13 | +0,31% |
| 6 | 58,86 | 57,99 | −1,48% |
Segno alternato, nessuna direzione consistente = **FLAT** (≠ WP-41: 4/4
più lento). Oracle di giornata 20,98/20,88; giornata progressivamente più
rumorosa (wall 75→86s). **Verdetto: KEEP** — precedente WP-36 (flat a
parità si tiene); il codice fa strettamente meno lavoro (meno cloni ⇒
meno traffico gc_note a monte) e la complessità è pari. Mini-leva CHIUSA:
il tetto ~0,5-1% dichiarato in WP-41 non emerge dal rumore — non
reinvestire su questo canale.

Divergenze PREESISTENTI trovate dalla probe (old==new su tutte; catalogo
vivo in `wp42-harness/probe-isset-div.php`):
1. PHP 8.5 emette Deprecated (chiave null / float con frazione) anche
   dentro isset/empty; `coerce_key_silent` è muto.
2. `isset($nonAA['k'])` / `isset($closure['k'])`: Zend lancia Error
   "Cannot use object as array", phpr risponde false (quiet).
3. `isset($mg->m['a']['b'])` via prefisso `__get`: oracle true, phpr false
   (il magic-probe dispatcha `__isset`/`__get` ma perde il walk annidato
   sul risultato) — questo è un BUG funzionale, candidato fix futuro.

## Punto 2 — Leva B: census + piano (deliverable, zero codice registri)

- **Census op-census WP-33** (binario dedicato feature `op-census`,
  target separato; run media; `wp42-harness/census-out/`): **743,9M op**;
  **data-movement puro 30,77%** (PushConst 9,71 · LoadVar 7,85 · DerefTop
  5,44 · Pop 4,23 · Dup 1,79 · StoreSlot 1,48); ThisPropGet 9,90 · Ret
  8,42 (bigramma Ret→DerefTop 40,5M → stadio call-ABI) · Sweep 7,73 (non
  target) · CmpJmpConst 4,85 · Stringify 4,20. Bigrammi da assorbire:
  ThisPropGet→CmpJmpConst 29,9M, ThisPropGet→Stringify 29,2M,
  Dup→StoreSlot+StoreSlot→Pop ~9M ciascuno, ecc.
- **Piano**: `REGISTER_BYTECODE_PLAN.md` (repo root) — registri = slot
  temporanei del Frame; operand-sourcing sugli op CALDI (`src/dst:
  Operand`), non un secondo ISA; 5 stadi (infra a delta zero → Binary/
  CmpJmp → temporanei d'espressione → call-ABI → consolidamento), parità
  a ogni commit, dual-mode opt-in per-funzione, A/B go/no-go per stadio;
  mago intatto; **tetto plausibile ~8-15% CPU sull'arco completo**.
  Rischio n.1: I-cache (sostituire, mai far convivere).

## ⚠️ Incidente disco (macchina utente, da sapere)

Durante il gate22 il volume root (Data) è arrivato a **0 byte liberi**
(228Gi, 100%): Bash del harness inutilizzabile (ENOSPC sul file di output
task), corpus/sess del gate falsati (+2/+18 fail spuri, tutti
session-write/temp-write). Diagnosi via Monitor-as-shell; liberati
**6,3G sicuri e ricostruibili**: `~/.npm/_cacache+_npx` (5,3G, cache npm
ufficiale) + `~/Claude/php-rust-output/debug/` (1,0G, artifact di MIA
pertinenza; i gate usano solo release/). Restano ~5,3G liberi = POCO: i
consumatori grossi sono dati utente (Library/Application Support 33G —
Google 8,9G, Claude 8,5G vm_bundles, Spark 2,5G…, Parallels 4,9G,
/private/var/folders 6,0G). ⭐ Lezione operativa: con Bash bloccato da
ENOSPC, il tool Monitor esegue script e streamma l'output → canale di
recovery. ⭐ Un gate che attraversa una finestra disco-pieno va RILANCIATO
per le suite che scrivono (corpus unset_cv*, ext/session) — verificare
sempre i "nuovi fail" in isolamento prima di crederci.

## Gate e stato a fine sessione

- **Gate22 (commit `c6e82c2`) TUTTO VERDE**: corpus **1447** IDENTICO ·
  sess 28 · date 351 · refl 290 IDENTICI (corpus+sess al RILANCIO
  post-ENOSPC; i primi numeri 1449/46 erano artefatti disco) · ORM 3E/13F
  identico per nome · hk 1665 0E/0F · cargo **1636/0** · gd/mysqli/media
  probe BYTE-ID · http battery DIFF-set = 16 nomi IDENTICO a WP-27a ·
  option/restapi IDENTICI per nome.
- **Full run32**: 30.472 test, 0E/2F/86W/73S, **fail-set BYTE-IDENTICO a
  run31 (88 righe)** — parità piena anche sulla suite intera. Master-CPU
  ~12:50 dal tail .rss (run31 ~11:39): +10% NOMINALE ma è la stessa
  giornata rumorosa dell'A/B (drift +3% intra-day sui round old, disco a
  ~5G liberi); il confronto same-day interleaved dice FLAT — fede a
  quello (lezione WP-36: mai confrontare assoluti cross-giornata).
- Footprint media (maxrss `/usr/bin/time -l`, caveat MADV): old==new
  4,72-4,78GB nei 6 round; oracle 0,36-0,38GB.
- Commit di sessione: `c6e82c2` (warm-up) · `fc5f431` (doc Gemini
  constraints) · `19b4d27` (piano Leva B + census) · + docs di chiusura.

## 📨 Direttive Gemini "vincoli safe-Rust vs C" (`20260723_gemini_rust_constraints.md`) — verdetti (verificati su codice, 2026-07-23)

- **§ Rc tax — PARZIALE**: vero il meccanismo (contatori strong/weak 16B
  per allocazione vs ~8B di zend_refcounted_h), ma l'ordine di grandezza
  del footprint 12× NON è spiegato da questo: l'attribuzione WP-26/27 dà
  dati workload (array/props), da cui dual-repr e slot WP-27.
- **§ RefCell CPU tax — PARZIALE**: i borrow-check esistono, ma il
  profilo WP-41 dice che il canale dominante è il churn clone/drop degli
  operandi (strutturale, → registri), non i check.
- **§ "Leak fisiologico dell'AST" — FALSO**: l'arena mago (`Bump`) è
  locale a `lower_source_impl` e muore a fine lowering. Ciò che resta
  vivo è HIR/bytecode nello unit-cache = un opcache in-process,
  deliberato (WP-5/6, prelude Rc-shared WP-20). I picchi multi-GB sono
  dati del workload, non AST.
- **§ arena per-request — GIÀ BOCCIATA** (collide con byte-parity
  dell'ordine dtor; decisione in vigore, NEXT_SESSION §decisioni).
- **§ HashTable "generalista" — SUPERATO**: PhpArray È custom dual-repr
  packed/hashed (WP-27), non IndexMap; la densità Zend resta superiore
  ma il claim descrive uno stato vecchio del progetto.
- **§ unsafe circoscritto nel value core — RESPINTO da RULEBOOK §0**
  (decisione owner ri-confermata due volte: NaN-boxing WP-32, SSO WP-38).
- Presa in carico REALE dal doc: il footprint 12× resta il fronte non
  aggredito → serve una **sessione di attribuzione memoria data-driven
  (metodo WP-26)** prima di qualsiasi leva, quando si aprirà quel fronte.
