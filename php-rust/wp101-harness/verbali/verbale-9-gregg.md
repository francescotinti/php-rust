# Verbale sedia 9 — Brendan Gregg (metodologia di misura, attribuzione) — Concilio WP-101

## §FONDAMENTALI

**Avanzamento oggetto (mandato inverso — che cosa sappiamo OGGI che ieri non sapevamo, per NOME):**
1. **D decomposto**: 6,27 ns/occ = 3,60 call/marshalling (57%) + 2,67 traffico Vec (43%), misurato con tripletta C0/INT1/C2 stessa finestra, R=5, raw pubblicati (`premisura-rollout99.out`). Conoscenza nuova e ben costruita.
2. **D_registro ≈ 0** (banda [0, 0,5] ns/occ, pre-registrata): le forme registro inlineano già `binary_fast` — il rollout Add lì è refutato PRIMA di scrivere codice. Nuovo.
3. **Emissione flag-on bit-identica sotto H-B2** (arith on 5,43→5,44): prima fotografia delle due gambe INSIEME post-H-B2; vantaggio composto −26,9%. Nuovo.
4. **Il pin server 365f4d40 non poteva funzionare** (feature `axum-server` assente): refutato a tavolino, sostituito e collaudato rc=0. Nuovo (sul perimetro, non sul motore).
5. **Parità WP chiusa per NOME sull'emissione H-A2+H-B2** (debito regola-n.2 saldato). Nuovo come fatto di collaudo.

**Solo ri-fotografato**: la coppia peak/CPU (gamba phpr piatta) e le sei categorie (17,5/13,8/8,6/6,9/4,9/3,8 — stesso ordinamento e grandezza di S-97.0; "H-C/H-D rianimate" è contabilità della gamba oracle sanata, non scoperta). Corretto che sia così: era l'ordine del concilio.

**Contatore misure**: azzerato con merito — sessione di SOLE misure, tre .out macchina con spread e pavimenti.

**Rischio trascurato**: la gamba ORACLE peak si muove del **+28,7% (media: 346,3→445,8 MB)** e **+12,1% (full: 745,6→836,0 MB)** tra WP-94 e S-99 — stessa ricetta, stesso brew 8.5.7 — e nessun file ne dà una causa.

## VERDETTO: PASS con tre refutazioni (nessuna capitale) — la sessione ha prodotto conoscenza vera; il rischio vive nel modo in cui le fotografie peak verranno usate.

## Refutazioni

**R1 — «i rapporti peak si muovono per la gamba oracle» era inferenza non pubblicata.** L'ho verificata io dai raw (`pair94-ratios.out` vs `pair99-ratios.out`): VERA (oracle +28,7%/+12,1%; phpr +2,7%/−0,45%). Ma i .out di S-99 la asseriscono senza la tabella a quattro gambe; e la gamba phpr media peak NON è piatta (+2,7%), è "piccola". La verifica era a un `paste` di distanza e non è stata fatta.

**R2 — l'anomalia oracle resta senza nome.** Un peak che oscilla ~29% tra due sere a ricetta identica dice che O il peak oracle ha varianza run-to-run enorme O qualcosa nell'ambiente è cambiato (brew bump? cache? ASLR/allocator?). Finché non è misurato, OGNI rapporto peak porta un'incertezza potenziale di quella taglia: 2,374× e 2,698× sono fotografie con barra d'errore ignota su un lato.

**R3 — «stessa finestra» ha una cucitura misurabile ma non dichiarata.** arith flag-off netto = 7,52 in `micro-rebaseline99.out` (R=3) e 7,44 in `premisura-rollout99.out` (R=5): 1,1%, FUORI dallo spread pubblicato (0,06). Il rapporto 12,7 (flag-on) incrocia gambe di due blocchi. Innocuo su effetti 13×; da dichiarare quando un rapporto attraversa i file. La parità server con build cargo concorrente è ACCETTABILE: giudice bit/fail, l'errore possibile è solo un falso FAIL da timeout — direzione sicura; quel run non diventi mai cronometro.

## Bozza S-100

Il giudice della promozione (punto 2: coppia stessa-sera in modo nuovo + corpus due modi + parità server) è giusto, ma **manca il criterio di accettazione pre-registrato**: nessuna banda entro cui la coppia flag-on deve cadere per non bocciare il flip. H-C: lo strumento c'è (tavola arith-decomposition S-97 + op census dietro feature); il «confronto col profilo oracle» non nomina lo strumento del lato oracle.

## Emendamenti
- **A-GR-101-1**: pubblicare in S-100 la tabella 2×2×2 delle gambe raw WP-94/S-99 (fatta qui, va nei .out) ogni volta che si attribuisce un movimento di rapporto a una gamba.
- **A-GR-101-2**: PRIMA della coppia flag-on, misurare lo spread run-to-run del peak ORACLE (R≥2 sul media group) e dare un nome all'anomalia +28,7%; fino ad allora i rapporti peak si pubblicano come banda.
- **A-GR-101-3**: pre-registrare le bande di accettazione del flip (full CPU e peak flag-on entro spread di WP-94/99 lato phpr) PRIMA del run.
- **A-GR-101-4**: per H-C, nominare lo strumento della gamba oracle (census opcode o profiler) nel programma, non in sessione.

## Kill-switch
- **KS-GR-101-1**: se la coppia flag-on viene eseguita SENZA banda pre-registrata (A-GR-101-3), il suo verdetto sulla promozione è VOID: resta fotografia, non gate.
- **KS-GR-101-2**: se una gamba oracle di una coppia si muove >10% vs il riferimento senza causa nominata, i rapporti di quella coppia non sono confrontabili cross-sessione finché lo spread oracle non è misurato.
