# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full PULITO = 1,758–1,805** (S-129 @ s127b, gate ictx/s
per gamba: leg1-off E leg3-off escluse, set confermato ANCHE con mediana per-motore;
coppie proprie 1,765–1,805 user+sys · 1,772–1,813 user-only; media CANONICA
user-only 2,447–2,463) · ultima leva SPEDITA **S-127 (stampo)**; S-128 F2 CADUTA;
**S-129 F4 TENTATA: avversa PER CRITERIO con direzione firmata 7/7 (+66,7/+71,7 su
UB 73) — rientro pronto** — sessioni-senza-leva-spedita = 2 · incidenti: storici
7 + **1 app. S-129** (dump tprobes assente su categoria a zero eventi).

## Scoreboard (pin **s127b ccb63dca**f565cffc INVARIATO + server bc95ba71; micro gate S-129)

**arith 5,3 · prop 5,6 · calls 5,0 (sciolta: phpr netto identico) · str 4,2 ·
arr 3,2 · re 2,5** · oggetti: **objalloc 7,7 · churn 8,9** · **MODELLO DEL TEMPO
seg.3 CHIUSO**: statement Field* 300–340 ns quasi invariante (oracle 23–37; locale
170); torta: E−E2 dispatch+prop_step 155 (52%, residuo — attribuzione resolve =
indizio) · preludio 73 (25%, SONDATO) · walk 48 (16%) · MAPPA (net): WP 1,76–1,81
≈ compoff 1,86–1,89 ≪ hf 2,55 ≪ hk 4,3 ≪ dbal 8,6 ≈ ORM 8,5 · corpus **1414** ×2.

## §S-130 — ordine proposto

1. **RIENTRO F4 prelude-gate** (s129-ab-f4-lettura.md; codice PRONTO = revert del
   revert di f4143a6, census 11/11 già valido): PRIMA di ogni run committare il
   criterio emendato — (a) rumore giudice robusto all'outlier singolo (trimmed
   drop-1 SIMMETRICO su A e B, dichiarato); (b) bande objmap/objalloc/objchurn
   FONDATE (spread R≥5 sul pin misurato nello stesso criterio), niente default 4;
   poi smoke R=2 → R=5 → se sopra soglia: promozione piena (s129-promozione.sh
   PRONTO: batteria→pin s129→corpus→fixture→micro→ORM→hk→server).
2. **Sonda E1a** (az.rev. #1): segmento dedicato alle sole `resolve_prop_access`
   dentro `field_write_prop_step` (estendere time-probes.patch, stessa ricetta):
   UB di resolve-once MISURATO, non residuale — decide la leva E1 di S-131.
3. Se F4 promossa: rimisurare micro-ORM sul pin nuovo (objdatains atteso ~−70 ns,
   churn ~−85) + coppia WP full/media SOLO se il tempo resta.
4. Mappa residui per NOME: lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
   strumento DENSITÀ evalcls · §3.20 dbal.
APPARATO: quiescenza = GATE SEPARATO (script s129-quiescenza.sh) PRIMA di ogni
run · Serena prima della prima misura, MAI edit .rs in finestra di misura · gate
ictx/s della pair: mediana PER MOTORE (addendum S-129) · verdetti avversi
committati PRIMA di ogni emenda.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

warm-up leg nella ricetta pair (le 2 gambe segnalate sono entrambe PRIME della
sequenza; oracle cpu più bassa con ictx 2×: meccanismo da nominare prima di
fidarsi del gate) · resolve-once in prop_step (E1a prima) · AssignOp/IncDec fuori
perimetro F4 (comporre dopo la promozione) · famiglia locale 170 ns (pop_keys
split_off; objmap 1 alloc) · evalcls 316,9× · refl 42,4× · re +2,00 alloc/iter ·
oracle-denominatore leg-first · morte-immediata al sito di nota · micro-trim morte
(is_empty SipHash · has_destruct · FxHash per-id) · fame frontend (kpc/sudo) ·
$z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 · get_gc · drift TODO.md ·
latin1-cliff · `$GLOBALS['x']->p` resta FieldAssign (caveat innocuo, rev. S-129).

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
= ricetta S-129) · leve micro senza banda v2 · alloc-removal senza modello del
costo SOSTITUTIVO · probe senza riferimento vivo · ordine FISSO di misura · delta
tra census di epoche diverse senza datare i raw · verdetti da script non
committati · SSO inline · inline-array init+drain args · claim di ASSENZA oltre
la risoluzione · smoke con fam-min > R · notti su PhpStr-full · guardie su giudici
diversi dalle loro bande · misure con LSP in volo · F2 keys-scratch · quiescenza
nello stesso comando del lancio · output TRACKED mossi da orchestratori ·
**rumore-soglia = range PIENO senza formula robusta (morso S-129)** · **guardia su
categoria senza banda propria spacciata per meccanismo (objmap S-129)**.

**Riscritto**: 2026-08-11 (chiusura S-129). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-130: pin phpr **s127b ccb63dca**f565cffc (il rebuild canonico lo
riproduce al byte — riprovato S-129 dopo il revert F4) · server **bc95ba71**
(ripristinato dallo stash: OGNI build canonica lo rilinka, ricontrollare l'hash)
· MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1414 ×2 modi · target
census e time-probes CALDI (phpr-census-l1-target, phpr-time-l1-target) · nessuna
run detached · ordine lettura: REGOLE.md → QUI → sessions/WP_SESSION_129.md →
wp129-harness/revisione.md → wp129-harness/s129-ab-f4-lettura.md → PERF_MAP.md.
