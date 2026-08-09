# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full ON 1,716 / OFF 1,837 · media 2,636/2,519 ·
peak ON 1933,6 MiB** (S-119, pin s119+server s119, serie A′ N=1 direzione-solo;
oracle mosso ~7% tra le gambe: magnitudine del calo non ripartita) · ultima leva
SPEDITA **S-119 (treno-2 V3-V5)** · sessioni-senza-Δ = 0 · incidenti «mai
collaudato»: 1 (S-106) · incidenti processo: 1 (S-115).

## Scoreboard (pin s119 **350582e5** @ 22e0cda = A′+L-A+H-P1+V3-V5; micro R=5; vs s118)

**arith 5,4 ↓ · prop 5,5 = · calls 5,0 ↑ · str 5,6 ↑ · arr 3,7 ↓ · re 3,3 =**
(A/B stessa-sera: guardie 6/6 TENGONO — calls D_med −0,50 = quanto layout
[banda calls ora N=4: +0,50/−0,50/+0,50/−0,50 ⇒ 0,50 CONFERMATA], str +5,00;
frecce dei rapporti = anche rumore oracle). S-119 ha STABILITO: (a) **C-lite**:
tabella 6×4 conteggi/iter sui DUE motori (s119-clite-verdetto.out) — int-pure
ZERO alloc entrambi ⇒ residuo prop = 5 cloni Zval/iter (1 Rc) vs 0 rc-op Zend;
**classifica delta: re +12 alloc/iter · str +3 · arr +2**; (b) **treno-2
PROMOSSO** (guardie 6/6, held-out N=3 3/3, poly −0,13 s direzione attesa, §6
pieno con resume dichiarato); (c) server **s119 gradato** ×2 modi + coppia WP
stessa sera; (d) ⭐⭐ ogni edit a php-types cambia il binario (span→svh→simboli):
strumentazione SOLO via `wp119-harness/census-clite.patch`, mai nei sorgenti.

## §S-120 — ordine proposto

1. **Leva re-alloc** (classifica C-lite voce 1: re = 17 alloc/iter phpr vs 5
   Zend nel giudice re): istruttoria BREVE (dove nascono: $m ricostruito, gruppi,
   preg-cache) → criterio ≤10 righe (bersaglio NEL giudice re; banda-v2 re 0,00
   ma banda micro N=2 re 10,00: trattarla nel criterio) → A/B.
2. **Str-alloc** (+3/iter) se il timebox regge: stessa forma (concat+substr = 5
   alloc/iter phpr vs 2 Zend).
3. **Coppia WP N=2 sul pin s119** (stessa coppia di stanotte ⇒ prime bande della
   famiglia A′; scioglie anche peak +94 MiB e media ON 2,636, oggi N=1).
4. **Stadio-2 PGO** (criterio §6 pre-registrato in s117-criterio-aprime.md;
   KS-A2: profdata non riproducibile ⇒ resta LTO).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

prop residuo: 4 cloni scalari + 1 Rc/iter ANCORA sul giudice (census su pin
s118; rimisurare col patch se serve una leva) · smoke-di-guardia da ricalibrare
(R=2 post-build mente: prop −2 svanito a R=5 — primo-giro; early-stop solo su
segno concorde a R≥3 o warmup escluso) · media ON 2,636 e peak 1933,6 N=1 (vedi
§3) · oracle tra-gambe ±7% stanotte (rumore tra-sere WP) · held-out: metodo N=3
stessa-sera promosso (banda poly 0,04 vs 0,01 di S-117) · str/arr bande larghe
7,50/6,67: zavorre N≥3 prima di leve str/arr · A′ da sola PEGGIORAVA prop
(−7,67, 0/5): meccanismo non indagato · Serena find_symbol/search_for_pattern
HANG su questo repo (Read mirato + agente Explore) · fame frontend (kpc/sudo —
azione utente) · §3.16/§3.17 warning · retro-A/B str s107b/s108/s109 · denti
rinviati (OBS-8; fx20; direct-bind; drop-order; hit/miss; checkout-staging) ·
$z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 · get_gc · drift TODO.md ·
(b-min)/(g) solo se census WP li mostra caldi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing (veto Hoare) · threaded-dispatch (veto Hejlsberg) ·
PGO addestrato sui giudici · verdetti su build emendata senza ri-banda ·
pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze tra
A/B distinti come cifra · componenti prezzate · magnitudine ripartita senza A/B
proprio · fixture su memory_get_usage · «icache» NON-premessa · pre-filtro che
tassa i freddi · guardie non-bersaglio BILATERALI · denominatori a memoria ·
output di run nel repo · rc di gate da pipe (6 morsi) · tee/log pre-mkdir ·
admission sul dump intero (deroga: leve runtime-only a emissione INVARIATA,
forma S-118, citata nel criterio) · xctrace senza guardie disco · run pesanti
come task · edit coi build in volo · promozione sotto banda · gate a soglia
fissa senza banda · bande pre-pipeline su binari post-pipeline · corpus-gate
solo-nomi · **edit di strumentazione nei sorgenti del pin (S-119: span→svh)**.

---
**Riscritto**: 2026-08-09 (chiusura S-119). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisione in `wp119-harness/revisione.md`.

Pre-flight S-120: pin phpr **s119 350582e5** @ 22e0cda (ricetta A′:
`[profile.release]` in Cargo.toml + `SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0`;
la batteria rilinka ⇒ build ricetta e pretendere 350582e5 al byte) · server
**b7bd6744 pin s119 GRADATO** · MySQL wp8 con l'elenco · uploads sotto guardia ·
disco Data ~4G (dichiarare, raw su Extreme Pro) · conservati: phpr-s117 ·
phpr-s118 · phpr-s118-treno1 · phpr-s119 (==pin) · phpr-s119-treno2 (==pin) ·
php-server-s118 · php-server-s119 · target census riusabili (census patch:
wp119-harness/census-clite.patch) · Serena in hang: Read mirato + Explore.
