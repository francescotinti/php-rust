# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **1,765–1,788 (S-142 @ s142; banda_ON unione 0,036)** ·
**⚖️ DELIBERATO S-143: via B (regola pre-registrata applicata: quota_obj 1,4% <25%)** ·
sessioni-senza-misura: 0 (S-143 census) · incidenti 14 (+ #4 S-142 all'utente).

## Scoreboard (pin s142 phpr bba8a7346d727e0e + server eeb284b681c4bf89 — INVARIATO in S-143)
**arith 5,5 · prop 5,6 · calls 4,7 · str 4,2 · arr 3,2 · re 2,6 · hintcall 7,3**
· oggetti (s136): objchurn 6,7 · objalloc 6,4 · objdatains 5,9 · objdropdef 7,5 ·
objallocni 7,9 · objmap 11,7 · MAPPA (net): WP 1,78 ≈ compoff 1,9 ≪ hf 2,55 ≪
hk 4,3 ≪ dbal 8,2 ≈ ORM 8,6 · corpus **1414 ×2** · **REPERTI S-143: concilio 9/9
(fascicolo `wp143-harness/COUNCIL_S143_REVIEWS.md` VINCOLANTE); census CH_*:
quota_obj 1,38% · quota_str 27,6% (129,9M creazioni/run!) · quota_arr 9,4% ·
other 61,7% dichiarato · galloc_n 471,3M ≡ dossier · `size_of::<Zval>()=16`
(B NON è riduzione-taglia: è Rc-traffic/clone-drop/niche) · bilancio bytes
CHIUSO (realloc disaggregato) · A cade sotto OGNI kill-switch (1,4% vs 15–40%).**

## §S-144 — ordine

1. **PROGETTAZIONE B su carta + criterio pre-registrato** (deliberato: B sola/
   B-poi-A): bersaglio RIMIRATO da zval_size=16 — non taglia ma ciclo-di-vita:
   clone/drop Rc (churn 4,4 s), memops (5,4 s), nota GC. Giudici churn-probe/
   memops (Klabnik R3), soglia banda ORM ±0,7% entro ≤3 sessioni; la
   promozione di B ASPETTA il profilo oracle (Stogov R4, p.2).
2. **ISTRUTTORIA voci restanti**: (b) profilo ORACLE per famiglia (budget =
   phpr−oracle per canale) · (c) sonda prezzi alloc/free+gc_note classe S-138 ·
   (e) «other»: 26,6% profilo S-140 E 61,7% census — chiudere o fuori-budget.
3. **Apertura dal census (per NOME): str 27,6%** — 129,9M creazioni/run: CHI le
   crea (census provenienza, monobinario)? Veti SSO/PhpStr-full restano.
4. **Az.rev. S-142 residue**: #4 decisione utente (near-miss = incidente 15?) ·
   #2 replica peak-only senza inserzione prima di ogni sonda sul bimodale.
5. CI: feed in apertura (coda ~60 in drenaggio, non-gate).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

str-provenienza 27,6% (p.3) · profilo oracle (p.2b) · sonda prezzi (p.2c) ·
other census 61,7% · peak due livelli (az.rev. #2) · cura §3.22 · media leg5
2,524 sopra tetto (oss.) · deprecation float→int · warning corsia ×2 ·
divergenze RMW · objdatains residuo · objmap 43,4 → piano GC · evalcls 316,9× ·
refl 42,4× · re +2 alloc · §3.13 · §3.12-i · §3.14 · §3.21 · get_gc · drift
TODO.md · latin1-cliff · dbal 10 nomi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

**S-143: A come scritta (arena-sweep senza refcount) RIFONDATA — ogni futura
A = pool+refcount+handle-generazione, giudici nuovi PRIMA del primo commit.**
BOLT su Mach-O · NaN-boxing (la niche di B ne compra la parte lecita in safe) ·
threaded-dispatch · PGO sui giudici · verdetti su build emendata senza ri-banda ·
pin/stash senza collaudo-nell'atto · contenitori sul call path (vale per la
tabella handle) · differenze tra A/B distinti come cifra · componenti prezzate ·
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
che eredita il verdetto A/B senza conferma post-pin · staleness di lock a
OROLOGIO · cmp secco vs oracle su fixture CON divergenze a catalogo · giudice
sotto-risoluto per il fenomeno · stash post-batteria senza rebuild ricetta A′ ·
giudice nuovo senza dry-run del parser del harness (S-141; **S-143: parser di
census senza match ESATTO del tag — exit_mi doppiava ogni chiave**) ·
byte-identità come gate di un edit .rs post-pin (S-142).
**Riscritto** 2026-08-16 notte (chiusura S-143); storia in `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-144: pin phpr **s142 bba8a734**6d727e0e + server **s142 eeb284b6**81c4bf89
(HEAD ≠ sorgente-pin ATTESO: contatori census S-142+S-143 post-pin; pin = stash) ·
MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1414 ×2 · CI feed · lock
misura da CREARE · Data ≥10G · pgrep rust-analyzer prima di ogni misura · lettura:
REGOLE.md → QUI → WP_SESSION_143 → COUNCIL_S143_REVIEWS.md (sintesi) →
s143-census-verdetto.out → PERF_MAP.md.
