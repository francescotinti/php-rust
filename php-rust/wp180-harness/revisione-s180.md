# Revisione S-180 — lente PROCESSO

## Verdetto: REGGE CON RILIEVI

## Ciò che regge
- Ri-pin s180: copia dichiarata di S-177 (`s180-promozione-copia.diff`), criterio pre-registrato, build al byte ×2, batteria rc dal comando, corpus 1412 zero flip, 29 fixture byte-id, ORM 16 nomi, micro R=5; prop-dq a sola direzione.
- Cap LOC (573e9dfa) alzato nel dente con cifre esatte; revert L-CM1 (b3e48919) al byte; keep-partial-wins non si applica (nessun guadagno mai misurato).
- Quattro commit di ratifica attribuiti; raw S-178/179 persistenti fuori repo.
- Il tetto S-179 basta a declassare la leva: azzerando l'intero cammino (0,42/35,7 s) il guadagno è ≤1,2 %; il 68 % per conteggio è irrilevante perché il tutto maggiora il sottoinsieme.

## Rilievi
1. **Ratifica senza ricalcolo**: S-180 non ha ricontato una cifra dai raw (`~/Claude/phpr-s179-cost-resume-results/dense-timings.json`); la «revisione indipendente» S-179 è dello stesso modello (WP_SESSION_180 p.8).
2. **Verdetto elevato oltre la fonte**: REPORT-resume dice «priorità bassa», «non tetto rigoroso»; S-180 scrive «CADUTA con meccanismo» ed estende a readonly NON misurato (WP p.3; NEXT «privato/readonly = CADUTO»): chiusura di fronte su misura singola.
3. **HEAD del pin**: `git rev-parse` DOPO la build (s180-promozione.sh:56), nessuna guardia tree-pulito; bffaaf8f (23:47:44) cade dentro la build (23:45:41→23:49) e tocca criterio (+2) e verdetto parziale. PIN_REGISTRY:74 «sorgente @ bffaaf8f» impreciso; dichiarato solo in WP p.6.
4. **hk gate**: `promo-out/hk.rc = 1` mentre il verdetto scrive 0E/0F dal summary (s180-promozione.sh:226-232); ereditato da S-177, mai dichiarato.
5. **Deviazione dal handoff**: S-177 NEXT:31 prescriveva l'A/B (rc=4 ⇒ revert); S-180 reverte senza A/B con regola nuova (non in REGOLE) e vantaggio collaterale «bytecode sotto cap» (573e9dfa): motivazioni sovrapposte, deviazione non nominata.
6. **Serie non dichiarata**: S-177→S-180 = 4 sessioni senza leva spedita; REGOLE §1 (≥70 % oggetto) violata senza quantificazione.
7. **E2 150 % totale**: campioni 67-100 % = ~1 core occupato durante le micro di record; accettabile senza attese, non per Δ fini.
8. **Licenza**: PHP 3.01 integrale (php-rust/LICENSE:25-45): clausole 4 (nome «PHP» — il prodotto è php-rust) e 6 («includes PHP software») non adattate; ratifica senza verifica (4e16d749).
9. `._*` AppleDouble vivi in promo-out/pair-out (veleno del runner CI), non purgati.

## Azioni S-181
- Ricalcolare le due mediane da dense-timings.json, registrarle; declassare readonly a «non misurato».
- Catena: HEAD + `git status --porcelain` PRIMA della build; `hk.rc` nel verdetto.
- Rettificare PIN_REGISTRY:74 (crates 4c2d3c4b == bffaaf8f).
- Portare all'utente clausole 4/6 della licenza.
- Purge `._*`; dichiarare la serie senza leva nel report S-181.
