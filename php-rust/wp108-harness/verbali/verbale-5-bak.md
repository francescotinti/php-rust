# Verbale Sedia 5 — BAK (V8/HotSpot: alloc-rate, code-cache, path caldi) — Concilio WP-108

**VERDETTO**: promozione H-A1 su arith CONFERMATA (criterio pre-registrato,
co-primario strutturale, ABAB 5/5, admission dichiarata: pulita). La riga
di scoreboard «prop ↓ −0,9 (H-A1)» è REFUTATA come attribuzione. Nessuna
refutazione capitale. Ordine S-107 approvato con emendamenti.

## Refutazioni

- **R-BA-108-1** — Il Δ prop NON è attribuito. Conti: 11,5×0,42−4,45 ≈
  0,38 s / 30M = **12,7 ns/iter** tra-sere, contro **7,0 ns/iter**
  misurati in ABAB su arith per la STESSA finestra (stessa forma: −2
  dispatch, −4 transiti di pila). L'eccesso ~5,7 ns/iter — sopra lo
  spread prop 0,09 s — è ORFANO: confuso con igiene D-12 (contatore
  census su calls.rs), reindex N_OPS=187, churn/relink, deriva del
  denominatore oracle, tra-sere. Il dump firma la DIREZIONE (BinarySTDst
  vive in `$s += $o->x`): «beneficiario nominato» sì, magnitudine no.
  Le KS-HE/GR-107 sui contrasti valgono identiche qui.
- **R-BA-108-2** — Il gate di chiusura pre-registrato recita «ogni altro
  rapporto fermo entro ±0,4 o la leva NON si promuove»; prop = −0,9. La
  lettera è stata attraversata SENZA emendamento dichiarato (S-106-R-3
  applicata a reg_lower_funnel e census, NON a questa clausola). Non
  capitale — movimento migliorativo con meccanismo dal dump — ma la
  sanatoria va scritta, non sottintesa come «scoperta».

## Emendamenti

- **A-BA-108-1** — Emendare DICHIARATAMENTE la clausola ±0,4: «fermo
  entro ±0,4 OPPURE movimento migliorativo con beneficiario nominato dal
  dump, registrato come direzione firmata / magnitudine NON attribuita».
- **A-BA-108-2** — Dente economico S-107 (~10′): retro-A/B prop ABAB R=5
  coi due stash esistenti (phpr-s105 d4d0fa52 vs phpr-s106 eb555106) —
  converte la direzione in magnitudine attribuita o smaschera il
  co-fattore. Regola prospettica: se l'istruttoria dichiara la finestra
  TRASVERSALE, l'A/B misura ANCHE le categorie beneficiarie nominate,
  nello stesso ABAB.
- **A-BA-108-3** — Budget di FORME: R-2 conta i byte, non le forme.
  BinarySTDst è la settima forma Binary-like in un dispatch a 187
  varianti: istituire al pin il LEDGER census per-forma (hit-count su
  giudici+corpus; forme ~0 hit flaggate fredde) con sveglia dichiarata
  (es. +8 forme da S-104 O run_loop +4 KB) ⇒ istruttoria cold-partition/
  outlining (A-HE-106-5/A-HE-106-4). Senza L1I «la icache paga» resta
  ipotesi: il ledger è il surrogato a eventi, non a pagine.

## Punti minori

- **arr −0,3**: lettura «in banda» GIUSTA (±0,4), ma registrare il
  SEGNO: 4,5→4,2 monotono; drift stesso verso su N≥3 sere oltre metà
  banda = trend, non rumore ⇒ istruttoria. Una banda difende dal rumore,
  non dalla deriva.
- **D-18**: esecuzione COERENTE (contatori non eseguiti ⇒ arith,
  verbalizzata, nessuna terza via). Nulla da refutare.

## KS

- **KS-BA-108-1** — «Beneficiario nominato» firma la direzione, mai la
  magnitudine: senza A/B per-categoria il Δ resta non attribuito.
- **KS-BA-108-2** — Il text-budget in byte non è un budget di forme: ogni
  forma monomorfa nuova entra nel fan-out del dispatch senza ledger di
  frequenza.
- **KS-BA-108-3** — Le finestre hanno un ORIZZONTE: phpr ~99 ns/iter su
  9 op (~11 ns/op) contro 8,6 ns/iter TOTALI dell'oracle — il collo è il
  costo per-op (ciclo di vita Zval), non il conteggio; l'elisione di
  dispatch compra ~0,5–1 punto/leva a costo marginale crescente. X≤3
  esige A-ZV2: le finestre non devono spiazzarla indefinitamente.

## Giudizio ordine S-107

Sequenza 1–5 APPROVATA. Denti (incl. hit/miss D-5) PRIMA della nomina
calls: giusto — tornare a calls senza contatore violerebbe D-5. Leva 3:
arith NON è dispatch-floor; ordine consigliato (i) **IncDecSlot+Pop**
(gemello di H-A1: discard-fold, diagnostic-safe, zero biforcazione),
(ii) **Sweep per-iter DOPO** istruttoria sul perché sta nel corpo —
tocca il ciclo di vita dei temporanei, rischio semantico superiore.
Integrare: A-BA-108-2 nei denti del punto 1 o nell'igiene 5; clausola
beneficiari trasversali nel criterio della leva; ledger forme al primo
pin S-107. §3.15 fuori perimetro mio: nessuna obiezione.
