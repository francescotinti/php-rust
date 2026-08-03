# Verbale sedia 8 — Stogov (Zend/opcache, contratto LSP A-DS35) — Concilio WP-92

VERDETTO: **CON EMENDAMENTI** (due refutazioni capitali sul formato pin e sul bersaglio t4).

## Q1 — Copertura RESIDUA del set v2: SETTE buchi, tutti oracle-morsi dal vivo (8.5.7, probe in scratchpad ds35-stogov/)

1. **Interfacce multiple in conflitto** (`implements I1, I2` con firme incompatibili) → fatal `Declaration of C::m(): int must be compatible with I2::m(): string`. Nessuna fixture v1/v2 lo copre (v13 copre solo il ctor di interfaccia).
2. **Interface extends interface** con firma incompatibile → fatal SENZA alcuna classe coinvolta: il check vive anche nel linking di interfacce; la sede duale deve coprirlo o t.b.d. a catalogo.
3. **Enum implements** con firma incompatibile → fatal (`E::m(string $x): int must be compatible with I::m(int $x): int`). Gli enum sono assenti dall'intero set.
4. **self vs static**: `P::m(): static` / `C::m(): self` → fatal col **self RISOLTO nel messaggio**: `Declaration of C::m(): C must be compatible with P::m(): static`. Vincolo di modellistica su `TypeHint::display_name`: `self` si stampa come nome-classe risolto, non «self». n3 non lo copre.
5. **Property hooks**: parent `public int $x { get => … }`, child ridichiarata → messaggio di FAMIGLIA DIVERSA da n8: `Type of C::$x must be subtype of int (as in class P)` («subtype of», non «must be int»). phpr modella gli hook: serve la fixture.
6. **Costanti final**: `C::X cannot override final constant P::X` → fatal, non coperto.
7. **readonly PROMOTED** (ctor promotion) → stesso messaggio di v11 ma path di lowering diverso; fixture a costo zero.

DNF `(A&B)|string → string` è LEGALE (verificato): manca il positivo di regressione. Trait+abstract è coperto (n9).

## Q2 — Formato pin: refutazione CAPITALE, il marcatore è forgiabile

`echo "x\n--stderr\ny--end"` è un programma PHP legale: il suo stdout contiene le righe-marcatore. Un comparatore futuro che parse-a `ds35-verify2.out` a scansione di marcatori è ambiguo per costruzione (oggi non morde solo perché le 37 fixture non emettono quei byte). Inoltre `42--stderr` su una riga (stdout senza newline finale) è decodificabile solo conoscendo i marcatori — stessa fragilità. **Pin v3: `--stdout bytes=N` / `--stderr bytes=N` per canale; parse = lettura di N byte esatti.** Vietato il comparatore a marcatori.

## Q3 — Bersaglio t4: «stdout integrale dell'oracle» non nomina il BRACCIO

Su t4 i due bracci divergono: plain = `pre|` PRIMA del fatal, persist = fatal PRE-output. phpr con lowering hoisted fatalerà pre-output ⇒ il bersaglio byte-fedele di t4 è il braccio **PERSIST**, da dichiarare per NOME nel pin. Contraddizione residua da sciogliere: appendice WP-89 dice «skip conservativo su nomi non risolvibili», §3.3-quinquies dice «mai skip silenzioso» (v15). Prevale la delibera WP-91: v15 fatala fedele al bind hoisted; per i bind dinamici decide il gate ORM/hk.

## Q4 — Ordine A-DS51 (rischio ORM/hk minimo)

(1) checker LSP puro + unit sui messaggi (nessun wiring); (2) wiring nel lowering HOISTED (lower/class.rs, dopo flatten trait, accanto al final-check) **con le esenzioni nello STESSO commit** (ctor-plain-class, private, RTWC, tentative-Deprecated A-DS46 — nota: il ctor NON è esente per interfacce/abstract, v13/v14) → gate ORM 3E/13F + hk 1665 + corpus per NOME; (3) bind-registry per condizionali/dinamiche (t2), gate di nuovo. Vincolo di fase 2: le condizionali restano fuori (t3 = REJECT se fatala). L'ordine inverso lascerebbe scoperti 35/37 pin nella finestra intermedia.

## Emendamenti

- **A-DS53** «fixture v3 — 7 buchi oracle-morsi»: multi-iface, iface-extends, enum-implements, self-vs-static, hook-subtype, final-const, readonly-promoted (+ positivo DNF).
- **A-DS54** «pin v3 a byte-count per canale; comparatore a marcatori bandito».
- **A-DS55** «bersaglio t4 = braccio PERSIST, dichiarato per NOME».
- **A-DS56** «v15: prevale “mai skip silenzioso”; fatal al bind hoisted, dinamici via gate».
- **A-DS57** «ordine A-DS51: checker→hoisted(+esenzioni stesso commit)→registry; gate ORM/hk dopo OGNI fase».

## Kill-switch

- **KS-DS-92-1**: comparatore che parse-a i pin per marcatori senza byte-count = REJECT.
- **KS-DS-92-2**: merge A-DS51 senza fixture A-DS53 per NOME nel gate = REJECT.
- **KS-DS-92-3**: fase-2 (hoisted) che fatala t3 o manca il persist-shape di t4 = REJECT.
