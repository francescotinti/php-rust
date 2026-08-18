# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **pin NUOVO s153 = PROMOZIONE L-BT2 con catena piena
rc=0** (debug_backtrace: chiavi statiche thread-local + BtFrame→ZStr; A/B R=5
D=+266,7 ns/iter, 733→467 = −36% sul giudice, segni 7/7, guardie 12/12,
fx-backtrace byte-id; FUORI-UB sopra ⇒ **sonda k post-leva DOVUTA**) ·
**L-TD1 CADUTA a R=5 gemello** (D=−3,3 vs soglia 4): 4 borrow/iter rimossi
per costruzione ⇒ **prezzo borrow in-contesto ≤~1 ns vs mock 4,27–4,41
FALSIFICATO** nei siti teardown/sweep ⇒ NO-GO A3c RAFFORZATO, fetta
PropSetPop retrocessa, coda A3 riordinata sulle fette ALLOC · **EMENDA
§7-bis** (s153-criterio-td1-emenda.md, PERMANENTE): a pin invariato ma tree
avanzato, il braccio A di OGNI A/B si COSTRUISCE gemello dal tree corrente
(i commit cfg-gated spostano il layout: drift ±5–10 ns/iter firmato) ·
ratifiche utente S-153: A2=T2-only · A3.0 confermato · census server in coda
(3° slittamento) · quesito NUOVO: col prezzo-borrow ≈0, T2/A2 conserva il
razionale o si sospende in attesa di fette ad attesa fondata? · leve spedite
S-153: **1** (tentate 2, una caduta a verdetto) · incidenti **19** (=).

## Scoreboard (pin s153 phpr 8370c257ae70cc8e + server f030c6fcbddfab96)
**arith 5,5 · prop 5,5 · calls 4,7 · str 4,3 · arr 3,2 · re 2,6** (promo
s153, ±1 tick vs s150) · WP t4 1,781 · media 2,480–2,555 · ORM 7,104–7,149 ·
dbal 7,283–7,491 (TUTTI rif s150: **coppia al pin nuovo DOVUTA**) · corpus
**1412×2 ZERO flip** · batteria 1748/0/2 (s125 + denti rczval+loc_dente;
cap dente AGGIORNATI con salita dichiarata: mod.rs 25707 · host.rs 7661).

## §S-154 — ordine
0. ⚠️ **BLOCCANTE apertura**: Data a **2G** liberi (<10G; probabile update
   macOS staged, snapshot MSUPrepareUpdate) — decisione UTENTE su come
   liberare PRIMA di ogni build fredda (~5G).
1. **Coppia WP t5 + rimisura ORM/dbal @ pin s153** (dovuta a ogni pin nuovo;
   attesa BT2 su ORM PRE-REGISTRATA: piccola ↓ ~0,05–0,15 s ≈ 473k chiamate
   × ~20 alloc × ~10 ns, sotto/al bordo risoluzione 0,26–0,30 — si dichiara
   qualunque esito; su WP «piccola/nulla»).
2. **Sonda k post-BT2** (fuori-UB da spiegare: probe census da RICOSTRUIRE
   sul tree corrente — la copia s151 non applica più pulita; contare k
   nuovo di debug_backtrace e rifondare la testa hostcall post-leva).
3. **Se resta finestra, fetta micro per NOME dalla coda ALLOC**:
   class_exists lookup case-insensitive no-alloc (k=2 esatte, ≈4,85M
   chiamate ORM) oppure get_declared_classes (4,6M, conteggio chiamate
   dovuto prima del criterio).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
sonda k post-BT2 (dovuta) · class_exists 2 alloc/call · get_declared_classes
4,6M · gamba server census (3°) · PropSetPop 3+1→1+1 (RETROCESSA: solo con
prezzo in-contesto fondato; run_loop ⇒ disasm) · MethodCall.borrow k=2 ·
§3.24+§3.23 backtrace · slot-load 0,99× · §3.22 · depr. float→int · warning
×2 · div. RMW · objmap 43,4 → GC · evalcls 316,9× · refl 42,4× · re +2 ·
§3.13/§3.12-i/§3.14/§3.21 · get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-153: A/B col braccio A dallo stash del pin quando il tree è avanzato
(emenda §7-bis: gemello SEMPRE) · leve borrow-unify su teardown/sweep senza
prezzo in-contesto NUOVO (falsificato ≤1 ns) · byte-identità del revert
verificata contro il PIN a tree avanzato (si verifica contro il GEMELLO).**
S-152: A3c in QUALUNQUE forma senza numeri nuovi oltre soglia · leve
debug_backtrace/class_exists a scala SUITE · calcolatori senza collaudo o
emenda. S-151: asset senza path+hash · cifre census pre-BT1 nei criteri ·
ObjectId Copy-senza-refcount · arena/bump-reset · spacchettare exec/ops_* ·
crate nuovi in A2 · Fase-5 registri. S-150: A/B fuori ricetta senza
gemello+ricetta · attese senza pavimento · forge non collaudate · revert
FR1. S-149: prezzi pair senza banda · splitoff3 senza replica · banda su
finestra singola. S-148..146: kill al perimetro dei tag · borrow-first sul
ponte · TakeSlot in OGNI forma · B1/B2 senza concilio. Trasversali:
BOLT/NaN-boxing/threaded-dispatch/PGO-sui-giudici · pin/stash senza
collaudo-nell'atto · differenze tra A/B come cifra · componenti prezzate ·
denominatori a memoria · rc da pipe · run pesanti come task · edit coi
build in volo · promozione sotto banda · strumentazione nel pin · claim di
ASSENZA oltre risoluzione · misure con LSP in volo · lock con trap altrui ·
giudice sotto-risoluto · byte-identità come gate di edit .rs post-pin.
**Riscritto** 2026-08-18 (chiusura S-153); storia in `sessions/` · `gaps/`.
Pre-flight S-154: pin phpr **s153 8370c257**ae70cc8e + server **f030c6fc**
bddfab96 (SOLO via pin-*.sh) · MySQL wp8 con l'elenco · uploads sotto
guardia · corpus **1412×2** · batteria = s125+rczval+loc_dente (cap 25707/
7661) · stash s153: gemelloA f95a1067 · td1-cand 297cffc9 · bt2==pin
8370c257 · probe ab02faec + d90dcdc6 ×2 · attesi smoke da SECONDO attore
prima d'ogni record · braccio A = GEMELLO dal tree (emenda §7-bis) · lock
misura da CREARE a ogni finestra · **Data ≥10G: OGGI 2G, BLOCCANTE** ·
rust-analyzer NON kill · lettura: REGOLE.md → QUI → wp153-harness/
s153-{td1-caduta.md,criterio-td1-emenda.md,teardown-conteggi.md,
ab-bt2-verdetto.out,promo-verdetto.out} → sessions/WP_SESSION_153.md →
PERF_MAP.md.
