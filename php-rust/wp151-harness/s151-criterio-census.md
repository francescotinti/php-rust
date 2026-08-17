# Criterio S-151 — census tranche-5 @ s150 (pre-registrato PRIMA di ogni run; concilio S-151)

1. Spec VINCOLANTE: `concilio/verbali/team-misura.md §Spec` + `concilio/sintesi.md §RATIFICA-A1`.
   Chiavi SIMBOLICHE (canale+simbolo/opcode/helper), mai file:riga.
2. Canali (partizione per TIPO, Object ≠ Str/Array ≠ Ref): C1 clone/drop
   handle Object per sito (rifonda il tetto movimenti — canale SEPARATO ed
   ESCLUSO dalla somma pro-A3) · C2 borrow/borrow_mut Object per sito ·
   C3 malloc/oggetto a costruzione (header+Vec props+dyn) · C4 gc_note con
   arg Object per sito · C5 clone/drop VALORI Zval a PropGet/PropSet.
   Più: testa hostcall per-NOME + scan outlier (none.other 94,6M ·
   class_exists 9,7M · __reflect_* 12,4M) + siti re-entranti contati +
   N1–N6 Leijen (footprint, distribuzione #props p50/p90/p99).
3. Identità OBBLIGATORIE (violazione ⇒ census NULLO, KS-G1): per canale
   Σsiti==tot dallo stesso hook · contatore OVERLAP tra canali atteso 0 e
   STAMPATO · conservazione nascite+cloni==drop+vivi_a_fine_request per
   classe · smoke probe a esito ESATTO prima del run.
4. Ricetta probe (Gregg R7): sorgente = ricetta pin s150 (candidato
   cbbe71735effb165) su COPIA census (mai il pin), diff probe agli atti
   (`s151-census-copia.diff`), hash probe REGISTRATO nell'atto di build,
   env ESPLICITO nel header dello script, workdir APFS ≥100 char dichiarato,
   stato di FUSIONE dichiarato e costante tra questo run e ogni ri-census.
   CONTEGGI, mai cifre di tempo dal binario census.
5. Re-istruzione scarto +3,2%: probe s149 CONSERVATO (hash f3a111ac92cac3ef)
   vs probe s150 NUOVO, STESSO workload piccolo; giudizio: |Δ hostcall.n|
   ≤1% ⇒ scarto istruito (indiziato: auto-conteggio probe s149); >1% ⇒
   resta aperto e — se non istruito entro la sessione census — i numeri A1
   sono declassati a INDICATIVI (Bak KS-3).
5-bis. EMENDA DICHIARATA (2026-08-17, PRIMA della prima esecuzione del §5):
   il binario probe s149 «conservato» NON è reperibile (stash phpr-old-target,
   /private/tmp, php-rust-output ispezionati; la conservazione era dichiarata
   SENZA path — osservazione di processo a verbale). Il §5 si esegue sui
   VALORI REGISTRATI, stesso workload ORM pieno: hostcall.n del probe s151 vs
   335.837.200 (probe s149, ×2 repliche + controllo path-lungo S-150) e vs
   325.416.908 (probe s148). Lettura pre-registrata: ≤1% da 325,4M ⇒ scarto
   ISTRUITO (sovra-conteggio del lignaggio s149 refutato dal probe nuovo);
   ≤1% da 335,8M ⇒ delta = proprietà STABILE del codice-conteggio di
   lignaggio s149 (che s151 EREDITA dall'albero — caveat dichiarato): scarto
   APERTO, istruttoria al diff SORGENTE s148tag vs s149name in S-152;
   altrove ⇒ aperto, testa nuova. I conteggi C1–C5 restano validi se le
   identità §3 reggono: lo scarto riguarda la sola testa hostcall.
6. GO/NO-GO A3c (pre-registrato ORA, prima di leggere qualunque numero;
   plenaria §3 della sintesi — la più severa governa, denominatore
   ARMONIZZATO): D_gap = gap ORM NETTO @ s150 = [30,52; 30,56] s (da
   `wp150-harness/s150-orm-coppia-verdetto.out`: phpr 35,52–35,53 −
   oracle 4,97–5,00). banda_netta = Σ_canali_non-movimento
   (conteggio_census × prezzo netto del SOSTITUTIVO-mock), pavimenti e
   componenti non prezzati DICHIARATI. GO(A3c) ⇔
   S1: banda_netta ≥ 0,50 s (Gregg R3: 2× soglia 0,7%×35,5) ∧
   S2: banda_netta ≥ 5%·D_gap = 1,53 s (Bak) ∧
   S3: somma canali Object ≥ 15%·D_gap = 4,58 s (Klabnik R6/Hoare R5/
   Hejlsberg KS-H3). Sotto una qualunque ⇒ A3c CHIUSA (veto stile
   NaN-boxing); restano A3a/A3b micro-judged e le leve per NOME.
7. Prezzi SOLO da sonde pair-style sul PIN + mock A/B dedicati; NESSUNA
   cifra Gemini in criterio; tetto 1,27 s NON citabile (stale, Gregg R5).
8. Gamba SERVER (Pedersen R4): dovuta in A1, può slittare a S-152 se la
   finestra chiude — slittamento DICHIARATO, non-gate per la gamba CLI.
