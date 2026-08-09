# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full = 1,810–1,889** (S-120) · **media
~2,51–2,53 · peak 1862–1983 MiB** · ultima leva SPEDITA **S-120 (L-RE1)** ·
**sessioni-senza-Δ = 3** (S-121/122/123: A/B eseguiti, ritmo rispettato) ·
incidenti: 1 (S-106) + 1 processo (S-115).

## Scoreboard (pin s120 **885d2c64** @ 86306c3 INVARIATO; micro vs oracle di S-120)

**arith 5,5 · prop 5,5 · calls 4,8 · str 5,3 · arr 3,7 · re 2,8**. S-123 =
METRO SANATO: (a) **BANDA_V2** (permutata quadrato-latino + P0b copia-pin,
timer getrusage-µs, N scalati ≥5 s): **arith 0,94 · prop 0,80 · calls 0,73 ·
str 2,89 · arr 2,49 · re 4,46** ns/iter, PAV_PIN 0,01–1,36, posizioni NON
monotone, predizioni 3/3 — SOSTITUISCE le provvisorie S-122; soglie future =
max(SOGLIA_LAYOUT; 2×spread_A; 4×quanto) (file macchina:
wp123-harness/metro-out/layout-bande-v2.txt, righe anche nel verdetto .out);
(b) **L-ST1 refutazione ACQUISITA** (str −5,08 vs soglia −3,50, 0/5 alternato);
(c) **L-RE2 ARCHIVIATA** (re −15,74, 0/6, R=6 alternato; anti-tesi mosse Caps
vince — 4ª caduta alloc-removal sul costo sostitutivo); (d) **classifica-v2
FUSA** (build SOLO mem-census, fuso vivo): Δalloc/iter vs oracle **re +5,00 ·
str +3,00 · arr +2,05**, resto 0; prop zvclone 5→3 (fusione, residui in
`$s+=$o->x`); **arr = 2 ZStr di chiave per lettura** ⇒ single-alloc → ~parità.

## §S-124 — ordine proposto

1. **PhpStr single-alloc, CRITERIO** (leva strutturale confermata dalla
   classifica-v2: attesa alloc str −2 · re −3 · arr −2,04≈parità): perimetro
   GIÀ misurato in wp123-harness/s123-istruttoria-phpstr.md (7 siti Rc-API,
   funnel unico zstr.rs:54, 3 rischi non testuali: RcEqIdent chiavi, hash in
   Cell, !Clone). Il criterio DEVE scrivere il modello del costo SOSTITUTIVO
   (refcount custom, regrow append, PartialEq manuale con ptr-fast-path) PRIMA
   del tempo; soglie dal metro S-123; gate: batteria + corpus 1415×2 + fixture
   + ricetta ORM/http-kernel (tocca php-types). Fasi: patch → census fuso
   (conferma −2/−3/−2) → A/B alternato → gate.
2. **PGO stadio-2** (criterio §6 s117-criterio-aprime.md: workload WP CON
   teardown, MAI le micro; profdata hashato; non riproducibile ⇒ resta LTO).
3. **prop oltre i cloni**: i 3 zvclone/iter residui stanno in `$s += $o->x`
   (secondo statement) — IC-probe/doppio borrow s122-istruttoria-prop, misure
   col census FUSO (s123-classifica-*.sh riusabili).
4. Full/media A/B WP quando una leva promuove (riferimento S-120 resta).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

PhpStr single-alloc (p.1) · PGO st.2 (p.2) · prop residui fusi (p.3) · cura
§3.18 (gate preg ROSSO + golden stesso commit) · re residuo SOLO via
single-alloc dei 3 gruppi (L-RE2 archiviata) · Serena HANG · fame frontend
(kpc/sudo) · §3.16/§3.17 warning · retro-A/B str s107b/s108/s109 · denti rinviati (OBS-8; fx20;
direct-bind; drop-order; hit/miss; checkout-staging) · $z++/$z-- undef non
warna · §3.13 · §3.12-i · §3.14 · get_gc · drift TODO.md · latin1-cliff.

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
strumentazione nei sorgenti del pin · leve micro senza banda v2 · zavorra
run-to-run come arbitro del layout · alloc-removal senza modello del costo
SOSTITUTIVO (**4 cadute**) · probe senza riferimento vivo · **classifiche da
census che spegne la fusione** · **ordine FISSO di misura: SEMPRE permutato o
alternato** · **delta tra census di epoche diverse senza datare i raw**.

---
**Riscritto**: 2026-08-09 (chiusura S-123). Storia: `sessions/` · `gaps/GAP_TREND.md` · revisione in `wp123-harness/revisione.md`.

Pre-flight S-124: pin phpr **s120 885d2c64**6ac7ff4c @ 86306c3 (ricetta A′; la
batteria rilinka ⇒ build ricetta e pretendere 885d2c64 al byte) · server
**s120 6b822369** GRADO PIENO · MySQL wp8 con l'elenco (se giù: mysqld_safe su
datadir esterno) · uploads sotto guardia · conservati: phpr-s118/s119/
s119-treno2/s120-re1(==pin)/s121-st1(reperto)/s122-lay1..3(probe)/
s122-re2(reperto)/**s123-p0b(==pin, fixture metro)** · php-server-s119/s120 ·
census-target FUSO riusabile (solo mem-census; catena s123-classifica-*.sh) ·
Serena in hang: Read mirato + Explore · ⚠️ **disco Data ~8G < 15G**: snapshot
aggiornamento macOS (MSUPrepareUpdate) — completare/annullare PRIMA delle run pesanti.
