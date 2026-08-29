# S-162 L-AM2 — bozza d'edit (si applica SOLO dopo lo stash del gemello A; il patch reale nasce dal diff del braccio B)

Tre pezzi, tutti specchi di forme già promosse (L-AM1/L-AL2):

## 1. calls.rs (+~22; file 1438, cap generico 2000)
```rust
/// L-AM2 (S-162): once-per-host-call resolution of a plain string callable
/// to a user function admitted for the arity-1 no-Vec path. Mirrors
/// `invoke_named`'s user-function resolution (leading-backslash strip,
/// `find_fn_ci`, then `linked_functions`); admission: `simple_call` with
/// exactly one parameter. Any other shape (builtin, "Class::method",
/// conditional not yet linked) returns None and stays on the generic loop.
pub(super) fn resolve_fn_one(&self, name: &[u8]) -> Option<(&'m Module, usize)> {
    let name = name.strip_prefix(b"\\").unwrap_or(name);
    let hit = self
        .module
        .find_fn_ci(name)
        .map(|i| (self.module, i))
        .or_else(|| self.linked_functions.get(LcKey::new(name).as_slice()).map(|&(m, i)| (m, i)))?;
    let f = &hit.0.functions[hit.1];
    (f.simple_call && f.n_params == 1).then_some(hit)
}

/// L-AM2 (S-162): arity-1 mirror of [`Self::call_callable`] for a
/// pre-admitted user function named by a string callable — same
/// baseline/drive contract as [`Self::call_closure_one`], no args-Vec.
pub(super) fn call_fn_one(&mut self, fmod: &'m Module, idx: usize, arg: Zval) -> Result<Zval, PhpError> {
    let baseline = self.frames.len();
    self.push_fn_frame_one(fmod, idx, arg)?;
    if self.frames.len() == baseline {
        return Ok(self.frames[baseline - 1]
            .stack
            .pop()
            .expect("host callable result on the caller stack"));
    }
    self.drive_to_return(baseline)
}
```

## 2. mod.rs (+~16; dente 25831 → salita PRE-dichiarata in loc_dente.rs nel braccio B)
```rust
/// L-AM2 (S-162): install a frame for a pre-admitted user function with a
/// single by-value argument, without materializing an args-Vec. Admission
/// (checked once by the caller, per host call, via `resolve_fn_one`):
/// `simple_call && n_params == 1`; a host value is never an `ArgPlace`.
/// Mirrors `invoke_named`'s user-function arm minus `bind_params`' Vec:
/// intake equals the simple-call arm at arity 1.
fn push_fn_frame_one(&mut self, fmod: &'m Module, idx: usize, arg: Zval) -> Result<(), PhpError> {
    let callee = &fmod.functions[idx];
    debug_assert!(callee.simple_call && callee.n_params == 1);
    let mut frame = self.pooled_frame(callee, fmod);
    frame.argc = 1;
    frame.slots[0] = decay_arg(arg);
    self.enter_callee(frame)
}
```

## 3. host.rs, ho_array_map ramo 1-array (+~18; dente 7708 → salita PRE-dichiarata)
Dopo il blocco L-AM1 (closure), PRIMA del loop generico:
```rust
// L-AM2 (S-162): 1 array + STRING-callable a funzione UTENTE simple_call
// arità-1 — risoluzione UNA volta per chiamata (niente to_vec del nome,
// scan "::" né find_fn_ci per-elemento), poi dispatch per-elemento senza
// args-Vec via call_fn_one. Ogni altra forma (builtin, "Class::method",
// array-callable, closure non ammessa) cade sul loop generico, invariato
// per costruzione ("::" non risolve mai in find_fn_ci/linked).
if let Zval::Str(s) = &cb {
    if let Some((fm, idx)) = self.resolve_fn_one(s.as_bytes()) {
        for (k, v) in entries {
            let mapped = self.call_fn_one(fm, idx, v)?;
            out.insert(k, mapped);
        }
        return Ok(Zval::Array(Rc::new(out)));
    }
}
```
(il blocco sta dentro `if !null_cb { … }` esistente accanto al ramo L-AM1.)

## Semantica dichiarata (per il criterio)
- Hoist della risoluzione: per le forme ammesse la risoluzione per-elemento è
  IDEMPOTENTE (funzione utente non ridefinibile; linked può solo AGGIUNGERE
  nomi nuovi, mai cambiare l'esito di un nome che GIÀ risolve) — semantica
  invariata per costruzione.
- `LcKey` import da verificare in calls.rs (già usato da invoke_named nello
  stesso file: nessun import nuovo).
- Stringify (`value_builtin_string_coerces`) tocca SOLO builtin: fuori
  dall'ammissione.
