# WP_SESSION_98 — S-98.0: una ipotesi caduta a tavolino con la sola misura, una spedita che morde il costo per opcode

**In una frase**: abbiamo misurato quanto costa davvero la "burocrazia" che
il motore paga a ogni singolo passo — scoprendo che eliminarla non farebbe
guadagnare quasi nulla, quindi quella strada è stata chiusa senza scrivere
codice — e abbiamo invece dato al motore una scorciatoia per la somma di
due numeri interi, che rende il caso più comune un sesto più veloce.

**Data**: 2026-08-05 mattina. **Ordine eseguito**: Concilio WP-99 (M1 →
decisione H-B1 → H-B2 asse → debiti bloccanti), con lo sbocco della
decisione utente 2026-08-05 (promozione flag-on = obiettivo nominato; le
leve si compongono).

## M1 — la misura del preambolo (zero codice VM) → H-B1 CADUTA A TAVOLINO

`wp98-harness/m1-preamble.out` (VERDICT). Tre strumenti convergenti:

- **noop da duecento milioni di iterazioni** (5 op/iter: LoadVar,
  CmpJmpConst, IncDecSlot, Pop, Jump):
  phpr 6,27 ns/op contro oracle 1,07 (3 op/iter) — anche gli op "economici"
  hanno corpi che dominano (4,5× il marginale del Sweep-noop).
- **ASM del loop head** (binario di parità, `m1-runloop-preamble.asm`):
  28 istruzioni dal back-edge al `br` del jump table; 11 rimovibili da
  H-B1 (guardia profondità, reload len/ptr da slot di SPILL, madd del
  frame); 17 restano (fetch op, bounds check ops[ip], store ip+1, jump
  table).
- **La sonda** (`m1-probe/`, arbitrata dal Concilio: «M1 è la sonda»):
  interprete sintetico col programma ESATTO del noop (OpS a 48 byte,
  FrameS a 176 byte), forma A = reload per-op, forma B = split-borrow
  H-B1. Build 1
  (registri): Δ = +0,01 ns/op (zero in banda). Build 2 (pressione, jump
  table come il vero): **B più LENTA di 0,53 ns/op** — su questo core OoO
  il reload L1 fuori dal cammino critico è GRATIS e la ristrutturazione
  può perfino perdere (layout/BTB).

**Predizione P scritta PRIMA di ogni codice**: D = 0,0 ns/op (banda
[−0,53, +0,55], tetto statico anti-hiding 1,4×11/28); P = 0% [0–6,7%]
< 10% ⇒ **H-B1 cade a tavolino** (KS-GR-99-1). Il tetto −17% di Bak/Gregg
era il CEILING; la parte rimovibile del marginale è nascosta dall'OoO.

## H-B2 — Binary Add int-int all'emissione → CONFERMATA E SPEDITA

`wp98-harness/hb2-addspec.out` (criterio PRE-REGISTRATO prima del codice:
cade se D_add < 0,7 ns/occorrenza; nullo se < 3× spread).

Forma (KS-ST-99-3 rispettata, −30,7% protetto): `Op::BinaryAdd` senza
payload, emesso al posto di `Binary(Add)` SOLO quando il pass registro è
OFF (il modo è già nella chiave della unit-cache); handler = pop rhs,
guardia tag (Long,Long) → checked_add con promozione overflow→Double
identica a `binary_fast`, scrittura in-place sulla cima della pila; MISS →
fallback INTEGRALE a `binary_value_ab` (stessi diags/overload/errori per
costruzione). Sotto `PHPR_REG_LOWER` l'emissione resta bit-identica
(census flag-on 1.100.019 invariato, zero BinaryAdd nel dump).

**Verdetto (coppia stessa-sera, macchina quieta, R=5)**: giudice `add.php`
(3 occ/iter × 50M): 5,63 → 4,72 s netti = **−16,2%**, **D_add = 6,07
ns/occorrenza = 8,7× la soglia** (guardia CONTATA nel tempo). Collaterale
`arith`: banda [−3, −5]% (nominale −5,2%, sotto 3× spread → banda, non
claim). Deriva inter-build 0,5% (noop, zero Add). Misure RIPRODOTTE sul
binario spedito finale.

**Gate di spedizione**: batteria 1723/0 · corpus Zend flag-OFF **1418 per
NOME identico** alla lista tracked wp82 (catena wp82→ha2→ha1 verificata;
`wp98-harness/evidence/corpus-hb2-flagoff.fails`) · corpus **flag-ON 1418
per NOME identico sullo stesso albero** (M4/KS-KL-99-1;
`corpus-hb2-flagon.fails`) · census: totali INVARIATI (1600020/1900020),
BinaryAdd 3/iter in add e 1/iter in arith (`Swap→BinaryAdd` = coda del
`+=`) · smoke semantico (overflow, coercizioni, union array) identico
all'oracle.

## Debiti del Concilio WP-99 saldati in sessione

