# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-167 = ⚖️ CONCILIO (GO-CONDIZIONATO 9/9 alla campagna
R1 «interno-handler prima, dispatch poi»; verbali VINCOLANTI in
wp167-harness/) + FETTA 0 A VERDETTO EMENDATO: reg-lower paga già 80,6
ns/iter · pila NON RILEVATA ≤0,5 (controllo positivo dovuto) · MISPREDICT
REFUTATO (phpr 0,005 vs oracle 0,031: il match è ben predetto) · ripartizione
INDIZIARIA del residuo: interno-handler ~23 ns (61%) dominante +
delivery-indiziato ~14,5 — l'«identità aritmetica» delle quote DICHIARATA:
la chiusura vera è ai mock** · az.rev. S-166: 5/5 chiuse (copia-gate-v2
collaudato con mutante; v1 S-134 ripristinato) · leve: 0 (sanzionato ⚖️:
fetta 0 = misura) · incidenti: 1 (ENOSPC da copia-albero non pulita, curato)
· revisione (misura): REGGE CON RILIEVI, 5 azioni sotto · QUESITI UTENTE:
(a) T2/A2; (b) census server (17° slitt.); (c) ratifiche; (d) emenda §3.

## Scoreboard (pin INVARIATO s166 phpr 092dcff431bef876 + server caa4e4b2638686a9)
**arith 5,4 · prop 5,5 · calls 4,8 · str 4,2 · arr 3,2 · re 2,5** · giudici
propri mc2 ~155 / mc3 181 · arith de-quantizzato 46,5 vs oracle 8,64 (gap
37,9) · WP 1,746-1,749 · ORM [7,023;7,053] (RIF) · corpus 1412×2 · batteria
1748 · denti: run.rs 6917 · mod.rs 25909 · host.rs 7726.

## §S-168 — ordine (⚖️ campagna, fetta 0-bis + mock)
1. **MOCK m1/m2/m3** (az.rev.1 = F0 del team-meccanismo, l'UNICA chiusura non
   tautologica): 3 build da tree==pin con patch MINIME dichiarate —
   m1 consts-predecode · m2 BinOp «cotto» · m3 hoist frames[top] — braccio
   via pin-phpr.sh --braccio, copia-gate v1+v2 sugli script, disasm, A/B
   micro-judged R=5 vs pin su arith-dq. GATE: chiusura sottrattiva ≥90%
   (Gregg) ⇒ sblocca F1/F2; kill: mock <10 ns nominati ⇒ delibera R4.
2. **Sanature strumento (az.rev.2-4)**: mutante proprio per c0 (lavoro utile
   puro +N esatto) + legenda template risolta · controllo positivo stride
   (mutante che DEVE produrre conflitto D-cache) · repliche xctrace R≥3 per
   lato + spiegare o-mut c1 0,031→0,004. Timebox: dentro la mezza sessione
   di misura, PRIMA di leggere di nuovo le quote.
3. **F1 predecode consts** SOLO se gate mock ≥90% passa (criterio suo,
   giudice arith-dq D≥5, guardie con banda-layout, two-request parity ⚖️).
4. Emende §3 + quesiti (a)-(d) se l'utente ratifica.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
mock m1-m3 · mutante-c0 · stride controllo-positivo · repliche xctrace ·
tetto-fuso (e) (in canna, KS-BAK-167-1) · F1 predecode · F2 BinOp-3ind ·
F3 outline · autoload statiche · sonda strmap · ri-fondazione banda
sentinella ORM (prima della prossima coppia) · gamba server census (17°) ·
§3.28 · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 ·
depr. float→int · warning ×2 · div. RMW · objmap → GC · evalcls 316,9× ·
refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi · attesa-AF1.

## NON riproporre (i veti restano)
**S-167: chiusura da QUOTE-che-sommano-a-1 spacciata per firma (identità
aritmetica) · lettura di strumento nuovo senza il SUO mutante · copione che
copia un albero senza pulizia nell'epilogo · commit concatenato all'esito di
un collaudo · sovrascrittura di script senza lettura del bersaglio.**
S-166: copione generato senza grep dei nomi di scrittura · tN senza carta ·
RIF fuori criterio · retry-wrapper fuori repo · etichette non verificate.
S-165: guardie senza banda-layout · quantizzati senza ri-risoluzione ·
azione di processo senza prezzo · batteria senza rebuild-ricetta · fast path
inline in run_loop. Trasversali: NaN-boxing/fn-table/arena (⚖️ CONFERMATI) ·
BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda · claim
di assenza oltre risoluzione · denominatori a memoria.
**Riscritto** 2026-09-01 (chiusura S-167; storia in `sessions/` · `gaps/`).
Pre-flight S-168: pin phpr **s166 092dcff4**31bef876 + server **caa4e4b2**
638686a9 (SOLO via pin-*.sh) · ⚠️ **Data 7G < 10G: liberare ≥4G PRIMA (i 3
mock costruiscono in target dedicati; lezione ENOSPC S-167: pulizia copie
nell'epilogo)** · MySQL wp8 con l'elenco · uploads sotto guardia · corpus
1412 · lock misura da CREARE · **NESSUNA coppia dovuta** · lettura: REGOLE.md
→ QUI → **wp167-harness/COUNCIL_WP167_REVIEWS.md (⚖️) + s167-f0-verdetto.out
(EMENDATO) + revisione.md** → sessions/WP_SESSION_167.md → PERF_MAP.md.
