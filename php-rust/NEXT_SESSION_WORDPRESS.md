# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-168+S-169 (stessa conversazione) = fetta 0-bis MOCK +
az.rev.: il KILL ⚖️ regola 4 è scattato A FILO (Σ nominata 9,92 <10; Σ grezza
12,9) e la decomposizione ha cambiato natura al quesito: DISPATCH PURO = 1,75
ns/op (= costo per-op INTERO dell'oracle), corpo di un handler BANALE ≈5,6
ns/op, corpo di BinarySCSCDst ≈27,4 (consts 4,7 + BinOp 5,4 nominati, ~17,5 NO)
⇒ il divario è il CORPO di OGNI handler (accesso slot, guardie Undef/Ref,
read_slot clone, funnel, reg_store_slot+gc_note, Result/`?`), non il dispatch
né il numero di op (tetto di ogni fusione/predecode: 1,75 ns/op eliminato) ·
pila REFUTATA (controllo positivo) · mispredict REFUTATO firmato (c3 fissata con
mutante casuale bilaterale) · c0 solo per esclusione · S-170 = DELIBERA R4
DELL'UTENTE PRIMA di ogni codice** · leve: 0+0 (sanzionato) · incidenti: 2+1
(ENOSPC ktrace) · revisioni: S-168 semantica REGGE CON RILIEVI (emende applicate),
S-169 processo: REGGE CON RILIEVI (pre-flight saltato: il feed CI diceva già
disk-low; e2 rimisurata ok; 4 casi di UNA classe «copione derivato non
collaudato contro l'atteso»; 1,75 = limite inferiore; c2 indiziata ⇒ c0 non
chiusa; mispredict «trascurabile», non firma; lock ha fermato la CI ~304 job)
· QUESITI: (a) delibera R4; (e) potare/azzerare la coda CI (≈100 h)?;
(b) T2/A2; (c) census server (19° slitt.); (d) ratifiche §3.

## Scoreboard (pin INVARIATO s166 phpr 092dcff431bef876 + server caa4e4b2638686a9)
**arith 5,4 · prop 5,5 · calls 4,8 · str 4,2 · arr 3,2 · re 2,5** · mc2 ~155 /
mc3 181 · arith-dq 46,6-47,1 vs 8,64 · E2 14,7 vs 3,6 · dispatch 1,75/op · WP
1,746-1,749 · ORM [7,023;7,053] (RIF) · corpus 1412×2 · batteria 1748 · denti:
run.rs 6917 · mod.rs 25909 · host.rs 7726.

## §S-170 — ordine
1. **DELIBERA R4 (utente)** con le cifre di S-168/169 (s168-mock-verdetto.out +
   s169-verdetto.out). Opzioni sul tavolo: (i) R4 = campagna «corpo del handler»
   generalizzata a TUTTI gli handler — fetta F3 outline/guardie estesa, accesso
   slot senza doppia indirezione+bounds (forma sigillata regola 8), gc_note e
   typed_refs fuori dal cammino Long, Result/`?` fuori dal fast path — misurata
   PRIMA con mock sul handler BANALE (giudice E2: bersaglio 5,6→~2 ns/op, il
   più economico da leggere); (ii) tenere R1 «compilatore» chiusa (tetto 1,75/op);
   (iii) concilio a 9 solo se l'utente lo chiede (cambio di rotta = suo diritto).
2. **Se delibera (i)**: criterio pre-registrato per mock «handler banale magro»
   (IncDecSlotJmp/CmpJmpSC senza guardie/bounds sul cammino Long) su E2 R=5,
   A=m0; poi decomposizione del residuo ~17,5 di BinarySCSCDst a bracci
   (guardie · read_slot clone · store+gc_note · funnel) — SOLA MISURA.
3. Az.rev. S-169 (processo): pre-flight OBBLIGATORIO anche in-conversazione ·
   copia-gate v2 per TOKEN + `[ -s ]` sui file letti + parità contro output
   ATTESO (rigenerare i manifest) · mutante c0 POSITIVO (throughput retiring:
   str_repeat/array_sum L1) e c2 declassata a «indiziata» · c3 anche in ns/iter
   · rimozione lock = passo esplicito di chiusura, coda CI nello scoreboard.
4. Quesiti (a)-(d) se ratificati.

## Aperture per NOME
delibera R4 · mock handler-banale-magro · residuo BinarySCSCDst a bracci ·
mutante c0 positivo · copia-gate per token · A==B vs atteso · Sweep/iter 2,9 ·
F1/F2 (SOSPESE: tetto 1,75/op) · tetto-fuso (e) · autoload statiche · sonda
strmap · banda sentinella ORM · gamba server census (19°) · §3.28 · §3.29 ·
§3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 · depr. float→int ·
warning ×2 · div. RMW · objmap → GC · evalcls 316,9× · refl 42,4× · re +2 ·
get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano)
**S-169: collaudo A==B senza output atteso (due errori uguali passano) ·
copia-gate per riga (etichetta stantia accanto a quella corrente passa) ·
xctrace senza purge dei `.ktrace` in $TMPDIR · mutante «lavoro utile» a catena
di dipendenza (crc32 = stallo) · sessione nella stessa conversazione senza
riaprire il pre-flight formale.** S-168: mock senza dump del loop · catena sotto
timeout tool · output di run nel repo · soglia controllo positivo che scala col
rumore · kill senza banda. S-167: quote-che-sommano-a-1 · strumento senza mutante
· copia d'albero senza pulizia. Trasversali: NaN-boxing/fn-table/arena (⚖️) ·
BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda.
**Riscritto** 2026-09-02 notte (chiusura S-169; storia in `sessions/` · `gaps/`).
Pre-flight S-170: pin phpr **s166 092dcff4**31bef876 + server **caa4e4b2**
638686a9 (SOLO via pin-*.sh; stash bracci `phpr-s168-*`/`phpr-s169-*` NON pin)
· Data ≥10G (**controllare $DARWIN_USER_TEMP_DIR/instruments*.ktrace e il
CI_FEED: disk-low = STOP**) · MySQL
wp8 con l'elenco · uploads sotto guardia · corpus 1412 · lock misura da CREARE ·
**NESSUNA coppia dovuta** · CI: coda lunga (~290 job da S-168/169) — leggere il
feed · lettura: REGOLE.md → QUI → wp169-harness/s169-verdetto.out +
s169-xctrace-verdetto.out + revisione.md → wp168-harness/s168-mock-verdetto.out
→ wp167-harness/COUNCIL_WP167_REVIEWS.md (regola 4) → WP_SESSION_169 → PERF_MAP.
