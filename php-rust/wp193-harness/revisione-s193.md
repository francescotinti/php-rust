# Revisione S-193 (lente: processo). Generato con assistenza AI (Claude Fable 5.2).

**Verdetto: REGGE CON RISERVE.** Cifre conformi ai file; «confermata» e «di record» dicono più dei criteri.

## Rilievi
1. **«Confermato dalla notte» non ha una regola scritta.** Il criterio (p.1) dice «conferma o smentisce» senza dire con quali esiti. Agli atti: `catena-s193.done` rc=8, `coda-guardia.done` rc=5, server rc=2; verdi solo t29 e t27, confronti fra notti e binari diversi (REGOLE §3: sola direzione): nessun A/B s185 contro s184 su WordPress o ORM.
2. **Kill scattati e non in riga 1.** I verdetti t4 scrivono «KS-189-5 (cura NON promossa)» e «KS-189-6 HALT (login=0.975)». REGOLE §8 li vuole in riga 1 del NEXT per nome: non ci sono. La delibera p.9, dichiarata, emenda una guardia senza rieseguirla (§3) e cita come prova un tripwire «mai cifra». Scoreboard server ancora sul pin s188.
3. **Guardia: stato più ottimista del criterio.** Il p.2 dice «rc=5 ⇒ resta aperta»; il NEXT scrive «K0 rientrato». La riga 18 conserva «mcalls-dq +3,0» mentre il punto 2 dà +4,33. «Ereditato da s184» somma b4 (+3,00, altra notte) e conferma diurna (+0,00, rumore 1,50): direzione, non cifra. Finestra b5b meno calma della catena (E2 91/112/74/35 % contro 36/11/11/25): +4,00 appaiata su soglia 4,00 è fragile nei due sensi.
4. **t29: la gamba bassa è la segnalata.** 6,449 viene da leg1, partita alle 19:00:20, lo stesso secondo della caduta di b5 per la sonda; ictx phpr1 801 contro 45; sentinella E5 fuori (4,79 < 4,81). Regge: leg2 6,576 < 7,045. Ma rc=0 certifica validità e parità, non «sotto t28»: il verdetto porta testo fossile («t20», «attesa a tetto 0.0-0.0»).
5. **t27**: l'etichetta «riferimento pin s185: t24, t25, t26» è falsa (notti su s184).
6. **Merge e chiusura prima della revisione.** `main` = a232ccf7 («S-193 closed») già su origin: Rust di sei sessioni su un ramo con CI GitHub rossa su Linux. Verificato: nessun sorgente Rust dopo e0a71c40; CI locale OK fino a 662943cc, non oltre.

## Azioni
1. Riga 1 del NEXT: KS-189-5 e KS-189-6 per nome, con «superati per delibera 20/11».
2. NEXT: «confermato in direzione (t27, t29); guardia rc=5 aperta»; aggiornare la riga 18 a +4,33.
3. Ripetere la guardia in finestra calma prima di ogni leva sulle chiamate.
4. Prossimo criterio di notte: regola esplicita esiti → conferma; t30 col confronto alla notte precedente nel giudice.
5. Leggere la CI GitHub di `main` dopo il merge.
