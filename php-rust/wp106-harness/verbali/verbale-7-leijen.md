# Verbale sedia 7 — LEIJEN (allocatore, footprint, contabilità memoria) — Concilio WP-106 su S-104

## VERDETTO: CON EMENDAMENTI (nessun MI OPPONGO; nessuna refutazione capitale)

Il free-hist (GA_FHIST, memcensus.rs:1409-1423) è implementato come chiesto:
stessa bucketizzazione, realloc fuori da entrambe le famiglie, `fhist_note`
dentro `gfree_note`. Le 4 attese pre-registrate (b106d3a) sono confermate con
aritmetica verificata da me (19.900.003/19,9M; 636.799.976 B free vs
636.799.870 alloc: asimmetria 106 B su 637 MB ≈ rumore di fondo).

## R-LE-106-n (refutazioni)

**R-LE-106-1 — La «simmetria byte» esclude le TAGLIE, non i SITI.**
L'argomento del soffitto regge: media 32,0000 con supporto ≤32 (bucket
(16,32]) forza la massa al massimo — un inquinante da 24 B è tollerato solo
fino al quantum di stampa (~10³ eventi su 19,9M, ≤50 ppm: «esatto» va letto
come BOUND, non come zero). Quindi sì: ogni alternativa di taglia (24 B, mix)
è esclusa oltre i ppm. Ma l'enumerazione «ret_cell O args-Vec» era una LISTA
D'IPOTESI, mai un censimento dei siti: qualunque altro sito che allochi
esattamente 32 B (Box di struct 32 B, buffer cap-2 diverso) produce lo stesso
istogramma. Layout+misura inchiodano la FIRMA, non il NOME. I file S-104 lo
dicono già («il tag decide»): la refutazione colpisce solo la tentazione di
promuovere senza nome.

**R-LE-106-2 — La lettura R=7 ha SOSTITUITO l'estimatore a posteriori, ed
era decision-relevant.** La regola pre-registrata (§2, che ho co-firmato)
dice «|Δ mediane| vs banda 34,64»: Δ mediane = +37,94 SFONDA. La lettura
passa alla mediana dei Δ accoppiati (+22,47, sotto banda) citando il
regime-shift. L'estimatore accoppiato è quello GIUSTO per un design
accoppiato — l'errore era nel testo della regola, mio — ma lo scambio è
avvenuto dopo aver visto i numeri, e cambia l'esito (bisect sì/no). Il
verdetto sopravvive SOLO grazie al §3 STOP (mai un terzo full-peak, quindi
il bisect era comunque ineseguibile). In più: con IQR [15,7; 40,0] a cavallo
della banda, «sotto la banda» è overclaim del point-estimate — la dizione
onesta è «magnitudine INDETERMINATA». Direzione firmata (7/7, p=0,0078): regge.

**R-LE-106-3 — La forma «pool/freelist utente» è refutata a priori.** Il
fast path TL di mimalloc È una freelist segregata: un pool utente la
re-implementa aggiungendo un branch e superficie di leak, guadagno atteso ≈
il solo size-class lookup, sotto banda-layout. Non si misura nemmeno.

## A-LE-106-n (emendamenti)

- **A-LE-106-1 (risposta a b)**: il SiteTag pieno (residuo≡0) NON è più
  l'unico gate: lo scopo (nominare il sito) si compra con la **probe
  cap-bump** (~45′: args-Vec cap 2→4; attesa: massa (16,32]→(32,64] 1:1
  ESATTA, byte/chiamata 32→64) — intervento firmato, pre-registrato. SiteTag
  pieno → backlog, solo se la probe è ambigua. La garanzia «nessun canale
  nascosto» migra sul lato promozione (KS-LE-106-1).
- **A-LE-106-2 (forma della leva)**: prima forma **SmallVec inline-2** (o
  array in-frame): sposta la coppia malloc/free sullo stack, minima
  variazione semantica. Seconda forma, se il disasm mostra bloat: args-stack
  contiguo a watermark (disciplina LIFO dei frame). Pool: escluso (R-3).
  Obbligo del protocollo S-104: bl-count + taglia run_loop prima/dopo
  (run_loop è icache-bound — la leva non deve gonfiare codice).
- **A-LE-106-3 (attese di banda su calls, 7,6)**: pre-registrare Δ ∈ **[6,14]
  ns/iter** (coppia fast-path TL mimalloc); conversione in rapporto SOLO con
  N emesso dal run (KS-GR-105-2). Soglie di promozione: eredita lo schema
  H-C2 (Δ≥8 ⇒ R=5; [4,8) ⇒ R≥9; sotto max(banda-layout, rumore ~3) ⇒
  registra e chiudi il braccio).
- **A-LE-106-4**: il criterio della leva DEVE enumerare i siti di FUGA degli
  args (func_get_args, varargs conservati, by-ref, catture) con fixture per
  ciascuno, PRIMA dell'implementazione.
- **A-LE-106-5**: correggere a registro la dizione R=7 («indeterminata», non
  «sotto banda») ed esplicitare il bound ppm sul «32,0000 esatti».
- **A-LE-106-6 (backlog per NOME, sponsor questa sedia)**: `memory_get_usage`
  stub → cablarlo ai contatori census già esistenti (galloc/gfree): ogni
  fixture di leak in-script resta VACUA finché non si fa.

## KS-LE-106-n

- **KS-LE-106-1**: la promozione della leva args esige DUE co-primari: A/B
  timing E census post-leva con **alloc/chiamata 1,0000→0,0000** su calls.
  Vittoria di timing senza lo zero census = canale non nominato ⇒ reject.
- **KS-LE-106-2**: in ogni design accoppiato l'estimatore è pre-registrato
  come la mediana dei Δ ACCOPPIATI; cambiare estimatore dopo aver visto i
  dati = lettura VOID (generalizza R-LE-106-2).

## Priorità S-105 (questa sedia)

1. Probe cap-bump (≤45′) → nome del sito. 2. **LEVA args** (SmallVec
inline-2) con criterio pre-registrato (fughe, attese [6,14] ns, disasm,
KS-LE-106-1) → A/B. 3. Gate PIN + coppia WP (salda il debito S-103).
4. Backlog: design per-fase peak (A-LE-105-5), memory_get_usage vero,
SiteTag pieno solo se serve.
