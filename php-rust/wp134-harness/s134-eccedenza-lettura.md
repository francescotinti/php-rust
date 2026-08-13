# S-134 — eccedenza D>UB su objalloc (az.rev. S-133 #4): lettura dal disasm

Metodo (lezione WP-104): disasm prima/dopo sui PIN STASHATI s132 (6af6e497) e
s133 (c87439a9), `objdump --disassemble-symbols` su `run_loop` e sulle funzioni
outlined; conteggio `bl` per NOME del bersaglio.

## Conteggio chiamate prima/dopo (statico, run_loop)

| grandezza | s132 | s133 | Δ |
|---|---|---|---|
| istruzioni run_loop | 70.295 | 70.468 | +173 |
| bl totali run_loop | 5.868 | 5.876 | +8 |
| siti bl → `magic_applies` | 12 | 2 | −10 |
| siti bl → `magic_applies_resolved` | 0 | 10 | +10 |
| siti bl → `resolve_prop_access` (esplicita) | 14 | 21 | +7 |
| corpo `magic_applies` (istr / bl) | 298 / 22 | 49 / 3 (wrapper: resolve+delega) | outlined nuovo |
| corpo `magic_applies_resolved` (istr / bl) | — | 290 / 21 | = vecchio − `resolve_prop_access` − 8 istr |
| corpo `resolve_prop_access` | 116 istr | 116 istr | INVARIATO |

I 12 siti di chiamata restano 12: dieci re-targettati sulla variante SENZA
resolve interna (perimetro della leva), due sul wrapper nuovo (49 istr =
resolve + delega) — i siti fuori perimetro. Il ±0 sui siti conferma che la
leva non ha mosso l'inliner del chiamante (nessuna recidiva H-C2/WP-104:
+173 istr e +8 bl su 70k = churn di layout, non flip strutturale).

## L'eccedenza ha un NOME

Al sito (evidenza: s132 `0x100268644` vs s133 `0x100268648`+`0x100268680`):
- **s132**: `bl magic_applies` (resolve DENTRO, sul nome) → al ritorno, il
  fallback PropSet ri-risolveva LO STESSO nome — seconda lookup hash
  **dipendente back-to-back**, con setup argomenti ripetuto (x0–x7
  ricostruiti due volte) e ri-borrow.
- **s133**: `bl resolve_prop_access` UNA volta → risultato in coppia di
  registri (`ldp x1, x2`) che alimenta magic-check E blocco key/slot/IC.

Il prezzo 17,7 ns/resolve dell'UB era il prezzo MEDIO dei siti ctor (modello
S-131). La resolve eliminata è la SECONDA di una coppia consecutiva sullo
stesso nome: il suo costo di sito include, oltre al corpo (116 istr), il
setup ripetuto + il borrow + la catena di load dipendenti tra le due lookup.
**Nome dell'eccedenza: costo di sito della seconda lookup dipendente, non
modellato dal prezzo medio per-resolve.** Direzione+meccanismo firmati dal
disasm; la magnitudine dei +11,3 resta NON ripartita (nessun A/B per
componente — vietato prezzare componenti, REGOLE §3/§4).

## Conseguenza per la scelta della leva S-134

La resolve residua (2/iter, "ultima resolve ctor") è ora un `bl
resolve_prop_access` ESPLICITO e nominato nel chiamante, col risultato già
canalizzato in registri: il sito è pulito per una IC (forma non-plain da
modellare PRIMA). Il dispatch 36,3 resta aperto ma tocca la struttura di
run_loop (classe di rischio H-C2/WP-104, threaded-dispatch VIETATO).
