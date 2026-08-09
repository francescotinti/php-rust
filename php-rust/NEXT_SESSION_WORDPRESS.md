# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full = 1,810–1,889** (16 celle S-120; mediano
~1,85) · **media ~2,51–2,53 · peak 1862–1983 MiB** · ultima leva SPEDITA
**S-120 (L-RE1)** · **sessioni-senza-Δ = 1** (S-121: leva TENTATA e refutata
dalla regola pre-registrata — ritmo rispettato, promozione no) · incidenti:
1 (S-106) + 1 processo (S-115).

## Scoreboard (pin s120 **885d2c64** @ 86306c3 INVARIATO; micro di S-120)

**arith 5,5 · prop 5,5 · calls 4,8 · str 5,3 · arr 3,7 · re 2,8** (non
rimisurate in S-121: pin fermo). S-121 ha STABILITO: (a) **L-ST1 FERMATA
(refutazione PROVVISORIA — az. rev.: banda 2,50 = 1 quanto, asimmetrica)** —
scratch args-Vec CallBuiltin: census str 5→4 ESATTO ma smoke str −5,00/−7,50
concorde ⇒ early-stop p.7 (PRIMA applicazione) + revert p.9; due ipotesi
APERTE: bookkeeping > malloc 32 B O layout; (b) **grado PIENO server s120**
off+on rc=0 voids=0 (option 413 + restapi 3508 per NOME; re-pin al byte);
(c) **ABAB s119↔s120**: L-RE1 su WP NON ripartibile (segno 2/2 s120 più
lento, DENTRO spread intra-pin 18,73 s; failnames 4/4); (d) az. rev. S-120
4/4 (gate preg `s121-fx-preg-gate.sh` + §3.18; colonna arr D2 +2,02/op-int).

## §S-122 — ordine proposto

1. **Banda-LAYOUT micro + full A/B L-ST1 dallo stash** (az. rev. S-121):
   banda tra-binari con no-op FIRMATO ricompilato ×N (criterio PRIMA); poi
   full L-ST1 (stash 2e1eda8d, costo basso) = refutazione-vs-layout chiusa;
   interim: early-stop str usa max(zavorra; 2×quanto = 5,00).
2. **Leva prop-cloni** (classifica: 5 cloni Zval/iter, 1 Rc — ciclo-di-vita,
   non alloc): census su pin s118 già in mano; istruttoria sito-per-sito che
   SOMMA, poi criterio col bersaglio prop (banda-v2 3,33).
3. **re residuo 10→8** (arg-Vec CallHostBuiltinOut): VINCOLATA da (1) e
   dalla refutazione L-ST1 — lo swap take/restore è refutato; serve un
   meccanismo senza swap (borrow disgiunto dei campi / refactor flush_diags).
   PRIMA: cablare s121-fx-preg-gate.sh nella catena fixture (az. rev. #4).
4. **Stadio-2 PGO** (criterio §6 pre-registrato in s117-criterio-aprime.md;
   KS-A2: profdata non riproducibile ⇒ resta LTO).
5. **arr**: SOLO istruttoria su D2 (+2,02/op-int; indiziata la chiave "k$i");
   nei census-verdetti anche le predizioni secondarie a verbale (az. rev.).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

banda-layout micro (punto 1) · L-ST1b senza swap (vincolata al punto 1) ·
cura §3.18 ((?J) dup-names + prefisso sintetico: alla cura il gate preg
diventa ROSSO, golden aggiornati nello stesso commit) · prop residuo 5
cloni/iter · A′ da sola PEGGIORAVA prop (−7,67, 0/5): non indagato · Serena
HANG su questo repo (Read mirato + Explore) · fame frontend (kpc/sudo —
azione utente) · §3.16/§3.17 warning · retro-A/B str s107b/s108/s109 · denti
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
nei sorgenti del pin · **leve micro ≤10 ns/iter senza banda-layout misurata**
· **zavorra run-to-run come arbitro del layout**.

---
**Riscritto**: 2026-08-09 (chiusura S-121). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisione in `wp121-harness/revisione.md`.

Pre-flight S-122: pin phpr **s120 885d2c64**6ac7ff4c @ 86306c3 (ricetta A′:
`[profile.release]` + `SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0`; la batteria
rilinka ⇒ build ricetta e pretendere 885d2c64 al byte) · server **s120
6b822369**89a4a0c4 GRADO PIENO (S-121) · MySQL wp8 con l'elenco (se giù:
mysqld_safe daemonizzato su datadir esterno) · uploads sotto guardia ·
conservati: phpr-s118 · phpr-s119 · phpr-s119-treno2 · phpr-s120-re1 (==pin
s120) · phpr-s121-st1 (REPERTO refutato, NON è un pin) · php-server-s119 ·
php-server-s120 · census target riusabile (wp119-harness/census-clite.patch)
· Serena in hang: Read mirato + Explore.
