# Verbale sedia 8 — STOGOV (Zend/opcache, semantica engine) — Concilio WP-108

## VERDETTO

**H-A1 è semanticamente SANA e AVVICINA phpr al modello Zend.** Verificato
sull'oracolo: `ZEND_ASSIGN_OP` (zend_vm_def.h:1257, handler 26) è UN'op RMW
unica — op2 (value) fetchato PRIMA, op1 (var_ptr) DOPO, deref del Ref, poi
`zend_binary_op` in place. La fusione `LoadSlot;Swap;BinaryDst → BinarySTDst`
riproduce esattamente quest'ordine (rhs già in pila = valutato prima; lhs
letto al momento dell'op, pre-store). Il tris era un'anomalia phpr; la forma
fusa è la forma Zend. Nessuna refutazione capitale.

Checklist semantica ESEGUITA (oracle 8.5.7 vs phpr pin, empirico +
lettura bracci):
- `$s += $s` → 10 su entrambi (lhs letto post-rhs, pre-store; = ordine Zend).
- Slot con Ref: `read_slot` segue il Ref (run.rs come arrays.rs:839),
  `reg_store_slot` replica StoreSlot INTERO (guardia typed-ref +
  write-through + gc_note, run.rs:295-311); empirico `$q += 3` → `$r`=5 ✓.
  Stesso `reg_store_slot` del braccio BinaryDst: zero biforcazione, come
  dichiarato.
- `$a += $arr` (array+array) e overflow `PHP_INT_MAX+1 → float`: funnel
  condiviso `binary_value_ab`, byte-parità empirica ✓.
- `.=` FUORI perimetro: `Concat` compila in `ConcatAssignSlot` (WP-55,
  expr.rs:96-99) — la finestra ST non lo vede mai.
- Forma value non fusa (`BinKind::ST, None` → nessun fold): corretto.
- Undefined lhs: oracle emette `Warning: Undefined variable $u`, phpr NO —
  ma è **§3.11 a catalogo dal S-100** ("identica flag-off e flag-on, NON è
  del pass", perimetro misurato S-103). La finestra è parity-preserving del
  comportamento PHPR; la divergenza è PRE-esistente e già per NOME. NON
  addebitata alla leva.

## R-ST-108-1 (refutazione documentale, non capitale)

Il doc modulo «Fold rules» (reg_lower.rs:17-21) dichiara ANCORA
«`LoadSlot` (silent, cold) is never folded» — la regola che H-A1 ha
emendato alla finestra (r.475-482) è rimasta INTATTA in testa al modulo.
Due fonti di verità nello stesso file: un autore futuro che si fida
dell'header conclude che BinarySTDst non può esistere. Da emendare.

## R-ST-108-2

Il commento della finestra dice «silenziosa per contratto ⇒ nessun warning
da risintetizzare». Il "contratto" È la divergenza §3.11 aperta 🔴: vero
oggi, ma non è semantica PHP — è il difetto catalogato. Il doc deve CITARE
§3.11, non un contratto.

## Emendamenti

- **A-ST-108-1**: rimando incrociato §3.11 ↔ finestra ST: la cura §3.11
  dovrà toccare ANCHE BinarySTDst, col TEMPLATE già in casa — le forme SS
  risintetizzano il warning esatto DENTRO l'op fusa (header r.18-20, nome
  byte-identico). Curare de-fondendo = regressione arith silenziosa
  (11,6→12,4): vietato senza micro come gate della cura.
- **A-ST-108-2**: la candidata S-107 «IncDecSlot+Pop» fonde un ALTRO membro
  della famiglia §3.11 (`$u++` senza warning, perimetro S-103): il criterio
  della nuova istruttoria DICHIARI l'accoppiamento §3.11.
- **A-ST-108-3** (opportunità nominata, solo con criterio suo): BinarySTDst
  non ha la guardia `binary_fast` senza-clone che SS/SC hanno; per il
  residuo 11,6 è un candidato, non un ordine.

## KS-ST-108-1

La disciplina dei DUE read è preservata dal pass: LoadVar (warning,
risintetizzato nelle SS) e LoadSlot (silente, ST usa `read_slot` identico
al braccio LoadSlot). Chi aggiunge finestre deve nominare QUALE disciplina
eredita.

## Giudizio ordine S-107

**§3.15 in testa: CONFERMATO** — aliasing variadic by-ref = corruzione
silenziosa di dati utente, classe peggiore delle diagnostiche. La cura
D-13 come scritta REGGE: specchio del binder dinamico, arbitro
by_ref_error.phpt (letterale⇒Error, place⇒MakeRef = SEND_REF Zend), fx21
stessa commit, attesa 1417→1415 citata (conforme D-21). Sequenza > get_gc >
§3.13 > §3.12-i > §3.14: confermata. Aggiunta Stogov: §3.11 non è in coda
S-107 mentre la leva candidata ne allarga la famiglia — vale A-ST-108-2.
