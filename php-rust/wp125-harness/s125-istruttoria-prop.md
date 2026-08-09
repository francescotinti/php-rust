# s125-istruttoria-prop.md — prop OLTRE i cloni (NEXT §S-125 p.3; segue s122-istruttoria-prop)

Loop caldo release (ramo FUSO L-A, `run.rs:4214-4281`): CmpJmpSC → [PropGetSlotRecv+BinaryTCPropSetPop FUSI] → PropGetSlot(x) → BinarySTDst(Add,$s) → IncDecSlotJmp.

## Inventario per-iter del ramo fuso (dal SORGENTE; conteggi esatti, tempi = INFERENZA)

| Sito | file:riga | n/iter | natura |
|------|-----------|--------|--------|
| probe IC get (`ic.get(sk)`) + `scope_key` | run.rs:4227,4231 | 1+1 | lookup |
| probe IC set (`set_ic.get(sk)`) | run.rs:4254 | 1 | lookup |
| `borrow()` get + `deref_clone` y (copia Long) | run.rs:4235,4240 | 1+1 | RefCell + copia enum |
| `borrow()` guardia set | run.rs:4258 | 1 | RefCell |
| `borrow()` Ref-check + `borrow_mut()` write | oop.rs:87,95 | 1+1 | RefCell ×2 |
| `consts[cidx].to_zval()` | run.rs:4247 | 1 | costruzione enum (no alloc) |
| `binary_fast` + `replace_slot` + `gc_note(old)` | run.rs:4248,4271-4277 | 1 | aritmetica + store |

Fuori dal fuso: PropGetSlot(x) = probe IC + borrow + deref_clone (run.rs:4293-4311); BinarySTDst = read_slot($s) copia Long (run.rs:1727); CmpJmpSC/IncDecSlotJmp in-place.
**Totale RefCell/iter = 5** (4 nel fuso + 1 in PropGetSlot) · **probe IC/iter = 3** · **copie Long/iter = 3** (zvclone census: la conferma A=3,00 e l'effetto-fusione B−A=+2,00 arrivano dal controllo ±zval s125, criterio census p.5).

## Candidati NOMINATI (nessuno promosso qui; ogni A/B esige SOGLIA_LAYOUT v2-s125)

- **C1 set-side single-borrow** (unico candidato con meccanismo pieno): nel ramo fuso la guardia (4258) ha già verificato shape/slot ma il borrow muore; `write_property_at` ri-borrowa ×2. Fondere guardia+Ref-check+write in UN `borrow_mut` ⇒ RefCell 4→2/iter. Prezzo sostitutivo: zero alloc mosse, ~2 attraversamenti RefCell risparmiati ≈ 2-4 ns/iter — **rischio sotto-banda**, criterio solo DOPO la banda v2-s125 e con disasm prima/dopo (run_loop).
- C2 hoist `to_zval` const: costruzione enum pura ~1 ns — sotto-quanto, NON candidata.
- C3 probe IC: strutturale del bigramma; la via è il dente «direct-bind» (aperture per NOME), istruttoria separata.
- Il resto del 5,6× vive in dispatch+IC+RefCell aggregati: attribuzione ferma a INFERENZA finché mancano i contatori L1I/branch (backlog S-105 §2) — nessuna cifra per-sito dichiarata come misura.

## Esito p.3

prop «oltre i cloni» = 3 copie Long (enum, no alloc, margine ~pochi ns) + 5 RefCell + 3 probe IC per iter; la classifica alloc-removal alloca le leve ALTROVE (str/cbargs, re host-path). C1 resta apertura per NOME con vincolo banda.
