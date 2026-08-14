# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full ON-ONLY = 1,767–1,781** (S-137 @ pin s136) —
**PIN RUOTATO s138 ⇒ COPPIA DOVUTA in S-139** (con rifondazione banda ON-config
N≥5, az.rev. S-137 #4 assorbita) · ultima leva SPEDITA **S-138 (FD1-ext RMW:
dimrmw −54%, diminc −58%)** · sessioni-senza-leva = 0 · incidenti: storici 14
(n.14 sed-copia, morso dal NO-CLOBBER).

## Scoreboard (pin s138 fa17dabd9eaa4bcb + server a9aded4516e6d46c; micro dal gate promo S-138)

**arith 5,6 · prop 5,6 · calls 4,8 · str 4,3 · arr 3,2 · re 2,6** · oggetti
(dal gate s136, NON rimisurate a s138): objdatains 5,9 (963,3) · objchurn 6,7 ·
objalloc 6,4 · objdropdef 7,5 · objallocni 7,9 · objmap 11,7 design-bound
(round-trip GC; cura = piano gc-cycle-collector) · **RMW nuove voci: m-dimrmw
320→147 ns/iter · m-diminc 270→113 (A/B S-138, conferma post-pin in banda)** ·
MAPPA (net): WP 1,77 on-only (@s136) ≈ compoff 1,86–1,89 ≪ hf 2,55 ≪ hk 4,3 ≪
dbal 8,36–8,45 ≈ ORM 8,43–8,56 (@s134) · corpus **1414** ×2 · **eccedenza FD1
CHIUSA (S-138: cross-giudice; D_mdw 63,3 vs UB 69,6 in banda) — dim-write
SBLOCCATO**.

## §S-139 — ordine

1. **COPPIA WP full a pin s138** (obbligo pin nuovo; server s138 pinnato):
   criterio pair PRE-REGISTRATO che (a) fonda la **banda ON-config da N≥5 gambe
   ON** (pensiona lo 0,041 S-134 — az.rev. S-137 #4); (b) firma per gamba come
   S-137. Atteso: RMW non muove WP (statement rara) — verdetto ai numeri.
2. **Az.rev. S-138** (`wp138-harness/revisione.md`): vincolanti al primo criterio.
3. **Leva successiva dai numeri**: candidata naturale = FieldRead/dim-read (IC
   sui cammini di lettura, famiglia sbloccata) previa modello; o objdatains residuo.
APPARATO: **CI runner.lock NON tiene (5 istanze concorrenti in S-138 = causa
batteria-FAIL del feed): fixare il mutex (flock) o spegnere l'auto-spawn**;
coda in smaltimento con 1 runner post-sessione.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

FieldRead/dim-read IC (famiglia FD1, sbloccata) · divergenze RMW del cammino
PIENO trovate da fixtures-rmw (undefined-key su RMW · float-key deprecation ·
str-increment deprecation · notice overloaded-property: per NOME, pre-esistenti)
· objdatains residuo (leaf 18,9 · plumbing 17,6) · objmap valore-oggetto 43,4 →
piano GC · dispatch 36,3 (solo dopo modello) · walk_driver 37,2 · cammini non
cacheabili (readonly, mangled, `__set`, slot assente, child Ref) · famiglia
locale 170 ns · evalcls 316,9× · refl 42,4× · re +2,00 alloc/iter · 14%
AssignPath · §3.13 · §3.12-i · §3.14 · §3.21 · get_gc · drift TODO.md ·
latin1-cliff · media bordo 2,529 (osservazione).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing · threaded-dispatch · PGO sui giudici · verdetti su
build emendata senza ri-banda · pin/stash senza collaudo-nell'atto · contenitori
sul call path · differenze tra A/B distinti come cifra · componenti prezzate ·
magnitudine ripartita senza A/B proprio · «icache» NON-premessa · pre-filtro che
tassa i freddi · guardie non-bersaglio BILATERALI · denominatori a memoria ·
output di run nel repo · rc di gate da pipe · tee/log pre-mkdir (recidiva ×2
S-138!) · admission sul dump intero · xctrace senza guardie disco · run pesanti
come task · edit coi build in volo · promozione sotto banda · gate a soglia
fissa senza banda · corpus-gate solo-nomi · strumentazione nei sorgenti del pin
· leve micro senza banda v2 · alloc-removal senza modello del costo SOSTITUTIVO
· probe senza riferimento vivo · ordine FISSO di misura · delta tra census di
epoche diverse senza datare i raw · verdetti da script non committati · SSO
inline · inline-array init+drain args · claim di ASSENZA oltre la risoluzione ·
smoke con fam-min > R · notti su PhpStr-full · guardie su giudici diversi dalle
loro bande · misure con LSP in volo · F2 keys-scratch · quiescenza nello stesso
comando del lancio · pattern quiescenza nell'argv del lancio (recidiva S-138:
shell di lancio VIVA) · output TRACKED mossi da orchestratori · rumore-soglia =
range PIENO · guardia senza banda propria · percentuale tra segmenti NON
annidati senza zero-check · sed di copia su righe ESEGUIBILI (n.14: solo Edit
mirato + manifest) · eccedenza sopra la parte modellata senza sonda · banda di
guardia da strumento DIVERSO senza drop-1 · lock di finestra con trap EXIT
altrui · quiescenza a 2 campioni senza STREAK · abort multi-gamba senza retry ·
probe che rompe l'inlining · leva GC note-time (WP-21) · `git add` di directory
harness · **identità tra GIUDICI diversi come gate (cross-giudice S-138)** ·
**gemello di relink che eredita il verdetto A/B senza conferma post-pin**.

**Riscritto**: 2026-08-14 (chiusura S-138). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-139: pin phpr **s138 fa17dabd**9eaa4bcb + server **s138 a9aded45**16e6d46c
(stash in phpr-old-target/release/) · MySQL wp8 con l'elenco · uploads sotto
guardia · corpus 1414 ×2 · CI: coda in smaltimento (1 runner legittimo
post-lock; runner.lock DA FIXARE) · lock misura `/private/tmp/phpr-measure.lock`
da CREARE a ogni finestra (oggi RIMOSSO a fine sessione) · disco Data ≥10G ·
pgrep rust-analyzer PRIMA di ogni misura · lettura: REGOLE.md → QUI →
WP_SESSION_138 → wp138-harness/revisione.md → PERF_MAP.md.
