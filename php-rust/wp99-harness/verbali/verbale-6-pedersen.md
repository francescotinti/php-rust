# Verbale Sedia 6 — Pedersen (Concilio WP-99)

Perimetro: confine per-richiesta/test, lifecycle, igiene di stato fra run.
Oggetto: report S-97.1 + programma H-B1. Mandato: refutare.

## VERDETTO: CON EMENDAMENTI

## Refutazioni capitali

**R1 — «Il flag è fisso per processo» è un COMMENTO, non un invariante.**
`reg_lower::enabled()` (reg_lower.rs:50-53) è un OnceLock letto alla PRIMA
chiamata; `putenv()` PHP (php-builtins/src/file.rs:1267-1284) chiama
`std::env::set_var` process-wide, e il suo stesso doc dice «safe under
per-process --isolate» — il server è esattamente il caso NON isolato. Su un
worker long-lived in cui la prima compile non è ancora avvenuta, una
RICHIESTA che fa `putenv("PHPR_REG_LOWER=1")` decide il modo dell'intero
processo per sempre: stato di richiesta promosso a configurazione di motore.
Nessun dente lo copre; l'ordinamento «la prelude compila prima» è un claim
mai pinnato.

**R2 — «Parità server da riverificare al primo uso» SOTTOSTIMA il debito.**
Non è igiene facoltativa: l'emissione flag-off È cambiata (l'elisione Sweep
di H-A2 è INCONDIZIONATA, non dietro il flag), quindi per la regola n.2 il
collaudo di parità sull'emissione nuova è dovuto, e lato server non è MAI
girato. Il binario 832568a72b925dd1 non è «lo stesso motore ricompilato».

**R3 — Handoff incoerente sul pin server.** NEXT_SESSION §Stato gate porta
ancora `php-server: f8f4295a1dcdb627`; WP_SESSION_97 dichiara la rotazione a
`832568a72b925dd1`. Due fonti di verità divergenti sulla riga che il
pre-flight della prossima sessione leggerà — violazione diretta della regola
single-source.

**R4 — I contatori NON distinguono le popolazioni.** `UnitKey.reg_mode`
separa correttamente le cache, ma UcStats/uc_log non portano il modo: un
mode-miss è indistinguibile da un `miss_cold` genuino, e un log condiviso da
due processi con env diversi mescola due popolazioni senza marcatore.

## Verificato (non refutato)

Il canale Ref del bridge (WP-33): il fast-path scarta `Ref` verso il funnel;
`reg_load_slot` (run.rs:228-240) ha lo STESSO predicato di `LoadVar`
(run.rs:636-653) — warning solo su `Undef` ESTERNO, un Ref che regge Undef
non avvisa in nessuno dei due, `read_slot` identico; testo byte-identico per
costruzione (fold solo se name==slot_names[slot] a pass-time, seed ceduto
DOPO il pass, contratto fp-guarded). Il re-lower deferred su unità già
cedute degrada chiuso (slot_names vuoto ⇒ nessun fold). MA:
`.unwrap_or(b"")` in reg_load_slot emette «Undefined variable $» (nome
vuoto) se il contratto è mai violato — silenziosamente sbagliato, contro
correct-or-absent e contro la disciplina fail-loud di `seed_prefix_breach`.

## Emendamenti

- **A-PE-99-1**: leggere il flag EAGER a un confine nominato (bootstrap
  processo/Vm), non lazy alla prima compile; test cargo che
  `putenv("PHPR_REG_LOWER=1")` da codice PHP non flippa il modo (dump
  bytecode invariato).
- **A-PE-99-2**: campo `reg=` nel vocabolario uc_log (o header one-shot per
  file); un A/B on/off è valido SOLO come due processi con env fissato allo
  spawn e registrato nel log.
- **A-PE-99-3**: `.unwrap_or(b"")` in reg_load_slot → fail-loud (stessa
  classe di seed_prefix_breach), mai un warning col nome sbagliato.
- **A-PE-99-4**: correggere SUBITO il pin server in NEXT_SESSION (R3).
- **A-PE-99-5 (H-B1, da scrivere nel criterio PRIMA del codice)**: «frame in
  registro, ricaricato ai confini call/ret/throw» è una lista INCOMPLETA:
  vanno enumerati TUTTI i siti dove `self.frames` può riallocare o mutare
  mid-opcode — rientranza builtin→VM (`call_method_sync`), `__toString` nel
  funnel di `binary_value_ab`, distruttori/GC che eseguono codice PHP. Un
  confine dimenticato è UB silenzioso, non un rallentamento.

## Kill-switch

- **KS-PE-99-1**: qualunque uso o misura del server su 832568a72b925dd1
  senza PRIMA restapi 3508 per NOME + option 413 per NOME sotto il launcher
  `env -i` a lista chiusa (PHPR_REG_LOWER ASSENTE dalla lista) e sentinella
  del binding output-capture verde = **VOID**.
- **KS-PE-99-2**: qualunque campagna che confronti flag-on/flag-off dentro
  UN processo = VOID by construction; e finché A-PE-99-1 non è chiuso, ogni
  run server deve conservare il log raw dell'ambiente allo spawn.
