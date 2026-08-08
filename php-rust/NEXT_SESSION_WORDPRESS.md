# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)
⏱ **FONDAMENTALI**: rif WP **full ON 1,867 / OFF 1,869** (S-110; lettura S-113
1,871/1,891 in famiglia, debito icache H-A2 ASSOLTO) · media 2,632/2,603 (voce
aperta, in discesa) · ultima leva SPEDITA **S-112 (H-A2)** · sessioni-senza-Δ =
1 (S-113: leva tentata, refutata dal criterio) · incidenti «mai collaudato»: 1
(S-106) · incidenti processo: +1 (S-113 run non-detached, guardia morsa).

## Scoreboard (pin s112 f71abd2a, micro R=5 di S-112 — S-113 non li muove)

**arith 5,5 · prop 7,6 · calls 5,2 · str 5,3 · arr 4,2 · re 3,4** · held-out
6,4·2,5·5,6. 🔬 S-113 ha provato che: (a) le oscillazioni di LAYOUT su giudici
non toccati arrivano a ±5,5 ns/iter (> soglia 4) — ogni prossima soglia è
cieca finché la banda non è MISURATA; (b) su prop una leva lifecycle da 1
clone Rc/op vale ~3,3 ns/iter (direzione firmata 5/5, sotto pavimento): si
compone o si cambia famiglia. Il peggiore resta **prop 7,6**.

## Stato gate

- **phpr pin s112 = f71abd2a16da7e71** (stash `phpr-s112`; release==pin
  VERIFICATO fine S-113 dopo revert al byte, diff crates/ vuoto) — gate del pin
  = quelli S-112 (batteria 1742/0 · corpus 1415×2 · fixture · micro R=5).
  Batteria S-113 sul CANDIDATO refutato: rc=0, 0 fail, **1741 vs 1742 = delta
  conteggio NON attribuito (aperto per NOME)**.
- **php-server 443ae42f ×2** (stash s109 INTATTO; release = relink gemella
  06e6d677 stesso sorgente, dichiarato S-113, non usato dalla coppia CLI) ·
  GitHub sync S-110 (drift TODO.md aperto).

## §S-114 — ordine provvisorio

1. **A/B LEVA-NULLA DI ATTRIBUZIONE LAYOUT (PREREQUISITO, promosso da S-113)**:
   binario con testo INERTE nel run_loop (~+2 KB, cold-branch mai preso o
   padding; admission a dump identici) vs pin, R=5 sulle 6 categorie: la
   **banda-layout si MISURA** (oggi solo punti raccontati: +1,50 S-112 e −5,50
   S-113 su calls non toccato) e ricalibra le soglie di guardia.
2. **LEVA PERF** (ritmo: una per sessione, A/B eseguito qualunque verdetto).
   Candidati con istruttoria PRIMA del criterio: **(h) prop composta** —
   bundle lifecycle multi-sito (2 get H-P1 + doppio borrow del set + recv
   push) con attesa > banda misurata al punto 1; o **(i) op-count su prop**
   (fusione LEGALE entro il primo helper sospendibile, dai dump). Vincoli:
   criterio PRE, soglia max(4; rumore; banda MISURATA), guardie solo-regressione,
   **soglie/mediane A/B A MACCHINA** (az.2 rev. S-113), held-out, §6 pieno.
3. **Delta batteria 1741 vs 1742**: attribuire per NOME (diff inventario
   #[test] come az.3 S-108) — chiude o apre una lettera-gate.
4. Se resta finestra: rumore tra-sere WP (az.2 S-110) · arr 4,2 vs 3,9 rerun.
5. Backlog PRODOTTO (Codex): SOLO su ordine esplicito dell'utente.
6. Chiusura lean: rotazione + revisore singolo (lente: SEMANTICA, a rotazione).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

banda-layout DA MISURARE (punto 1) · delta batteria 1741/1742 per NOME ·
arr 4,2 vs 3,9 (tra-sere) · media voce aperta · rumore tra-sere WP (az.2
S-110) · causa fine frontend (kpc/sudo — azione utente; quota arith da
RIMISURARE post-H-A2) · §3.16/§3.17 righe warning · retro-A/B str
s107b/s108/s109 · denti rinviati (OBS-8; fx20; direct-bind; drop-order;
hit/miss) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 · get_gc ·
drift TODO.md · (b-min) ConcatNFetchDim solo con istruttoria sopra pavimento ·
(g) BinaryDst/CmpJmpConst solo se un census WP li mostra caldi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate nei criteri · magnitudine
ripartita senza A/B proprio · finestre estese senza criterio+dente ·
simple_call allargato senza dente+fx21 · fixture su memory_get_usage (stub) ·
«icache» NON-premessa · pre-filtro/hot-cluster senza forma che non tassi i
freddi · finestre fuse oltre un helper sospendibile · guardie non-bersaglio
BILATERALI · denominatori a memoria · output di run nel repo · rc di gate da
pipe (5 morsi) · tee/log prima del mkdir · admission sul dump intero ·
xctrace senza guardie disco · path del volume esterno nel figlio tracciato
(TCC) · **leve lifecycle mono-clone su prop senza composizione (H-P1
refutata S-113)** · **run pesanti come task di sessione (detached DA SUBITO)**.

---
**Riscritto**: 2026-08-08 (chiusura S-113). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisioni in `wp1*-harness/`.

Pre-flight S-114: pin phpr **f71abd2a** (release AL pin, verificato fine
S-113) · server 443ae42f (stash s109; release relink 06e6d677 dichiarato) ·
MySQL wp8 con l'elenco database · debug/ da rimuovere · uploads sotto guardia ·
nessuna run in volo · disco Data ~16G (guardie xctrace s110/s111) · held-out 6,4·2,5·5,6.
