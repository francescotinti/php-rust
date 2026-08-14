# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full ON-ONLY = 1,767–1,781** (S-137 @ **pin s136**,
N=3 coppie proprie, COMPATIBILE col precedente 1,777–1,779 su banda off 0,041;
off 1,805 N=2; media 2,445–2,529; peak 1743–1819 RIENTRATO) — pin INVARIATO ⇒
coppia SOLO a pin nuovo · ultima leva SPEDITA S-136 (FD1) · **sessioni-senza-leva
= 1 (ANOMALIA S-137, 3 ragioni per NOME)** · incidenti: storici 13 (0 nuovi).

## Scoreboard (pin s136 1e14793ec0d9650c + server 91c4e04321309936; micro dal gate S-136)

**arith 5,5 · prop 5,6 · calls 4,7 · str 4,2 · arr 3,3 · re 2,6** · oggetti:
objdatains 5,9 (963,3) · objchurn 6,7 · objalloc 6,4 · objdropdef 7,5 ·
objallocni 7,9 · **objmap 11,7 (116,7): valore-oggetto 43,4 RIQUALIFICATO
design-bound (round-trip GC nota→sweep→demozione, census S-137; cura = piano
gc-cycle-collector, NON leva micro)** · MAPPA (net): WP 1,77 on-only (@s136) ≈
compoff 1,86–1,89 ≪ hf 2,55 ≪ hk 4,3 ≪ dbal 8,36–8,45 ≈ ORM 8,43–8,56 (@s134) ·
corpus **1414** ×2 · **eccedenza FD1 +13,7 NON CHIUSA (sonda S-137: artefatto
inlining del probe; indizio: plumbing +13,0) ⇒ blocco leve dim-write ATTIVO**.

## §S-138 — ordine

1. **Sonda FD1 v2 per SBLOCCARE dim-write** (criterio NUOVO dichiarato): la v1
   cade con IPOTESI artefatto-inlining al call-site di `field_assign_fast` (arm
   probe 56,7 vs ~34,9 dall'A/B) — **disasm DOVUTO PRIMA (az.rev. S-137 #1)**;
   poi vie senza rottura d'inlining: (a) A/B a
   COPPIE DI BUILD, un canale rimosso per build (prezzi per differenza, metodo
   H-D) · (b) disasm del pin (bl-count, protocollo S-104) + census. Se chiude ⇒
   **leva estensione FD1 a FieldAssignOp/FieldIncDec** (RMW; UB dai prezzi chiusi).
2. **Criterio leva con az.rev. S-136 #1 e #5** (OBBLIGO al primo criterio): ruolo
   smoke sulle guardie PRE-REGISTRATO · working tree DICHIARATO pulito al commit.
3. **Az.rev. S-137** (`wp137-harness/revisione.md`): vincolanti.
4. Se p.1 non chiude: leva NON-dim-write con prezzi propri (dispatch 36,3 SOLO dopo modello; 14% AssignPath).
APPARATO: CI batteria-FAIL = ambiente work-dir (access/fstat): sanare o dichiarare noto.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

eccedenza FD1 +13,7 (sonda v2 = p.1) · estensione FD1 AssignOp/IncDec (sblocca
con p.1) · objdatains residuo (leaf 18,9 · plumbing 17,6) · objmap: chiave 10,0
sotto soglia, valore-oggetto 43,4 SPOSTATO al piano GC · dispatch 36,3 ·
walk_driver 37,2 (tutti i Field*) · cammini non cacheabili (readonly, mangled,
`__set`, slot assente, child Ref) · famiglia locale 170 ns · evalcls 316,9× ·
refl 42,4× · re +2,00 alloc/iter · 14% AssignPath · §3.13 · §3.12-i · §3.14 ·
§3.21 · get_gc · drift TODO.md · latin1-cliff · media bordo 2,529 (osservazione).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing · threaded-dispatch · PGO sui giudici · verdetti su
build emendata senza ri-banda · pin/stash senza collaudo-nell'atto · contenitori
sul call path · differenze tra A/B distinti come cifra · componenti prezzate ·
magnitudine ripartita senza A/B proprio · «icache» NON-premessa · pre-filtro che
tassa i freddi · guardie non-bersaglio BILATERALI · denominatori a memoria ·
output di run nel repo · rc di gate da pipe · tee/log pre-mkdir · admission sul
dump intero · xctrace senza guardie disco · run pesanti come task · edit coi
build in volo · promozione sotto banda · gate a soglia fissa senza banda ·
corpus-gate solo-nomi · strumentazione nei sorgenti del pin (#[path] fuori-crates
= ricetta) · leve micro senza banda v2 · alloc-removal senza modello del costo
SOSTITUTIVO · probe senza riferimento vivo · ordine FISSO di misura · delta tra
census di epoche diverse senza datare i raw · verdetti da script non committati ·
SSO inline · inline-array init+drain args · claim di ASSENZA oltre la risoluzione
· smoke con fam-min > R · notti su PhpStr-full · guardie su giudici diversi dalle
loro bande · misure con LSP in volo (anche via Serena/IDE: **verificare pgrep
rust-analyzer PRIMA di ogni finestra**) · F2 keys-scratch · quiescenza nello
stesso comando del lancio · output TRACKED mossi da orchestratori · rumore-soglia
= range PIENO senza formula robusta · guardia su categoria senza banda propria ·
percentuale tra segmenti NON annidati senza controllo a zero eventi · pattern del
gate quiescenza dentro l'argv del lancio · sed di copia senza collaudo delle
righe NON toccate · eccedenza sopra la parte modellata senza sonda · banda di
guardia da strumento DIVERSO senza drop-1 del run · lock di finestra con trap
EXIT altrui · quiescenza a 2 campioni senza STREAK · abort multi-gamba al PRIMO
morso senza retry · **identità di sonda con probe che rompe l'inlining del
bersaglio · leva GC note-time contro il precedente WP-21 · `git add` di
directory harness (imbarca gli output di run: add per FILE)**.

**Riscritto**: 2026-08-14 (chiusura S-137). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-138: pin phpr **s136 1e14793e**c0d9650c + server **91c4e043**21309936
(INVARIATI; stash in phpr-old-target/release/) · MySQL wp8 con l'elenco · uploads
sotto guardia · corpus 1414 ×2 · nessuna run detached · CI: coda post-lock
(batteria-FAIL = ambiente work-dir, non regressione); lock misura
`/private/tmp/phpr-measure.lock` da CREARE a ogni finestra (oggi RIMOSSO) · disco
Data ≥10G · pgrep rust-analyzer PRIMA di ogni misura · lettura: REGOLE.md → QUI →
WP_SESSION_137 → wp137-harness/revisione.md → PERF_MAP.md.
