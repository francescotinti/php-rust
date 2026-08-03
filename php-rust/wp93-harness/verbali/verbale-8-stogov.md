# Verbale sedia 8 — Dmitry Stogov (Concilio WP-93, revisione S-91.0)

Perimetro: Zend/opcache, contratto LSP A-DS35/A-DS51. Oracle vivo: PHP 8.5.7 (probe in /tmp/stogov93, MAI nel repo).

## VERDETTO

Fase 0 SOLIDA (pin length-prefixed regge anche il multibyte, Q2), ma le 8 fixture v3 NON chiudono il perimetro: QUATTRO buchi residui morsi dall'oracle (Q1), regole di formattazione mancanti in fase 1 (Q3), e UNA refutazione capitale sull'esenzione ctor (Q4) che, non emendata, produce falsi fatal su ORM/hk in fase 2.

## Q1 — Buchi residui: SÌ, quattro (+1 di famiglia)

1. **Enum abstract method** — famiglia NUOVA, non "Declaration of": `Fatal error: Enum method E::m() must not be abstract` (line della decl). Buco → v4.
2. **Hook SET** — famiglia NUOVA, senza "Declaration of" e senza tipi nel messaggio: `Fatal error: Type of parameter $v of hook C::$x::set must be compatible with property type`. Il set si giudica contro il TIPO DELLA PROPERTY, non contro il set del parent. Buco → v4.
3. **Intersection pura**: param `A&B`→`A` è LEGALE (contravarianza, exit 0); il negativo è il ritorno `A&B`→`A`: `Declaration of C::m(): A must be compatible with P::m(): A&B`. Negativo non pinnato (w8 copre solo il positivo DNF) → v4.
4. **By-ref hook, property invariance**: `Declaration of & C::$x::get(): string must be compatible with & P::$x::get(): int` — prefisso `& ` CON SPAZIO, non modellato dal vincolo w5. Buco → v4.
5. Interface `final const`: `C::X cannot override final constant I::X` — stessa famiglia di w6 (route implements ≠ extends); fixture v4 opzionale.

## Q2 — Length-prefixed su multibyte: REGGE

Probe classe UTF-8 (`class Città` / `Provincia extends Città`): fatal `Declaration of Provincia::m(): string must be compatible with Città::m(): int`; `stdout_bytes=163` (wc -c) vs 162 caratteri (`à`=2 byte); rilettura per OFFSET di 163 byte = `cmp` byte-exact. Il formato A-DS54 è corretto per costruzione sui byte. Però NESSUNA delle 45 fixture ha payload multibyte: il caso non è pinnato → fixture UTF-8 in v4 (regressione anti-`wc -m`).

## Q3 — Vincoli fase 1: INSUFFICIENTI, cinque regole mancanti

Oracle: `(B&A)|string` stampato COME DICHIARATO (nessun riordino); `string|int` idem; `int|null` E `null|int` collassano a `?int`; MA `string|int|null` resta ESTESO — il collasso `?T` vale SOLO per l'unione binaria con null. Quindi: (a) "ordine canonico Zend" va emendato in "ordine DICHIARATO, zero sorting"; (b) collasso `?T` solo binario; (c) by-ref param grafia `int &$x` (spazio tipo/&, & attaccata a `$x`: `Declaration of C::m(string &$x): void must be compatible with P::m(int &$x): void`); (d) prefisso `& ` sugli hook by-ref (Q1.4); (e) la famiglia hook-set NON usa la forma "Declaration of" (Q1.2).

## Q4 — Esenzioni: INCOMPLETE, con trappola semantica

- **q4a**: ctor CONCRETO in classe ABSTRACT è ESENTE (oracle: vive, `concrete-ctor-in-abstract-EXEMPT`). L'esenzione è keyed sul PROTOTIPO (ctor da interfaccia o dichiarato `abstract`), NON sull'abstract-ness della classe. La dicitura "ctor NON esente per iface/abstract" nella lettura per-classe è REFUTATA — e Doctrine è piena di abstract con ctor concreto: falsi fatal garantiti.
- **q4b**: il vincolo iface-ctor si PROPAGA ai nipoti: `Declaration of C::__construct(string $y) must be compatible with I::__construct(int $x)` due livelli sotto. Nessun pin (v13 copre solo l'implements diretto) → v4.
- **w7 = anti-esenzione**: il check readonly-promoted morde DENTRO il ctor esente — l'esenzione copre il SIGNATURE check, non il lowering delle promoted. Va detto per NOME nel commit fase 2.

## Emendamenti

- **A-DS58**: sei fixture v4 per NOME coi messaggi esatti sopra: x1 enum-abstract, x2 hook-set, x3 intersection-return-widen, x4 byref-hook-get, x5 utf8-classname, x6 iface-ctor-grandchild; stesso pin length-prefixed.
- **A-DS59**: esenzione ctor riformulata: checked SOLO contro prototipo da interfaccia o ctor `abstract`; classe abstract con ctor concreto = ESENTE.
- **A-DS60**: le cinque regole di formattazione Q3 entrano nei vincoli di modellistica fase 1 (unit dedicate).

## Kill-switch

- **KS-DS-93-1**: merge fase 2 senza le fixture A-DS58 per NOME nel gate = REJECT (estende KS-DS-92-2).
- **KS-DS-93-2**: fatal del checker su ctor concreto di classe abstract (pattern q4a) = REJECT.
- **KS-DS-93-3**: comparatore in CARATTERI (wc -m) anziché BYTE = REJECT.

## Refutazioni capitali

**SÌ, una**: lettura per-classe dell'esenzione ctor (Q4/q4a) — refutata dall'oracle vivo. Emendative (non capitali): "ordine canonico" → ordine dichiarato; collasso `?T` solo binario.
