# Verbale sedia 4 — Hejlsberg (pipeline di compilazione, unit-cache/modes, dedup) — Concilio WP-102

## VERDETTO

S-100 ha eseguito il flip con disciplina, ma il claim di testa — «il modo è un
INPUT del funnel, batteria SENZA premesse ambientali» — era FALSO al momento
della chiusura: REFUTAZIONE CAPITALE, indipendentemente confermata (vedi R1).
La bozza §S-101 è ammissibile dal mio seggio SOLO con i denti sotto.

## Refutazioni

**R1 (CAPITALE) — un sito ambientale residuo dentro il funnel.** Nel commit
fb861e4, `emit_binary` (mod.rs:780) leggeva `reg_lower::enabled()` — il
globale OnceLock — a UNA chiamata di distanza dal campo `ctx.reg_lower`
creato apposta per eliminarlo (`FnCompiler` porta già `ctx`, riga 598).
Conseguenza: il braccio OFF in-process (`compile_mode(src,false)`) sotto
batteria default-ON emetteva `Binary(Add)` generico, una emissione che la
produzione flag-off NON produce mai (produce `BinaryAdd`, H-B2). Il claim
«lowered() eliminato, il braccio dei test è il funnel VERO» valeva per il
braccio ON soltanto. Nota di concordanza: in-flight ho trovato nel working
tree l'edit NON COMMESSO A-HO-102-1 (sedia 1) che sposta la lettura su
`self.ctx.reg_lower` — la refutazione resta a verbale su S-100 come spedita.

**R2 — il fix senza dente è una regressione futura garantita.** Nessun dente
in-process asserisce che il braccio OFF contenga `Op::BinaryAdd`: la
violazione R1 era invisibile alla batteria (e `BinaryAdd` NON è in
`is_reg_form`). Peggio: il commento di `stage2v3_flag_off_emits_no_register_forms`
(reg_lower.rs:936) dichiara «né BinaryAdd» — non asserito, e col fix di
sedia 1 diventa FALSO (l'emissione OFF lo contiene per H-B2). Commento ≠ dente.

**R3 — il tripwire «zero Binary(Add) flag-on» è falso a perimetro-modulo.**
La fixture del progetto stesso lo refuta: `public $d = self::K + 1`
(BODY_ZOO) produce un prop-init FUORI dal pass che flag-on SHIPPA un
`Binary(Add)` generico (emesso da `emit_binary`, mai riscritto). Il funnel
test lo scampa solo perché il grep è scoped a `{main}`/`fn probe`; il
commento a reg_lower.rs:289 («l'emissione non contiene MAI») non dichiara il
perimetro. Corollario: il handler generico `Binary` resta CALDO flag-on.

**R4 — l'esaustività di `all_funcs` è 2/3.** Il commento (reg_lower.rs:622)
promette «un campo nuovo di Module O DI CompiledClass NON COMPILA» — ma
`CompiledClass` NON è destrutturata (dot-access su 4 campi): `enum_cases`,
`attributes`, `abstract_sigs` e ogni campo futuro con corpi sfuggono in
silenzio all'assert flag-off «nessuna forma registro OVUNQUE».

**R5 — A-HE-101-3 non è solo aperto: oggi è INESERCITABILE.**
`UnitKey.reg_mode` è ri-letto dal globale (vm/mod.rs:16058) che è
OnceLock-costante intra-processo: il discriminante di chiave non può MAI
differire in vivo, quindi il «controllo positivo» non ha un percorso
pubblico che lo eserciti. Il modo ora vive in TRE posti (OnceLock, ctx,
UnitKey) tenuti d'accordo per convenzione, non per costruzione.

**R6 — dump e ereditarietà: duplicazione non pinnata.** `prop_info` è
flattened parent-first: l'hook del padre si ridumpa sotto OGNI figlio.
Deterministico (sort per nome) ma nessuna fixture con ereditarietà lo
dichiara; un census-da-dump double-conta i corpi ereditati.

## Emendamenti

- **A-HE-102-1**: dente in-process che pinna il braccio OFF con `BinaryAdd`
  PRESENTE e il braccio ON senza `Binary(Add)` (via `compile_mode`); sanare
  il commento di stage2v3_flag_off.
- **A-HE-102-2**: alla rotazione la batteria cargo gira nei DUE modi
  espliciti (`=0` e `=1`) — la purezza del funnel si prova eseguendo.
- **A-HE-102-3**: destrutturare `CompiledClass` in `all_funcs` (match
  esaustivo vero).
- **A-HE-102-4**: perimetro del tripwire dichiarato per NOME + controesempio
  prop-init (`self::K+1`) pinnato come eccezione ATTESA flag-on.
- **A-HE-102-5**: stampare il modo NEL `Module`; `UnitKey.reg_mode` derivato
  da lì; dente a chiave costruita a mano (chiude A-HE-101-3).
- **A-HE-102-6**: fixture con ereditarietà per il dump; politica di dedup
  dichiarata (o duplicazione pinnata).

## Keystones

- **KS-HE-102-1**: nessuna iscrizione/promozione H-C1 prima della batteria
  nei due modi espliciti nella stessa rotazione.
- **KS-HE-102-2**: A-HO-102-1 non si committa senza A-HE-102-1 — il fix
  entra solo col dente che l'avrebbe morso.
