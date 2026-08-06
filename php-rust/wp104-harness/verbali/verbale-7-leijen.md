# Verbale sedia 7 — Leijen (allocatore mimalloc, footprint fisico) — Concilio WP-104

## VERDETTO: REGGE CON RISERVE (non benedico)

S-102 ha eseguito A-LE-103-1 alla lettera (GA_*_N eventi, mai più stats a
pagine) e RC-LE-103-1 ha morso alla prima categoria nuova: il churn
bilanciato di calls era invisibile alle pagine. Metodo giusto. Ma la CIFRA
«2 alloc/chiamata, ~35 B» non è ancora verdict-grade — vedi RC-LE-104-1.

## (a) Gamba alloc — A-LE-103-1 soddisfatta? Sì come ESISTENZA, no come cifra

1. **Realloc gonfia i conteggi**: `CountingMi::realloc` (main.rs:62-69) nota
   `galloc_note(new_size)` + `gfree_note(old)` anche quando mimalloc
   riusa il blocco in-place (crescita nella stessa size-class). Una Vec che
   cresce a ogni chiamata apparirebbe come «1 alloc + 1 free» senza traffico
   allocatore reale. Le «2 alloc/chiamata» potrebbero essere 2 alloc fresche,
   oppure 1 fresca + 1 realloc: specie diverse, leve diverse.
2. **I ~35 B includono l'avvio**: 711934806/20196691 = 35,25 B è la media
   LORDA. Al netto (Δbytes 639,98 M / Δeventi 19,999 M) = **32,0 B/alloc**.
   Cifra piccola ma il fingerprint per size-class si fa sul netto.
3. **Costante d'avvio PRESA IN PRESTITO da prop**: calls non ha il suo
   due-punti (prop_small↔prop 300:1 c'è; calls_small NO). Il ~197k di prop
   regge (±50k non sposta 2/iter), ma la disciplina dice: ogni giudice il
   SUO controllo di linearità.
4. **Gap empty.php**: innocuo per i verdetti dati (il due-punti cancella
   l'avvio), ma il dump atexit agganciato alla prima nota zval è un
   accoppiamento fragile — promuovere il backlog.

## (b) Banda rumore: credibile per la decisione bisect, NON universale

Spread max–min di R=5 è uno stimatore SOTTOSTIMATO del range vero (il max
di 5 campioni non è il max della popolazione; il peak è coda unilaterale,
come il criterio stesso dichiara). Rischio: falsa «CRESCITA REALE» quando
|Δ| supera di poco una banda troppo stretta. Il criterio pre-registrato è
buono (mediana, mai media; max dei due spread; tripwire segni opposti;
KS-LE-103-3 fail-safe) ma ha DUE buchi: la zona marginale e la riusabilità.

## (c) A/B in volo nella finestra pomeridiana

Il peak footprint per-processo è relativamente robusto al carico ALTRUI
(conta dirty+compressed del processo), ma la contesa CPU stira il wall e
sposta i tempi di purge/teardown ⇒ gli SPREAD di fase 2 si gonfiano. ABAB
protegge il Δ delle mediane (contaminazione simmetrica in media) e il
tripwire coppie-opposte pesca i burst. Il buco è ASIMMETRICO: banda gonfia
⇒ verdetto RUMORE più facile ⇒ la voce si chiude con POTENZA degradata.
Fail-safe verso il bisect, NON verso la chiusura.

## (d) Le 2 alloc/chiamata: candidati e strumento

Candidati (~32 B netti, coppia es. 48+16 o 32+32): (1) Vec args spillata
(2 Zval); (2) locals/Frame heap (Vec slot o Box frame); (3) Rc fresca di un
valore di ritorno; (4) buffer diag/String per-call. NON HashMap (troppo
grande). Strumento S-103: **niente backtrace** — (i) istogramma per
size-class in CountingMi (due picchi a ~10M ciascuno = fingerprint); (ii)
tag di sito thread-local settato nei costruttori del call-path, letto dal
wrapper; (iii) chiusura contabile: somma per-sito = 2/iter, residuo non
taggato ≡ 0.

## Emendamenti

- **A-LE-104-1**: disaggregare realloc in CountingMi (GA_REALLOC_N + delta
  byte) PRIMA di ogni attribuzione H-D.
- **A-LE-104-2**: due-punti calls_small↔calls (linearità propria, non
  prestata da prop).
- **A-LE-104-3**: istogramma size-class + tag per-sito thread-local, con
  chiusura contabile residuo=0.
- **A-LE-104-4**: dump atexit anche senza note zval (empty.php) — da
  backlog a igiene S-103.
- **A-LE-104-5**: zona marginale A/B — se |Δ| ∈ (banda, 2×banda] con R=5,
  si ripete con R≥7 prima di dichiarare crescita reale.

## Kill-switch

- **KS-LE-104-1**: nessuna leva H-D finché la somma per-sito non chiude a
  2/iter con realloc disaggregato.
- **KS-LE-104-2**: verdetto RUMORE con banda fase-2 > 1,5× la banda quieta
  (34,64) NON chiude la voce — si ripete in finestra quieta.
- **KS-LE-104-3**: la banda R=5 vale SOLO per la decisione bisect di questo
  A/B; mai riusarla come banda universale senza rimisura stessa-sera.

## Refutazione capitale

**RC-LE-104-1**: «~2 alloc + ~2 free/chiamata» è CENSUS-grade come
ESISTENZA (>0, RC-LE-103-1 vendicata), ma la cifra «2» e i «35 B» NON sono
verdict-grade: realloc conta doppio anche in-place, la media lorda include
l'avvio (netto = 32,0 B), e calls non ha il suo due-punti. Ogni banda o
pavimento per leve H-D derivata oggi da «2 siti» sarebbe prematura.
