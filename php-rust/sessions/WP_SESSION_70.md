# WP_SESSION_70 — debiti d'apertura del concilio CHIUSI (🔴 H-70.1 walker a disciplina di confine + 🔴 S-70.1..3 declare-fallito Zend-fedele) + GATE70 PASS al 1° run + CACCIA AL RESIDUO: esiste su RELEASE, tripla 2,1121 KiB/req / 20,000 obj/req DENTRO, attribuzione APERTA (axum resta fermo)

> ⚡ **WP-70 (2026-07-29 notte, `2a308ff`→`1f73b2b`)** — sintesi a 9
> recepita INTEGRALE in `wp70-harness/design70.md` PRIMA del codice;
> predictions70 lockate (cbf251fe…, con addendum PRE-letto) prima di
> ogni letto. GATE70 **PASS fails=0 AL PRIMO RUN** (28 ✓). Stash
> **phpr-wp70 (a666382e…)**; census phpr-memgc70 (9201ed94).

## 🔴 H-70.1 ≡ M-70.1 (bloccante Hoare+Matsakis) — CHIUSO (4c0a3bc)

Panic riprodotti alla riga (`&$a[0][0]` → field_cell; `&$o->self->x`
→ object borrow). Fix = **disciplina di confine**: i walker si fermano
a OGNI confine di cella (Ref/Object), consegnano l'handle clonato a un
driver a loop (token `CellWalk`/`PathWalk`) e il guard cade PRIMA del
borrow successivo — una cella rivisitata non trova mai il proprio
guard vivo. **Scoperta di sessione: il panic viveva ANCHE sul path di
SCRITTURA** (`$b[0]="x"` sul ciclo → `set_returning_displaced`
array.rs:730; OpSet old-read; IncDec): chiusi con `LeafWrite::Busy` +
parcheggio `RefLeaf` drenato da `path_op` FUORI dai guard (la store di
Zend attraverso il puntatore collassato). M-70.2 contatore
`tag=cellskip`. Batteria K-M70.1 `gate-h70cycle` **PASS 8/8** (7 pin
oracle; wopset pin phpr con PROVENANCE — divergenza PRE-esistente
riga-warning-attraverso-alias, riprodotta su wp69 non-ciclica ⇒
backlog famiglia diag-line). Cargo belt test. **KH70-1 SCIOLTO.**

## 🔴 S-70.1..3 (bloccante Stogov) — CHIUSO (87b0162)

Causa radice: `lower_source_seeded` pre-popolava il class_index con
OGNI seed, incluse le CONDIZIONALI mai dichiarate ⇒ bind fantasma.
Fix S-70.2: predicato `declared` (= class-table RUNTIME) sul ci del
seed ⇒ il declare con supertype non-runtime-dichiarato DEFERISCE al
punto di esecuzione, dove risolve da class-table + spl_autoload in
ORDINE ZEND (b_chain/b2/b3 → Error+255; b4 ordine; b5 fantasma morto;
m_auto → `load:M` stampato, anche via require). Trappola cache S-69.5
governata: bind eager su seed-condizionale runtime-dichiarato ⇒
`used_conditional_seed` ⇒ `pure=false` (mai pubblicato). Guardia
attempted anti-loop in run_deferred. S-70.3: class_alias →
`Cannot redeclare class %s` (grafia argomento, kind sempre "class").
Batteria `gate-s70neg` **PASS 12/12**. **Corpus 1422→1421: −1 fix
REALE (class_order_autoload1.phpt), zero regressioni — RI-PIN con
PROVENANCE.**

## Emendamenti (feca5b2) + gate

H-70.2+H-70.3 fold==mint a zip PER NOME su ENTRAMBI i rami · H-70.4
batteria trait-dietro 5/5 (t4 ha esposto trait-redecl "unsupported" ⇒
fatal Zend col sito della prima, oracle-pinned) · H-70.5/M-70.3 note
di prova · K-70.4 gate70.sh etichette corrette · E-70.3/B-70.2
rettifiche applicate anche in design69/WP_SESSION_69 · K-70.3 timbri
(D e P69-S-a/b NON-ESEGUITE) in design70 §retro. **GATE70 PASS
fails=0 al 1° run**: cargo 1650/0 · sentinelle 5 assi · fixture
WP-67..70 TUTTE · corpus 1421 · refl 290 · ORM 3E/13F · hk 1665 ·
reverse 2F per nome. K-69.3 = 178 invariato.

## 🔴 CACCIA AL RESIDUO (predictions70 LOCK cbf251fe PRIMA dei letti)

