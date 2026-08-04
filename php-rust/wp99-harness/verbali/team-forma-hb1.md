# Nota di team «forma-hb1» — Concilio WP-99

**Relatore**: team forma-hb1. **Fonti**: SOLO verbale-1-hoare.md e verbale-2-matsakis.md.

## 1. Convergenze — la riformulazione di H-B1 che entrambe le sedie accettano

Le due sedie refutano in indipendenza la stessa premessa da due lati (Hoare: l'insieme dei confini «call/ret/throw» è sottostimato di un ordine; Matsakis: la forma letterale è E0499 da manuale) e convergono sulla stessa forma sostitutiva:

> **H-B1 riformulata**: loop interno del run_loop su **split-borrow strutturale di campo** — `let fr = self.frames.last_mut()` (Matsakis) / handler su `(&mut Frame, &mut VmRest)` (Hoare) — con `fr.ip` che resta il **campo vero del frame** mutato attraverso il prestito vivo; **confine = ogni opcode che richiede un metodo `&mut self`** (funnel, gc_note, typed_ref_assign, flush_diags, chiamate): lì si fa break al loop esterno e il borrow checker FORZA la resa del frame. Guardia di profondità dove `frames` può crescere.

Vincoli KS congiunti, tutti compatibili tra loro (nessuna tensione):
- **No `unsafe`, no raw pointer, no cache di `as_mut_ptr()`/`NonNull<Frame>`** (KS-HO-99-1 ≡ KS-MA-99-3).
- **No `mem::take` del frame** attraverso archi di ri-entrata: Hoare per GC root-scan/backtrace ciechi (KS-HO-99-2), Matsakis per unsoundness semantica verso gli osservatori cross-frame — backtrace, `current_frame_args`, `var_dyn_read` (KS-MA-99-3). Stessa proibizione, due giustificazioni complementari.
- **No `ip` locale con write-back «ai confini»** (KS-MA-99-2): ogni sito che emette diag legge `frames[top].ip` via `cur_line`; un ip staccato sposta le righe dei messaggi in silenzio. Coerente con l'osservazione di Hoare che `flush_diags` è esso stesso un arco di ri-entrata (`set_error_handler`).
- **Criterio numerico scritto PRIMA del codice** (A-HO-99-4 ≡ A-MA-99-2): entrambe bollano «sotto il rumore R=3» come predicato quasi vacuo; la predizione di canale (ns attesi dalle 4 indicizzazioni + 2 `len()`) precede l'implementazione.
- Convergenza implicita anche sul **conto dei guadagni**: il beneficio copre solo il fast-path, mentre S-97.1 mostra che il costo sta negli opcode che entrano nel funnel `&mut self`.

## 2. Conflitti reali

Nessun conflitto di sostanza. Due differenze di posizione, da NON appianare:
- **Meccanismo della sonda**: Matsakis prescrive una **sonda per ADDIZIONE** (+2 indicizzazioni/opcode dietro flag; se `arith` non sale oltre il rumore, H-B1 cade senza essere scritta — refutazione capitale n.2). Hoare chiede solo la predizione numerica a priori, senza sonda. La sonda è più forte e la ingloba; ma la paternità e l'obbligo sono di Matsakis.
- **Granularità del confine**: Hoare enumera gli **archi di ri-entrata** come insieme da censire (gc_note→`__destruct`, flush_diags→`set_error_handler`, binary_value_ab→`__toString`/GMP/BcMath/lazy-init, typed_ref_assign, to_bool su oggetti); Matsakis dà il criterio **sintattico** («ogni metodo `&mut self`»). Il criterio di Matsakis è il sovra-insieme verificato dal compilatore; la lista di Hoare è il censimento semantico che dice QUANTI confini sono, cioè se il risparmio sopravvive.

## 3. Priorità proposte per l'ordine S-98.0 (vista di questo team)

1. **Enumerazione degli archi di ri-entrata PRIMA di ogni design** (A-HO-99-5): senza il conteggio della loro frequenza sul workload, il risparmio non è stimabile.
2. **Sonda per addizione + predizione di canale scritta in apertura** (A-MA-99-2 + A-HO-99-4): se la sonda non morde, H-B1 muore a costo zero.
3. Solo dopo, eventuale implementazione nella forma vincolata §1, con **KS-MA-99-1** attivo (`binary_fast` resta `fn(BinOp, &Zval, &Zval) -> Option<Zval>`, pura, senza canale diags).
4. **Fixture dovute** (flag-only, non bloccanti ma prima di ogni promozione): typed-ref by-ref su `*Dst` (A-HO-99-2), drop-order su TypeError in catch (A-MA-99-3), riga su espressione multilinea (A-HO-99-3); nessuna promozione flag-on senza corpus 1418 per NOME flag-ON (KS-HO-99-3).
