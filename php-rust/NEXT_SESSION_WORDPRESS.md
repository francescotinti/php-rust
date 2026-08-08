# NEXT_SESSION — phpr ≤ 3× l'oracle (processo LEAN v1: la lista unica è REGOLE.md)
⏱ **FONDAMENTALI**: rif WP **full ON 1,867 / OFF 1,869** (S-110) · media
2,632/2,603 (voce aperta, in discesa) · ultima leva SPEDITA **S-112 (H-A2)** ·
sessioni-senza-Δ = 2 (S-113 H-P1 e S-114 L-A: entrambe tentate con A/B pieno e
verdetto) · incidenti «mai collaudato»: 1 (S-106) · incidenti processo: +3
S-114 (batteria contaminata da edit concorrente · redirect pre-mkdir ·
checkout-staging di run.rs, corretto index-only).

## Scoreboard (pin s112 f71abd2a, micro R=5 di S-112 — S-113/114 non li muovono)

**arith 5,5 · prop 7,6 · calls 5,2 · str 5,3 · arr 4,2 · re 3,4** · held-out
6,4·2,5·5,6. 🔬 S-114 ha MISURATO: (a) **banda-layout: arith 0,40 · prop 4,33 ·
calls 5,50 · str 5,00 · arr 6,67 · re 0,00 → globale 6,67** (leva-nulla, N=1
perturbazione; calls −5,50 = replica ESATTA del morso S-113: era layout; una
leva nulla fa 5/5 segni concordi su 3 categorie); (b) **L-A trigramma prop:
+30,33 mediano 5/5** — direzione FIRMATA ben oltre banda, magnitudine NON
stabilita (2 run del PIN a ~150 vs ~107 → spread_A 47 → soglia 47 FAIL;
guardia calls −6,50 in famiglia layout). Il peggiore resta **prop 7,6**.

## Stato gate

- **phpr pin s112 = f71abd2a16da7e71** (stash `phpr-s112`; release==pin
  VERIFICATO ×3 in S-114 dopo ogni revert, diff crates/ vuoto) — gate del pin
  = S-112 (batteria 1742/0 · corpus 1415×2 · fixture · micro R=5). **Batteria:
  inventario 1744 nomi CONSERVATO nei raw S-114** (1742 ok + 2 ignored:
  a_ds9_supersede_scan_cost_bound, rendered_null_array_offset_deprecation);
  delta 1741 S-113 CHIUSO (one-off ambientale, non il codice: tree H-P1
  ricostruito dà 1742 con inventario identico). Candidati conservati:
  `phpr-s114-nulla` 846d0df4 · `phpr-s114-la` 052ea417.
- **php-server 443ae42f ×2** (stash s109 INTATTO; release relink 06e6d677
  dichiarato S-113) · GitHub sync S-110 (drift TODO.md aperto).

## §S-115 — ordine provvisorio

1. **LEVA PERF = L-A RIPETUTA con criterio EMENDATO** (keep-partial-wins: la
   direzione è firmata, si rimisura il quanto). Emendamenti da pre-registrare
   nel criterio PRIMA della run: (a) **scheduling osservabile** — esclusione
   per NOME pre-registrata delle run fuori famiglia (es. lato>1,3× la mediana
   della PROPRIA serie, dichiarata run per run dallo script) O affinità/QoS se
   praticabile; (b) soglia promozione max(4; spread_A DEPURATO; banda 4,33);
   (c) guardie con banda S-114; (d) R=7 se resta rumore. Il codice è pronto:
   commit 2c18b2e (revert f6fbf6a), candidato conservato phpr-s114-la.
2. **Seconda perturbazione leva-nulla** (banda N=2, ~1 ora): variante zavorra
   (taglia o posizione diversa) per irrobustire banda(cat) prima di L-B
   dispatch — pesa su ogni soglia futura.
3. Se L-A promossa: pin SOLO via `scripts/pin-phpr.sh` §6 PIENO + micro R=5
   nuovi + held-out.
4. Se resta finestra: rumore tra-sere WP (az.2 S-110) · arr rerun. Backlog
   Codex SOLO su ordine esplicito. Chiusura lean: revisore (lente PROCESSO).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

L-A criterio emendato (punto 1) · banda N=2 (punto 2) · bimodalità run
(~107/150 pin, 78/102 candidato — ipotesi P/E-core da rendere osservabile) ·
arr 4,2 vs 3,9 (tra-sere) · media voce aperta · rumore tra-sere WP · causa
fame frontend (kpc/sudo — azione utente) · §3.16/§3.17 righe warning ·
retro-A/B str s107b/s108/s109 · denti rinviati (OBS-8; fx20; direct-bind;
drop-order; hit/miss) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 ·
get_gc · drift TODO.md · (b-min) ConcatNFetchDim · (g) BinaryDst/CmpJmpConst
solo se census WP li mostra caldi · dente checkout-staging (show --stat dopo
ogni checkout parziale).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

pin/stash senza collaudo-nell'atto · contenitori sul call path · differenze
tra A/B distinti come cifra · componenti prezzate nei criteri · magnitudine
ripartita senza A/B proprio · finestre estese senza criterio+dente ·
simple_call allargato senza dente+fx21 · fixture su memory_get_usage (stub) ·
«icache» NON-premessa · pre-filtro/hot-cluster senza forma che non tassi i
freddi · finestre fuse oltre un helper sospendibile · guardie non-bersaglio
BILATERALI · denominatori a memoria · output di run nel repo · rc di gate da
pipe (5 morsi) · tee/log prima del mkdir (6° morso S-114) · admission sul dump
intero · xctrace senza guardie disco · path del volume esterno nel figlio
tracciato (TCC) · leve lifecycle mono-clone su prop senza composizione ·
run pesanti come task di sessione (detached DA SUBITO) · **edit ai sorgenti
con una batteria/build in volo (contaminazione S-114)** · **promozione con
segni concordi ma sotto banda-layout (la concordanza non firma)**.

---
**Riscritto**: 2026-08-08 (chiusura S-114). Storia: `sessions/` ·
`gaps/GAP_TREND.md` · revisioni in `wp1*-harness/`.

Pre-flight S-115: pin phpr **f71abd2a** (release AL pin, verificato fine
S-114) · server 443ae42f (stash s109; release relink 06e6d677) · MySQL wp8
con l'elenco database · debug/ da rimuovere · uploads sotto guardia · nessuna
run in volo · disco Data ~15G (guardie xctrace) · held-out 6,4·2,5·5,6 ·
candidati conservati phpr-s114-{nulla,la}.
