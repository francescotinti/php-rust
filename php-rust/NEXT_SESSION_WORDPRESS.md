# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)

⏱ **FONDAMENTALI**: riferimento WP = **full ON 1,855× / OFF 1,885×** (S-108,
stessa-sera su pin s107b; WP-102 1,894 resta il riferimento storico) · media
2,677/2,747 (voce APERTA) · **coppia WP DOVUTA in S-109 sul pin NUOVO
(debito: lotto-2 = run_loop +14,7 KB da collaudare sull'aggregato)** ·
ultima leva = S-108 (lotto-2: prop sopra soglia, 3 direzioni firmate) ·
sessioni-senza-Δ-rapporti = 0 · incidenti «mai collaudato»: 1 (de67cb64, S-106).

## Scoreboard (pin 3b3d25e2, micro R=5, S-108)

**arith 9,4 · prop 8,0 · calls 5,3 · str 6,2 · arr 3,8 · re 3,5**
Il conto del target: oracle arith ≈8,6 ns/iter ⇒ 3× ≈ 26; phpr ≈ 80.
Il corpo arith è 4 op/iter (1 sola op di lavoro + Sweep + 2 di loop): i
colpi restanti sono dispatch (threaded, gated dai contatori L1I), ciclo di
vita Zval (Sweep) e il costo interno dei funnel — non altre peephole su arith.

## Stato gate

- **phpr pin 3b3d25e2a467c8b0** @ HEAD S-108 (stash `phpr-s108`; = il binario
  dell'A/B, conservato) — batteria 1740/0 rc=0 · corpus **1415 per NOME ×2** ·
  fixture ×5 rc=0 · ORM 3484 3E/13F per NOME · hk 1665 0E/0F · run_loop
  288.920 B (bl 29 invariato) · default flag-ON · oracle 07b0df8d. Stash
  62a4df65 (s107b) resta per retro-A/B.
- **php-server dde2a64d GRADATO ma PRE-lotti** (S-106): per CIFRE server serve
  regrade (scripts/pin-server.sh + grado); release churnato (aa8cc563) NON
  stashato, dichiarato.
- Census: build strumentata separata (phpr-census-target, op-census); dump
  S-108 in wp108-harness/census-out (fuori repo).

## §S-109 — ordine provvisorio

1. **COPPIA WP FULL+MEDIA BIMODALE in APERTURA sul pin 3b3d25e2** (debito
   lotto-2; criterio PRIMA: full ON fuori banda sopra 1,93 BLOCCA la leva;
   riferimenti 1,855 (S-108) / 1,894 (WP-102); media = direzione, terza
   lettura della voce aperta).
2. **Regrade server** (pin-server.sh + grado): sblocca le cifre server —
   rinviato per NOME da S-108.
3. **Leva S-109: census terzo giro** sul pin nuovo e scelta per NOME fra i
   residui (W12 CallHostBuiltinOut+JumpIfFalse differita; Neg-fold str via
   consts-append; RMW-su-dim arr FetchDim+BinarySTDst; PropGetSlot+BinarySTDst
   prop) O **contatori L1I** (prerequisito del threaded dispatch, S-106).
   Criterio pre-registrato PRIMA, come da REGOLE §3.
4. Azioni del revisore S-108 (wp108-harness/revisione.md) in coda d'apertura.
5. Chiusura lean: rotazione + revisore singolo (lente: PROCESSO).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

media fuori banda (2,677 on / 2,747 off: terza lettura con la coppia S-109) ·
dente zval-census: entrare in un gate COMPILATO o dichiararlo dormiente
(lettera sanata in S-108 dopo 2 sessioni di rottura silenziosa) · retro-A/B
prop coi due stash s107b/s108 · denti rinviati (OBS-8 terza mutazione;
mutante fx20; dente direct-bind; dente drop-order; contatore hit/miss) ·
fedeltà: $z++/$z-- su undefined non warna · §3.13 · §3.12-i · §3.14 ·
get_gc · contatori L1I · text-budget run_loop (288.920 B; +31,1 KB in due
sessioni: si ricontratta se la crescita continua).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate nei criteri · magnitudine
ripartita senza A/B proprio · estensioni di finestre senza criterio+dente ·
allargare simple_call senza dente+fx21 · fixture su memory_get_usage (stub) ·
«icache-bound» come premessa firmata · denominatori a memoria · output di
run nel repo · rc di gate da pipe (morso 3 volte: vale anche per gli echo
di verifica) · tee/log prima del mkdir · **finestre fuse OLTRE un helper
sospendibile (prop_get/prop_set entry: il frame __get/hook tronca la
finestra — vincolo S-108)**.

---
**Riscritto**: 2026-08-07 (chiusura S-108). Apertura/chiusura = skill
`apri-sessione`/`chiudi-sessione` v2. Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisioni in `wp10*-harness/revisione.md`.

Pre-flight S-109: pin phpr **3b3d25e2** (fa fede HEAD, la build churna) ·
MySQL wp8 con elenco DB · debug/ da rimuovere · uploads sotto guardia ·
nessuna run in volo.
