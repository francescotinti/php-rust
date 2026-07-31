# MEASURE81_RESULTS.md — A/B churn verdict of the A-BB6 lever (S-81.0 step 7)
# RETRO-ANNOTATO S-82.0 p1 (Council WP-83: A-BG26/27, A-SK20, A-BB27/29/30, A-AH27)

**Cifre di misura** = ricomputo SCRIPTATO dei raw (KG-82-1:
`wp81-harness/verdict81.sh` v2 → `verdict81.out`, committati entrambi;
P* giudicati sulle MEDIE R=3 dietro md5-gate, A-SK20/A-BG28).
**Cifre DERIVATE** (percentuali, delta cross-campagna) = righe `D*` di
verdict81.out, emesse dallo script con formula+sorgenti (A-BG26).
⚠️ Nota storica: l'header v1 dichiarava «nessuna trascrizione a mano» —
FALSO alla lettera (Gregg, Concilio WP-83): le percentuali erano trascritte.
Da v2 ogni cifra del documento è vincolata al corpus committato dal gate
`gate-measure-cifre.sh` (KG-83-3). Cifre = churn LORDO, upper bound
(gross=1 in-band).

## Identità (ENFORCED dal driver su OGNI run)

- git tree **3f32c16** == `git=` del feature-matrix.log; SESTETTO: union
  9f9f8d92ff969729 · census **12a8777c8c38fdc4** · census-axum-only 13c429be
  · axum-only e2183043 · default cee8c63e · **mem-census d5bba760069639e0**
  (KS-AH-82-4: riga matrix + lane CI same-commit).
- **driver_sha=436b453ef0980c03** (A-AH21: measure78.sh+gate-feature-matrix
  fingerprint) nell'header di OGNI run e in coda a ogni raw `.log`; porcelain
  esteso agli script harness — la campagna abortita a metà dal proprio
  tripwire (verdict81.sh scritto mid-campaign ⇒ 13 run rifiutate) è la PROVA
  che il check morde; i raw misti sono stati rimossi DICHIARANDOLO
  (commit 3f32c16) e l'intera campagna è stata rifatta a UN rev.
  ⚠️ **Retro-annotazione A-AH27 (Concilio WP-83)**: i 13+13 raw VOID furono
  `rm`-ati SENZA quarantena — mai committati, quindi il claim «13 rifiutate»
  NON è ri-verificabile dal repo (testimoniale; superstiti verificabili:
  26/26 summary a git uniforme, driver_sha uniforme — ricomputo Gregg).
  Da S-82.0 vige la policy quarantena: mai `rm` di raw, anche VOID —
  `evidence/void/<stamp>/` + manifest committato (KS-AH-83-2), e le run
  VOIDate si CONTANO nell'header di campagna (KH83-2).
- Equivalenza sorgenti tra i commit harness-only della sessione: COMANDATA
  (`git diff --stat <rev1> <rev2> -- crates/` vuoto, KH82-2) — verificata
  empiricamente dagli hash sestetto IDENTICI su 552e6fb→2dc11eb→3f32c16.
- Baseline pre-leva: `wp80-harness/MEASURE80_RESULTS.md` (git 6910767,
  census 5c9c6eec). Il confronto è tra campagne, non build-adiacente
  stessa-sera per il churn: i contatori sono DETERMINISTICI su entrambe le
  campagne (spread 0,00% osservato IN-campagna qui, non ereditato —
  A-SK18/KS-SK-82-4/KG-82-2).

## Protocollo

measure81-campaign.sh = R=3 × (2 arm × 4 fixture) + idle60 × 2 arm, 26 run,
tutte ENFORCE (righe==110, depth≤1, inflight≤1, a3_trip==0, W=1, boot-probe
fuori canale, pin idle ==4 righe con self-test A-SK14). rc=0, zero FAIL.

## Verdetto churn (da verdict81.out v2 — TUTTE le predizioni §10 PASS su MEDIE R=3)

