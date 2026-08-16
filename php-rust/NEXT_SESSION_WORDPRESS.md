# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **resta 1,765–1,788 su banda 0,036 CONGELATA** —
**t2 (S-148) FUORI BANDA bordo BASSO (1,722–1,742, N=5 pulite; leg3 1,722
vs limite 1,729) ⇒ nessun claim, REPLICA t3 DOVUTA; spread CROSS-finestra
(t1 0,090 · t2 0,020 · unione 0,101)** · ORM 8,370–8,427 / dbal 8,20–8,37
(S-147 @ s145) · **⚖️ S-148 SECONDO ATTO COMPIUTO — census ATTRIBUZIONE per
TAG (identità Σtag==galloc_n ESATTA ×2, repliche 0,056% worst, workload==s144
−0,73%): other 269,3M NOMINATO = hostcall 165,6M (61,5%; 6,7× soglia) + none/
VM-inline 94,6M (3,8×) + code; KILL per CONTEGGI: growth-hashbrown 0,23× e
pool-Frame 0,12× e gc MORTI (zero codice)** · FR1 in ISTRUTTORIA (az.#4
SLITTATA da S-148) · leve spedite S-148: 0 (dichiarato: attribuzione-ordinata)
· sessioni-senza-misura: 0 · incidenti 16 (=).

