# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full = 1,758–1,909** (S-128 @ s127b, FRESCO; ⚠ bordo
alto = gamba leg1-off con ictx 2–3× — coppie proprie PULITE **1,765–1,805**, rev.
S-128 az.1-2; media 2,447–2,539; peak 2,30–2,37×) · ultima leva SPEDITA
**S-127 (stampo)**; S-128: F2 keys-scratch TENTATA (A/B eseguito) e **CADUTA** con
meccanismo — sessioni-senza-leva-spedita = 1 · incidenti: 1 (S-106) + 2 proc. S-125
+ 2 app. S-125 + 1 S-126 + 1 app. S-127 + **2 proc. + 1 app. S-128**.

## Scoreboard (pin **s127b ccb63dca**f565cffc INVARIATO; micro dal gate s127b)

**arith 5,3 · prop 5,6 · calls 4,9(*) · str 4,2 · arr 3,2 · re 2,6** ((*) da osservare
al prossimo gate) · oggetti: **objalloc 7,7 · churn 8,9** (residuo churn: Δins 320 ns
= 5 alloc/insert + macchineria field_write 1297 ins/92 bl) · MAPPA (net): **WP
1,76–1,91 ≈ compoff 1,86–1,89** ≪ hf 2,55 ≪ hk 4,3 ≪ dbal 8,6 ≈ ORM 8,5 · corpus
congelato **1414** ×2 modi · compoff MISURATA (S-128, tarball CON composer-x).

## §S-129 — ordine proposto

1. **L-OL1 seg.3 — MODELLO DEL TEMPO prima di ogni forma** (lezione F2): il census
   attribuisce 5 alloc/insert ma il Vec chiavi (1 di quelle 5) rimosso ha PEGGIORATO
   il giudice di 16,7 ns ⇒ i residui ~2 alloc/statement ignoti E il costo per-ins di
   field_write vanno MODELLATI (sonde temporali per passo + disasm mirato, criterio
   pre-registrato) PRIMA di nominare una forma. Candidati da spiegare: chi alloca i
   2 residui (Key? walk? dropped?), perché overwrite (+4) > secondo insert (+3).
2. **Gate micro R=5 su s127b per osservare calls 4,9(*)** (voce a bordo rumore da
   2 gate: REGOLE §4 — blocca la leva successiva se resta senza rerun).
3. Mappa residui per NOME: lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
   strumento DENSITÀ evalcls · §3.20 dbal (Portability 9 + parser unicode).
4. Fedeltà in coda: §3.19-quater · §3.19-quinquies (phpcs) · iconv parziale.
APPARATO: az.rev. S-128 in `wp128-harness/revisione.md` VINCOLANTI · quiescenza =
GATE separato PRIMA di ogni lancio di misura (mediaanalysisd S-128) · Serena PRIMA
della prima misura · cattura summary per-suite (az.rev. S-126 #3, ancora aperta).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

costo fisso per-statement Field* (residui ~2 alloc + tempo; keys-Vec da SOLO non
paga) · oracle-denominatore: gamba off1 veloce 423,6 s (spread 5,1% vs 1,2% S-125)
da tenere d'occhio alla prossima coppia · evalcls 316,9× (dopo densità) · refl
42,4× · re +2,00 alloc/iter · prop C1 single-borrow · cbargs2 su CallHostBuiltin/
Ref/Spread · grado pieno server · morte-immediata al sito di nota · micro-trim
morte (is_empty 2 remove SipHash · has_destruct precompilato · FxHash mappe per-id)
· AssignPath/pop_keys split_off (gemello locale del costo fisso; objmap 1 alloc) ·
fame frontend (kpc/sudo) · denti rinviati (OBS-8; fx20; direct-bind; drop-order;
hit/miss; checkout-staging) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14
· get_gc · drift TODO.md · latin1-cliff.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing (veto Hoare) · threaded-dispatch (veto Hejlsberg)
· PGO addestrato sui giudici · verdetti su build emendata senza ri-banda ·
pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate · magnitudine ripartita
senza A/B proprio · fixture su memory_get_usage · «icache» NON-premessa ·
pre-filtro che tassa i freddi · guardie non-bersaglio BILATERALI ·
denominatori a memoria · output di run nel repo · rc di gate da pipe (6
morsi) · tee/log pre-mkdir · admission sul dump intero (deroga forma S-118)
· xctrace senza guardie disco · run pesanti come task · edit coi build in
volo · promozione sotto banda · gate a soglia fissa senza banda · bande
pre-pipeline su binari post-pipeline · corpus-gate solo-nomi · strumentazione
nei sorgenti del pin · leve micro senza banda v2 · zavorra run-to-run come
arbitro del layout · **alloc-removal senza modello del costo SOSTITUTIVO (2°
morso: F2 S-128)** · probe senza riferimento vivo · classifiche da census che
spegne la fusione · ordine FISSO di misura: SEMPRE permutato o alternato ·
delta tra census di epoche diverse senza datare i raw · verdetti da script
d'arbitrio non committati · SSO inline (WP-38) · forma inline-array init+drain
args (S-125) · claim di ASSENZA oltre la risoluzione · smoke con fam-min > R ·
altre notti su PhpStr-full · guardie su giudici diversi dalle loro bande (S-127)
· misure con LSP in volo (S-127) · **F2 keys-scratch drain+put-back (caduta
S-128)** · **quiescenza nello stesso comando del lancio (S-128)** · **output di
run TRACKED mossi/cancellati da orchestratori (2 morsi S-128: fuori dal
tracking, mai allentare il gate)**.

**Riscritto**: 2026-08-11 (chiusura S-128). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-129: pin phpr **s127b ccb63dca**f565cffc (stash phpr-s127b; il rebuild
canonico lo RIPRODUCE al byte — provato S-128) · server **s127b bc95ba71** grado
minimo (ripristinato dallo stash dopo relink accidentale S-128: il build workspace
rilinka ANCHE php-server con feature diverse ⇒ dopo ogni build canonico ricontrollare
l'hash server) · MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1414 ×2
modi · compoff-work.tgz ORA CON composer-x · nessuna run detached in volo · ordine
lettura: REGOLE.md → QUI → sessions/WP_SESSION_128.md → wp128-harness/revisione.md
→ PERF_MAP.md.
