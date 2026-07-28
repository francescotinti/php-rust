# WP_SESSION_68 — LEVA DEFER-ALWAYS SPEDITA (autoload fuori dal lowering; le 15 diventano hit_cross=512; ordering-s68 8/8 BYTE-ID oracle a 2 lati; gate68 PASS; 88 BYTE-ID ×3) + fix KS-P67.2 + fix drift seed-fold

> ⚡ **WP-68 (2026-07-28 mattina, `9f4c8a9`→`f663765`)** — sintesi a 9
> recepita INTEGRALE in `wp68-harness/design68.md` PRIMA del codice;
> predizioni committate PRIMA dei letti (9bc603f, locked sha256
> 3deb3b1b…). Verdetti integrali: design68 §10 (disposizione K-68.3
> per OGNI predizione: §10.11).

## Precondizioni KS-P68.1 (P-68.3) — eseguite PRIMA del codice leva

- (a) KS-S67.2 static su unit cacheata: PASS (N:1 ogni R, hit cross
  asserito). (c) S-67.4 g2′ include-vuoto: PASS.
- **(b) P-67.2: FAIL(3) ONESTO — violazione REALE di KS-P67.2**: nome
  con assegnazione condizionata mai eseguita affiorava in $GLOBALS
  come NULL (oracle: assente; anche CLI e nested). Causa: `make_cell`
  del bridge include/eval definiva Undef→Null per OGNI slot aliasato.
  **Fix (16869da)**: `make_cell_bridge` PRESERVA Undef ai 4 siti del
  bridge; `make_cell` resta promozione DEFINENTE (`&$x`/`global $x` →
  Null, oracle-pinned) anche su Ref(Undef). Test
  `scope_bridge_does_not_define_unassigned_names` su `run_source_with`
  (il path `run_module` senza HIR lowera gli eval standalone —
  harness-only, pre-esistente, dichiarato). Rerun PASS(0); verdict
  FAIL archiviato `.superseded-*` (K-68.1).

## LEVA DEFER-ALWAYS (b8dca2c; decisione concilio a 5 sedie)

- **Forma**: `lower_unit` → `DeferPolicy::All` (lo stesso late binding
  del MAIN script): un supertype irrisolvibile da seed∪unit-corrente
  NON autoloada nel lowering ma deferisce al DECLARE (`run_deferred`),
  dove l'autoload fira nell'ordine di Zend ⇒ lowering senza
  side-effect ⇒ `pure` ⇒ le ex-impure PUBBLICABILI dal path esistente
  (guard pure&&main_hir INVARIATO). Siti non-deferrable (use hoisted
  di trait): autoload eager, pure=false, never-published (KS67-1).
  Guard di terminazione = set `attempted`. `DeferPolicy::Set`
  ELIMINATA (H-68.5). Frame di include nel trace come `require()`
  (`unit_call` via `pending_unit_call` — a-func lo esige).
- **Batteria ordering-s68 (5 casi Stogov) + declaration-order H-68.1
  (3 casi nuovi, pin oracle con PROVENANCE): 8/8 BYTE-ID all'oracle a
  2 LATI** (cold CLI + hit -S R1/R2, `hit cross` asserito — persino il
  Fatal di a-func riprodotto byte-id DALLA CACHE). **La divergenza
  semantica S-67.2 è CHIUSA**; gate-ordering RI-PINNATA a oracle
  (atto deliberato S-68.3/P-68.5, P68-B-d).
- **KB67-2 CONSEGNATO**: probe68-ksp1 (release): miss_cold=0, dc=0,
  fp=0, **hit_cross=512/512** U8/U9/U10, fp-seq identiche, body
  byte-id ×10, self-traffic 0.
- **Quota S-68.4** (G-68.4: mediana R4→R10, ×3 run, ms su build
  census): Δdefer=16/req, **7,84 ms/req [7,79-7,85]**, 490 µs/DECL;
  **Δ(l+c) whole-file 21→0,00 ms/req**. KS68-2 NON scatta (≤10) ⇒
  fallback publish+dep-replay DECADE. Guadagno netto ≈13 ms/req su
  census; canale di riduzione (cache dei defer-mini) NOMINATO per il
  concilio, non aperto.
- **P-67.4 ARMATA** (gate-editmiss): figlio editato ⇒ MISS + body
  nuovo; padre resta hit.

## 🔴 SCOPERTA del gate (run1 FAIL): drift seed-vs-runtime PRE-ESISTENTE

hk/reverse panic "prefix misaligned at id 180 (\0deferred\0180),
runtime len=179". Diagnosi: `seed_delta_of` piegava nel seed anche le
classi di coda con nome GIÀ REGISTRATO (es. `class ValueError extends
Error` GUARDATA nei bootstrap symfony/polyfill) che a link dedupano
SENZA mint ⇒ seed > runtime, e l'identity-arm POSIZIONALE del remap
elided si rompeva alla prima entry non-registrata successiva. Latente
da sempre; i placeholder di defer-always l'hanno reso comune.
**Fix (f663765): il fold del seed filtra i nomi già in class_index —
fold == mint, invariante posizionale vero PER COSTRUZIONE.** Effetto
spiegato alla cifra: sentinelle65 main.elide 177→176 (una dup in meno)
⇒ ri-pin deliberato con PROVENANCE + copia pre-fix.

## Ri-pin sem (S-65.3 forma WP-68)

Il fix KS-P67.2 ha RISTRETTO il diff $GLOBALS verso l'oracle (i nomi
mai assegnati spariti da G1/D1). probe68-sem: DOPPIO diff pinnato
(vs oracle = residuo pre-esistente ristretto; vs stash wp67 = firma
esatta del fix); movimento oltre i pin = FAIL.

