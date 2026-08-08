# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)
⏱ **FONDAMENTALI**: rif WP **full ON 1,867 / OFF 1,869** (S-110) · media
2,632/2,603 (voce aperta) · ultima leva SPEDITA **S-112 (H-A2)** ·
sessioni-senza-Δ = 4 (S-113..116: tutte con leva ad A/B pieno) · incidenti
«mai collaudato»: 1 (S-106) · incidenti processo: 1 S-115 (BUILD_RC da pipe).

## Scoreboard (pin s112 f71abd2a, micro R=5 di S-112 — S-113..116 non li muovono)

**arith 5,5 · prop 7,6 · calls 5,2 · str 5,3 · arr 4,2 · re 3,4** · held-out
6,4·2,5·5,6. S-116 ha STABILITO: (a) L-A rigiudicata a criterio emendato:
**prop +29,33 5/5 PASS, guardia calls −6,50 < −5,50 SFONDATA ⇒ NON promossa**;
la tassa calls è FATTO (5 campioni: L-A −6,5/−7,0/−6,5 vs nulle −5,5/−5,5);
(b) batteria 8bb395c CHIUSA (3×1742/0/2, inventari identici — 1741 S-113 non
riproducibile); (c) banda micro N=2: 0,40·4,33·5,50·5,00·6,67·10,00 — ⚠️ le
bande e le baseline DECADONO alla prima build A′ (build emendata, concilio).

## ⚖️ CONCILIO A 9 (S-116, convocato dall'utente — CAMBIO DI ROTTA)

`wp117-harness/COUNCIL_WP117_REVIEWS.md` = VINCOLANTE per S-117 (9/9 concordo
con emendamenti; in conflitto vince il verbale individuale). Delibera: basta
leve micro una-a-una. **BOLT ESPUNTO (non esiste su Mach-O)**. Rotta:
**A′ (pipeline build) → ri-banda → treno B → C-lite/D dai contatori**;
«C in riserva» REFUTATA dall'aritmetica (prop chiede −65 ns/iter; A′+L-A
danno −31..−45): istruttoria C-lite comunque entro S-119.

## §S-117 — ordine (dal concilio, vincolante)

1. **Spike A′ stadio-1** (criterio PRE ≤10 righe PRIMA di toccare Cargo.toml):
   `[profile.release]` lto=fat + codegen-units=1 (order_file ld64 se regge);
   **determinismo: build ×2 → hash IDENTICO**; gate PIENI sul candidato
   (batteria rc in FILE · corpus 1415 per NOME ×2 · fixture · parità micro);
   timebox ½ sessione. Se regge: **stadio-2 PGO** — profilo PINNATO/versionato,
   workload ≠ sei giudici e ≠ held-out, DEVE includere request_end/teardown.
2. **Ri-banda su A′**: ≥2 leve nulle micro (metro riparato se max ≤5 ns/iter)
   + banda held-out N≥2. Nessun verdetto su A′ prima della ri-banda.
3. **Rigiudizio L-A** (cherry-pick 2c18b2e, MAI checkout parziale) sotto A′
   con bande NUOVE — è il test della tassa calls; micro R=5 = nuove baseline;
   se promossa: pin via scripts/pin-phpr.sh §6 PIENO.
**KS-A** (kill-switch): uplift A′ mediano <2% E banda_new >5 ⇒ treno B su
pipeline vecchia + istruttoria C-lite ANTICIPATA a S-118. Soglie/logica E-O
dei kill: dissenso registrato — fissarle nel criterio PRE di S-117.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

Treno B: spec gate (manifest per NOME cap 5, nulla-treno, bisezione — verbale
klabnik/team-struttura) · istruttoria C-lite: harness contatori rc-op/alloc
per categoria su ENTRAMBI i motori (team-engine; entro S-119) · classifica D
dai contatori (stogov) · dissensi concilio da conciliare nel criterio PRE
(statuto C; workload PGO klabnik-vs-pedersen; soglie KS-A) · banda held-out
N=1 da irrobustire · bimodalità P/E-core (non attribuita) · arr 4,2 vs 3,9
(tra-sere) · media voce aperta · rumore tra-sere WP · fame frontend (kpc/sudo
— azione utente) · §3.16/§3.17 warning · retro-A/B str s107b/s108/s109 ·
denti rinviati (OBS-8; fx20; direct-bind; drop-order; hit/miss;
checkout-staging) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 ·
get_gc · drift TODO.md · (b-min)/(g) solo se census WP li mostra caldi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

**BOLT su Mach-O (inesistente)** · **NaN-boxing (unsafe per costruzione — veto
Hoare)** · threaded-dispatch come vagone D (veto Hejlsberg; già refutato S-111)
· PGO addestrato sui giudici · verdetti su build emendata senza ri-banda ·
pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate · magnitudine ripartita
senza A/B proprio · finestre estese senza criterio+dente · fixture su
memory_get_usage · «icache» NON-premessa · pre-filtro che tassa i freddi ·
guardie non-bersaglio BILATERALI · denominatori a memoria · output di run nel
repo · rc di gate da pipe (6 morsi) · tee/log pre-mkdir · admission sul dump
intero · xctrace senza guardie disco · leve lifecycle mono-clone su prop ·
run pesanti come task (detached DA SUBITO) · edit coi build in volo ·
promozione su segni concordi sotto banda · gate a soglia fissa senza banda.

---
**Riscritto**: 2026-08-08 (chiusura S-116). Storia: `sessions/` · `gaps/GAP_TREND.md` · concilio in `wp117-harness/`.

Pre-flight S-117: pin phpr **f71abd2a** (release AL pin, MAI toccata in S-116)
· server 443ae42f (stash s109; relink 06e6d677) · MySQL wp8 con elenco
database · uploads sotto guardia · nessuna run in volo · disco Data ~11-12G
(sotto soglia 15G: dichiarare e usare Extreme Pro per i raw) · candidati
conservati phpr-s114-{la,nulla} + phpr-s115-nulla2 · ⚠️ prima build A′
invalida bande/baseline: ordine ri-banda PRIMA di ogni verdetto.
