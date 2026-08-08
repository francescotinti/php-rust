# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)
⏱ **FONDAMENTALI**: rif WP **full ON 1,867 / OFF 1,869** (S-110, banda [1,81;1,88])
· media 2,673/2,612 (voce aperta) · ultima leva di codice SPEDITA **S-112 (H-A2:
arith −41%)** · sessioni-senza-Δ-rapporti = 0 · incidenti «mai collaudato»: 1
(de67cb64, S-106).

## Scoreboard (pin s112 f71abd2a, micro R=5 di S-112)

**arith 5,5 · prop 7,6 · calls 5,2 · str 5,3 · arr 4,2 · re 3,4**
(vs S-109: arith 9,3→5,5 leva H-A2 · prop 7,9→7,6 · arr 3,9→4,2 entro
tra-sere ±0,4, A/B fermo — rerun con la coppia). HELD-OUT S-112: **poly 6,4
(MIGLIORATO da 6,7) · err 2,5 · wploop 5,6**. 🔬 Da sfruttare: censire i
BinOp/sentieri ancora FUORI dal fast-path sui corpi caldi (il −41% era un
hoisting di poche righe). Il peggiore ora è **prop 7,6**.

## Stato gate

- **phpr pin s112 = f71abd2a16da7e71** @ HEAD (stash `phpr-s112` da
  pin-phpr.sh; release == pin verificato) — batteria 1742/0 rc=0 (relink
  gemella b70e049a dichiarato, stesso sorgente) · corpus **1415 per NOME ×2**
  (fresco S-112) · fixture chain rc=0 (hc1+move+recv+fx20+fx21+w9) ·
  run_loop 291.316 B (+3.372 dichiarati, bl 5864, br 24) · flag-ON ·
  oracle 07b0df8d. Held-out baseline AGGIORNATA: 6,4·2,5·5,6
  (wp112-harness/heldout-out/candidato.out).
- **php-server 443ae42f ×2** (stash s109; NON toccato in S-112) · GitHub
  sync S-110 (drift TODO.md aperto).

## §S-113 — ordine provvisorio

1. **COPPIA WP BIMODALE** (DOVUTA: collaudo icache del +3.372 B di H-A2;
   criterio PRE con banda [1,81;1,88]; parità per NOME ×2; raw fuori repo).
2. **LEVA PERF** (ritmo: una per sessione). Candidati con istruttoria PRIMA
   del criterio: **(f) census dei sentieri fuori-fast-path su prop** (il
   peggiore 7,6: PropGetSlotRecv/BinaryTCPropSetPop — l'IC path e la coda TC
   vanno letti nei dump/braccio come in S-112) o **(g) BinOp residui fuori
   da binary_fast sui corpi caldi degli altri giudici** (metodo S-112
   generalizzato). Vincoli invariati: criterio PRE con soglia max(4;
   rumore; banda-layout) + guardie non-bersagli A SOLO-REGRESSIONE (lezione
   S-112) + held-out a leva conclusa su soglia PRE + disasm bl-count +
   admission bipartita + commit+push + **ordine §6 PIENO (az.5 rev. S-112:
   batteria→re-hash→stash sullo STESSO binario, o deroga dichiarata norma)**.
3. Se resta finestra: rumore tra-sere WP (az.2 S-110) o A/B pre-filtro vuoto.
4. Backlog PRODOTTO (Codex): SOLO su ordine esplicito dell'utente.
5. Chiusura lean: rotazione + revisore singolo (lente: MISURA, riparte).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

arr 4,2 vs 3,9 (tra-sere: rerun con la coppia S-113) · banda-layout RESTA
0,67 (az.2 rev. S-112: +1,50 calls non misurato — A/B leva-nulla prima di
promuoverlo) · media voce aperta · rumore
tra-sere WP (az.2 S-110) · A/B pre-filtro VUOTO (az.2 rev. S-111) · causa
fine frontend (kpc/sudo — azione utente; dopo H-A2 la quota su arith va
RIMISURATA prima di investirci) · §3.16/§3.17 righe warning · retro-A/B str
s107b/s108/s109 · denti rinviati (OBS-8; fx20; direct-bind; drop-order;
hit/miss) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 · get_gc ·
drift TODO.md (gh-status-sync) · (b-min) ConcatNFetchDim solo se
un'istruttoria nuova alza l'attesa sopra il pavimento.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate nei criteri · magnitudine
ripartita senza A/B proprio · finestre estese senza criterio+dente ·
simple_call allargato senza dente+fx21 · fixture su memory_get_usage (stub) ·
«icache» NON-premessa · pre-filtro/hot-cluster senza forma che non tassi i
freddi · finestre fuse oltre un helper sospendibile · guardie non-bersaglio
BILATERALI (S-112: solo-regressione) · denominatori a memoria · output di
run nel repo · rc di gate da pipe (**5 morsi**: S-112 fixture chain,
rieseguita col rc dal comando) · tee/log prima del mkdir · admission sul
dump intero · xctrace senza guardie disco · path del volume esterno nel
figlio tracciato (TCC).

---
**Riscritto**: 2026-08-08 (chiusura S-112). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisioni in `wp1*-harness/`.

Pre-flight S-113: pin phpr **f71abd2a** (HEAD fa fede; release AL pin,
verificato fine S-112) · server 443ae42f (stash s109) · MySQL wp8 con
l'elenco dei database · debug/ da rimuovere se rigenerata · uploads sotto
guardia · nessuna run in volo · disco Data ~16G (stretto: xctrace SOLO con
guardie s110/s111) · held-out baseline 6,4·2,5·5,6 (wp112-harness/).