Tabella = medie R=3 con spread per-campo (A-SK20/A-BG28; md5 r1==r2==r3
BYTE-IDENTICI su tutte le 8 coppie, sezione IDENT di verdict81.out — il
determinismo è pieno, quindi media==r1). Byte accanto alle call (A-BG27).

| arm | fixture | a_calls m | spr% | a_bytes | a1 c/B | a2 c/B | a3 c/B | b c/B | resid c/B | retain |
|---|---|---|---|---|---|---|---|---|---|---|
| census | hello | 2 | 0.00 | 196 | 0/0 | 2/196 | 0/0 | 731/95659 | 44.1/43463 | 1 |
| census | include_gate | 2 | 0.00 | 210 | 0/0 | 2/210 | 0/0 | 767/98333 | 43.1/43896 | 3 |
| census | include_heavy | 2 | 0.00 | 212 | 0/0 | 2/212 | 0/0 | 27982/1867510 | 44.1/45156 | 6 |
| census | bare | 2 | 0.00 | 194 | 0/0 | 2/194 | 0/0 | 722/95390 | 41.1/43778 | 1 |
| censuscli | hello | 1140 | 0.00 | 126238 | 0 | — | 0 | — | — | — |
| censuscli | include_gate | 1176 | 0.00 | 128947 | 0 | — | 0 | — | — | — |
| censuscli | include_heavy | 28405 | 0.00 | 1899697 | 0 | — | 0 | — | — | — |
| censuscli | bare | 1132 | 0.00 | 126061 | 0 | — | 0 | — | — | — |

- **hello a_calls su HIT: 2 call/req** (baseline 80.476 → **−99,9975%**
  [derivata: D1 verdict81.out]; KS-AH-80-4 v2 «≥90%» superata di 3 ordini;
  puntuale <4.000 ✓). **Il floor 2 è CONDIZIONALE (A-BB27)**: vale per path
  canonico <384 B (buffer CString di stack in std) ed è CIECO al C-malloc di
  realpath (fuori GlobalAlloc); i due siti contati per nome: (1) PathBuf da
  `fs::canonicalize`, (2) Vec del path in `unit_key_for`. **Prova aritmetica
  (Bak/KB-83-4): a_bytes(HIT) == 2×len(path canonico)** — 196/210/212/194
  in tabella. **Il −99,9975% è della FINESTRA a, mai «costo per richiesta»
  (A-BB29)**: il `fs::read` del source vive nel resid.
- **Floor ex-post (A-BB23): a_calls(bare,HIT)=2 ≤ 200** — il floor ex-ante
  NON è bucato (KB-82-3); delta hello−bare su HIT = 0 call (i BYTE
  differiscono di 2: 196 vs 194, un char di filename — A-BG27) ✓; nessuna
  decomposizione dovuta (A-BB24: 2 ≪ 5×floor). Il probe
  (canonicalize+stat+hash) è quasi alloc-invisibile — che NON dice nulla
  sulla CPU (A-BB25/KB-82-4: bound su ALLOCAZIONI; la CPU si giudica solo
  con la slope due-N, residuo dichiarato sotto).
- a1==0 e a3==0 steady (niente lower_prelude, include ancora HIT); **le 2
  call residue vivono nel canale a2 — NOMINATO (A-BG27, D6 verdict81.out):
  a2=2 call / 196 B su OGNI riga steady** (la unit MAIN contiene il prelude
  compilato; la frase v1 «a2 assorbita» era smentita dai raw — Gregg).
- **b invariato**: hello 731 vs 730 (**+0,14%** [derivata: D2]),
  include_heavy 27.982 == baseline ESATTO (**+0,00%** [derivata: D5]; la
  cifra corretta da A-BG22 — il run non è toccato). **Pin req=1 (A-BB30)**:
  put+park vivono SOLO nella riga MISS req=1 — sezione REQ1 di
  verdict81.out (include_heavy req1 a_calls=128551) — fuori dalla finestra
  steady: «b invariato» si legge accanto a quel pin, non al posto suo.
