# design89 — delibere S-89.0 p8 (Concilio WP-90): ritiro KL-85-2 + A-BB60 nested-guard v2 + A-PP49 path-in-band

Ordine Concilio WP-90 §Sintesi p8. NESSUNA riga di runtime cambia con
questo documento: A-BB60 e A-PP49 sono DESIGN (attuazione = prima
sessione che ne consuma l'esito); il ritiro KL-85-2 è una delibera di
registro con effetto immediato sui gate (già attuato in
gate-measure-cifre A-SK53-bis).

## REGISTRO BANDE — KL-85-2 RITIRATA (KB-90-2, effetto immediato)

**Delibera**: la banda KL-85-2 (3.605.572 B ±5% per-worker) è RITIRATA
come costante cross-protocollo. Fondazione (Bak, Concilio WP-90 Q2):
nasce a W=10 steady su UN fixture con altra metrica di forma; l'IC 2σ di
b misurato in m88 (21,20M ± 1,17M, se(b)=586.270 B) non la sfiora; b è
stabile fra i regimi (~20-25 MiB/worker W1..16) — non è una banda di
QUESTO protocollo e non è una costante universale.

**Stato a registro**:
- CITARLA cross-protocollo ⇒ claim VOID (KB-90-2, kill-switch attivo).
- Legale SOLO come bersaglio storico di una NAMED-DEVIATION già
  giudicata (MEASURE87/88) — allowlist per-forma in gate-measure-cifre
  (A-SK53-bis: `3.605.572B±5%`).
- Ri-derivazione ammessa SOLO per-protocollo, con fascia δ ex-ante e
  b±2σ in-band (A-BB57/KB-90-1); mai riproposta tal quale
  (NON-riproporre WP-88, invariato).

## A-BB60 — nested-guard v2 (supera i tre buchi di A-BB54, KB-90-3)

**Problema (Bak WP-90 Q4)**: A-BB54 (design88) ha tre buchi: (i) la
regex `function\s+(\w+)` è cieca alle closure anonime (un piccolo con
una closure extra passa il subset senza essere annidato); (ii) i nomi
PHP sono case-insensitive (confronto senza normalizzazione = falsi
refuse/pass); (iii) il witness usa costanti di collaudo (996.838 B) come
oracolo — per coppie NUOVE il criterio è non-falsificabile.

**Design vincolante**:
1. **lowercase**: ogni nome estratto normalizzato `tolower` PRIMA del
   confronto di insiemi (funzioni, `Class::method` inclusi).
2. **closure-subset per HASH del corpo**: le closure anonime non hanno
   nome — si confrontano per sha256 del CORPO normalizzato (whitespace
   collassato); l'insieme dei body-hash di S deve essere ⊆ di quello di
   L. Una closure extra nel piccolo = refuse (KB-90-3: coppia con
   closure senza confronto dei corpi ⇒ NESTED non provato,
   scomposizione MODEL-GRADE).
3. **witness derivato dalla CALIBRAZIONE di L**: il Δfloor atteso non è
   una costante di collaudo ma si DERIVA a macchina dal raw di
   calibrazione di L nella STESSA campagna (floor_inc(L, cal) −
   floor_inc(S, cal)); ε scalato per classe-di-coppia (default:
   max(1.040 B, 0,5% del Δfloor atteso), ENTRAMBI nominati nel raw del
   guard).
4. Selftest che MORDE (invariato da A-BB54 + i tre casi nuovi: closure
   extra nel piccolo ⇒ refuse; nomi case-variant ⇒ pass; coppia nuova
   senza calibrazione di L nella campagna ⇒ refuse "witness underivable").

**Consumo**: ogni scomposizione additiva VERDICT-GRADE su coppie
annidate esige il guard v2 eseguito con esito in-band (KB-89-3
invariata).

## A-PP49 — `worker_dispatch` con path= in-band (KS-PP-90-3)

**Problema (Pedersen WP-90 Q2)**: la riga `tag=worker_dispatch thr=N
count=M arm=union` non porta QUALE fixture è andata su quale thread —
la composizione warm per-worker ("hello+pad per worker") poggia sul
round-robin da next_worker=0: assunzione vera ma non provata
(KS-PP-90-3: composizione warm citata senza path= ⇒ ASSUNZIONE).

**Design vincolante**:
1. La riga diventa `tag=worker_dispatch thr=N count=M arm=union
   paths=<p1>,<p2>,…` — l'elenco ORDINATO dei basename serviti dal
   thread (bounded: primi 8 + `+K` per il resto; il canale resta
   stderr NON-census, emissione fuori da ogni finestra temporizzata,
   WP-64).
2. `dispatch-union-guard.pl` (A-PP44, design88) si estende: oltre a
   thr-set esatto e Σcount==nreq, verifica la MAPPA fixture→thr attesa
   dove il protocollo la garantisce (warm 2+2: hello+pad per ciascun
   thr; sweep: un pad per thr).
3. In alternativa equivalente (stessa forza): cross-check in VORD della
   coppia (`tag=unitcache_main_entry` porta già il path per-thread nel
   census) — se si sceglie questa lane, la delibera è soddisfatta
   SENZA riga nuova ma il verdetto deve NOMINARE il cross-check
   (mai assumere il round-robin in silenzio).

**Consumo**: ogni claim di composizione warm per-worker in grado
VERDICT esige path= (o il cross-check nominato) in-band.
