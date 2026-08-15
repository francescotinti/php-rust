# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full ON-ONLY = 1,765–1,777 (S-140 @ s140, N=6)** ·
**banda_ON = 0,033 CONFERMATA cross-finestra (az.rev. S-139 #1 CHIUSA)** ·
leva SPEDITA S-140 (HC1) · senza-leva = 0 · incidenti 14.

## Scoreboard (pin s140 f2708b75660803a7 + c7a03e2aaa7c7cba; micro = guardie promo HC1)

**arith 5,6 · prop 5,6 · calls 4,8 · str 4,3 · arr 3,2 · re 2,6 · hintcall
7,3 (NUOVA)** · oggetti (s136): objdatains 5,9 · objchurn 6,7 · objalloc 6,4 ·
objdropdef 7,5 · objallocni 7,9 · objmap 11,7 (→ piano GC) · RMW: m-dimrmw 147
· m-diminc 113 · MAPPA (net): WP **1,77 @ s140** ≈ compoff 1,86–1,89 ≪ hf 2,55
≪ hk 4,3 ≪ dbal 8,15–8,23 ≈ ORM 8,59–8,71 (S-139) · corpus **1414 ×2** ·
**REPERTO S-140: profilo SUITE = CHURN 32% vs DIMPROP 6%; 44% dei clone INLINE
da run_loop (ciclo vita Zval → TakeSlot); i canali per-statement piccoli
(hint-check 0,13%) NON muovono le suite.**

## §S-141 — ordine

1. **LEVA dai numeri**: canale GROSSO = clone/drop del ciclo vita Zval in
   run_loop (44% dei clone) = famiglia **TakeSlot** (S-95: liveness.rs, F2 e
   contatori would_take_safe GIÀ FATTI; opcode MAI scritto; WP-98: metodo
   vincolato, ordine sospeso). Primo passo: census would_take_safe_{rc,str}
   sulla SUITE ORM (probe zval-census, come s140-census); se STR-only multi-%
   ⇒ istruttoria opcode (F3/F4 design95, divergenze S-96 rilette PRIMA).
   Alternativa se delude: Repr-drop 11% o frame-lifecycle 9%.
2. **Peak WP**: reperto S-140 (leg1 1807 SOTTO banda vecchia, 2–6 a 1838–1853)
   indizia STATO/ordine-finestra ⇒ bisezione PER POSIZIONE (gamba fredda /
   ordine ruotato), non per binario; osservativo, mai gate.
3. **CI**: feed locale in apertura; GH Actions esito run s140 (ef66b79/
   54d7334/d690a64); fase 2 SOLO su decisione utente.
4. Coppia WP: NON dovuta (s140 coperto t1 6/6); a pin nuovo, su banda_ON 0,033.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

TakeSlot census suite (p.1) · Repr-drop 11% · frame-lifecycle 9% ·
peak-per-posizione (p.2) · bisezione peak s136 (declassata) · deprecation
float→int da catalogare (riga doppia CLI + linea decl-callee vs call-site) ·
warning corsia zval+mem-census ×2 (pre-esistenti) · fix echo s140-pair.sh ·
divergenze RMW del PIENO · objdatains residuo (18,9/17,6) · objmap 43,4 →
piano GC · dispatch 36,3 · cammini non cacheabili · locale 170 ns · evalcls
316,9× · refl 42,4× · re +2,00 alloc/iter · §3.13 · §3.12-i · §3.14 · §3.21 ·
get_gc · drift TODO.md · latin1-cliff · dbal 10 nomi.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing · threaded-dispatch · PGO sui giudici · verdetti su
build emendata senza ri-banda · pin/stash senza collaudo-nell'atto · contenitori
sul call path · differenze tra A/B distinti come cifra · componenti prezzate ·
magnitudine ripartita senza A/B proprio · «icache» NON-premessa · pre-filtro che
tassa i freddi · guardie non-bersaglio BILATERALI · denominatori a memoria ·
output di run nel repo · rc di gate da pipe · tee/log pre-mkdir · admission sul
dump intero · xctrace senza guardie disco · run pesanti come task · edit coi
build in volo · promozione sotto banda · gate a soglia fissa senza banda ·
corpus-gate solo-nomi · strumentazione nei sorgenti del pin · leve micro senza
banda v2 · alloc-removal senza modello del costo SOSTITUTIVO · probe senza
riferimento vivo · ordine FISSO di misura · delta tra census di epoche diverse
senza datare i raw · verdetti da script non committati · SSO inline ·
inline-array init+drain args · claim di ASSENZA oltre la risoluzione · smoke
con fam-min > R · notti su PhpStr-full · guardie su giudici diversi dalle loro
bande · misure con LSP in volo · F2 keys-scratch · quiescenza nello stesso
comando del lancio · pattern quiescenza nell'argv del lancio · output TRACKED
mossi da orchestratori · rumore-soglia = range PIENO · guardia senza banda
propria · percentuale tra segmenti NON annidati senza zero-check · sed di copia
su righe ESEGUIBILI (n.14: MORSA ANCORA in S-140, errata header pair) ·
eccedenza sopra la parte modellata senza sonda · banda di guardia da strumento
DIVERSO senza drop-1 · lock di finestra con trap EXIT altrui · quiescenza a 2
campioni senza STREAK · abort multi-gamba senza retry · probe che rompe
l'inlining · leva GC note-time (WP-21) · `git add` di directory harness ·
identità tra GIUDICI diversi come gate · gemello di relink che eredita il
verdetto A/B senza conferma post-pin · staleness di lock a OROLOGIO ·
cmp secco vs oracle su fixture CON divergenze a catalogo · **giudice
sotto-risoluto per il fenomeno (si scala la DENSITÀ del giudice, mai la
soglia — S-140)** · **stash post-batteria senza rebuild ricetta A′ riprodotto
al byte (il relink di cargo test MORDE — S-140)**.
**Riscritto**: 2026-08-15 (chiusura S-140). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-141: pin phpr **s140 f2708b75**660803a7 + server **s140 c7a03e2a**aa7c7cba ·
MySQL wp8 con l'elenco · uploads sotto guardia · corpus 1414 ×2 · CI: feed
locale + esito GH run s140 · lock misura da CREARE (oggi RIMOSSO) · Data ≥10G ·
pgrep rust-analyzer PRIMA di ogni misura (Serena lo rilancia) · lettura:
REGOLE.md → QUI → WP_SESSION_140 → wp140-harness/revisione.md → PERF_MAP.md.
