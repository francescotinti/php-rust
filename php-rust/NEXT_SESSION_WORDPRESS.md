# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-156 = leva HD2-hostcall PROMOSSA (pin NUOVO s156) +
census ORM post-CE1 rifondato + fix dbal diagnosticato** · giudice m-hostargs
**D=+16,0 ns/iter** (soglia 4,0; rumore 3,0/1,0; UB 13,8+3,0 in banda: canale
alloc spiega tutto), 13 guardie ok, disasm bl +19 localizzato · promozione t2
rc=0 (t1 STOP dente loc A4 → **salita DICHIARATA** mod.rs 25742 / run.rs 6815,
candidato al byte) · census: attese (b)/(d) FUORI spiegate — scope s149
annidante: il run_loop dell'AUTOLOADER resta sul nome ⇒ chiamate class_exists
∈ [1,23M;2,46M], **residuo miss/autoload E ∈ [4,82M;6,05M]** (fetta nuova per
NOME); funnel CE1(b) apporzionato: ce −2,46M · __reflect_class_real_name
−1,28M · __reflect_class_loc −452k · leve S-156: **1 (promossa)** · incidenti
**19** (=) · reperti dichiarati: riconc. smoke↔R5 fuori banda 0,5 · guardia
backtrace al bordo (2 tick) · conferma post-pin D=+5,0 segni 5/5 (finestra) ·
QUESITI UTENTE: (a) T2/A2 sospendere (raccomandazione S-155, ratifica al
contatto); (b) census server (6° slittamento).

## Scoreboard (pin s156 phpr 42efea3e34feb390 + server ef89630f9c7408c3)
**arith 5,4 · prop 5,5 · calls 4,7 · str 4,2 · arr 3,2 · re 2,6** (micro R=5
promo s156) · WP t6 1,771 · media 2,456–2,510 · ORM 6,972–7,053 · dbal
7,385–7,422 (tutti @ pin s154: **coppia al pin s156 DOVUTA**) · corpus
**1412×2** (promo s156) · batteria 1748/0/2 (cap loc 25742/6815).

## §S-157 — ordine
1. **Coppia WP t7 + ORM/dbal al pin s156** (dovuta: pin nuovo). Nella COPIA:
   fix `summ()` e failnames con `LC_ALL=C tr | grep -a` (az.rev. S-155 #4,
   diagnosi in wp156-harness/s156-dbal-summ-fix.md) — dichiarare nel manifest
   e mettere a verbale il reperto dbal phpr 3921 test/626 skip vs oracle
   3929/594 (visibile solo col fix). Attesa HD2-hostcall PRE-REGISTRATA:
   sotto-risoluzione su entrambe (WP «piccola/nulla»; ORM ≤ ~0,05 s).
2. **LEVA TENTATA (obbligo di ritmo)** — candidate per NOME, istruttoria di
   forma PRIMA dell'attesa: (a) **miss/autoload class_exists** (E 4,8–6,05M
   alloc attribuite: capire il cammino — autoloader Composer per miss
   ripetuti? — prima di ogni criterio); (b) **seconda tranche slice HD2**
   (__reflect_* famiglia ~9,5M alloc: corpi da verificare SOLO-LETTURA, la
   macro a due sezioni è già in piedi); (c) array_map 7,68M (forma: k dipende
   da arità/callback).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
miss/autoload class_exists (E 4,8–6,05M, S-156 istruttoria) · __reflect_*
slice tranche-2 · array_map 7,68M (istruttoria forma) · gamba server census
(6°) · MethodCall.borrow k=2 · §3.24+§3.23 · slot-load · §3.22 · depr.
float→int · warning ×2 · div. RMW · objmap 43,4 → GC · evalcls 316,9× ·
refl 42,4× · re +2 · §3.13/§3.12-i/§3.14/§3.21 · get_gc · latin1 (morde
anche l'estrazione dbal) · dbal 10 nomi · gdc DECLASSATA · CI: corpus-FAIL
d'ambiente 3 test backtrace (nei gate di record passano).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-156: attese-census per-NOME senza i termini ANNIDATI dello scope
(plumbing E miss/autoload: derivare dal perimetro, non dalla semantica) ·
conferma post-pin come giudice di MAGNITUDINE (arbitra solo il segno) ·
snellire il sorgente per un dente su leva già misurata (invalida il
candidato: salita dichiarata sul file di test).** S-155: fette <risoluzione
senza micro-judge · attese per-nome per cure su FUNNEL · claim «sotto N×» a
intervallo a cavallo. S-154: identità pin a hash esatto su build fredda ·
guardie quantizzate sotto il tick. S-153: braccio A non-gemello (§7-bis) ·
leve borrow senza prezzo in-contesto. S-152: leve a scala SUITE. S-151:
cifre census pre-BT1 · arena/bump-reset. S-150: A/B fuori ricetta · attese
senza pavimento. Trasversali: BOLT/NaN-boxing/threaded-dispatch/PGO ·
pin/stash senza collaudo-nell'atto · differenze tra A/B come cifra ·
componenti prezzate · denominatori a memoria · rc da pipe · edit coi build
in volo · promozione sotto banda · claim di ASSENZA oltre risoluzione ·
misure con LSP in volo (conteggi esenti, dichiarare) · giudice sotto-risoluto.
**Riscritto** 2026-08-23 (chiusura S-156); storia in `sessions/` · `gaps/`.
Pre-flight S-157: pin phpr **s156 42efea3e**34feb390 + server **ef89630f**
9c7408c3 (SOLO via pin-*.sh) · MySQL wp8 con l'elenco · uploads sotto
guardia · corpus 1412×2 · batteria cap loc 25742/6815 · stash: phpr-s156 ·
php-server-s156 · phpr-s156-hd2host-B (==pin) · **gemello A per S-157 da
RICOSTRUIRE sul tree s156** (il 2023cbb9 è del tree s154) · lock misura da
CREARE · CI: coda S-156 in smaltimento a lock rimosso · lettura: REGOLE.md →
QUI → wp156-harness/{s156-census-verdetto.out,s156-census-istruttoria.md,
s156-abhd2h-verdetto.out,s156-promo-verdetto.out,s156-dbal-summ-fix.md,
revisione.md} → sessions/WP_SESSION_156.md → PERF_MAP.md.