## Scoreboard (pin s145 phpr a89faf32c62142f9 + server 4a9adc51a62b21ba INVARIATO)
**arith 5,5 · prop 5,5 · calls 4,8 · str 4,3 · arr 3,2 · re 2,5 · hintcall
7,3 (n.r.) · dimread 4,3** (micro n.r. S-146/147/148: pin invariato) ·
oggetti (s136): objchurn 6,7 · objmap 11,7 · MAPPA (net): WP 1,78 (rif
1,765–1,788; t2 1,722–1,742 SENZA claim) ≈ compoff 1,9 ≪ hf 2,55 ≪ hk 4,3 ≪
dbal 8,20–8,37 ≈ ORM 8,370–8,427 · corpus **1414 ×2** · media 2,454–2,569
(t2) · **REPERTI S-148**: 69,6% delle alloc grezze DENTRO i builtin ·
~6,7 alloc/chiamata nei corpi · pop_keys/split_off = 1 Vec a CallHostBuiltin
(11 siti; l'args-Vec S-104 ha il canale) · shape hostcall ≤48B 107,9M +
≤16B 98,8M · FramePool WP-30 già ricicla (frame.other 3,0M).

## §S-149 — ordine (dal verdetto S-148: dentro la testa, poi prezzare)
1. **Tranche-4: census per-NOME-builtin** dentro hostcall.other 165,6M
   (apparato s148 RIUSABILE: tag dinamico sul nome nel dispatch; criterio
   ≤10 righe + soglie in CONTEGGI pre-registrate PRIMA dei dati; monobinario,
   ×2 repliche, identità e parità come s148). Emette il ranking dei builtin
   per alloc-other → le teste NOMINATE.
2. **Sonda-PREZZO pair COLLAUDATA** (monobinaria, churn 16–48 B sul pattern
   reale; criterio con banda e giudice proprio): converte i conteggi in
   secondi con prezzi PROPRI (zcell/arr0 restano INDIZIO, veto S-146). POI la
   decisione di leva sulla scala arbitrata S-146 (≥2× soglia ⇒ scommessa
   ammessa). Candidate GIÀ nominate (s148-anatomia-hostcall.md): pop-diretti
   su CallHostBuiltin (forma-2 estesa) · args-Vec user-call (none.other) ·
   corpi dei builtin di testa (dalla tranche-4).
3. **Replica coppia WP t3** (DEBITO: t2 fuori banda dal basso, nessun claim):
   stesso harness s148-pair.sh (t3); il test DERIVA (az.rev. S-146 #3) si
   applica SOLO con N=6 pulite; a valle, banda_ON da RIFONDARE multi-finestra
   (unione t1+t2+t3), mai da finestra singola.
4. **Istruttoria FR1 dimrmw** (az.rev. S-146 #4, slittata ×2 — dichiarare se
   slitta ancora): mutante a parità di layout + disasm bl-count PRIMA di ogni
   revert (regressione +3,0 ns/iter confermata; indiziato layout run_loop).
5. Coppia WP di pin: NON dovuta (pin invariato); dovuta a OGNI pin nuovo.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
spread WP cross-finestra 0,101 da capire (banda multi-finestra) · confine
slot-load da nominare (ponte+This 0,99×, S-147) · attribuzione Zval-move dei
memops (VOCE PROPRIA) · ORM ↓ 8,59→8,37 da confermare al prossimo pin · cura
§3.22 · depr. float→int · warning corsia ×2 · divergenze RMW · objdatains
residuo · objmap 43,4 → piano GC · evalcls 316,9× · refl 42,4× · re +2 alloc
· §3.13 · §3.12-i · §3.14 · §3.21 · get_gc · drift TODO.md · latin1-cliff ·
dbal 10 nomi · leg-max media ballerina (leg5 S-142/146, leg3 t2).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-148: growth-hashbrown come bersaglio-solo (0,23× soglia) · pool-Frame
come bersaglio-solo (0,12×; FramePool già ricicla) · gc come bersaglio alloc
(0,36M) · conversione conteggi→secondi coi prezzi zcell/arr0 come CIFRA
(INDIZIO, mai record) · uso di rif/banda nuovi PRIMA della t3 · hist per-tag
letto come other-only (è sul tag intero).**
**S-147: borrow-first sul PONTE slot-load (kill 0,83×) · scommessa SUITE sul
canale movimenti (tetto 1,27 s su binario census) · TakeSlot in OGNI forma ·
cifre census senza qualifica «tetto su binario census».**
**S-146: arena-conteggi senza definizione ≤1 pagina · prezzi pair come budget
· quota memops come giudice (KS-G3) · convenzioni mescolate fuori dal ponte.**
**S-145/144/143**: B1/B2 senza concilio · àncore replace testuale · inventari
su nomi VOLATILI · memops senza attribuzione BILATERALE · denominatore coi
thread parcheggiati · A rifondata = pool+refcount+handle-gen.
BOLT su Mach-O · NaN-boxing · threaded-dispatch · PGO sui giudici · verdetti
su build emendata senza ri-banda · pin/stash senza collaudo-nell'atto ·
contenitori sul call path · differenze tra A/B distinti come cifra ·
componenti prezzate · magnitudine ripartita senza A/B proprio · «icache»
NON-premessa · denominatori a memoria · output di run nel repo · rc di gate
da pipe · run pesanti come task · edit coi build in volo · promozione sotto
banda · corpus-gate solo-nomi · strumentazione nei sorgenti del pin · leve
micro senza banda v2 · alloc-removal senza modello del costo SOSTITUTIVO ·
probe senza riferimento vivo · delta tra census di epoche diverse senza
datare i raw · verdetti da script non committati · claim di ASSENZA oltre la
risoluzione · misure con LSP in volo (sentinelle) · lock con trap EXIT
altrui · staleness di lock a OROLOGIO · probe che rompe l'inlining · `git
add` di directory harness · giudice sotto-risoluto · byte-identità come gate
di un edit .rs post-pin.
**Riscritto** 2026-08-16 notte (chiusura S-148); storia in `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-149: pin phpr **s145 a89faf32**c62142f9 + server **s145
4a9adc51**a62b21ba (stash `phpr-old-target/release/phpr-s145`) · MySQL wp8
con l'elenco · uploads sotto guardia · corpus 1414 ×2 · CI feed (mutex col
measure-lock) · lock misura `/private/tmp/phpr-measure.lock` da CREARE a ogni
finestra · Data ≥10G · rust-analyzer NON kill (sentinelle arbitrano) ·
lettura: REGOLE.md → QUI → wp148-harness/s148-attrib-verdetto.out →
wp148-harness/s148-anatomia-hostcall.md → sessions/WP_SESSION_148.md →
wp148-harness/s148-pair-verdetto-t2.out → PERF_MAP.md.
