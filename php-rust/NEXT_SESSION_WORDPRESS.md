# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **1,765–1,788 (S-142 @ s142; banda_ON unione 0,036)** ·
**⚖️ S-145: SONDA-B CHIUSA — KS-B4 SCATTATO (memcpy 69,5% ≥60%): B1/B2 NON si
aprono, filone conteggi (B3) TORNA AL CONCILIO** · **leve spedite: 1 (L-FR1
dim-read, pin s145)** · sessioni-senza-misura: 0 · **incidenti 16**
(nuovo n.16, rev. S-145 az.5: rc-da-pipe sulla build + braccio A su target
condivisa — morsi dai gate PRIMA del giudizio, contati comunque).

## Scoreboard (pin NUOVO s145 phpr a89faf32c62142f9 + server 4a9adc51a62b21ba)
**arith 5,5 · prop 5,5 · calls 4,8 · str 4,3 · arr 3,2 · re 2,5 · hintcall 7,3
(n.r.) · dimread 4,3 (NUOVA, era 6,0)** · oggetti (s136): objchurn 6,7 ·
objmap 11,7 · MAPPA (net): WP 1,78 ≈ compoff 1,9 ≪ hf 2,55 ≪ hk 4,3 ≪ dbal
8,2 ≈ ORM 8,6 · corpus **1414 ×2** · **REPERTI S-145: partizione churn
memcpy 69,5% / inc-dec 14,1% / nota 16,4% (prezzi per-movimento firmati:
scalar 2,88 · str/arr +0,97 · obj +0,49 · nota-cont 2,40 ns; pair a banda
8,71–8,80 / 11,57–11,79 ns) · riparse simmetrico: phpr idle=0 4/4, memops
FUORI / churn IN confermati · L-FR1: D=+16,7 ns/iter (−28%) su m-dimread.**

