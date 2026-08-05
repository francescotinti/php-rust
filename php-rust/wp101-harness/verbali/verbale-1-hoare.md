# Verbale Sedia 1 — Tony Hoare (design linguaggio/runtime, safe-only) — Concilio WP-101

**Oggetto**: report S-99.0 + bozza §S-100. **Perimetro**: sigillo eager, controfattuale statico, forma INT1, catena pin, bozza promozione.

## VERDETTO: CON EMENDAMENTI (una refutazione capitale sulla bozza §S-100, punto 2)

Riconosco ciò che regge: il controfattuale 4a è STATICO ma onesto — ho riletto `run.rs:1005-1074` e il corpo BinarySS/SC/Dst è esattamente come descritto (borrow su slot, guardie `matches!(Undef|Ref)`, `binary_fast` inline, zero call/marshalling/pop); il rimovibile è davvero solo carico+match del payload, e il criterio di riapertura (D_registro ≥ 0,7 misurato) è pre-registrato, non un editto. La patch INT1 è genuinamente di sola misura: pop/pop/push preserva la semantica (miss path identico, smoke byte-id), albero ripristinato, target-dir separato. Ho verificato il buco-sigillo richiesto: **l'unico lettore di `PHPR_REG_LOWER` è `reg_lower::enabled()`** — nessun canale secondario legge l'ambiente dopo il boot.

## REFUTAZIONE CAPITALE

**R1 — Il flip del default (§S-100 punto 2) è sottospecificato al punto da rendere incoerenti i suoi stessi gate.** `enabled()` è PRESENCE-based (`var_os(..).is_some()`: anche `PHPR_REG_LOWER=0` significa ON, trappola già latente oggi). "Pass registro ON senza env" non dice COME si ottiene il modo OFF dopo il flip: senza un contratto value-based, il gate A-PE-100-4 (braccio flag-OFF nel funnel) diventa irraggiungibile, i due bracci del dente anti-putenv (set→on rifiutato / unset→off rifiutato) vanno RI-DERIVATI sulla mappatura nuova, e la guardia di premessa M5 in `reg_lower.rs:597` (`assert!(!enabled())`) FALLIRÀ o mentirà nella batteria standard — la cifra 1726/0 non sopravvive al flip così com'è scritto. Un invariante non nominato non è mantenibile: il contratto d'ambiente va scritto PRIMA di flippare.

## EMENDAMENTI

- **A-HO-101-1 "Sigillo di tipo, non allowlist di call-site"**: la garanzia attuale è una convenzione su DUE main; ogni terzo embedder del runtime (test in-process via `run_source`, futuri bin, pool) resta non sigillato — cura enumerabile contro attacco non enumerabile (lezione WP-96). Prima del flip, forzare l'init a un chokepoint di costruzione del motore (token di boot richiesto dalla compile-entry, stile VmGate ZST di WP-83) o, minimo, dente che ogni `[[bin]]` linkante php-runtime sigilli.
- **A-HO-101-2 "Contratto d'ambiente del flip"**: parse value-based nominato (es. assente→ON, `=0`→OFF), ri-derivazione dei due bracci anti-putenv e della premessa M5, unit-cache key che continua a distinguere i modi. Rimedio di R1.
- **A-HO-101-3 "Timing non prova bit-identità"**: "5,43→5,44 ⇒ emissione davvero bit-identica" è un'inferenza invalida (uguaglianza di misura → proprietà statica). Declassare a ipotesi; l'unica evidenza ammessa è il dump-diff (A-HE-100-4), da eseguire sull'albero candidato al flip.
- **A-HO-101-4 "Identità del pin"**: un hash che "churna col relink — fa fede HEAD" rende il campo hash del PIN_REGISTRY non autoritativo: due binari diversi possono reclamare lo stesso pin. Registrare la tripla (commit, ricetta, hash esatto) e legare ogni `collaudato: sì` all'hash SU CUI il collaudo è girato.
- **A-HO-101-5 "57/43 non è una tariffa"**: la decomposizione INT1 assume additività su tre binari distinti (layout/I-cache non controllati) e la banda [0, 0,5] di 4a è convenzione (½ pavimento sonda), non misura. Pubblicare sempre come bande; vietare l'ereditarietà delle quote 57/43 fuori dal percorso pila (stessa classe dell'errore D=6,07 appena sepolto).

## KILL-SWITCH

- **KS-HO-101-1**: flip VOID se eseguito prima di (a) contratto d'ambiente nominato + dente anti-putenv ri-derivato, (b) dump-diff off/on sullo stesso albero candidato.
- **KS-HO-101-2**: ogni claim d'identità d'emissione fondato su timing anziché dump-diff ⇒ il gate corrispondente è NON soddisfatto.
- **KS-HO-101-3**: batteria post-flip con premesse invertite non aggiornate (M5 e affini) ⇒ la cifra 1726/0 non vale come gate di promozione.
