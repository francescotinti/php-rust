# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **1,765–1,788 (S-142 @ s142; banda_ON unione 0,036)** ·
**⚖️ S-144: numeratore CHIUSO PER MISURA (quota_obj_max_loose 2,38% <25% ⇒ via B senza appello)**
· **bersaglio vivo di B = churn Rc + gc-nota; memops FUORI budget (oracle ne paga ~2/3)** ·
sessioni-senza-misura: 0 (censimenti S-144) · **leve: 0 da DUE sessioni (istruttorie deliberate)
⇒ la prima fetta B in S-145 è DOVUTA** · incidenti 15.

## Scoreboard (pin s142 phpr bba8a7346d727e0e + server eeb284b681c4bf89 — INVARIATO in S-143/S-144)
**arith 5,5 · prop 5,6 · calls 4,7 · str 4,2 · arr 3,2 · re 2,6 · hintcall 7,3**
· oggetti (s136): objchurn 6,7 · objalloc 6,4 · objdatains 5,9 · objdropdef 7,5 ·
objallocni 7,9 · objmap 11,7 · MAPPA (net): WP 1,78 ≈ compoff 1,9 ≪ hf 2,55 ≪
hk 4,3 ≪ dbal 8,2 ≈ ORM 8,6 · corpus **1414 ×2** · **REPERTI S-144: tranche-2
rczval 1,01%/vecargs 2,79%/objsynth 48 (r1==r2 ESATTO, parità rc=0; maggiorante
8–15% del rev. S-143 REFUTATO) · profilo oracle (emenda v2 thread parcheggiati):
churn_zval 0,2% vs phpr 10,3% IN BUDGET · memops 7,8–8,1% vs 12,6% FUORI (severa)
· niche GIÀ attiva (Option<Zval>==16) · attribuzione other 38,3→42,1%, residuo
57,9% fuori-budget dichiarato.**

## §S-145 — ordine
1. **Sonda-B monobinaria** (voce c istruttoria, unica residua; criterio GIÀ
   firmato `wp144-harness/s144-criterio-B.md` p.2–3): ripartizione del churn
   in memcpy / inc-dec / nota + prezzi pair alloc/free (classe S-138) ⇒
   decide l'ordine delle fette B1 «uniform-rc» / B2 «root-at-decrement».
2. **PRIMA FETTA B spedita** (leva DOVUTA): criterio proprio ≤10 righe,
   giudici micro churn + famiglia; promozione ora SBLOCCATA (profilo oracle
   fatto, gate Stogov R4 assolto). Progetto: `s144-progettazione-B.md`.
3. **Az.rev. S-144** (`wp144-harness/revisione.md`, VINCOLANTI).
4. **str 27,6% provenienza** (129,9M creazioni: CHI le crea — census
   provenienza monobinario; veti SSO/PhpStr-full restano).
5. Residui: az.rev. S-142 #2 (replica peak-only senza inserzione) ·
   tranche-3 other (growth-alloc interni PhpArray/hashbrown, pool Frame) ·
   CI feed in apertura (non-gate).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
sonda-B (p.1) · prima fetta B (p.2) · str-provenienza (p.4) · tranche-3
growth-alloc · attribuzione Zval-move dei memops (unica via per riaprirli) ·
peak due livelli · cura §3.22 · media leg5 2,524 (oss.) · depr. float→int ·
warning corsia ×2 · divergenze RMW · objdatains residuo · objmap 43,4 → piano
GC · evalcls 316,9× · refl 42,4× · re +2 alloc · §3.13 · §3.12-i · §3.14 ·
§3.21 · get_gc · drift TODO.md · latin1-cliff · dbal 10 nomi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-144: memops come bersaglio B senza attribuzione Zval-move BILATERALE
(l'oracle ne paga ~2/3) · denominatore `sample` senza esclusione dei thread
PARCHEGGIATI (69% dei campioni oracle era workqueue idle).**
**S-143: A come scritta (arena-sweep senza refcount) RIFONDATA — ogni futura
A = pool+refcount+handle-generazione, giudici nuovi PRIMA del primo commit.**
BOLT su Mach-O · NaN-boxing (la niche è GIÀ attiva: comprata) · threaded-dispatch
· PGO sui giudici · verdetti su build emendata senza ri-banda · pin/stash senza
collaudo-nell'atto · contenitori sul call path · differenze tra A/B distinti
come cifra · componenti prezzate · magnitudine ripartita senza A/B proprio ·
«icache» NON-premessa · pre-filtro che tassa i freddi · guardie non-bersaglio
BILATERALI · denominatori a memoria · output di run nel repo · rc di gate da
pipe · tee/log pre-mkdir · admission sul dump intero · xctrace senza guardie
disco · run pesanti come task · edit coi build in volo · promozione sotto banda
· gate a soglia fissa senza banda · corpus-gate solo-nomi · strumentazione nei
sorgenti del pin · leve micro senza banda v2 · alloc-removal senza modello del
costo SOSTITUTIVO · probe senza riferimento vivo · ordine FISSO di misura ·
delta tra census di epoche diverse senza datare i raw · verdetti da script non
committati · SSO inline · inline-array init+drain args · claim di ASSENZA oltre
la risoluzione · smoke con fam-min > R · notti su PhpStr-full · guardie su
giudici diversi dalle loro bande · misure con LSP in volo · F2 keys-scratch ·
quiescenza nello stesso comando del lancio · pattern quiescenza nell'argv ·
output TRACKED mossi da orchestratori · rumore-soglia = range PIENO · guardia
senza banda propria · percentuale tra segmenti NON annidati senza zero-check ·
sed di copia su righe ESEGUIBILI · eccedenza sopra la parte modellata senza
sonda · banda di guardia da strumento DIVERSO senza drop-1 · lock di finestra
con trap EXIT altrui · staleness di lock a OROLOGIO · quiescenza a 2 campioni
senza STREAK · abort multi-gamba senza retry · probe che rompe l'inlining ·
leva GC note-time (WP-21) · `git add` di directory harness · identità tra
GIUDICI diversi come gate · gemello di relink senza conferma post-pin · cmp
secco vs oracle su fixture CON divergenze a catalogo · giudice sotto-risoluto ·
stash post-batteria senza rebuild ricetta A′ · parser senza golden-test
(S-143/S-144: ora il golden è nel repo) · byte-identità come gate di un edit
.rs post-pin (S-142).
**Riscritto** 2026-08-16 (chiusura S-144); storia in `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-145: pin phpr **s142 bba8a734**6d727e0e + server **s142 eeb284b6**81c4bf89
(HEAD ≠ sorgente-pin ATTESO: census S-142..S-144 post-pin; pin = stash
`phpr-old-target/release/phpr-s142`) · MySQL wp8 con l'elenco · uploads sotto
guardia · corpus 1414 ×2 · CI feed · lock misura da CREARE · Data ≥10G · pgrep
rust-analyzer prima di ogni misura (e KILL prima dei profili campionari) ·
lettura: REGOLE.md → QUI → WP_SESSION_144 → wp144-harness/revisione.md →
s144-progettazione-B.md + s144-criterio-B.md → PERF_MAP.md.
