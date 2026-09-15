# Revisione S-175 — lente PROCESSO (revisore singolo, sola lettura)

## Verdetto: REGGE CON RILIEVI

### Ciò che regge
- Promozione: identità a contenuto (48 B), gate batteria/corpus/fixture/ORM/hk rc=0; copione emendato nei soli tag (manifest), collaudato dal run.
- Criteri committati PRIMA dei lanci: coppia 02:41/02:56 vs t20 03:06; ORM 02:41 vs 05:48; mock 02:45 vs 05:59; mutante-gc 02:38 vs 02:56.
- Copie dichiarate con manifest; divergenze solo quelle scritte.
- ORM tentativo 3 = ri-esecuzione INTEGRALE del criterio invariato (progress.txt 06:16-06:20); la pre-attesa anti-flare è guardia più stretta: legittimo per §3.
- Mock: mutanti dichiarati SCORRETTI, «mai promozione»; controllo nullo A−Z 0,24/0,07.

### Rilievi
1. **Divergenza GC NON a catalogo.** `s175-mutante-gc.sh:7` e il suo verdetto dicono «momento della raccolta … a catalogo», ma `PHPR_DIVERGENCES_FROM_PHP.md` non la contiene (ultimo commit 2026-09-10). Per §9 l'az.rev. (a) NON è chiusa.
2. **Mock lanciato contro il suo criterio.** p.5 vuole `rimisura.done rc=0`; `s175-lancio-mock.sh` gata sull'ESISTENZA del file e ha lanciato su `rc=8` (log 05:55:08), tra due quiescenze ORM fallite per flare. Tetto +4,67: margine 0,67 sul pavimento 4, rumore B' 0,93 — cifra a filo. Rischio: tetto (flag a costo ZERO) preso per cifra di leva.
3. **Gate micro non di record.** Micro R=5 e conferma post-pin con updater attivo e Data 0,47G, igiene assente dal `.out` (incidente #1, dichiarato): «tutti i gate» vale a GUARDIA; scoreboard 2,3/2,6 provvisorio.
4. **Demone con `sed`+commit+push** su un copione di misura: prosegue anche a commit FALLITO ⇒ ammette un lancio con criterio non committato. `s175-lancio-orm-calmo.sh` committato DOPO il run (06:21 vs 06:11), senza manifest.
5. ORM: sentinella oracle 4,97 fuori banda (t19 a filo, t20 fuori) ⇒ Delta_norm non giudicante: coppia valida per PARITÀ, non per cifra; §4 blocca alla terza sessione. Corsa 5 del mutante eseguita in finestra di promozione (02:38), verdetto letto in PRE sovrascritto (corsa 4 conservata).

### Azioni S-176
1. Catalogare per NOME la divergenza «momento della raccolta GC» con fixture fx-sw2-gc; solo allora (a) è chiusa.
2. Rimisura micro R=5 + conferma post-pin con igiene (Data, swap, top) nel `.out`; watchdog disco nel lanciatore.
3. Rerun del mock DOPO ORM rc=0 con lanciatore che gata su `^rc=0`; criterio della leva flag gc-idle: attesa < tetto, KILL pre-registrato se prop-dq < 4.
4. Catena: STOP a commit fallito; ogni lanciatore nuovo committato con manifest prima del run.
5. Istruttoria drift sentinella oracle ORM prima della prossima coppia.
