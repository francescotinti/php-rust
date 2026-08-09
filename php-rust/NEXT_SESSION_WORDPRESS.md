# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full = 1,810–1,889** (S-120) · **media
~2,51–2,53 · peak 1862–1983 MiB** · ultima leva SPEDITA **S-120 (L-RE1)** ·
**sessioni-senza-Δ = 2** (S-121/S-122: leve TENTATE con A/B, ritmo
rispettato) · incidenti: 1 (S-106) + 1 processo (S-115).

## Scoreboard (pin s120 **885d2c64** @ 86306c3 INVARIATO; micro di S-120)

**arith 5,5 · prop 5,5 · calls 4,8 · str 5,3 · arr 3,7 · re 2,8**. S-122:
(a) **BANDA_LAYOUT PROVVISORIA** (K=4, criterio PRIMA): arith 1,00 · prop
1,00 · calls 0,00 · str 5,00 · arr 3,33 · re 5,00 ns/iter — **VIZIATA da
ordine FISSO di misura** (P0 sempre primo; P0 min 4/6 — revisore MISURA):
si rimisura PERMUTATA + pin-vs-pin; (b) **L-ST1**: macchina dice «refutazione
confermata» (str −5,00 = soglia, margine ZERO) ma revisore RIDIMENSIONA a
**NON ACQUISITA** (TA sempre prima di TB = stesso artefatto; soglia_ref senza
2×spread_A); (c) **L-RE2 fermata PRUDENTE** (census re 10→9,00 ESATTO,
smoke −20/−10 R=2 stesso ordine fisso: retry R≥5 alternato prima di
archiviare; anti-tesi mosse Caps 176 B nominata; reperto 4eda5d6b); (d) il
census SPEGNE il peephole fuso (run.rs:4282): classifica delta-alloc
sovrastima il release; (e) gate preg §3.18 CABLATO (8 gate); PGO rinviato.

## §S-123 — ordine proposto

1. **METRO prima di tutto (az. rev. S-122 #1-5)**: banda-LAYOUT rimisurata a
   ordine PERMUTATO per round + 2ª copia del pin (pavimento pin-vs-pin); run
   bersaglio ≥5 s (N×5) o timer sub-ms; A/B a ordine ALTERNATO per coppia;
   soglia_ref simmetrica (con 2×spread_A) → ri-giudizio L-ST1 dal TSV nuovo
   e smoke L-RE2 R≥5 alternato. NESSUNA soglia < ~4 quanti è dicibile.
2. **Classifica-v2 col census nel ramo FUSO** (prerequisito leve micro):
   contatori per-sito senza spegnere la fusione; tabella 6 categorie sul
   sentiero RELEASE vero; predizioni secondarie a verbale.
3. **Istruttoria STRUTTURALE PhpStr single-alloc** (radice comune: str 2
   concat + arr +2,02/op-int + re 3 Rc gruppi — s122-istruttoria-arr/-re):
   tocca php-types ⇒ PERIMETRO prima, niente patch senza criterio col metro
   sanato (p.1).
4. **PGO stadio-2** (criterio §6 s117-criterio-aprime.md: workload WP CON
   teardown, MAI le micro; profdata hashato; non riproducibile ⇒ resta LTO).
5. **prop oltre i cloni** (IC-probe + doppio borrow() RefCell —
   s122-istruttoria-prop): misura SOLO con lo strumento del p.2.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

classifica-v2 ramo fuso (p.2) · PhpStr single-alloc (p.3) · prop IC/RefCell
(p.5) · cura §3.18 (gate preg ROSSO + golden stesso commit) · re residuo 9:
SOLO dopo il metro sanato e classifica-v2 · A′ sola PEGGIORAVA prop (−7,67,
0/5) · Serena HANG (Read mirato + Explore) · fame frontend (kpc/sudo) ·
§3.16/§3.17 warning · retro-A/B str s107b/s108/s109 · denti rinviati (OBS-8;
fx20; direct-bind; drop-order; hit/miss; checkout-staging) · $z++/$z-- undef
non warna · §3.13 · §3.12-i · §3.14 · get_gc · drift TODO.md · latin1-cliff.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing (veto Hoare) · threaded-dispatch (veto Hejlsberg)
· PGO addestrato sui giudici · verdetti su build emendata senza ri-banda ·
pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate · magnitudine ripartita
senza A/B proprio · fixture su memory_get_usage · «icache» NON-premessa ·
pre-filtro che tassa i freddi · guardie non-bersaglio BILATERALI ·
denominatori a memoria · output di run nel repo · rc di gate da pipe (6
morsi) · tee/log pre-mkdir · admission sul dump intero (deroga forma S-118
citata nel criterio) · xctrace senza guardie disco · run pesanti come task ·
edit coi build in volo · promozione sotto banda · gate a soglia fissa senza
banda · bande pre-pipeline su binari post-pipeline · corpus-gate solo-nomi ·
strumentazione nei sorgenti del pin · leve micro ≤10 ns/iter senza
banda-layout · zavorra run-to-run come arbitro del layout · **alloc-removal
senza modello del costo SOSTITUTIVO (3 cadute)** · **probe senza riferimento
vivo (ld64 dead-strippa: nm + hash ≠ pin)** · **classifiche da census che
spegne la fusione**.

---
**Riscritto**: 2026-08-09 (chiusura S-122). Storia: `sessions/` · `gaps/GAP_TREND.md` · revisione in `wp122-harness/revisione.md`.

Pre-flight S-123: pin phpr **s120 885d2c64**6ac7ff4c @ 86306c3 (ricetta A′;
la batteria rilinka ⇒ build ricetta e pretendere 885d2c64 al byte) · server
**s120 6b822369** GRADO PIENO · MySQL wp8 con l'elenco (se giù: mysqld_safe
su datadir esterno) · uploads sotto guardia · conservati: phpr-s118/s119/
s119-treno2/s120-re1(==pin)/s121-st1(reperto) · **phpr-s122-lay1..3 (probe)**
· **phpr-s122-re2 (reperto, 4eda5d6b)** · php-server-s119/s120 · census
riusabile (wp119 census-clite.patch; bande in wp122-harness/layout-out/) ·
Serena in hang: Read mirato + Explore · disco Data ≥15G da ricontrollare.
