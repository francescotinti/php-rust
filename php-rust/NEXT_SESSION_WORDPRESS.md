# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full = 1,810–1,889** (16 celle S-120; mediano
~1,85) · **media ~2,51–2,53 · peak 1862–1983 MiB** · ultima leva SPEDITA
**S-120 (L-RE1)** · **sessioni-senza-Δ = 2** (S-121 L-ST1, S-122 L-ST1-full +
L-RE2: leve TENTATE con A/B e verdetto — ritmo rispettato, promozioni no) ·
incidenti: 1 (S-106) + 1 processo (S-115).

## Scoreboard (pin s120 **885d2c64** @ 86306c3 INVARIATO; micro di S-120)

**arith 5,5 · prop 5,5 · calls 4,8 · str 5,3 · arr 3,7 · re 2,8**. S-122 ha
STABILITO: (a) **BANDA_LAYOUT** (K=4 layout stesso sorgente-pin, criterio
PRIMA): **arith 1,00 · prop 1,00 · calls 0,00 · str 5,00 · arr 3,33 · re
5,00** ns/iter — le soglie micro d'ora in poi la includono; (b) **L-ST1
REFUTAZIONE CONFERMATA** al full (str D_med −5,00 = soglia esatta, margine
ZERO a verbale; superstite bookkeeping, non layout); (c) **L-RE2 FERMATA**
(SmallVec inline Caps.groups: census re 10→9,00 ESATTO ma smoke −20/−10
fuori banda 10,00; anti-tesi mosse Caps 176 B nominata NEL criterio; reperto
phpr-s122-re2 4eda5d6b); (d) il census SPEGNE il peephole fuso (run.rs:4282):
la classifica delta-alloc SOVRASTIMA il release e ha esaurito i guadagni
facili — 3 leve alloc-removal consecutive cadute sul tempo; (e) gate preg
§3.18 CABLATO in s109-fixture-chain (8 gate); PGO stadio-2 rinviato.

## §S-123 — ordine proposto

1. **Classifica-v2 col census nel ramo FUSO** (prerequisito di OGNI leva
   micro futura): contatori per-sito che NON spengano la fusione (estensione
   census-clite dentro il ramo fused di PropGetSlotRecv e gemelli); rifare la
   tabella 6 categorie sul sentiero RELEASE vero; predizioni secondarie a
   verbale nel verdetto (regola S-122 già applicata).
2. **Istruttoria leva STRUTTURALE PhpStr single-alloc** (radice comune:
   str 2 alloc concat + arr +2,02/op-int + re 3 Rc gruppi — vedi
   s122-istruttoria-arr.md/-re.md): layout single-alloc o small-string
   inline; tocca php-types ⇒ istruttoria di PERIMETRO prima (siti, ABI,
   rischio), niente patch senza criterio con banda-layout.
3. **PGO stadio-2** (criterio §6 s117-criterio-aprime.md: workload WP
   request-loop CON teardown, MAI le micro; profdata hashato e congelato;
   KS-A2: profdata non riproducibile ⇒ resta LTO).
4. **prop oltre i cloni**: il gap 5,5× è indiziato su IC-probe + doppio
   borrow() RefCell per iter (istruttoria s122-istruttoria-prop.md) — misura
   per-componente SOLO con lo strumento della classifica-v2 (punto 1).
5. **Cura §3.18** ((?J) dup-names + __phprbg): alla cura il gate preg diventa
   ROSSO e i golden si aggiornano NELLO STESSO commit.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

classifica-v2 ramo fuso (p.1) · PhpStr single-alloc (p.2) · prop
IC/RefCell (p.4) · cura §3.18 (p.5) · re residuo 9: SOLO dopo classifica-v2
(2 refutazioni: swap E inline-176B; un terzo tentativo vuole un meccanismo
NUOVO) · A′ da sola PEGGIORAVA prop (−7,67, 0/5): non indagato · Serena HANG
su questo repo (Read mirato + Explore) · fame frontend (kpc/sudo — azione
utente) · §3.16/§3.17 warning · retro-A/B str s107b/s108/s109 · denti
rinviati (OBS-8; fx20; direct-bind; drop-order; hit/miss; checkout-staging)
· $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 · get_gc · drift
TODO.md · latin1-cliff preg (freddo sui giudici).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing (veto Hoare) · threaded-dispatch (veto
Hejlsberg) · PGO addestrato sui giudici · verdetti su build emendata senza
ri-banda · pin/stash senza collaudo-nell'atto · contenitori sul call path ·
differenze tra A/B distinti come cifra · componenti prezzate · magnitudine
ripartita senza A/B proprio · fixture su memory_get_usage · «icache»
NON-premessa · pre-filtro che tassa i freddi · guardie non-bersaglio
BILATERALI · denominatori a memoria · output di run nel repo · rc di gate
da pipe (6 morsi) · tee/log pre-mkdir · admission sul dump intero (deroga
forma S-118: leve runtime-only a emissione INVARIATA, citata nel criterio) ·
xctrace senza guardie disco · run pesanti come task · edit coi build in volo
· promozione sotto banda · gate a soglia fissa senza banda · bande
pre-pipeline su binari post-pipeline · corpus-gate solo-nomi · strumentazione
nei sorgenti del pin · leve micro ≤10 ns/iter senza banda-layout misurata ·
zavorra run-to-run come arbitro del layout · **leve alloc-removal micro senza
modello del costo SOSTITUTIVO (3 cadute: L-ST1, L-RE2, e il take/restore)** ·
**probe/no-op senza riferimento vivo (ld64 dead-strippa: nm + hash ≠ pin)** ·
**classifiche da census che spegne la fusione**.

---
**Riscritto**: 2026-08-09 (chiusura S-122). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisione in `wp122-harness/revisione.md`.

Pre-flight S-123: pin phpr **s120 885d2c64**6ac7ff4c @ 86306c3 (ricetta A′:
`[profile.release]` + `SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0`; la batteria
rilinka ⇒ build ricetta e pretendere 885d2c64 al byte) · server **s120
6b822369** GRADO PIENO · MySQL wp8 con l'elenco (se giù: mysqld_safe
daemonizzato su datadir esterno) · uploads sotto guardia · conservati:
phpr-s118 · phpr-s119 · phpr-s119-treno2 · phpr-s120-re1 (==pin) ·
phpr-s121-st1 (REPERTO refutato) · **phpr-s122-lay1..3 (probe layout)** ·
**phpr-s122-re2 (REPERTO refutato, 4eda5d6b)** · php-server-s119/s120 ·
census target riusabile (wp119 census-clite.patch; bande in
wp122-harness/layout-out/layout-bande.txt) · Serena in hang: Read mirato +
Explore · disco Data: liberata cache uv (1,6G) in S-122, ricontrollare ≥15G.
