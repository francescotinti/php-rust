# Verbale sedia 4 — Hejlsberg (Concilio WP-93, revisione S-91.0)

Perimetro: catene di evidenza (ledger, attempts, supersessioni), identità toolchain. Mandato: REFUTARE.

## VERDETTO
La delibera unica di formato (design91-ledger.md, d783a74) è ben scritta ma **applicata a metà**: la grammar v2 delle battery morde su UN solo percorso di consumazione, la grammar v2 campaign non ha alcun dente, e il corpus non distingue «max perché ultima» da «max perché valida». Due refutazioni capitali.

## Q1 — battery-91pre v7 (trap/att_row/head-move)
- **Doppio writer=: NO.** `att_row` (r.47-52) appende `writer=script:` SOLO nel ramo `*)`; la riga del trap (r.58) contiene `writer=operator` ⇒ ramo `*writer=*)` la scrive verbatim, UN solo writer. Residuo minore: il case è substring-match, non field-anchored — un valore k=v contenente «writer=» sopprimerebbe la firma script (oggi non raggiungibile: FIRSTFAIL è sed-vincolato `[a-z0-9-]*`).
- **assert_head_unmoved NON copre tutti i terminali.** È chiamato solo a r.161 (prima di PASS/FAIL). Il REFUSE porcelain (r.78) e l'ABORT del trap (r.58) emettono la riga terminale SENZA il check — la lettera della delibera («prima di OGNI riga terminale», design91-ledger r.18-19) è violata. Il trap è il caso serio: può scattare DOPO un HEAD-move mid-battery e la riga stampa `rev=$GIT_REV` con `reason=signal` che maschera il head-moved.
- **Trap armato prima del primo att_row: SÌ** (trap r.61, primo att_row possibile r.78). Finestra residua non dichiarata: un segnale nel preambolo r.24-60 lascia il ledger muto.

## Q2 — battery-equivalence grammar v2
- Riga vuota in V2ROWS: impossibile per costruzione (output di grep, guardia `[ -n ]`); se iniettata, `grep -cvE` la conta come violazione (verificato: count=1) — fail-closed.
- `writer=script:SHA` a fine riga: il ramo `$` in `( |$)` funziona su grep BSD/macOS (verificato: match=1). Le righe PASS, che chiudono con writer=, passano.
- Esiti fuori set: riga senza `esito=` o con esito ignoto ⇒ conteggiata (grep -cv, fail-closed). Sfuggono: (a) **TUTTO il blocco v2 (r.389-404), come A-AH50/A-AH54, vive DENTRO `if [ "$SAME_REV" = 1 ]`** — una consumazione in modalità EQUIVALENZA di una battery 9x salta writer=, àncore e triangolo attempts: la delibera dice «alla consumazione», il codice dice «alla consumazione same-rev». **REFUTAZIONE CAPITALE.** (b) `battery-9[1-9]*` non copre battery a 3 cifre (100pre) — buco futuro, da dichiarare. (c) `sha256=[0-9a-f]{64}` non ancorato: 65-hex passa (minore).

## Q3 — ledger_supersession_proved / cite_max
Il dente (r.578-591) è consultato SOLO per g<max (r.740). Per g non-max con prova FAIL: citarla come «verità» verbale passa (il wording è morto per design A-SK-72), ma le sue CIFRE sono fuori corpus (r.451-453) ⇒ residuo solo narrativo, accettabile. **Il buco reale è il max-FAIL**: il filtro corpus (r.444-453) tiene la generazione MASSIMA per solo numero G, senza check di esito — una campagna terminata con FAIL finale mai superseduto lascerebbe il suo .out NEL corpus e le sue cifre legalizzerebbero token come verità. Oggi non-live (m89 max=g3 PASS, m90 max=g2 PASS, verificato dai ledger), ma il meccanismo è aperto. **REFUTAZIONE CAPITALE** (mechanism-level).

## Q4 — campaign v2 solo deliberata
**Nessun dente, solo delibera.** measure91-campaign.sh/verdict91.sh NON esistono (verificato); measure90-campaign.sh è riusabile a mano per una m91 con grammatica v1; `ledger_supersession_proved` parsa felicemente un ledger v1 (regex supersede_of/esito=FAIL, zero check su campaign_sha/reason/authorize); nessun checker chiavato su `m9[1-9]`. La lezione di A-AH58 («la provenienza era una convenzione, non un campo») si applica alla delibera stessa.

## Emendamenti
- **A-AH61**: il blocco grammar v2 + denti attempts (A-AH50/54/58/59) esce dal ramo same-rev: morde su ENTRAMBI i percorsi di consumazione.
- **A-AH62**: assert_head_unmoved chiamato anche nel REFUSE porcelain e nel trap (HEAD mosso ⇒ reason=signal+head-moved sulla riga).
- **A-AH63**: dente pre-nascita campaign v2, committato ORA: qualunque riga di `m9[1-9]*.campaign.ledger` senza disciplina v2 ⇒ rifiuto alla prima consumazione.
- **A-AH64**: ammissione al corpus della generazione max richiede riga `esito=PASS` committata; max-FAIL = storia (classe judge=no), mai verità.

## Kill-switch
- **KS-AH-93-1**: consumazione (qualunque percorso) di battery 9x senza verifica grammar v2 ⇒ consumazione VOID.
- **KS-AH-93-2**: riga m9[1-9] fuori grammatica v2 ⇒ campagna VOID.
- **KS-AH-93-3**: cifra legalizzata da un .out max senza riga PASS committata ⇒ doc FAIL.

## Refutazioni capitali
**SÌ, due**: (1) enforcement v2 battery solo same-rev + campaign v2 senza dente (Q2a+Q4 — la revisione di formato appena deliberata è per metà convenzione); (2) corpus ammette la generazione max senza esito (Q3).
