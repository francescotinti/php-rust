# WP_SESSION_43 — archivio storico della sessione WP-43

> Convenzione: un file per sessione; il handoff tiene solo l'ultima sintesi.
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> ⚡ **WP-43 (2026-07-23, commit `9cc141b`)** — **STADIO 1 Leva B ESEGUITO
> E ACCETTATO: infrastruttura registri a delta zero.** Tutti e tre i criteri
> di accettazione passati: (1) diff bytecode VUOTO a flag spento; (2) gate22
> completo verde per nome + cargo 1637/0; (3) A/B interleaved "infra
> presente ma spenta" vs old = RUMORE ZERO (6 round, segno alternato,
> new 55,70 vs old 56,38 medie). + ⚠️ incidente MySQL datadir
> (recuperato, vedi sotto).

## Stadio 1 — cosa è entrato (piano `REGISTER_BYTECODE_PLAN.md` §5)

- **`Func.max_temps: u32`** (=0 in tutti e 6 i siti di costruzione, tutti in
  `compile/func.rs`): slot-registro = slot ordinario del frame con indice
  statico in `n_slots..n_slots+max_temps`. ⭐ DISTINTO dai temporanei del
  compiler (`n_temps_max`), che sono già FUSI dentro `n_slots` — il campo
  nuovo è additivo e vive PAST quel range.
- **Frame**: `Frame::with_buffers` (vm/mod.rs, UNICO sito di allocazione
  slot nel workspace) dimensiona `n_slots + max_temps`. Nessun campo nuovo
  nel Frame ⇒ nessun rischio layout/drop-order (fisica WP-32 rispettata:
  il layout non è stato toccato).
- **`bytecode::Operand`** `{Stack | Slot(u16) | Temp(u16) | Const(u16)}`:
  operand-sourcing per gli op caldi (piano §4, non un secondo ISA). Non
  consumato da nessun op fino allo stadio 2.
- **`compile/reg_lower.rs`** (nuovo modulo): `enabled()` legge
  `PHPR_REG_LOWER` una volta (OnceLock, pattern `gc_verify_enabled`);
  `lower_func()` è il pass di riscrittura, DELIBERATAMENTE VUOTO allo
  stadio 1, agganciato in coda a `compile_body` (il funnel di TUTTI i corpi
  caldi: funzioni, closure, metodi, hook, main) — il call-site wiring è già
  quello che userà lo stadio 2.
