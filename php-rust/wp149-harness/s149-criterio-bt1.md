# Criterio S-149 p.2b — leva BT1 «debug_backtrace onora options/limit» — commit PRIMA dell'A/B

1. Perimetro: `ho_debug_backtrace` fa il parse di `options` (default
   `DEBUG_BACKTRACE_PROVIDE_OBJECT`=1) e `limit` (default 0=tutti);
   `collect_backtrace_opt(limit, ignore_args)` si ferma a `limit` frame e con
   IGNORE_ARGS NON raccoglie gli args (il clone per-arg non avviene);
   `object` solo con PROVIDE_OBJECT. `ho_debug_print_backtrace` INVARIATO
   (`collect_backtrace()` = delega a `_opt(0,false)`); altri chiamanti:
   NESSUNO (verificato a sorgente, 2 soli siti).
2. Fedeltà (REGOLE §9): la cura chiude una DIVERGENZA — oggi args presenti
   anche con IGNORE_ARGS, frames oltre limit, object sempre. Fixture
   bilaterale `fx-backtrace.php` (default · options=0 · IGNORE_ARGS ·
   limit=1 · limit=2 · IGNORE_ARGS,limit=2 · dentro metodo con $this)
   byte-id vs oracle 8.5.7 PRIMA di ogni promozione; corpus 1414×2 al gate:
   flip eventuali DICHIARATI per nome.
3. Giudice NUOVO `m-backtrace.php` (forma Doctrine IGNORE_ARGS,2; pila 48;
   150k iter; args nei frame), BILATERALE. A/B R=5 ABAB monomacchina su
   A=pin s145 (stash) vs B=build leva (target separato): cifra = user CPU
   netto pavimento per-binario / iter; soglia = max(4 ns/iter, rumore
   drop-1, spread-batch); attesa di segno: B piu' veloce, grande (dal
   census: ~2 frame costruiti invece di ~50, niente clone args).
4. Costo SOSTITUTIVO NOMINATO: parse di 2 argomenti + walk fino a limit con
   ~2 frame costruiti per chiamata sulla forma Doctrine; sulle chiamate
   full-default il cammino resta l'attuale per costruzione (nessuna nuova
   tassa nominata).
5. Guardie SOLO-REGRESSIONE (R=5, formula del giudice): sei micro wp97 +
   m-dimread + m-dimrmw; batteria `cargo test --release` al gate di
   promozione; disasm bl-count di run_loop atteso INVARIATO (leva fuori dal
   dispatcher), registrato.
6. Scommessa suite (scala S-146): attesa_alta = conteggio bersaglio (tetto
   census tranche-4) × prezzo_max dalla sonda (s149-sonda-pair-verdetto.out);
   ≥2× (≥~0,6 s) ⇒ scommessa ORM AMMESSA e pre-registrata (al prossimo pin
   la coppia ORM deve dare direzione ↓; denominatore KS-146-1 0,293 s);
   1×–2× ⇒ leva resta micro-judged (giustificata comunque dalla fedeltà
   p.2); l'aritmetica si scrive nel verdetto A/B PRIMA di guardare l'esito.
7. Esiti pre-registrati: A/B sotto soglia o guardia rossa ⇒ leva NON
   promossa, revert al byte; promozione = catena piena collaudo-nell'atto
   (scripts/pin-phpr.sh) SOLO dopo la coppia t3 @ s145 (il debito t3 vincola
   il pin corrente; pin nuovo ⇒ coppia WP dovuta, REGOLE/feedback 2026-08-12).
