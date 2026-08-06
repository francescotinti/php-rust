# VERBALE 3 — SEDIA KLABNIK (spec, testabilità, matrici e gate) — Concilio WP-105

## VERDETTO

S-103: metodo SOLIDO (audit che ribalta il verdetto meccanico, denominatore riletto dal sorgente, regola set-che-scende scritta). Ma la chiusura NON è gradabile come «parity-null PROVATO»: è parity-null DICHIARATO — la prova copre la funzione, non i giudici. Bozza S-104: approvabile SOLO con gli emendamenti sotto.

## REFUTAZIONI

**R1 (punto a) — L'oracle non è pinnato.** Nei tre fixture-gate e nel collaudo server `ORACLE` è sovrascrivibile da env, senza hash né controllo di versione. Un `brew upgrade php` cambia il giudice in silenzio e il gate resta «verde». Fail-closed su UN solo lato è un gate mezzo aperto: A-KL-104-2 ha pinnato l'imputato, non il tribunale.

**R2 (punto a) — Il mode-probe può dare falso OK.** (i) Prova il modo su un programma SINTETICO (`for`+add), non sulle unità giudicate: se il lowering non copre i corpi property-heavy delle fixture, il probe «OK» non testimonia il modo del codice sotto giudizio (il collaudo server, con C4 per-unità su cb2.php, fa meglio: quello è il modello). (ii) Il grep gira su `2>&1`: una stringa d'errore che contenga una forma-registro simula ON. (iii) La lista forme è chiusa A MANO: nessun tripwire la lega all'enum sorgente — se le forme vengono rinominate il gate va VOID (accettabile), ma se ne nascono di NUOVE accanto alle vecchie il probe resta cieco alla copertura.

**R3 (punto b) — «Fa fede HEAD» con hash che churna al relink non è un pin** (precedente: d45b578 irreproducibile). Il doppio rerun del corpus (4e1cc41a→f45a5d19) dimostra che la sequenza di chiusura NON era specificata: è stata improvvisazione corretta, non regola. Il churn non si insegue a valle: si congela a monte.

**R4 (punto c) — La prova «corpus+batteria+server» prova la parità FUNZIONALE, non quella dei GIUDICI.** I numeri micro in baseline (12,3/11,5/7,4/…) sono misurati su d0b01362 (pin S-102), NON sul pin di chiusura f45a5d19 — e la tabella NEXT_SESSION non lo dichiara nella riga. `dcn!`/memcensus «gated» costano comunque un branch per evento: «i rapporti non cambiano per costruzione» è un'asserzione, non una misura. Il debito coppia WP è nominato onestamente, ma la definizione di parity-null oggi non è verificabile.

**R5 (punto d) — La lezione da5c2948 è a registro ma non è un GATE**: nulla impedisce di rifarlo. `collaudo.done` non porta né hash del pin né nome dello stash; il cross-mode confronta con `collaudo-out-$OTHER` PERSISTENTE senza verificare che l'altro braccio girasse sullo STESSO pin — un braccio stantio di un pin precedente può firmare il «cross-mode byte-id».

**R6 (minore)** — `s103-recv-fixtures.sh` dichiara in testa «rc: 0/1» ma esce 66 su VOID: contratto rc incompleto nella spec del file.

## EMENDAMENTI

- **A-KL-105-1** — Oracle pinnato fail-closed: ogni gate esige `ORACLE_PIN_ATTESO` (hash del binario o `php -v` esatto), simmetrico ad A-KL-104-2. Assente o mismatch ⇒ VOID.
- **A-KL-105-2** — **REGOLA DEL PIN DI CHIUSURA** (scritta come set-che-scende): sequenza atomica «build finale → hash → PIN file + STASH → batteria → fixture → corpus ×2», TUTTA sullo stesso hash verificato in testa a ogni launcher; QUALUNQUE rebuild azzera la sequenza da capo.
- **A-KL-105-3** — Parity-null VERIFICABILE = funzionale (corpus+batteria+fixture sul pin) **E** strumentale (micro 6 categorie SUL pin di chiusura, Δ entro banda pre-registrata vs pin precedente). Senza il braccio strumentale la coppia WP NON è differibile.
- **A-KL-105-4** — `collaudo.done` arricchito: pin hash + esito + nome stash; cross-mode VOID se il `.done` dell'altro braccio non porta lo stesso PIN_SRV.
- **A-KL-105-5** — Mode-probe per-unità sulle fixture giudicate (modello C4), grep separato da stderr; header rc dei gate allineato (0/1/66).

## KILL-SWITCH

- **KS-KL-105-1** — Un pin citato «GRADATO» senza stash contestuale al grading è retroattivamente NON-GRADATO: incitabile in registro e rotazione.
- **KS-KL-105-2** — Nessun numero entra in baseline se misurato su pin ≠ pin di chiusura senza dichiararlo NELLA riga della tabella.

## PRIORITÀ S-104

1. Verdetto A/B R=7 (invariato). 2. A-KL-105-1/4/5 nei launcher PRIMA del gate H-C2 (costo ~minuti, chiude R1/R5). 3. LEVA H-C2 col gate pieno = prima applicazione integrale di A-KL-105-2 e -3; la coppia WP bimodale salda il debito R4. 4. H-D SiteTag. 5. Generator-in-cycle (decisione catalogata).
