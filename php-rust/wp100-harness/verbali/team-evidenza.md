# Team «evidenza» — Concilio WP-100 (relatore)
Fonti: verbale-3-klabnik.md · verbale-4-hejlsberg.md · verbale-7-leijen.md (vincolanti; questa nota riconcilia).

## (a) Convergenze
1. **Le misure di S-98.0 reggono; i CLAIM di evidenza sono più forti dei test scritti.** Klabnik R1: il «controllo positivo» del funnel confronta i dump (`off_out == on_out` + tre forme nel `{main}`) ma non pinna stdout atteso né exit status — un binario che sbaglia UGUALE nei due modi passa. Hejlsberg RC-1/RC-2 rincara: il dump è cieco per costruzione dove serve di più.
2. **Il gate corpus per NOME è un bit per fail** (Klabnik R2, verificato: set 1418 byte-uguale flagoff/flagon/canone): parità di regressione sì, **gate di PROMOZIONE no** — un fail con diff diverso tra i due modi è invisibile. Serve il riga-per-riga degli output dei fail, salvato come evidenza (A-KL-100-2).
3. **Punti ciechi del dump/battery** (Hejlsberg RC-1/RC-2, Klabnik R1 coda): flag-on i corpi **hook** VENGONO riscritti dal pass ma sono esclusi da `dump_module_ops` (clausola violata oggi); `lowered()` abbassa **prop_init** che in produzione (`compile_prop_init`, func.rs:361, senza gate `enabled()`) non è MAI lowered. Il battery testa una pipeline che la produzione non esegue e non testa quella che esegue. Guardia M5 in un solo test; `cargo test` con env esportato applica il pass due volte in silenzio.
4. **Due rialzi di budget cifre nella stessa sessione** (24561→24643→24645, Klabnik R4): il primo è forgia legittima, il secondo è gate ri-adattato all'artefatto dopo il morso. Breakdown per fonte nel file (A-KL-100-4); terzo rialzo senza enumerazione token = gate non-mordente (KS-KL-100-3).
5. **Strumentazione S-99 sottospecificata**: coppia peak senza strumento nominato = forgia che fallisce in silenzio (Leijen R1: `/usr/bin/time -l`, spot-check vmmap Physical, MAI RSS ps, env `MIMALLOC_PURGE_DELAY=0` come WP-94, accanto a 1901,11 MiB); dente **N_OPS≤255** in BACKLOG cioè DOPO le occorrenze che lo consumano, con N_OPS=186 e rollout moltiplicativo (Leijen R2).

## (b) Conflitti per sedia
- **Klabnik vs Hejlsberg sul controfattuale**: Klabnik emenda la matrice smoke Add (R3, 5 celle su tag-space ≥12) tenendo D=6,07 come base; Hejlsberg RC-3 dichiara D=6,07 il controfattuale SBAGLIATO per BinarySS/SC/Dst (pop/push già eliso) → il criterio va ri-registrato per-forma. Componibili: matrice per la correttezza, controfattuale per-forma per il criterio.
- **Hejlsberg vs Klabnik su bin_op_of**: per Hejlsberg neutralizza un tripwire (A-HE-100-1: assert ZERO BinaryAdd flag-on); per Klabnik il rischio vive nella matrice fixture. Non contraddittori.
- **Leijen r3-r5** sono minori dichiarate (retro-annotazioni, offset_of, pin env process-level): nessuna sedia le contesta, ma nessuna le eleva a bloccante.

## (c) Lista MINIMA che blocca i PASS futuri (regola di ammissione: blocca l'oggetto)
1. **Promozione flag-on→default** bloccata da: diff riga-per-riga dei 1418 fail off/on (KS-KL-100-1) + dente anti-putenv/eager-init su `enabled()` (KS-HE-100-1) + sanatoria dump/`lowered()` al funnel vero — hook dentro, prop_init gated o dichiarato fuori (A-HE-100-4) + funnel pinna stdout/exit (A-KL-100-1).
2. **Rollout famiglia (Sub/Mul, punto 3)** bloccato da: gate compile-time N_OPS≤255 + canary noop pre-registrato (KS-LE-100-2/A-LE-100-3) + matrice fixture Add per NOME (KS-KL-100-2) + criterio ri-derivato per-forma (KS-HE-100-3) + tripwire ZERO BinaryAdd flag-on (A-HE-100-1).
3. **Coppia peak S-99**: strumento nominato o confronto VOID (KS-LE-100-1/A-LE-100-1).

## (d) Priorità
P1 sanatoria dump/lowered+funnel (blocca promozione E rollout) · P2 diff riga-per-riga + dente enabled() · P3 gate N_OPS + strumento peak · P4 matrice Add + criterio per-forma.

**BACKLOG per NOME** (non blocca l'oggetto): A-KL-100-4 breakdown budget; A-KL-100-5/debito B2 (13 snippet al funnel vero); A-HE-100-2 `visit_addrs` esaustivo (blocca solo shape Addr-bearing future, KS-HE-100-2); A-HE-100-3 test differenziale BinaryAdd≡Binary(Add); A-LE-100-2 profilo alloc nei .out; A-LE-100-4 offset_of sonda M1; pin env process-level (r5).
