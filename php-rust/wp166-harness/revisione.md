# Revisione S-166 — lente PROCESSO (revisore singolo, adversariale)

## VERDETTO: REGGE CON RILIEVI

La promozione L-MCk regge: criterio pre-registrato (s166-criterio-mck.md), smoke +20,0 e R=5 +21,5 in banda [4;30] con riconciliazione |1,5|, 21/21 guardie, disasm Δ=−1 centrato, churn batteria dichiarato e curato col rebuild tornato al byte ×2, ordine §6 rispettato per entrambi i pin (commit e582e16f→3391c3dc→26ae107d→017dec94/8a113f96→036f7a46). Il conjunct «criteri PRE-registrati + copia-gate» del claim, invece, NON regge per intero.

## Rilievi
1. **Copia-gate prodotto, non eseguito come revisione dei path di scrittura** (pista 1 CONFERMATA): due omissioni dichiarate (pair164; VERD orm2) più una TERZA non dichiarata — il verdetto orm2 titola «pin s165 MISURATO 092dcff431bef876» (etichetta stantia con hash s166: `s166-orm2-coppia-verdetto.out` r.1) e `s166-orm2-coppia.sh` si contraddice: r.5 «RIF INVARIATO = registrato s162», r.188 «RIF AGGIORNATO s166-mattina». La lezione ⭐⭐ del session file lo ammette: il grep dei nomi di scrittura NON c'era.
2. **Criterio fantasma t16**: `s166-pair2.sh` e `s166-pair-verdetto-t16.out` citano `s166-criterio-pair-t16.md`, che NON ESISTE nel repo (find vuoto); t15=1,746 è entrato nella storia mediane via copia, senza carta. Sostanza salva (banda [1,738;1,799] INVARIATA), ma per t16 «criteri PRE-registrati» è falso alla lettera.
3. **RIF ORM aggiornato fuori criterio** (pista 4): REF2 34,47→34,27 e RREF→[7,023;7,053] su DUE gambe, dichiarato solo nel commento del copione mentre `s166-criterio-orm.md` pre-registrato dice «RIF s162 REGGE». Direzione difendibile (ultima finestra valida quieta), forma carente.
4. **Emenda del rinvio** (pista 2): appesa all'atto originale con motivazione sostanziale (finestra quieta, non numero di sessione) e in direzione conservativa — ma il VERD promesso `s166-orm-coppia-verdetto-r2.out` non esiste; la catena mattina si ricostruisce solo da git.
5. **Retry-wrapper fuori repo** (pista 3): `/private/tmp/s166-retry-mckr5.sh` porta il predicato di quiete (c<5,0 su mediaanalysisd/mdworker, 6×30s) che negli atti non c'è; `mckr5-tentativo1-gate-fail.log` = 35 byte senza timestamp né processo colpevole.
6. **Refutate**: pista 5 (sentinella: 2 finestre su 3 annullate erano DAVVERO sporche, la terza un tick; ri-fondazione pre-registrata in apertura) e pista 6 (§6 rispettato, churn dichiarato).

## Azioni
1. Copia-gate v2: grep AUTOMATICO di tutti i path di scrittura (.done/.rc/VERD/log) E delle etichette pin/sessione, esito nel manifest.
2. Carta t16 sanata (retro-dichiarata) + regola: ogni tN nuovo = carta prima del lancio.
3. Retry-wrapper e predicato di quiete DENTRO wp166-harness; log tentativo-gate con timestamp e colpevole.
4. Regola di aggiornamento RIF (quando, quante gambe) emendata in criterio-orm prima della prossima coppia; sanare l'auto-contraddizione di orm2.
5. Contare la terza omissione di classe copia (header «pin s165») nel registro incidenti.
