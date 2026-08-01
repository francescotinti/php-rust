# WP_SESSION_83.md — S-83.0 "DENTI DI VERBALE + MISURE IN FORMA NUOVA" — ✅ 8 punti Concilio WP-84 + VERDICT83 PASS + partizione-per-TIPO

**Data**: 2026-08-01 notte
**Scope**: ordine vincolante Concilio WP-84 §Sintesi (8 punti) eseguito
integrale. Modello verificato all'apertura: Fable 5.
**Commit**: 050c891 → fc6cbb7 → 6644981 → ebdd0d8 → 0424e27 → f4c3ece →
7b90614 → ac35d24 → 925da3b → e5f2a5e → d62ff33 → **857da59** (tutti su
main, pushati).
**Binari**: phpr **7a6104575134de27** (nuova baseline parità, stash
additivo `phpr-wp83`; bit mossi da A-MS17/A-MS22/partizione A-MS24;
corpus 1418 + refl 290 per NOME IDENTICI, verificato DUE volte: p1a e
p6); campagna-2 a matrix e5f2a5e; base arm 75108f696feec495 (7593d8e).

## Ordine WP-84 eseguito (p1-p8)

| # | Esito | Commit |
|---|---|---|
| 1a | **A-MS17 al PRIMO commit** (KS-MS-84-1 onorata): `VmGate` ZST a costruttore privato su vm_new/park_main — rustc giudice, awk declassato a cintura (sezione 1b: 3 mint pinnati per NOME); rustc ha trovato ESATTAMENTE i 5 call-site noti (KS-MS-83-2 non innescata). **A-MS22**: lower_net → `MainPublishTicket{key,fp,lower_net}`, letto SOLO in main_publish_ticket | 050c891 |
| 1b | A-TH23 (split prod==2/selftest==7 ancorato su `retained_walk_selftest` + class-sweep workspace con allowlist NOMINATA — phpt-runner/lib.rs:287 nominato; crates/*/tests esclusi DICHIARATI) · A-TH24 (dente cast epoch case-insensitive + pin forma celle IC u64-first) · A-DS23 (inventario 10 mappe per NOME + verbi estesi; il dente nuovo ha MORSO in audit: 6 siti order-insensitive di class.rs che il regex a suffisso non vedeva, allowlistati dopo audit) · A-TH25 (timeout dichiarato 2s, tempo nel fail, polarità documentata) | fc6cbb7 |
| 2 | A-PP22 `uc_main_entry_count()` key-blind + delta==0 in a_pp20 + controllo positivo (+1 su main pulito) · A-PP23 metà macchina (pin lessicali: flush in request_shutdown ==1 region-anchored, send-after-execute) · A-SK30/A-AH34 `battery-equivalence.sh` (vincoli i-iv, il `-ge 14` è MORTO) · A-SK29 `SUPPLEMENT_NORM.md` (lista chiusa, ledger, max 1/fase) | 6644981 |
| 3 | **A-DL20/21/22 strumento scomponibile**: `main_put_ordinal` in CachedUnit, righe `tag=unitcache_main_entry` in ordine di put (floor_inc = incremento walk, il primo put paga il prelude), `program_floor_share` + `rw_budget` + `arm=cli-server` + `alloc_id` in-band; A-AH33 const `memcount-v2-s82` pinnata in census-twin | ebdd0d8 |
| 4 | F14b (matrice (o,c) completa con atteso C2 PINNATO + pin-HIT put==1/hit≥6) · F15b interleavata (pre-mitigazione: FIFO confermato, main_evicted==1 col main toccato) · fixture path 415B + autoload83_reg · **A-DS22: il mutation-test ha morso DUE volte** — sig() cieco ai corpi dei METODI e al CONST POOL; comparatore rafforzato (ops+consts di main/fn/metodi), 5 bracci verdi sotto il giudice forte | 0424e27 |
| 5 | A-BG32 probe `__reqns` (buffer in-memory, drain via router FUORI dalle finestre, WP-64; ends_with-anchored) + hook drain in measure78 · A-AH31/32 `base-arm-build.sh` (worktree-subdir check, lock copiato, --offline dichiarato, lock-cmp post-build, header A-TH26) · campagna83 + verdict83 (min-of-R R=9, f=5% ex-ante) | f4c3ece+7b90614 |
| 6 | **MISURE (campagna-2 a e5f2a5e, 0 fail, VERDICT83 PASS)**: vedi `wp83-harness/MEASURE83_RESULTS.md` — VC PASS (slope 110,0 vs 6920,0 µs/req, risoluzione 20,0<123,7 — KB-83-3 CHIUSA nella forma nuova, ~63×), VR algebra ESATTA (budget scorporato 19,69MB ×W), VA split (+4 register / ≤+52 include-HIT), VW modello 2×len FALSIFICATO al confine (esito nominato) | d62ff33 |
| 7 | **p6 partizione-per-TIPO IMPLEMENTATA**: `UnitSlot{main, includes}` (KS-MS-84-3, mai skip-per-tag; main = one persistent_script per path, replace mai evizione; includes = FIFO ways; bound ways+1); main_evicted resta CABLATO sulla corsia include come tripwire KS-DS-84-4; **F15/F15b FLIPPATI e verdi** (main_evicted==0 CON macchina di evizione viva: 5° fp evince un include, main HIT post-thrash) | 857da59 |
| 8 | Revert policy KS-DS-80-3: MAI innescata (zero body ≠ oracolo in sessione) | — |

## 🔵 Scoperte

1. **Il "flat 1,85MB/main" di S-82.0 era un artefatto dell'aggregato÷4**
   (KL-84-2 risolta): per-entry, ord=1 net=7.349.977 B (residuo one-time
   del PRIMO lower del processo: prelude/interner ≈35,6% del budget),
   ord≥2 = 4-8KB. Il residuo è di PROCESSO, non per-entry.
2. **Il dente lock-cmp A-AH32 ha morso alla prima uscita**: il build base
   PRUNA l'edge dev-dep `"mimalloc"` (una riga, zero drift di versioni).
   Il byte-cmp letterale è insoddisfacibile tra manifest che differiscono
   per un dev-dep ⇒ emendato a PRUNE-ONLY (diff = solo cancellazioni +
   subset (nome,versione) identico), DICHIARATO per il giudizio WP-85.
3. **sig() di a_ds17 era doppiamente cieco** (KS-DS-84-2 aveva ragione):
   né corpi dei metodi né const pool nel comparatore — una mutazione
   reale confrontava UGUALE. Rafforzato; la purezza regge sotto il
   giudice forte (bracci polluted/permuted/mutation verdi).
4. **Il modello a_bytes==2×len è FALSO al confine** (VW): a len=415,
   a_calls=4 e a_bytes=1662 = 2×len + 2×(len+1) — due copie EXTRA
   NUL-terminate sul path lungo. Sito della soglia da nominare.
5. La quota register dell'autoload è PICCOLA: +4 call/req; l'include-HIT
   è ≤+52 (upper bound vs opcache inheritance-cache).

## ⭐ Lezioni

1. ⭐⭐ **Il sigillo di tipo è il giudice giusto**: rustc ha certificato in
   una compilata ciò che l'awk garantiva per disciplina (e i due test
   hand-replica sono emersi DA SOLI come errori di compilazione).
2. ⭐⭐ **Un dente nuovo va fatto mordere su codice vero prima di fidarsi
   del suo zero**: A-DS23 (6 iterazioni invisibili al suffisso), A-DS22
   (comparatore doppio-cieco) e A-AH32 (prune inevitabile) hanno TUTTI
   morso al primo giro — tre zeri storici erano zeri ciechi.
3. ⭐⭐ **Il cambio di FORMA chiude ciò che il raddoppio di N non può**:
   min-of-R R=9 ha risolto in una campagna (res 20 µs/req) quello che
   la coppia a fattore 3 non avrebbe mai risolto (Bak aveva ragione
   ad aritmetica).
4. ⭐⭐ **Un dente che non può mai passare non certifica nulla** (duale
   della lezione WP-72): il byte-cmp del lock era strutturalmente
   insoddisfacibile — la forma onesta è quella che distingue il prune
   legittimo dal re-resolve.
5. ⭐ La partizione-per-TIPO ha ucciso la classe thrash SENZA numeri
   nuovi: F15b legge ora "esenzione per corsia" (put==1/hit==5/evicted==0
   con evizione include viva) — la prova è strutturale.

## Residui / NON fatto (dichiarati, per NOME)

- **A-PP18** (ledger drained esatto): APERTA, calendarizzata PRIMA di
  ogni riconciliazione Δglobal a W>1 (A-PP24 onorata: nessuna tentata).
- **A-BB34** (pin identità peak 232±1MB W=10): da rieseguire alla
  prossima campagna su matrix fresca (la partizione non tocca il
  retained ×W; la rimisura formale VP è della campagna WP-84).
- **VW modello nuovo**: soglia della 4ª copia non localizzata nel codice.
- **Emendamento lock-cmp PRUNE-ONLY**: al giudizio del Concilio WP-85.
- Registry condivisa (frazione condivisibile ~69% misurata): SOLO con
  A-BB35 budget ex-ante + riapertura ESPLICITA KH81-3 (KH84-4).
