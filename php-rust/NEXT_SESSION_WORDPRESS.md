# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **⚖️ A1 CHIUSA in S-152: sonde-prezzo + GO/NO-GO A3c =
NO-GO ⇒ A3c CHIUSA a verdetto** (stile veto NaN-boxing; «robusto»
QUALIFICATO dalla revisione: maggiorante nel modello hot-hot + cuscinetto
pair-per-evento ~2× pro-GO ~1,1 s, criterio §9; riapribile SOLO con
sonde working-set ~180k slot su ENTRAMBI i lati oltre soglia): banda_netta
[1,109;1,324] s < S2 1,53 · S3 lorda [3,627;3,995] s < 4,58, entrambe
all'estremo FAVOREVOLE ad A3 (mock LB-ottimistico) — restano **A3a/A3b
micro-judged** e leve per NOME · prezzi (4 repliche, probe d90dcdc6f9f154af
×2 in sonda-prep/+stash): c2_borrow 4,27–4,41 · c2_borrow_mut 1,68 ·
c1_pair 5,43–5,46 · mock_deref 1,13 ns · **testa hostcall QUIET Δ=0
(82.211.532 ×2) = CITABILE** · pesca outlier ESAURITA a scala suite (bt
k=45 alloc/call ≈473k chiamate; ce k=2 ≈4,85M; testa intera ~0,6–1,0 s) ·
+3,2% lato sorgente CHIUSO (auto-conteggio refutato) · leve spedite S-152:
0 (tentativo d'obbligo = GO/NO-GO eseguito a verdetto) · incidenti **19** (=).
**QUESITI UTENTE**: 1) con A3c chiusa, perimetro A2 ridotto a **T2-only**
(mod.rs zone teardown/sweep; T3 run.rs DECADE, T1 solo se una fetta la
chiede) — ratifica; 2) A3.0 sweep-preserving resta adottato per le fette
A3a/b (dissenso agli atti); 3) gamba SERVER census slittata ANCHE in S-152
(dichiarato): commissionarla o lasciarla in coda?

## Scoreboard (pin s150 INVARIATO phpr cbbe71735effb165 + server 18c2740774336c82)
**arith 5,5 · prop 5,5 · calls 4,8 · str 4,3 · arr 3,3 · re 2,5** (rif.
promo s150, non rimisurate) · WP t4 MEDIANA 1,781 · media 2,480–2,555 ·
**ORM 7,104–7,149 · dbal 7,283–7,491** · corpus **1412×2** · census s151
(conteggi) + prezzi s152: il residuo prezzato più grosso è il **borrow C2
nei siti teardown/sweep ~0,91–1,09 s netti** (frame_teardown.borrow 61,0M ·
PropSetPop.borrow 57,4M · Sweep.borrow 50,0M su 340,9M canale).

## §S-153 — ordine (A3a/b micro-judged coi numeri di S-151/152)
1. **Leva famiglia borrow-teardown (A3a candidata)**: i 3 siti dominanti
   valgono ~170M borrow ≈ 0,55 s (al bordo della scala-suite 2×0,26–0,30);
   istruttoria a sonda conteggi per SITO → criterio pre-registrato (giudice
   micro dedicato o objchurn; soglia max(4; rumore; banda-layout)) → A/B.
   Forma candidata: borrow hoistato/unificato nel teardown (oggi più borrow
   dello stesso handle per frame), SENZA toccare lo store (A3c chiusa).
2. **Se resta finestra**: fetta micro per NOME dalla coda (BT2-alloc su
   m-backtrace 5,50× — chiavi statiche + ZStr condivisi + presize, attesa
   ~30 alloc/call; oppure class_exists lookup case-insensitive senza alloc).
3. **Touch-map/A2**: solo DOPO la ratifica del quesito 1 (T2-only);
   `wp152-harness/s152-touchmap-a2.md` è la mappa vincolante.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
famiglia borrow-teardown C2 (~0,91–1,09 s) · BT2-alloc m-backtrace ·
class_exists 2 alloc/call · get_declared_classes 4,6M (conteggio k dovuto) ·
gamba server census · §3.24 debug_print_backtrace + §3.23 fx-backtrace
estesa (fedeltà, in coda) · confine slot-load 0,99× · §3.22 · depr.
float→int · warning corsia ×2 · div. RMW · objmap 43,4 → piano GC ·
evalcls 316,9× · refl 42,4× · re +2 alloc · §3.13/§3.12-i/§3.14/§3.21 ·
get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-152: A3c in QUALUNQUE forma senza numeri nuovi oltre soglia (verdetto
s152-gonogo) · leve debug_backtrace/class_exists a scala SUITE (solo fette
micro-judged) · calcolatori di verdetto senza collaudo o emenda dichiarata
(path con SPAZI; assunzioni d'ordine sui prezzi).** S-151: asset senza
path+hash · cifre census pre-BT1 nei criteri · ObjectId Copy-senza-refcount ·
arena contigua/bump-reset · spacchettare exec/ops_* di run_loop · crate
nuovi in A2 · Fase-5 registri (veto WP-44). S-150: A/B fuori ricetta senza
gemello+ricetta · attese senza pavimento · forge coi rami non collaudati ·
revert FR1. S-149: prezzi pair senza banda · splitoff3 senza replica ·
banda su finestra singola. S-148/147/146: kill al perimetro dei tag ·
borrow-first sul ponte · TakeSlot in OGNI forma · B1/B2 senza concilio.
Trasversali: BOLT/NaN-boxing/threaded-dispatch/PGO-sui-giudici · pin/stash
senza collaudo-nell'atto · differenze tra A/B distinti come cifra ·
componenti prezzate · denominatori a memoria · rc da pipe · run pesanti
come task · edit coi build in volo · promozione sotto banda ·
strumentazione nei sorgenti del pin · claim di ASSENZA oltre risoluzione ·
misure con LSP in volo · lock con trap altrui · giudice sotto-risoluto ·
byte-identità come gate di edit .rs post-pin.
**Riscritto** 2026-08-18 (chiusura S-152); storia in `sessions/` · `gaps/`.
Pre-flight S-153: pin phpr **s150 cbbe7173**5effb165 + server **18c27407**
74336c82 (SOLO via pin-*.sh) · MySQL wp8 con l'elenco · uploads sotto
guardia · corpus **1412×2** · batteria = s125+rczval+loc_dente · probe
census s151 **ab02faec0abfab67** + probe sonda s152 **d90dcdc6f9f154af**
conservati ×2 (repo harness + phpr-old-target/release/) · attesi di smoke
verificati da un SECONDO attore prima di ogni run di record · CI mutex lock
`/private/tmp/phpr-measure.lock` da CREARE a ogni finestra · Data ≥10G ·
rust-analyzer NON kill · lettura: REGOLE.md → QUI →
`wp152-harness/s152-{gonogo-verdetto.out,pesca-lettura.md,touchmap-a2.md}`
→ `sessions/WP_SESSION_152.md` → PERF_MAP.md.
