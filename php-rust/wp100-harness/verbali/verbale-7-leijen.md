# Verbale Sedia 7 — Leijen (allocatore mimalloc, footprint fisico) — Concilio WP-100

**Oggetto**: report S-98.0 + programma S-99.0. **Mandato**: refutare.

## VERDETTO: APPROVATO CON EMENDAMENTI VINCOLANTI — due refutazioni capitali sul mio perimetro.

## Refutazioni capitali

**R1 (capitale) — La «coppia peak al collaudo WP» promessa da S-99 è una forgia
senza strumento.** L'ordine S-99 punto 1 la contiene DAVVERO («coppia peak nello
stesso giro (chiude il debito Leijen)») ma non nomina né `/usr/bin/time -l` né
`vmmap` Physical footprint, né l'env mimalloc: senza lo stesso
`MIMALLOC_PURGE_DELAY=0` e lo stesso strumento della coppia WP-94, il confronto
col riferimento **1901,11 MiB** non è un confronto (RSS di `ps` mente per
regola permanente). Una promessa di misura senza strumento nominato è la classe
di forgia che fallisce in silenzio (lezione WP-96). Aggravante: la riga
⏱ FONDAMENTALI di NEXT_SESSION traccia SOLO full/media **CPU** — il footprint
non è misurato da WP-90/94 e il debito non ha contatore, quindi non scade mai;
«roadmap footprint ferma» + nessun contatore = deriva invisibile mentre due
sessioni (S-97.1, S-98.0) hanno cambiato emissione e binari.

**R2 (capitale) — Il rollout S-99 punto 3 spende un budget che nessuno conta.**
Il tag di dispatch è un byte (`ldrb` nell'ASM di M1) ⇒ tetto duro 256; N_OPS è
salito a **186** (test census aggiornato in S-98.0) e il rollout per famiglia
(Add/Sub/Mul/compare × stack/BinarySS/SC/Dst) è MOLTIPLICATIVO, ma il «dente
N_OPS<256» giace in BACKLOG, cioè DOPO le occorrenze che lo consumano. Inoltre
ogni variante è un corpo caldo in più nel run_loop (lezione WP-33: branch
mai-preso = +2,9%; M1: già a 185 bracci gli indirizzi di frames.len/ptr
SPILLANO) — il canary noop inter-build (0,5% misurato per la prima occorrenza)
non è pre-registrato come obbligo per le successive.

## Refutazioni minori (forma, non sostanza)

**r3** — `hb2-addspec.out` non dichiara né misura l'impatto footprint di H-B2.
Per MIA ispezione di run.rs:944-975 il hit-path è alloc-neutro (pop di `Vec`
+ scrittura in-place; `Zval::Long` senza heap) e il high-water dello stack è
identico al generico (pop+pop+push ≡ pop+write-in-place, picco uguale, mai
superiore); il MISS ricalca il funnel. Ma è ispezione del revisore, non claim
del report: un claim assente non è un claim vero.

**r4** — La sonda M1 pinna la TAGLIA (OpS=48B, FrameS=176B) e la catena critica
(madd frame → ldr ip → madd op), ma non gli OFFSET di ip/func del `Frame` vero
(repr(Rust) non li garantisce) né l'allineamento del frame sulle linee da 64B
(176B ne attraversa 3). La conclusione «reload L1-resident gratis» è robusta
per dati L1, quindi non cade; ma la fedeltà venduta è di taglia, non di layout.

**r5** — `reg_lower::enabled()` sta nella chiave della unit-cache: corretto per
la parità («nessun ibrido»), ma se mai il flag diventasse togglabile nel
processo la cache terrebbe DUE emissioni residenti = footprint doppio del
codice compilato. Oggi è env process-level: va pinnato che resti tale.

## Emendamenti

- **A-LE-100-1**: ordine S-99 punto 1 riscritto con lo strumento: coppia peak
  stessa-sera via `/usr/bin/time -l` (peak fisico; `vmmap` Physical footprint
  come spot-check; MAI RSS di ps), env mimalloc identico a WP-94, cifre accanto
  a 1901,11 MiB. La riga ⏱ FONDAMENTALI acquista il campo «ultima misura
  peak = WP-N (k sessioni fa)».
- **A-LE-100-2**: ogni .out di specializzazione dichiara il profilo di
  allocazione del hit/miss-path (per H-B2: retro-annotare «alloc-neutro per
  costruzione, high-water invariato»).
- **A-LE-100-3**: dente N_OPS≤255 promosso a GATE compile-time PRIMA del
  rollout punto 3, con budget di varianti nominato; canary noop inter-build
  pre-registrato per OGNI occorrenza nuova.
- **A-LE-100-4**: nella sonda m1-probe, controllo positivo `offset_of!` di
  ip/func contro il Frame vero prima di ogni riuso come strumento.

## Kill-switch

- **KS-LE-100-1**: VOID ogni confronto peak non prodotto da `/usr/bin/time -l`
  (o vmmap Physical footprint) stessa-sera sui due lati con env mimalloc
  identico al riferimento.
- **KS-LE-100-2**: VOID la spedizione di una nuova variante `Op` se porta
  N_OPS oltre 255 o se il canary noop devia oltre 3× lo spread.

— Leijen, Sedia 7, 2026-08-05
