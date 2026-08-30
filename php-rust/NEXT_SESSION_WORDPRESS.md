# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-164 = ordine COMPLETO sui punti 1-4: coppia t14 @ s163
COMPATIBILE 1,761 (6/6 PULITE, banda_ON 0,011 RECORD, anti-flare
PRE-registrato ha retto) + ORM [7,066;7,111] VALIDO (attesa-AU1 COMPATIBILE
tetto ~0) + census AU1 RIESEGUITO rc=0 Δ=600000 ESATTO (incidente S-163
CURATO; +1 incidente S-164 curato: path harness chiuso, copia-gate parziale)
+ indagine arith CHIUSA su s161→s163 (tick 5,3→5,5 = QUANTIZZAZIONE del
giudice; rapporti VERI 5,426/5,454/5,417; creep da s158 = nota aperta) +
leva L-AL3 TENTATA E CADUTA A VERDETTO (D=+0,0 col census che conferma la
rimozione del Box: alloc mimalloc ≈0 ns ⇒ classe Box-pooling puro
RIDIMENSIONATA; revert AL BYTE fea4a2d0)** · leve S-164: 0 promosse, 1 A/B
completo (ritmo ok) · revisione: REGGE CON RILIEVI (5 azioni sotto) ·
QUESITI UTENTE invariati: (a) T2/A2 sospendere; (b) census server (14°
slitt.); (c) ratifica #20 #21 + census.done S-163 + #S-164; (d) emenda §3.

## Scoreboard (pin s163 INVARIATO phpr fea4a2d040a0d8d0 + server 8d76d6f129bfd4af)
**arith 5,5 = (vero ~5,43) · prop 5,5 = · calls 4,7 = · str 4,2 = · arr 3,2 =
· re 2,6 =** (micro promo s163, pin invariato; arith de-quantizzato
5,426/5,454/5,417 su s161/s162/s163) · WP t14 mediana 1,761 (6/6) ·
**banda_ON 0,011 RECORD** (t12 0,018) · media 2,341-2,450 · ORM [7,066;7,111]
(registrato resta s162 [7,035;7,086]) · dbal [7,459;7,491] riserva ictx
3ª coppia ⇒ **istruttoria MATURA** · corpus **1412×2** · batteria cap
**25909/7726** (tree == pin al byte, nessun edit residuo).

## §S-165 — ordine
1. **Leva (obbligo di ritmo)** — candidata per NOME: **MethodCall.borrow k=2**
   (alternativa s163) o autoload array-callable k≥2/forme statiche; il
   Box/Vec-pooling puro è RIDIMENSIONATO (s164-al3-STOP.md: si torna a leve
   che CAMBIANO il dispatch, non che spostano alloc). Az.rev.5: un verdetto
   «non pagante» pretende smoke a R tale che soglia=4 (non rumore 6);
   unreachable! ×2 (call_fn_one+call_method_one) DA MONTARE su questo edit.
2. **Istruttoria dbal ictx-oracle** (MATURA, 3 punti: s162 217,8/69,1 · s163
   80,9/48,2 · s164 87,1/101,5) + az.rev.3: banda NUMERICA pre-registrata per
   la sentinella oracle ORM (2 giri lato-veloce 4,86/4,85 e 4,85/4,84) —
   R=5 oracle-only in finestra quieta o ri-fondazione ORA_REF dichiarata.
3. **Azioni revisione S-164 residue**: (az.2) rerun census AL3 con attesa
   che NOMINI il +1 (warm o with_capacity nel probe) — chiude il post-hoc;
   (az.4) istruttoria non-riproducibilità php-server: RICETTA server scritta
   e provata build ×2 (stash duplicato in php-server-s163.dup, fatto S-164);
   (az.1) creep arith da s158: giudice a 3 decimali col braccio s158, quando
   una finestra quieta avanza.
4. **Sonda FUORI-UB strmap** (az.3 S-162, PRIMA di leve sul sito strmap).
5. **Emenda §3 + quesiti (a)-(d)** se l'utente ratifica.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
MethodCall.borrow k=2 · autoload k≥2/statiche · istruttoria dbal ictx (MATURA)
· sentinella-oracle ORM banda numerica · census AL3 rerun (+1 da nominare) ·
ricetta php-server · creep arith s158 (3 decimali) · unreachable! ×2 (sul
prossimo edit) · sonda strmap · elementi array_map-filter non censiti · gamba
server census (14°) · §3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load ·
§3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC · evalcls
316,9× · refl 42,4× · re +2 · §3.13/§3.12-i/§3.14/§3.21 · get_gc · latin1 ·
dbal 10 nomi · attesa-AF1 (pool 6 gambe, serve ampiezza).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-164: Box/Vec-pooling puro senza cambio di dispatch (caduto a verdetto
con census esatto) · verdetto «non pagante» da smoke con rumore>soglia ·
copia-gate verificato sulle sole righe attese (diff INTERO + grep path) ·
build di gemello SENZA ricetta · `git apply` dalla sottodir senza verifica
del diff (no-op silenzioso) · declare -A negli script (bash 3.2).**
S-163: census.done (o rc autoritativo) scritto dalla sessione · estensioni
fixture nei harness CHIUSI · attesa census per analogia (unità dal SORGENTE)
· attesa di flare con campioni radi. S-162: gemello in target dedicato ·
attesa per analogia di sito. S-161..S-152: coeff di famiglia trasportato ·
attese con oracle fuori rif · gate-fixture senza marcatore · copia-gate senza
verifica · giudizio da un estremo · banda smoke senza denti · cifra dalla
sola finestra · leve a scala SUITE. Trasversali: BOLT/NaN-boxing/PGO · pin
senza collaudo · differenze A/B come cifra · denominatori a memoria · rc da
pipe · edit coi build in volo · promozione sotto banda · claim di ASSENZA
oltre risoluzione.
**Riscritto** 2026-08-30 (chiusura S-164; storia in `sessions/` · `gaps/`).
Pre-flight S-165: pin phpr **s163 fea4a2d0**40a0d8d0 + server **8d76d6f1**
29bfd4af (SOLO via pin-*.sh) · Data 13G al prune · MySQL wp8 con l'elenco ·
uploads sotto guardia · corpus 1412 · batteria cap 25909/7726 · stash:
phpr-s163 · php-server-s163 (+.dup) · phpr-s164-gemelloA (==pin) ·
phpr-s164-al3-B (b0f0f766, leva caduta: solo storia) · lock misura da
CREARE · NESSUNA coppia dovuta (pin invariato) · lettura: REGOLE.md → QUI →
wp164-harness/{s164-pair-verdetto-t14,s164-orm-coppia-verdetto,
s164-arith-dq-verdetto,s164-al3sm-verdetto,s164-census-al3-verdetto}.out +
s164-al3-STOP.md → revisione.md → sessions/WP_SESSION_164.md → PERF_MAP.md.
