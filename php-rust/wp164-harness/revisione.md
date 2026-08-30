# Revisione S-164 — REVISORE SINGOLO, lente MISURA

## VERDETTO: REGGE CON RILIEVI

## Rilievi
1. **Arith: "indagine CHIUSA, nessuna regressione phpr" è più larga della misura.** La fase 2 copre SOLO s161→s163; il creep dichiarato in fase 1 (+0,6 ns/iter da s158) su N=250M vale 0,15 s — dentro la banda (a) di 0,40 s. La banda pre-registrata non avrebbe potuto refutare un creep della taglia osservata, e il braccio s158/s159 non c'era. Il claim vero è: «nessuna regressione TRA s161 e s163; tick=quantizzazione». Il giudice ×5 è derivato dichiarato (stesso driver, N×5): confrontabile, non è il rilievo.
2. **L-AL3 "non pagante" è NON-REFUTATO, non dimostrato.** Smoke R=2 con rumore drop-1 A'=6,0 > soglia nominale 4: un D vero di 4–6 ns è indistinguibile da +0,0. Il p.3b è stato applicato alla lettera, ma la lezione esportata («la classe pooling puro è ridimensionata») generalizza da una misura che non poteva vedere metà della banda attesa [4;10]. Le guardie morse (arrload −8,5 = presidio DIRETTO di L-AU1; objdropdef −6,7) muoiono col revert al byte verificato: rischio residuo nullo sul pin, ma l'arbitrato dovuto (emenda S-161 #4) non è mai avvenuto.
3. **Census AL3: due assoluzioni off-by-one consecutive sullo stesso arbitro.** Attesa emendata pre-run 200000→199999 (cold-start), esito 199998, spiegazione +1 = buffer del Vec — post-hoc e NON verificata da rerun (bastava `with_capacity` o un rerun warm). In più il criterio emendato è internamente incoerente: la clausola (a) dice ancora «Δ ≠ 200000 ⇒ STOP» mentre p.3 dice 199999. Il verdetto leva non ne dipende (p.3b), ma il «meccanismo nominato» sì.
4. **Oracle ORM di nuovo lato veloce (4,86/4,85 vs 4,87/4,90): seconda coppia consecutiva.** Con ORA_REF=4,885 fisso, un drift oracle di −0,03 s produce da solo Δ_norm ≈ −0,21 — quasi l'intero segnale osservato [−0,27;−0,01]. La sentinella p.5 («dintorni RES») non ha banda numerica: non può mordere.
5. **php-server NON riproducibile dal sorgente (661b490c ≠ 8d76d6f1).** t14 ha misurato il binario giusto (gate rc=9 sull'hash a inizio script, header MISURATO), ma il pin server ora esiste SOLO come artefatto di stash — eco del pin d45b578 irreproducibile. Nessuna istruttoria aperta.

## Azioni (S-165)
1. Riformulare a verbale il claim arith come «chiusa su s161→s163»; il creep da s158 resta nota aperta con giudice a 3 decimali nel promo.
2. Rerun census AL3 singolo (pool warm o `with_capacity`) per convertire il +1 da post-hoc a verificato; emendare i criteri in TUTTE le clausole, non solo nel preambolo.
3. Banda numerica pre-registrata per la sentinella oracle ORM + istruttoria drift (R=5 oracle-only o ri-fondazione ORA_REF).
4. Istruttoria non-riproducibilità php-server: ricetta server scritta e provata; duplicare lo stash pinnato prima della prossima promozione.
5. Se il pooling rientra in coda: smoke con R tale che soglia=4 (non rumore 6) prima di ogni verdetto «non pagante».