- **`UnitKey.reg_mode: bool`**: la chiave unit-cache porta la modalità
  bytecode (unità compilata in un modo non può servire l'altro); derive
  Hash ⇒ propaga da sola anche nella catena fingerprint.
- **Dump diagnostico `PHPR_DUMP_OPS`** (stderr, compile-time only, freddo):
  main/functions/closures/metodi/prop-init con ops+consts+n_slots+max_temps.
  Scope documentato (thunk reflection e hook NON dumpati — uno stadio che li
  riscrive deve prima allargarlo). È il canale con cui si proverà il diff
  di ogni stadio futuro.
- **Test `stage1_pass_is_identity_and_no_temps`** (cargo 1636→**1637**):
  compila uno snippet rappresentativo, asserisce `max_temps==0` ovunque e
  `lower_func` = identità sui corpi compilati.

## Prova di accettazione (tutti e tre i criteri)

1. **Diff bytecode VUOTO a flag spento**: dump `PHPR_DUMP_OPS` flag-off vs
   flag-on **byte-identico su 162.615 righe** (3 probe: dump-probe nuovo
   ricco — classi/closure/generatori/try-finally/ref/variadic/hook/include
   — + probe-isset + probe-isset-div WP-42, prelude incluso); stdout
   identico; probe nuovo **oracle==phpr byte-id**; old==new byte-id.
2. **Gate22 completo VERDE per nome** (su `9cc141b`): corpus **1447**
   IDENTICO · sess 28 · date 351 · refl 290 · ORM 3E/13F per nome ·
   hk 1665 0E/0F · cargo **1637/0** · gd 11/11 BYTE-ID · mysqli 11/11
   BYTE-ID · media BYTE-ID · http DIFF-set IDENTICO a WP-42 · option 413
   IDENTICO per nome · restapi 3508 IDENTICO per nome.
3. **A/B interleaved 6 round** (media group, user CPU dai .time; old =
   `phpr-5cca65c` tree WP-42, new = stadio 1 flag spento):
   | round | old | new | Δ |
   |---|---|---|---|
   | 1 | 56,09 | 55,46 | −1,12% |
   | 2 | 55,81 | 55,52 | −0,52% |
   | 3 | 55,63 | 55,89 | +0,47% |
   | 4 | 58,71 | 56,15 | −4,36% ⚠️ outlier old (sys 6,84 alto) |
   | 5 | 56,51 | 55,57 | −1,66% |
   | 6 | 55,54 | 55,61 | +0,13% |
   Medie 56,38/55,70 · mediane 55,95/55,59 · oracle di giornata
   20,86/20,68. Segno alternato, magnitudo <1,7% fuori outlier ⇒
   **RUMORE ZERO** (semmai new marginalmente sotto). Il solo layout
   (campo Func + somma nel resize + branch flag in compile) non costa —
   la guardia fisica WP-32 è soddisfatta; summary phpunit old==new
   identiche in tutte le run.
   Verdetto: **STADIO 1 ACCETTATO — si apre lo stadio 2**.

## ⚠️ Incidente MySQL datadir (macchina utente, da sapere)

Durante il primo gate22 le sezioni DB-dipendenti sono passate con **mysqld
GIÙ**: mysqli DIFF spuri, media probe morta, e — trappola — **http battery
e option/restapi FALSI VERDI** (error-page==error-page ⇒ BYTE-ID vacuo;
junit vuoto vs vuoto ⇒ "IDENTICO per nome (0 nomi)"). ⭐⭐ Lezione: un
"IDENTICO" di gate DB-dipendente va sempre validato col CONTEGGIO nomi
(option deve dire 413, restapi 3508) — per questo il regate43 stampa il
conteggio nel messaggio.

Diagnosi del guasto: il datadir VERO del progetto è
**`/Volumes/Extreme Pro/Claude/mysql-wp8/data`** (drive esterno, creato in
WP-8; l'indizio era nel commento di `mysqli-probe/run-probes.sh`) con
socket `/private/tmp/mysql-wp8.sock`, servito da un mysqld avviato il
17/07 e morto oggi senza shutdown. Il primo tentativo di restart
(`mysql.server start`) ha invece aperto il datadir brew di default
(`/opt/homebrew/var/mysql`, VERGINE — inizializzato dal postinstall del
15/07 e mai usato): lì l'utente 'wp' non esiste (ERROR 1410 sul GRANT) e
wp_o/probe/wp mancano. Recovery: mysqld_safe daemonizzato sul datadir
esterno; crash-recovery InnoDB pulito; wp_o integro (97 post).

⭐ Danno collaterale auto-sanante: `run-media.sh` fa
`mysqldump wp_o > wp_o-baseline.sql` come PRIMO passo — coi gate falliti la
redirect ha troncato il baseline a 778B (header-only); il run buono lo ha
rigenerato dal wp_o live. ⭐ Attenzione operativa: un demone avviato dentro
un task Claude muore col TaskStop del task (process group) — avviare i
demoni SEMPRE col daemonizer double-fork+setsid.

## Note operative

- OLD per A/B: `phpr-old-target/release/phpr-5cca65c` (worktree
  `/tmp/phpr-old-43`, CARGO_TARGET_DIR sul drive esterno, build 57s
  incrementale sulla cache di `phpr-old-target`).
- Disco root a 18Gi liberi a inizio sessione (pre-flight ok, soglia 15G);
  puliti 3,2G di `php-rust-output/debug/` rigenerati da cargo test.
- Harness sessione: `wp43-harness/` (dump-probe.php + inc, build-old-43.sh,
  ab43.sh, regate43-mysql.sh).

## Gate e stato a fine sessione

- Gate22 (commit `9cc141b`) TUTTO VERDE — dettagli sopra.
- Full-suite: NON rilanciata (delta zero provato a livello bytecode + A/B;
  run32 resta la baseline, fail-set 88 righe byte-id a run31).
- Commit di sessione: `9cc141b` (stadio 1) + docs di chiusura.

## Prossimo lavoro (stadio 2 — dal piano §5)

**Binary/CmpJmp a operandi diretti**: `Binary{l,r,dst}` con sorgenti
Slot/Const/Temp; il pass riscrive i trigrammi LoadSlot,LoadSlot,Binary;
ASSORBE binary_fast/CmpJmpConst WP-33/34 (sostituzione, mai convivenza).
Bigrammi target dal census: ThisPropGet→CmpJmpConst 29,9M ecc. A/B
go/no-go. Il diff di stadio si prova col dump `PHPR_DUMP_OPS` (flag-on vs
flag-off: devono differire SOLO le sequenze riscritte).
