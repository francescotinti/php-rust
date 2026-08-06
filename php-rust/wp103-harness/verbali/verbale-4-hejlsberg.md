# Verbale sedia 4 — Hejlsberg (compilatori incrementali, emissione, dedup) — Concilio WP-103

Fonti lette: NEXT_SESSION §S-102; WP_SESSION_101 punto 5; `compile/mod.rs`
`emit_binary` (783-789, ctx da riga 278: UN `ProgramCtx.reg_lower` per
l'intero Module); `compile/reg_lower.rs` tests (helper 594-776, denti
1001-1056).

## VERDETTO: APPROVATO CON RISERVE — una refutazione capitale sul dente A-KL-102-3; i punti 2-3 di §S-102 passano SOLO con dump-diff e budget Op pre-registrati.

### Tema 1 — dente A-HE-102-1 (`in_process_off_arm_emits_production_stack_add`)
La polarità è ora quella VERA del corpo (`!ctx.reg_lower ⇒ BinaryAdd`):
OFF ≥1 `BinaryAdd` e zero `Binary(Add)`, ON tripwire zero-`Binary(Add)`.
Il conteggio usa `all_funcs` (destructuring esaustivo A-HE-100-4) — il
predicato È sul modulo intero. **MA la fixture non ha classi**: i corpi
FUORI funnel (prop_init, const-thunk) condividono `ctx.reg_lower` (un solo
campo, riga 278) e il pass non li visita MAI (RC-2) — sotto ON un add in
prop_init resterebbe `Binary(Add)` e il tripwire "modulo intero" sarebbe
ROSSO. Oggi l'invariante fuori-funnel è NON ESERCITATO, forse falso: verde
per fortuna della fixture. I BinOp non-Add non hanno polarità di modo in
`emit_binary` — la copertura Add-only è corretta per QUESTO sito, ma il
bug-classe (lettura ambientale di `enabled()`) non è enumerabile per op.

- **A-HE-103-1**: estendere la fixture del dente a BODY_ZOO (classe con
  prop-init `self::K + 1`): o l'invariante ON regge ovunque (pinnarlo), o
  si scrive il carve-out fuori-funnel per NOME. Attesa scritta PRIMA.
- **A-HE-103-2**: dente di BUDGET sui call-site di `enabled()`: in
  produzione UN solo sito (l'entry, riga 218 + il print). Guardia statica
  (grep in batteria o dente `#[cfg(test)]` sul conteggio) — cura la CLASSE,
  non l'istanza (lezione A-SK WP-96).

### Tema 2 — dente A-KL-102-3 (`absent_env_is_identical_to_explicit_one`)
**REFUTAZIONE CAPITALE R-HE-103-1 (copertura fabbricata)**: la metà
"stessa emissione" confronta `compile_mode(src, a)` con
`compile_mode(src, b)` DOPO aver asserito `a == b` — è
`f(x) == f(x)`: pinna il determinismo del compilatore, non «assente ≡ =1»
del funnel. In-process, stesso env in entrambi i bracci: un sito
ambientale residuo colorerebbe i due bracci allo stesso modo — falso
verde per costruzione. Il contenuto reale del dente è la sola riga di
grammatica. E il confronto su `main.ops` soltanto è il vizio minore:
funzioni/hook invisibili.
- **A-HE-103-3**: la coppia assente↔`=1` si giudica in SOTTOPROCESSO
  (apparato `env -i` A-SK-93..97): due invocazioni CLI con
  `PHPR_DUMP_OPS`, dump-diff BYTE su BODY_ZOO, modulo intero. Nota: il
  corpus "nei 2 modi" esercita default(assente) e `=0` — la coppia
  assente↔`=1` end-to-end oggi non la esercita NESSUNO.
- **A-HE-103-4**: la metà in-process del dente si dichiara per ciò che è
  (determinismo) o si elimina — un dente che finge copertura è peggio
  dell'assenza.

### Tema 3 — §S-102 punto 3 (PropGet/PropSet a slot-operando) = EMISSIONE
Macchinario S-97.1 già in casa: finestre del pass su `Binary(Add)`, forme
`*SS/*SC/*Dst`, `dump_module_to`/`PHPR_DUMP_OPS`, denti dump-scope. PRIMA
del cronometro:
- **A-HE-103-5**: dump-diff 2-modi con attese scritte PRIMA su fixture per
  FORMA di ricevitore: `$o->p`, `$this->p`, catena `$o->a->b`, ricevitore
  da call, ref nello slot (deve conservare `deref_clone`, braccio Ref di
  H-C1b), `__get`/hook, lazy ghost, ricevitore undef (famiglia §3.13:
  il PUNTO del warning non deve slittare di riga). Fuori dalle forme
  nominate: diff ZERO.
- **A-HE-103-6**: controfattuale CONTATO dal census (`recv via slot` vs
  `via pila`), atteso = 3 coppie × ~2 ns MISURATO, dichiarato banda LARGA
  (lezione H-C1b: il profilo a campioni sovraconta ~2×).
- **KS-HE-103-1**: `stage2v3_op_size_unchanged` pinna `size_of::<Op>()==48`
  — le varianti nuove si PROGETTANO dentro il budget PRIMA di scrivere il
  pass; se non ci stanno, il widening si porta al Concilio col suo costo
  D-cache misurato, non si allarga il pin in silenzio.
- **KS-HE-103-2**: emissione toccata ⇒ coppia WP bimodale NON derogabile
  (regola 2) + corpus 1418×2 per NOME entrambi gli stadi.

### Tema 4 — `cargo check --features zval-census` in batteria
Beneficio reale (liveness.rs è sfuggito in S-101: costo di UNA variante
enum non classificata). Costo warm: secondi (fingerprint separato, primo
run freddo una tantum). **MA `cargo check` nudo è profilo DEV: rigenera
la `debug/` che il pre-flight impone di rimuovere** (disco ~22G).
- **A-HE-103-7**: `cargo check --release --features zval-census` — resta
  nell'albero release; misurare il costo warm UNA volta e registrarlo.
- **KS-HE-103-3**: NO `--all-features` alla cieca: un rosso per feature
  estranee trasforma la guardia in rumore; la lista feature del check è
  chiusa e nominata.

### Punto 2 §S-102 (pila operandi): nessuna obiezione d'impianto — il
census push/pop per categoria e il dump-diff-prima-del-cronometro sono la
sequenza giusta; vale KS-HE-103-1/2 anche qui se nascono forme nuove.

**Refutazioni capitali: SÌ — R-HE-103-1** (dente absent≡=1: la metà
emissione è tautologica in-process, copertura fabbricata).