## GATE68 PASS (0 ✗) a VERDETTO MACCHINA (terzo run, 8eea807e)

Run1 FAIL(5) e run2 FAIL(1) ARCHIVIATI (.superseded — K-68.1);
il PASS esercita TUTTA la matrice nello stesso run (K-68.2):
**K-68.4 trap self-test ESERCITATA** · **M-68.1 assert park** (3
call-site+1 def, Box::leak(=0) ) · cargo **1648/0** · sentinelle 5
assi · fixture gate-ordering(oracle)+**s68 2 lati**+**droporder
P-68.1**+**editmiss P-67.4** · sentinels65(ri-pin 176)+KS-S6+
seed_prefix_short=0 · KE-e · P1 · sem doppio pin · corpus **1421
IDENTICO** · refl **290** · ORM **3E/13F** · hk **1665 OK** · reverse
**2F PER NOME**.

## Catena serale run56-triple + B-68.3 + metro

- **run56**: new **790,31u** · old(wp67) **791,10u** · new′ 815,27u —
  coppia **−0,10%** ≪ spread A-A′ **3,16%** ⇒ costo CPU leva
  INDISTINGUIBILE DA ZERO; **fail-set 88 BYTE-ID = run33 ×3**.
- **B-68.3 FAIL ONORATO (KH68-3: banda axum NON citabile)**: pendenza
  wpdev +62,6 KiB/req ma spiegata ALLA CIFRA come census-own
  (CENSUS_HITS mai potata: 512 righe/req × ~120 B ≈ 61,4) — ipotesi
  quantificata, prova = L-68.1 (WP-69). **Dato forte: de-leak wpdev
  LETTO alla scala (G-68.3 chiuso)** — dead 16.009/17.045 (~16/req),
  vivi PIATTI ≈1.036 (plateau ~2 entry/path della cache 4-way — input
  per B-67.4/ways).
- **Metro KL67-1 terza run ESEGUITA** (19 min): counted master PIATTO
  231,9 ≈ 231,8 MB pre-P2 — **errore di MECCANISMO della predizione**
  (il master CLI è UNA richiesta: RetainSet vivo fino all'exit ⇒ il
  metro full NON esercita il canale per-richiesta; giudice = probe
  server N={1,100,1000}). Σcommitted 1412,7 MiB (informativo).
  Per-causa CONTAMINATA (swap 7,5 GB pre-run, KL68-1) — phys standing
  RESTANO SOSPESI; metro NON-promosso (KL68-2 → WP-69 con no-swap da
  reboot + L-68.1).

## Debiti P-2 e puntuali consegnati

E-68.2 prune Weak al dump (CENSUS_UNITS+SPLITS in coppia; dead
cumulativi): pendenza nk **2,549→0,586 KiB/req** — bookkeeping
riconciliato 1,963 (≈77%, H-68.2); KS68-1 non scatta. P-68.2 prova
viva STUBS: elision-off ⇒ stubs_entries **0→173**, Δ=0 ×300 (Δbytes:
colonna assente → WP-69). H-68.3: stable_deref_trait **1.2.1**
(6ce2be8d…) = marker-crate pura, `StableDeref for Rc<T>` sound.
M-68.4 SAFETY ristretta; M-68.5; H-68.4; E-68.3 ceiling dichiarato.

## ⭐ Lezioni

- ⭐⭐ **Il fold del seed deve rispecchiare ESATTAMENTE il set che
  minta**: ogni asimmetria tra immagine di lowering e tabella runtime
  è un debito che la prima entry "posizionale" riscuote — l'invariante
  va reso vero per costruzione, non per adiacenza.
- ⭐⭐ **La macchineria del main è la forma fedele anche per le unit**:
  defer-always non ha aggiunto un meccanismo — ha rimosso l'eccezione
  (autoload-in-lowering) che rendeva le unit sia infedeli sia impure.
- ⭐⭐ **Un kill-switch che scatta è un risultato, non un fallimento**:
  B-68.3 FAIL + ipotesi quantificata alla cifra (512×120 B) vale più
  di un PASS a soglia non discriminante.
- ⭐⭐ **Le predizioni dichiarano anche il MECCANISMO del workload**:
  P68-M assumeva morte per-richiesta su un processo che è UNA
  richiesta — l'errore era nel modello, non nel motore; la
  disposizione K-68.3 lo rende visibile invece di seppellirlo.
- ⭐ Il probe di precondizione che FALLISCE prima della leva è il
  sistema che funziona: KS-P67.2 ha fermato un bleed reale prima che
  la cacheabilità lo moltiplicasse.
- ⭐ I pin che si muovono per un MIGLIORAMENTO si ri-pinnano a coppia
  (vs oracle + vs stash = firma del fix): il tripwire resta armato su
  ogni movimento ulteriore.

## Parità e stash

Release **phpr-wp68 (8eea807e…, tree `f663765`)**, stash ADDITIVO
accanto a wp67 (c0f5cfff…) — 24° in archivio. Delta engine vs wp67:
make_cell_bridge (KS-P67.2) + defer-always (DeferPolicy::All, Set
eliminata) + unit_call nel trace + seed-fold fold==mint + prune
E-68.2 (census-only) + colonne defer_relowers/defer_relower_ns.
Census: phpr-memgc68 (5027e4e8, tree f663765) in phpr-mem-target/.

## Prossimo (WP-69) — vedi NEXT_SESSION §WP-69
