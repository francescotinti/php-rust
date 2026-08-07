# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)

⏱ **FONDAMENTALI**: riferimento WP = **full 1,894×** (WP-102, citabile) ·
media 2,64× · **coppia WP DOVUTA in S-108 (debito, run_loop +16,4 KB da
verificare sull'aggregato)** · ultima leva = S-107 (lotto superistruzioni,
4/6 sopra soglia; arr/re direzione firmata) · sessioni-senza-Δ-rapporti =
0 · incidenti «mai collaudato»: 1 (de67cb64, S-106).

## Scoreboard (pin 62a4df65, micro R=5, S-107)

**arith 9,7 · prop 8,5 · calls 5,3 · str 6,2 · arr 3,9 · re 3,4**
Il conto del target: oracle arith ≈8,6 ns/iter ⇒ 3× ≈ 26; phpr ≈ 83.
Mancano ~57 ns/iter su arith: il corpo è già 5 op/iter — i prossimi colpi
sono dispatch (threaded) e ciclo di vita Zval, non altre peephole.

## Stato gate

- **phpr pin 62a4df65578282b5** @ HEAD S-107 (stash `phpr-s107b`; contiene
  lotto superistruzioni + cura §3.15) — batteria 1739/0 rc=0 (inventario
  #[test] 1767, =S-106) · corpus **1415 per NOME ×2** (lista wp82
  aggiornata −2 in 3223150) · ORM 3E/13F per NOME · hk 0E/0F · fixture ×5
  (fx21 golden ora IDENTICI oracle/phpr) · run_loop 274.192 B · default
  flag-ON · oracle 07b0df8d (8.5.7). Pin intermedio b4b1a87d (stash
  phpr-s107, solo lotto) resta per retro-A/B.
- **php-server dde2a64d GRADATO PIENO** (S-106) ma runtime PRE-lotto: per
  CIFRE server serve regrade (scripts/pin-server.sh + grado); per la
  coppia CLI basta il pin phpr.
- Census: build strumentata separata (phpr-census-target, feature
  op-census); dump per giudice in wp107-harness (fuori repo).

## §S-108 — ordine provvisorio (azioni revisore S-107 recepite)

1. **COPPIA WP FULL+MEDIA BIMODALE in APERTURA** (debito + azione-4
   revisore: è il test dell'ipotesi icache del +16,4 KB e BLOCCA la leva
   successiva se fuori banda, REGOLE §4). Bande: full [1,86;1,93] su
   WP-102; media = solo direzione (voci S-105 fuori banda: prima rerun).
2. **Rerun A/B mirato arr e re** (azione-2: rumore abbattuto — misura in
   ns/inner-iter con N interno maggiorato) prima di citarne cifre; nello
   stesso giro, prova d'identità .text a0543213↔b4b1a87d o marca «build
   gemella» a verbale (azione-5) e diff per NOME degli eseguiti di
   batteria fra pin S-106 e S-107 (azione-3, `cargo test -- --list`).
3. **Leva S-108: threaded dispatch O census secondo giro** — rifare il
   census sul pin nuovo (i bigrammi residui dopo il lotto sono un'altra
   classifica) e scegliere fra (a) nuove finestre se restano coppie ≥5%
   e (b) riesame threaded-dispatch CON misura (candidato da S-106).
   Criterio pre-registrato PRIMA, come da REGOLE §3.
4. Se la finestra regge: **regrade server** (pin-server.sh + grado) —
   sblocca cifre server attribuibili.
5. Chiusura lean: rotazione + revisore singolo (lente: SEMANTICA).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

retro-A/B prop coi due stash s106/s107 (riparte il −0,9 di S-106) ·
Neg-fold su PushConst;Unary (richiede append a f.consts nel pass: −1
op/iter su str) · voci coppia S-105 fuori banda (full-off 1,947; media
2,697/2,734: rerun) · denti rinviati (OBS-8 terza mutazione; mutante
fx20; dente direct-bind; dente drop-order; contatore hit/miss) ·
fedeltà: $z++/$z-- su undefined non warna (catalogare o curare) ·
§3.13 · §3.12-i · §3.14 · get_gc · contatori L1I · text-budget run_loop
da ricontrattare se la crescita continua (+16,4 KB in S-107).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path ·
differenze tra A/B distinti come cifra · componenti prezzate nei criteri ·
magnitudine ripartita senza A/B proprio · estensioni di finestre senza
criterio+dente · allargare simple_call senza dente+fx21 · fixture su
memory_get_usage (stub) · «icache-bound» come premessa firmata ·
denominatori a memoria · output di run nel repo · rc di gate da pipe ·
tee/log prima del mkdir.

---
**Riscritto**: 2026-08-07 (chiusura S-107). Apertura/chiusura = skill
`apri-sessione`/`chiudi-sessione` v2. Storia: `sessions/` ·
`gaps/GAP_TREND.md` · concili in `wp107/wp108-harness/`.

Pre-flight S-108: pin phpr **62a4df65** (fa fede HEAD, la build churna) ·
MySQL wp8 con elenco DB · debug/ da rimuovere · uploads sotto guardia ·
nessuna run in volo.