## §S-146 — ordine
1. **Coppia WP @ s145** (obbligo da pin nuovo, utente 2026-08-12): attesa
   FERMO (FR1 non morde WP: pattern dim-read-const raro nel suite); banda_ON
   0,036; replica peak-only SENZA inserzione prima di ogni sonda (az.rev.
   S-142 #2, ancora dovuta).
2. **CONCILIO a 9 su B3/filone conteggi** (cambio rotta da sonda, REGOLE §7):
   il bersaglio vivo è il PAVIMENTO per-movimento (2,88 ns × 367,6M) ⇒
   muovere MENO, non muovere più a buon mercato — TakeSlot S-140, liveness,
   arena-conteggi; il fascicolo è il verdetto sonda-B + progettazione S-144.
3. **str 27,6% provenienza** (129,9M creazioni: census provenienza
   monobinario; veti SSO/PhpStr-full restano).
4. **Az.rev. S-145 (VINCOLANTI, wp145-harness/revisione.md)**: #2 rimisura
   m-dimrmw pre-leva vs pin a N≥10× (regressione confermata ⇒ leva in
   istruttoria, dimread resta — keep-partial-wins) · #3 giudice delle
   guardie DENTRO lo script A/B (banda calcolata, rc dal giudizio) · #4
   collaudo del criterio PRIMA della firma (giudici esistenti per nome,
   banda ≥ risoluzione) · #5 pair: run fresca sotto gate 5% o declassare
   a indizio.
5. Residui: az.rev. S-142 #2 (replica peak-only) · tranche-3 other
   (growth-alloc hashbrown, pool Frame) · CI feed in apertura (non-gate).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
FR1-ext: chiave da SLOT (`$o->d[$k]`, pattern LoadSlot) e famiglia
FieldRead/isset · str-provenienza · tranche-3 growth-alloc · attribuzione
Zval-move dei memops (unica via per riaprirli) · peak due livelli · cura
§3.22 · media leg5 2,524 (oss.) · depr. float→int · warning corsia ×2 ·
divergenze RMW · objdatains residuo · objmap 43,4 → piano GC · evalcls
316,9× · refl 42,4× · re +2 alloc · §3.13 · §3.12-i · §3.14 · §3.21 ·
get_gc · drift TODO.md · latin1-cliff · dbal 10 nomi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-145: B1 uniform-rc / B2 root-at-decrement SENZA nuovo concilio (KS-B4
scattato per misura) · àncore di dente emendate con replace testuale (prefix
collision) · inventari su nomi VOLATILI senza normalizzazione · braccio di
contrasto su target dir CONDIVISA tra worktree.**
**S-144: memops come bersaglio B senza attribuzione Zval-move BILATERALE ·
denominatore `sample` coi thread PARCHEGGIATI.**
**S-143: A come scritta RIFONDATA — ogni futura A = pool+refcount+handle-gen.**
BOLT su Mach-O · NaN-boxing (niche GIÀ attiva) · threaded-dispatch · PGO sui
giudici · verdetti su build emendata senza ri-banda · pin/stash senza
collaudo-nell'atto · contenitori sul call path · differenze tra A/B distinti
come cifra · componenti prezzate · magnitudine ripartita senza A/B proprio ·
«icache» NON-premessa · pre-filtro che tassa i freddi · guardie non-bersaglio
BILATERALI · denominatori a memoria · output di run nel repo · rc di gate da
pipe · tee/log pre-mkdir · admission sul dump intero · xctrace senza guardie
disco · run pesanti come task · edit coi build in volo · promozione sotto
banda · gate a soglia fissa senza banda · corpus-gate solo-nomi ·
strumentazione nei sorgenti del pin · leve micro senza banda v2 ·
alloc-removal senza modello del costo SOSTITUTIVO · probe senza riferimento
vivo · ordine FISSO di misura · delta tra census di epoche diverse senza
datare i raw · verdetti da script non committati · SSO inline · inline-array
init+drain args · claim di ASSENZA oltre la risoluzione · smoke con fam-min >
R · notti su PhpStr-full · guardie su giudici diversi dalle loro bande ·
misure con LSP in volo · F2 keys-scratch · quiescenza nello stesso comando
del lancio · pattern quiescenza nell'argv · output TRACKED mossi da
orchestratori · rumore-soglia = range PIENO · guardia senza banda propria ·
percentuale tra segmenti NON annidati senza zero-check · sed di copia su
righe ESEGUIBILI · eccedenza sopra la parte modellata senza sonda · banda di
guardia da strumento DIVERSO senza drop-1 · lock di finestra con trap EXIT
altrui · staleness di lock a OROLOGIO · quiescenza a 2 campioni senza STREAK
· abort multi-gamba senza retry · probe che rompe l'inlining · leva GC
note-time (WP-21) · `git add` di directory harness · identità tra GIUDICI
diversi come gate · gemello di relink senza conferma post-pin · cmp secco vs
oracle su fixture CON divergenze a catalogo · giudice sotto-risoluto · stash
post-batteria senza rebuild ricetta A′ · parser senza golden-test · byte-
identità come gate di un edit .rs post-pin (S-142).
**Riscritto** 2026-08-16 (chiusura S-145); storia in `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-146: pin phpr **s145 a89faf32**c62142f9 + server **s145
4a9adc51**a62b21ba (HEAD ≠ sorgente-pin ATTESO: doc post-pin; sorgente-pin =
4a968b7; pin = stash `phpr-old-target/release/phpr-s145`) · MySQL wp8 con
l'elenco · uploads sotto guardia · corpus 1414 ×2 · CI feed · lock misura
`/private/tmp/phpr-measure.lock` da CREARE a ogni finestra · Data ≥10G ·
pgrep rust-analyzer prima di ogni misura (NON killarlo: respawn+indicizza —
il gate CPU della quiescenza arbitra) · lettura: REGOLE.md → QUI →
WP_SESSION_145 → wp145-harness/revisione.md → s145-sonda-b-verdetto.out →
PERF_MAP.md.