- **B1+M3**: `crates/php-cli/tests/reg_lower_funnel.rs` — test
  d'integrazione al funnel VERO (spawna il binario, `PHPR_REG_LOWER=1` +
  `PHPR_DUMP_OPS=1`) che PRETENDE BinarySC/CmpJmpSC/BinaryDst nel dump del
  `{main}` (il controllo positivo che mancava sui top-level). PASS al
  primo run.
- **M5**: il test flag-off ora FALLISCE rumorosamente se l'ambiente
  esporta `PHPR_REG_LOWER` (era uno skip silenzioso).
- La batteria ha MORSO davvero in sessione: `stage2v3_rewrites_hot_windows`
  è fallito alla prima corsa (compila senza flag → emissione BinaryAdd →
  finestra `Binary(Add)` non matchava): è il buco di pipeline RC-1 di
  Hejlsberg visto dal lato opposto. Chiuso con `bin_op_of` nel matcher
  (BinaryAdd ≡ Binary(Add)).

## NON fatti (dichiarati)

- **Parità server** (le suite restapi e option per NOME sotto env -i):
  NON eseguita — l'oggetto è rimasto CLI (la clausola di Pedersen la fa
  rientrare in ordine al primo uso del server). php-server è stato
  ricostruito con H-A2+H-B2: **pin 365f4d4069513de3, parità MAI
  verificata** (eredita KS-PE-99-1: VOID ogni uso/misura del server prima
  del collaudo). PRIMO DEBITO di S-99.
- Nessuna misura full/media WordPress (l'emissione flag-off È cambiata ⇒
  il collaudo di parità WordPress È DOVUTO alla prossima occasione utile,
  regola n.2 — da mettere in ordine S-99 col launcher backup/wipe/restore).
- Coppia peak: rinviata al collaudo WP (backlog per NOME, invariato).
- Delibera manifest 94/95 e il resto del BACKLOG per NOME del Concilio
  WP-99: non toccati.

## ⭐ Lezioni

- ⭐⭐ **Una predizione misurata può chiudere un'ipotesi a costo zero**: la
  sonda (60 righe fuori dal VM) ha refutato H-B1 meglio di quanto avrebbe
  fatto scriverla — e in direzione OPPOSTA al tetto del concilio: le
  istruzioni di plumbing L1-resident fuori dal cammino critico sono
  GRATIS su un core OoO largo. Corollario: mai derivare un criterio dal
  CONTEGGIO di istruzioni senza chiedersi se stanno sul cammino critico.
- ⭐⭐ **Il plumbing di CHIAMATA invece si paga**: 6 ns/occorrenza per
  call + marshalling di due Zval per valore + pop/push contro una
  scrittura in-place. Il fattore ~8 del costo per opcode vive nei CORPI,
  e dentro i corpi nel plumbing generico — la specializzazione per tipo
  è la leva giusta e COMPONE (flag-off oggi; le forme registro flag-on
  hanno lo stesso plumbing da togliere).
- ⭐⭐ **La forgia che morde si è pagata due volte in un giorno**: il test
  del pass ha colto il disallineamento di emissione alla prima corsa; e
  il controllo positivo del census (arith_small = 1900020 pinnato) ha
  ri-collaudato lo strumento prima di usarlo.
- ⭐ **Due recidive di lezioni note, entrambe prese dai denti**: un `|
  tail` ha mascherato l'exit della batteria (2 FAIL letti dal file di
  log); un `awk '{print $1}'` ha troncato i path allo spazio di "Extreme
  Pro" (il diff urlava 1418≠1418). E una NUOVA: una serie di misure è
  girata con una build concorrente in background (bimodale 4,8/6,5) —
  SCARTATA e rifatta; mai misurare mentre compila qualcosa.
- ⭐ **Un check di processo sbagliato ha ucciso un corpus al 90%**:
  `ps aux | grep` non mostrava il runner (output largo/travisato) mentre
  `pgrep -fl` lo vedeva; ho killato il padre sulla falsa evidenza.
  Regola: prima di uccidere, DUE conferme indipendenti che il processo
  sia davvero morto/appeso (pgrep + log che non cresce + CPU).

## Stato binari e processi

- phpr parità: **4e268c3f61e6573d** (HEAD 23d9cce; stash additivo
  `phpr-s98-hb2`; il bin è rilinkato dal test d'integrazione — sorgente
  identico al 39e0a359 intermedio). Precedente 0dd98ebbb7eb2d96 resta in
  `phpr-s97-ha1`.
- php-server: **365f4d4069513de3** (H-A2+H-B2 dentro, parità NON
  verificata — debito S-99). Precedente 832568a72b925dd1 (anch'esso mai
  collaudato) in `php-server-s97`.
- Census: `phpr-opcensus-target` ricostruito con BinaryAdd (controllo
  positivo su arith_small PASS prima dell'uso).
- Nessun processo orfano; uploads non toccati (nessun full run).
