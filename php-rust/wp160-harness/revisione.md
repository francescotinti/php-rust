# Revisione S-160 — lente PROCESSO (revisore singolo adversariale)

## VERDETTO: REGGE CON RILIEVI

La finestra del criterio af1 p.10 è rispettata (pair done 04:31 → orm rimisura.done 04:38 → smoke 04:53 → R5 05:06 → promo 05:14+); gemello stashato prima dell'A/B; cap rotazione 40/80 esatti, Data 6G dichiarata. Ma la rettifica, pur onesta, non ha dichiarato la conseguenza più grave del difetto di forgia.

## Rilievi

1. **GRAVE, NON DICHIARATO: il run con H stantio ha DISTRUTTO il record della promozione S-159.** `OUT="$H/promo-out"` ha sovrascritto in wp159-harness/promo-out gli artifact autoritativi di s159 (rcb, batteria.rc/log, orm.rc/log, pin.log, fixture-chain, corpus — mtime 2026-08-28; sopravvive solo micro-pin-s159.out per nome unico). La rettifica p.4 dichiara la "posizione" ma non la sovrascrittura: il verdetto s159 ora è orfano delle sue prove. Va nell'incidente #21.
2. **Doppia fonte di verità**: wp159-harness/s160-promo-verdetto.out (2,2K, scritto dal run) contiene i falsi verdi SENZA rettifica; il file corretto (4,2K) sta in wp160. La copia stantia è un verdetto alternativo in circolazione.
3. **La rettifica è essa stessa fuori-script**: le ri-derivazioni (fx-af, fx-am, conferma post-pin) hanno artifact (conferma-rett-runs.tsv, quiesce-rett.rc=0) ma nessuno script agli atti né rc proprio; s160-promozione.sh corretto DOPO il run. Collaudo-nell'atto (§2) parziale.
4. **Riconciliazione UB meccanicamente incoerente col criterio**: p.4 dichiara 12,0±2,5, il verdetto stampa "UB=12.0+rumore=1.0" (soglia 13, non 15,5). Con la formula del criterio D=+16,0 è al bordo (+0,5) e la conferma ri-derivata +15,0 cade DENTRO: il reperto "fuori-UB" non è robusto. Conservativo (sonda comunque dovuta), ma la dichiarazione è discrezionale, non meccanica.
5. **Istruttoria phpr1**: predizioni pre-registrate, (b) vince netta, rettifica onesta del "3ª finestra". Ma "causa CHIUSA" poggia su N=1 a freddo amplificato (~2 giorni), condizione diversa dall'in-finestra s159. Il RIMEDIO è validato in campo (4/4 pulite); la CAUSA resta "meccanismo firmato, N=1".

## Azioni

1. Integrare l'incidente #21 con la sovrascrittura del record S-159; annotare in wp159-harness/s159-promo-verdetto.out che i suoi promo-out sono del run S-160.
2. Rimuovere o marcare INVALIDA la copia wp159-harness/s160-promo-verdetto.out.
3. Formalizzare la rettifica: salvare agli atti lo script effettivo delle ri-derivazioni con rc, o rieseguire la conferma dallo script corretto.
4. Emenda §3 proposta: aggiungere che OUT del copia-gate deve essere directory NUOVA/versionata (il clobber è l'altro morso dello stesso difetto).
5. Riclassificare il reperto UB come "al bordo, non robusto" (formula 12,0±2,5+rumore agli atti); la sonda S-161 decide.