- **resid invariato**: 44,1 call / 43.463 B == baseline ESATTO (media
  steady; nessun leak nel canale dichiarato, KG-81-3). ⚠️ **La media
  ingloba uno spike periodico NON ancora NOMINATO a req=11** (r1 hello:
  157 call / 166.771 B vs 43/42.199 tipici — Bak/KB-83-2): finché la
  sorgente non è identificata, ogni «resid invariato» di WP-82 è BLOCCATO;
  l'etichetta «steady» della finestra è quindi imprecisa (req=11 è ancora
  transiente — Gregg).
- retain_len = park-EVENTI: bare/hello 1 (main), include_gate 3 (2+main),
  include_heavy 6 (5+main) — pin A-DS8/F6 per NOME.
- cli arm (asimmetria superglobali DICHIARATA, A-DS10/KS-DS-81-3; confronti
  solo totale/a1/a3): hello 81.613 → **1.140** (**−98,60%** [derivata: D3]);
  include_heavy 109.415 → 28.405 (**−74,04%** [derivata: D4] — resta la
  quota include-compile run-side b, come da contratto: la leva rimuove
  a1+a2 del MAIN).

## Idle window (churn-only)

Post-leva: **drift idle = 0 call / 0 B su entrambi gli arm a 60s**
(finestra == self-cost esatto: axum 38/41.524, cli 49/9.639). KL-82-1: la
frase precisa è «0 allocazioni Rust-allocator, tutti i thread» — canali
CIECHI per costruzione: (i) dealloc (il contatore non le incrementa),
(ii) malloc FFI (zlib/gd/tidy fuori dal GlobalAlloc), (iii) arene/metadata
mimalloc (mmap diretto; senza purge-thread: con PURGE_DELAY=0 e zero
traffico non scatta nulla), (iv) page-dirtying di memoria già allocata.
La residenza idle esige il twin union + vmmap + floor (KL-81-3).

## Fixture e gate (tree 3f32c16, battery COMPLETA PASS)

F1-F13 + F-probe + F-oneshot(3 denti) + F5/F8b/F8c(contatori) TUTTE VERDI
(gate-lever-fixtures{,2}.sh); corpus 1418 per NOME IDENTICO + refl 290
IDENTICO + workspace 0 fail; run-gate union + census-twin (full-body vs
oracolo sul binario census = battery CON main-HIT) + concurrent +
worker-panic + stdout-tandem + capture-order + doc-purge + DR-1 +
lever-pins. F4: deviazione DICHIARATA (nessun path main-impuro sul tree —
Concilio WP-83 giudica). Battery-su-HIT col pin ESPLICITO main_hit
per-richiesta (KS-SK-82-3): il claim formale resta «path misto» finché il
pin non è contato — residuo dichiarato.

## RESIDUI DICHIARATI (A/B NON completo su questi assi — mai chiusi in silenzio)

1. **Footprint twin** (V2 N vs 2N con floor vmmap 0,1MB + peak W=num_cpus):
   NON misurato in S-81.0 — il verdetto memoria RESIDENTE della leva è
   APERTO (il churn sopra non lo sostituisce, KH80-3/KL-80-1).
2. **CPU slope due-N** (100/200 req + costo scan supersede A-DS9, con
   risoluzione ex-ante KL-82-3): NON misurata — ogni claim CPU della leva
   resta ADVISORY d'ufficio.
3. **Retained ×W**: walker+controllo positivo ESISTONO (KL-82-2 onorata),
   ma la cifra retained su run reale mem-census + budget ×W (KS-MS-82-2)
   NON è stata prodotta — budget NULLO finché non misurata.
4. **Fixture autoload-run** (KB-81-3/KB-82-5): assente — «il HIT salta a3»
   resta ADVISORY.
5. Tolleranze: spread 0,00% = determinismo, NON base statistica (A-BG25);
   le bande usate (±15/±5KB/±5%) sono concessioni dichiarate della
   baseline WP-80.
