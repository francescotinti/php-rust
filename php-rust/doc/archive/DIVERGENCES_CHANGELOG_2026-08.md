# Archivio — changelog di PHPR_DIVERGENCES_FROM_PHP.md (2026-08-03 → 2026-08-13)

> Congelato il 2026-09-10 (revisione del catalogo, decisione utente).
> Copre le sessioni S-96 → S-134. Il changelog precedente (2026-07) è in
> `DIVERGENCES_CHANGELOG_2026-07.md`; il catalogo vivo resta
> `PHPR_DIVERGENCES_FROM_PHP.md`, che per regola contiene SOLO voci aperte.

---


- 2026-08-13 (S-134, az.rev. S-133 #5): DICHIARATA la risoluzione del gate
  `teardown` (wp133-harness/s133-fx-teardown-gate.sh, in catena s109): la
  parità sulla finestra dei distruttori a fine richiesta è PROVATA sui 7
  vettori nominati (dtor self/peer, gc annidata, resurrezione, dtor-walk,
  WeakReference, doppio gc) byte-id oracle==s131==s132 — e NON OLTRE: il
  buco «by construction» della finestra teardown è RISTRETTO dai 7 vettori,
  non chiuso; ogni claim di assenza oltre quei vettori eccede la risoluzione
  del gate (REGOLE: claim di ASSENZA oltre la risoluzione = vietato).
- 2026-08-09 (S-121): AGGIUNTA §3.18 — preg_match: `(?J)` nomi duplicati
  → false, e nome utente col prefisso sintetico `__phprbg` nascosto
  (fixture bilaterale per NOME, az. rev. S-120 #1; gate fail-closed in
  wp121-harness).
- 2026-08-08 (S-111): AGGIUNTA §3.17 — riga sbagliata nel warning «A
  non-numeric value encountered» (famiglia §3.13/§3.16), scoperta dal
  revisore semantica collaudando il giudice held-out `err.php` senza `@`.
- 2026-08-06 (S-103 sera, Concilio WP-105): §3.12 emendata coi TRE regimi
  (strict_types e `.=` CONSERVANO — il censimento 4/4 era mono-regime);
  §3.13 RIAPERTA sul canale unit (include/eval: file del flush, provato
  su HEAD); da S-104: divergenza symlink-docroot del server da catalogare
  (php -S canonicalizza, phpr no — osservata nel collaudo S-103).
- 2026-08-06 (S-103): §3.13 chiusa per la famiglia PropGet (fix S-102) con
  claim ridimensionato (5 siti su ~435, A-ST-104-2); perimetri §3.11/§3.12
  MISURATI con probe (§3.12 rititolata typed-LVALUE: azzera anche senza
  ref); nuova divergenza generator-in-cycle PROVATA
  (`wp103-harness/recv-fixtures-gen/gen1-verdetto.out`: il ciclo via
  Generator non è mai raccolto, dtor solo a shutdown — buco A-HO-103-2).
- 2026-08-06 (S-101): §3.13 — warning «Undefined property» attribuito alla
  riga dello statement successivo (famiglia fetch-undef, trovata dalla
  fixture 09 di H-C1; identica nei due modi).
- 2026-08-05 (S-100): §3.11 — AssignOp con lhs indefinito senza warning
  «Undefined variable» (famiglia di §3(c), percorso compound-assign);
  §3.12 — typed-ref azzerato da Zend dopo AssignOp fallito, phpr conserva.
  Entrambe trovate dalle trappole A-ST-99-3 (b)/(e), identiche nei due modi.
- 2026-08-04: §3.10 — argomenti `string` dei builtin: coercizione con warning
  invece di `TypeError` (trovata di lato in S-96.0 costruendo le fixture di
  liveness; verificata sul binario di parità; perimetro NON misurato).