- **P70-0-bis DENTRO ⇒ ESISTE-SU-RELEASE**: two-boot vmmap sul
  RELEASE strumento-free (185,0→190,8 MiB, Δ=5,8 ∈ [5,15] su
  ΔN=4000). L'ipotesi strumento è REFUTATA: il residuo è ENGINE.
  (P70-0 forma mi-stats: NON-CALCOLABILE — niente stats su SIGTERM;
  addendum dichiarato e ri-lockato PRIMA del letto.)
- **P70-T DENTRO SU TUTTO — KL70-1 leak=SI**: 3 leg indipendenti
  (memgc70, HITS-off, cron-free, UNIT_CACHE_LOG, finestre con assert
  macchina TUTTE valide): used_b LS mediana **2,1121 KiB/req**
  ∈ [1,7-2,6]; used_n **20,000 obj/req** (intero alla 3ª cifra su 3
  leg!) ∈ [15-25]; firma per-bin 5/5 in ±15% per leg; segno 3/3.
  **Self-spread fondato: 0,024 KiB/req · 0,003 obj/req (= MDE cacce
  future).** Run1 analisi FAIL(1) onesto (bug parser win=) →
  superseded, rifatta SUGLI STESSI grezzi (analyze70-tripla.pl).
- **K-M70.2 non scatta**: cellskip=0 su 3 leg wpdev.
- **P70-D FUORI (per costruzione) — canale VIVO, attribuzione
  APERTA**: defermini calls **16,00/req ESATTO** (= le 16 DECL
  defer-always WP-68); net_b 1526 KiB/req = churn del path (~95 KiB/
  defer re-lower+compile), il net-form clampato NON misura il
  ritenuto. Attribuzione ≥80% per-causa NON raggiunta ⇒ **fronte
  axum FERMO** (KG70-1). WP-71: contatore RETAINED-form (trap
  bin 112/128 o tag per-sito, L-70.1).

## Spike dispatch — evidenza di costo a verbale (§7.5→7.6 design70)

run.rs: **83 `continue` + ~115 `return` DENTRO i bracci** ⇒
l'outlining non è un wrap `#[inline(never)]` ma una trasformazione
per-braccio a enum-di-stato del control-flow. Conferma quantitativa
del "non si improvvisa": build outlined = sessione dedicata a
macchina quieta (stanotte assorbita dalla caccia, priorità 1).
predictions69 (b91b4799) + recinto P70-S restano lockati (KK70-3).

## ⭐ Lezioni

- ⭐⭐ **La disciplina di confine batte il try-puntuale**: H-69.2
  aveva messo un try su UNA foglia; il panic viveva un frame sopra e
  ANCHE sul path di scrittura — la proprietà giusta è "nessun guard
  vivo attraverso un confine di cella", non "questo borrow è safe".
- ⭐⭐ **Un registro di compilazione non è una class-table**: il seed
  ci con le condizionali mai dichiarate creava classi fantasma; la
  risolvibilità eager è una proprietà RUNTIME (class-table + autoload
  in ordine Zend), e ogni eccezione va marcata impura per la cache.
- ⭐⭐ **Un contatore di flusso non attribuisce un ritenuto**: il
  net-form clampato per-call ha dato 1526 KiB/req su un residuo di
  2,1 — utile per dire che il canale FIRA (16,00/req esatto), inutile
  per l'attribuzione; serve il retained-form per-sito.
- ⭐⭐ **La firma intera-esatta è oro**: 20,000 obj/req su 3 boot
  indipendenti con spread 0,003 = set FISSO deterministico — il
  residuo si caccia per costruzione, non per statistica.
- ⭐ Un pin di fixture scopre i gap adiacenti PRIMA della leva (t4:
  trait-redecl "unsupported" trovato pinnando la batteria H-70.4).
- ⭐ Il two-boot Δ cancella lo startup e rende citabile un metro
  lossy (vmmap) come verdetto di ESISTENZA — mai come slope.

## Parità e stash

Release **phpr-wp70 (a666382e…, tree `feca5b2`; +1f73b2b census-only
cfg-gated, parità invariata)**, stash ADDITIVO — 26° in archivio.
Census phpr-memgc70 (9201ed94) con tag=cellskip + tag=defermini.
Delta engine vs wp69: CellWalk/PathWalk/RefLeaf/LeafWrite (walker) +
elem_cell_step + cell_skip_note + predicato `declared` e
`seed_conditional`/`used_conditional_seed` (lowering seeded) +
run_deferred attempted-guard + class_alias msg + trait-redecl fatal +
assert_fold_equals_mint (zip per nome, 2 rami).

## Prossimo (WP-71) — vedi NEXT_SESSION §WP-71
