# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full ON-ONLY = 1,752–1,785 (S-139 @ s138, N=5)** ·
**banda_ON CANONICA = 0,033** (fondata S-139; la banda off 0,041 è PENSIONATA) ·
ultima leva SPEDITA S-138 (FD1-ext RMW) · **sessioni-senza-leva = 1 (S-139,
anomalia dichiarata)** · incidenti: storici 14.

## Scoreboard (pin s138 fa17dabd9eaa4bcb + server a9aded4516e6d46c; micro dal gate promo S-138)

**arith 5,6 · prop 5,6 · calls 4,8 · str 4,3 · arr 3,2 · re 2,6** · oggetti
(gate s136): objdatains 5,9 · objchurn 6,7 · objalloc 6,4 · objdropdef 7,5 ·
objallocni 7,9 · objmap 11,7 (→ piano gc-cycle-collector) · RMW: m-dimrmw 147 ·
m-diminc 113 · MAPPA (net): WP **1,77 @ s138** ≈ compoff 1,86–1,89 ≪ hf 2,55 ≪
hk 4,3 ≪ **dbal 8,15–8,23 ≈ ORM 8,59–8,71 (S-139 @ s138)** · corpus **1414**
×2 · **REPERTO S-139: le TRE leve dim-write (AP1+FD1+RMW) NON muovono le suite
Doctrine — la prossima leva object si sceglie dal profilo SUITE.**

## §S-140 — ordine

1. **LEVA dai numeri (obbligo ritmo, 2ª sessione)**: il REPERTO S-139 orienta al
   **profilo SUITE ORM** (churn clone/drop Zval, insert/lookup — già indicato da
   S-135) PRIMA della candidata micro dim-read; se il profilo conferma il churn,
   leva lì; dim-read resta candidata con istruttoria PRONTA
   (`wp139-harness/s139-istruttoria-dimread.md`: lowering dump → riferimento
   m-dimread R=5 bilaterale → modello → criterio → A/B). Prerequisito collaudo
   RMW: GIÀ CHIUSO (S-139, verdetto s139-rmw-collaudo).
2. **Peak WP 1831–1849 OSSERVAZIONE** (+80 MiB sopra banda oss. s136/s137 su
   TUTTE le gambe): attribuire per NOME prima di qualunque claim (candidate:
   celle IC per-sito RMW, warmup on-config) — misura dedicata, non gate.
3. **CI**: leggere `phpr-ci/CI_FEED.log` (coda ~42 in smaltimento col mutex
   NUOVO PID-based — verificare 1 solo runner); GH Actions: esito ultimo run
   (fix corsie census + stub mach committati bf6ab09; se rosso residuo →
   iterare); fase 2 (corpus-gate col tarball php.net) SOLO su decisione utente.
4. Coppia WP: NON dovuta (pin invariato s138); torna dovuta a pin nuovo, col
   confronto sulla banda_ON 0,033.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

Profilo SUITE ORM (REPERTO S-139) · FieldRead/dim-read IC (istruttoria pronta) ·
peak-osservazione 1831–1849 · divergenze RMW del PIENO (undefined-key ·
float-key · str-increment · overloaded-notice: per NOME, pre-esistenti) ·
objdatains residuo (leaf 18,9 · plumbing 17,6) · objmap 43,4 → piano GC ·
dispatch 36,3 · walk_driver 37,2 · cammini non cacheabili (readonly, mangled,
`__set`, slot assente, child Ref) · famiglia locale 170 ns · evalcls 316,9× ·
refl 42,4× · re +2,00 alloc/iter · 14% AssignPath · §3.13 · §3.12-i · §3.14 ·
§3.21 · get_gc · drift TODO.md · latin1-cliff · media bordo 2,529 (oss.) ·
warnings Linux php-builtins ×4 (dependency-position, non bloccanti).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing · threaded-dispatch · PGO sui giudici · verdetti su
build emendata senza ri-banda · pin/stash senza collaudo-nell'atto · contenitori
sul call path · differenze tra A/B distinti come cifra · componenti prezzate ·
magnitudine ripartita senza A/B proprio · «icache» NON-premessa · pre-filtro che
tassa i freddi · guardie non-bersaglio BILATERALI · denominatori a memoria ·
output di run nel repo · rc di gate da pipe · tee/log pre-mkdir · admission sul
dump intero · xctrace senza guardie disco · run pesanti come task · edit coi
build in volo · promozione sotto banda · gate a soglia fissa senza banda ·
corpus-gate solo-nomi · strumentazione nei sorgenti del pin · leve micro senza
banda v2 · alloc-removal senza modello del costo SOSTITUTIVO · probe senza
riferimento vivo · ordine FISSO di misura · delta tra census di epoche diverse
senza datare i raw · verdetti da script non committati · SSO inline ·
inline-array init+drain args · claim di ASSENZA oltre la risoluzione · smoke
con fam-min > R · notti su PhpStr-full · guardie su giudici diversi dalle loro
bande · misure con LSP in volo · F2 keys-scratch · quiescenza nello stesso
comando del lancio · pattern quiescenza nell'argv del lancio · output TRACKED
mossi da orchestratori · rumore-soglia = range PIENO · guardia senza banda
propria · percentuale tra segmenti NON annidati senza zero-check · sed di copia
su righe ESEGUIBILI (n.14) · eccedenza sopra la parte modellata senza sonda ·
banda di guardia da strumento DIVERSO senza drop-1 · lock di finestra con trap
EXIT altrui · quiescenza a 2 campioni senza STREAK · abort multi-gamba senza
retry · probe che rompe l'inlining · leva GC note-time (WP-21) · `git add` di
directory harness · identità tra GIUDICI diversi come gate · gemello di relink
che eredita il verdetto A/B senza conferma post-pin · **staleness di lock a
OROLOGIO (si giudica sul PID vivo — S-139)** · **cmp secco vs oracle su fixture
CON divergenze a catalogo (si pre-registra il filtro per NOME — S-139)**.

**Riscritto**: 2026-08-15 (chiusura S-139). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-140: pin phpr **s138 fa17dabd**9eaa4bcb + server **s138 a9aded45**16e6d46c
(INVARIATI; sorgente AVANTI al pin di soli attributi lint + cfg(test) +
ic-stats cfg-gated: churn dichiarato, prossimo pin li ingloba) · MySQL wp8 con
l'elenco · uploads sotto guardia · corpus 1414 ×2 · CI locale: coda in
smaltimento col mutex nuovo · lock misura `/private/tmp/phpr-measure.lock` da
CREARE a ogni finestra (oggi RIMOSSO) · disco Data ≥10G · pgrep rust-analyzer
PRIMA di ogni misura · lettura: REGOLE.md → QUI → WP_SESSION_139 →
wp139-harness/revisione.md → PERF_MAP.md.
