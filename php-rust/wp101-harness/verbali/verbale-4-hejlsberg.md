# Verbale SEDIA 4 — Hejlsberg (pipeline di emissione, unit-cache/modes) — Concilio WP-101

## VERDETTO

**L'obiettivo della promozione è AMMISSIBILE; la bozza §S-100 come scritta è VOID nei punti 1–2**: l'ordine dei gate è invertito, il contratto di selezione del modo dopo il flip non è nemmeno nominato, e uno dei "gate già soddisfatti" poggia su un'evidenza sovradichiarata che l'unico strumento capace di verificarla (il dump) oggi non può verificare.

## Refutazioni capitali

**R1 — Il flip non ha un contratto di modo: il collaudo "nei DUE modi" rischia il falso verde stesso-modo.** Oggi `enabled()` è `env::var_os("PHPR_REG_LOWER").is_some()` (reg_lower.rs:50-53): QUALUNQUE valore, incluso `PHPR_REG_LOWER=0` o stringa vuota, significa ON. La bozza ordina "flip del default + corpus per NOME nei DUE modi" senza dire COME si ottiene l'OFF a default invertito. Se l'opt-out viene speso come `=0` sull'attuale `is_some()`, i due bracci del funnel e di `s99-corpus-gate.sh` girano ENTRAMBI on: gate identici per costruzione = forgia che fallisce in silenzio [[feedback-forge-silent-failure]]. Anche i due bracci del dente anti-putenv (set→on rifiutato, unset→off rifiutato) sono scritti per default=off e vanno riscritti, come i tre launcher.

**R2 — "Emissione flag-on davvero bit-identica" è dedotta da un delta timing (5,43→5,44), non da un diff di emissione.** Invarianza di cronometro ≠ identità di bit. Lo strumento che può dichiararla — dump/`lowered()` — è CIECO sugli hook riscritti (è esattamente A-HE-100-4), che la bozza mette ULTIMO nel punto 1. Ordine sbagliato: la sanatoria del dump è il PRE-REQUISITO del dump-assert di A-PE-100-4 e del tripwire A-HE-100-1; asserire "zero forme registro" con un dump cieco è un controllo positivo fallito.

**R3 — `visit_addrs` ha `_ => {}` su `Op` (reg_lower.rs:74): la promozione trasforma un footgun latente in un vettore di corruzione di default.** Il commento stesso ammette: variante nuova con `Addr` non aggiunta lì ⇒ il pass corrompe gli indirizzi. È la stessa classe che S-96 ha eliminato altrove ("una variante nuova di `Op` ora NON COMPILA"). Con flag-on opt-in, il danno colpisce chi opta; con default ON colpisce TUTTI, silenziosamente. A-HE-100-2 non è backlog: è BLOCCANTE per il flip.

**R4 — RC-1 si INVERTE col flip e la bozza non lo vede: il percorso OFF diventa il pin incollaudato.** Dopo il flip, OFF è il kill-switch di rollback — e nessun ordine della bozza lo tiene collaudato nel tempo. Un rollback mai eseguito è il pin 365f4d40: refutabile per costruzione al primo uso.

## Classificazione gate (mandato)

**BLOCCANTI per il flip**: A-HE-100-4 (primo: è lo strumento), A-HE-100-2 (match esaustivo), A-HE-100-1 (tripwire, col dump sanato), A-HE-100-3 (differenziale BinaryAdd≡Binary(Add) su overflow/coercizioni/union/warning — costa un file, il corpus non è prova: "corretto per fortuna del corpus" ≠ corretto). **Backlog**: nulla dei quattro; restano backlog i gate non-HE non nominati dal mio perimetro.

## Emendamenti

- **A-HE-101-1**: PRIMA di ogni riga del flip, definire il contratto di modo (opt-out value-parsed, non `is_some()`), riscrivere i due bracci del dente e i tre launcher per il default nuovo.
- **A-HE-101-2**: riordinare il punto 1: A-HE-100-4 in testa; A-HE-100-2 promosso bloccante; poi A-PE-100-4/A-KL-100-1/2 sugli strumenti sanati.
- **A-HE-101-3**: un controllo positivo sulla claim "la chiave unit-cache porta il modo" (doc di reg_lower.rs, mai collaudata): con cache TL-al-MAIN e modo sigillato è cintura ridondante — va o provata o riclassificata, non contata come gate soddisfatto per documentazione.
- **A-HE-101-4**: il funnel-probe esca da `{main}`: almeno un corpo non-main (funzione/chiusura) asserito nei due bracci — è il punto cieco stesso di A-HE-100-4.

## Kill-switch

- **KS-HE-101-1**: flip VOID se i due bracci del funnel non provano emissione DIVERSA su una probe (hash dei due dump pubblicati) — anti falso-verde stesso-modo.
- **KS-HE-101-2**: nessun flip finché `visit_addrs` conserva un braccio wildcard su `Op`.
- **KS-HE-101-3**: post-flip, ogni rotazione pin include un braccio flag-OFF collaudato (campo `modo:` in PIN_REGISTRY); OFF non eseguito ⇒ pin non collaudato.
- **KS-HE-101-4**: "bit-identico" si dichiara SOLO da diff di dump/emissione, mai da un cronometro.
