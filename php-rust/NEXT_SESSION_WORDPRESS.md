# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)
⏱ **FONDAMENTALI**: rif WP **full ON 1,867 / OFF 1,869** (S-110) · media
2,632/2,603 (voce aperta) · ultima leva SPEDITA **S-112 (H-A2)** ·
sessioni-senza-Δ = 3 (S-113/114/115: tutte con leva TENTATA ad A/B pieno) ·
incidenti «mai collaudato»: 1 (S-106) · incidenti processo: +1 S-115 (BUILD_RC
letto dopo pipe — dichiarato, mai usato come gate).

## Scoreboard (pin s112 f71abd2a, micro R=5 di S-112 — S-113/114/115 non li muovono)

**arith 5,5 · prop 7,6 · calls 5,2 · str 5,3 · arr 4,2 · re 3,4** · held-out
6,4·2,5·5,6. 🔬 S-115 ha STABILITO: (a) **L-A magnitudine +26,33 ns/iter su
prop, 5/5, spread_A depurato 2,00** (famiglie 1,3×min: il metro inquinato S-114
RECUPERATO senza toccare la leva) — NON promossa SOLO per held-out poly
9,86>9,71; (b) gate held-out REFUTATO come DIAGNOSTICO: la nulla-2 fa poly 9,80
(0,20s, N=1) e sfonderebbe lo stesso gate a ZERO semantica; (c) **banda micro N=2:
max = arith 0,40 · prop 4,33 · calls 5,50 · str 5,00 · arr 6,67 · re 10,00**
(re 0→10: banda N=1 mente; globale 10,00 stabile <13,34). Il peggiore resta
**prop 7,6**; L-A promossa lo abbatterebbe (~26 su ~107 lato pin).

## Stato gate

- **phpr pin s112 = f71abd2a16da7e71** (stash `phpr-s112`; release AL pin,
  verificato fine S-115, diff crates/ vuoto) — gate S-112 (batteria 1742/0 ·
  corpus 1415×2 · fixture · micro R=5). Candidati CONSERVATI: `phpr-s114-la`
  052ea417 (codice 2c18b2e) · `phpr-s114-nulla` 846d0df4 · `phpr-s115-nulla2`
  d9093a6b (patch s115-zavorra2.patch).
- **php-server 443ae42f ×2** (stash s109; relink 06e6d677) · GitHub sync S-110
  (drift TODO.md aperto). Batteria 1741 S-113: flaky non escluso.

## §S-116 — ordine provvisorio

1. **LEVA PERF = PROMOZIONE L-A**: rieseguire il criterio L-A PER INTERO
   (REGOLE §3) con le SOLE modifiche (azioni revisore S-115): (a) banda micro
   = max N=2 (riga (c), incl. re 10,00); (b) soglia held-out = baseline +
   max(2×spread; 0,12; banda_heldout 0,20/0,01/0,06 — N=1 dichiarato), con
   «spread» DEFINITO = max(pin, candidato) ed ENTRAMBI pubblicati nei raw;
   (c) tie ESATTI sulle soglie pre-registrati; (d) rc di gate scritti dagli
   script in FILE (mai da echo/pipe), esiti nei verbali appesi A ESITO
   ACQUISITO; (e) resto INVARIATO (famiglie 1,3×min, parità output, admission
   hit/miss — binari conservati, zero rebuild). Smoke R=2 prima del full.
2. Se PROMOSSA: cherry-pick 2c18b2e (MAI checkout parziale) + pin SOLO via
   `scripts/pin-phpr.sh` §6 PIENO + micro R=5 + held-out sul pin nuovo.
3. Finestra: batteria N≥3 su 8bb395c · rumore tra-sere WP · arr rerun. Codex
   solo su ordine. Chiusura lean: revisore (lente MISURA, dopo PROCESSO S-115).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

L-A promozione (p.1) · banda held-out N=1 da irrobustire · batteria N≥3 su
8bb395c ·
bimodalità P/E-core (le famiglie la gestiscono, resta non attribuita) · arr
4,2 vs 3,9 (tra-sere) · media voce aperta · rumore tra-sere WP · fame frontend
(kpc/sudo — azione utente) · §3.16/§3.17 warning · retro-A/B str s107b/s108/
s109 · denti rinviati (OBS-8; fx20; direct-bind; drop-order; hit/miss;
checkout-staging) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 ·
get_gc · drift TODO.md · (b-min) ConcatNFetchDim · (g) BinaryDst/CmpJmpConst
solo se census WP li mostra caldi.
## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate · magnitudine ripartita senza
A/B proprio · finestre estese senza criterio+dente · simple_call senza dente+
fx21 · fixture su memory_get_usage · «icache» NON-premessa · pre-filtro che
tassi i freddi · finestre fuse oltre helper sospendibile · guardie non-bersaglio
BILATERALI · denominatori a memoria · output di run nel repo · rc di gate da
pipe (6 morsi: +BUILD_RC S-115) · tee/log pre-mkdir · admission sul dump intero
· xctrace senza guardie disco · path volume esterno nel figlio tracciato · leve
lifecycle mono-clone su prop · run pesanti come task (detached DA SUBITO) ·
edit ai sorgenti con build/batteria in volo · promozione su segni concordi
sotto banda · **gate a soglia fissa su giudice SENZA banda misurata**.

---
**Riscritto**: 2026-08-08 (chiusura S-115). Storia: `sessions/` · `gaps/GAP_TREND.md` · revisioni in `wp1*-harness/`.

Pre-flight S-116: pin phpr **f71abd2a** (release AL pin, verificato fine
S-115) · server 443ae42f (stash s109; relink 06e6d677) · MySQL wp8 con elenco
database · debug/ da rimuovere · uploads sotto guardia · nessuna run in volo ·
disco Data ~12G (bundle VM Claude Desktop RIMOSSO su ordine utente, fine
S-115) · held-out 6,4·2,5·5,6 · candidati
phpr-s114-{la,nulla} + phpr-s115-nulla2 · banda micro N=2 (riga (c)) +
held-out N=1 (0,20/0,01/0,06).
