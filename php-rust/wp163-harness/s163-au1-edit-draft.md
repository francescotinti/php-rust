# S-163 L-AU1 — draft d'edit (l'edit VERO si applica SOLO dopo lo stash del gemello A; criterio s163-criterio-au1.md p.6)

## 1. calls.rs — dopo `call_fn_one` (specchio dichiarato)

```rust
/// L-AU1 (S-163): arity-1 mirror of [`Self::call_callable`] for a
/// pre-admitted `[object, method]` array callable — same baseline/drive
/// contract as [`Self::call_fn_one`], no args-Vec, no elems-Vec, no
/// method-name copy. Admission (per loader, per miss, by the caller):
/// public non-static user method, `simple_call && n_params == 1`, no
/// private homonym in the receiver's ancestor chain (the `MethodIc` fill
/// predicate — resolution is scope-independent, so skipping
/// `parent_private_rebind` + visibility is sound for every caller scope).
pub(super) fn call_method_one(
    &mut self,
    defc: usize,
    midx: usize,
    cid: usize,
    this: Zval,
    arg: Zval,
) -> Result<Zval, PhpError> {
    let baseline = self.frames.len();
    self.push_method_frame_one(defc, midx, cid, this, arg)?;
    if self.frames.len() == baseline {
        // Mirror of call_callable's no-frame arm: unreachable for a user
        // method body (which always pushes a frame); kept for structural
        // equality with the full path.
        return Ok(self.frames[baseline - 1]
            .stack
            .pop()
            .expect("host callable result on the caller stack"));
    }
    self.drive_to_return(baseline)
}
```

## 2. mod.rs — dopo `push_fn_frame_one` (specchio del braccio usable di `dispatch_instance_call`, deref=false al sito autoload)

```rust
/// L-AU1 (S-163): install a frame for a pre-admitted instance method with
/// a single by-value argument, without materializing an args-Vec. Mirrors
/// `dispatch_instance_call`'s usable arm at `deref: false` minus
/// `bind_params`' Vec: the intake equals its simple-call arm at arity 1
/// (`argc = 1; slots[0] = decay_arg(arg)`); `this`/`class`/`static_class`
/// set exactly as there (LSB = receiver's actual class).
fn push_method_frame_one(
    &mut self,
    defc: usize,
    midx: usize,
    cid: usize,
    this: Zval,
    arg: Zval,
) -> Result<(), PhpError> {
    let callee = &self.classes[defc].methods[midx].func;
    debug_assert!(callee.simple_call && callee.n_params == 1);
    let m = self.class_mod(defc);
    let mut frame = self.pooled_frame(callee, m);
    frame.argc = 1;
    frame.slots[0] = decay_arg(arg);
    frame.this = Some(this);
    frame.class = Some(defc);
    frame.static_class = Some(cid);
    self.enter_callee(frame)
}
```

## 3. mod.rs `try_autoload` — ammissione PER-LOADER accanto al blocco L-AL2

Dopo il blocco `fast` (closure), aggiungere:

```rust
// L-AU1 (S-163): loader `[obj, metodo]` Composer-style — metodo utente
// PUBLIC non-static `simple_call` arità-1 senza ombra private nella
// catena (predicato IC-fill: risoluzione scope-indipendente). La
// chiamata per-miss va via call_method_one, senza args-Vec né
// elems-Vec né copia del nome. Elementi DIRETTI (non-Ref): ogni altra
// forma resta su call_callable INVARIATO (criterio s163-criterio-au1.md).
let mut fastm = None;
if fast.is_none() {
    if let Zval::Array(a) = &loader {
        if a.len() == 2 {
            let mut it = a.iter();
            let e0 = it.next().map(|(_, v)| v);
            let e1 = it.next().map(|(_, v)| v);
            if let (Some(t @ Zval::Object(_)), Some(Zval::Str(mname))) = (e0, e1) {
                let cid = object_class_id(t).expect("object class id");
                let mb = mname.as_bytes();
                if let Some((defc, midx)) = resolve_method_runtime(&self.classes, cid, mb) {
                    let cm = &self.classes[defc].methods[midx];
                    if cm.visibility == Visibility::Public
                        && !cm.is_static
                        && cm.func.simple_call
                        && cm.func.n_params == 1
                        && !private_shadow_in_chain(&self.classes, cid, mb)
                    {
                        fastm = Some((defc, midx, cid, t.clone()));
                    }
                }
            }
        }
    }
}
let call = match (&fast, fastm) {
    (Some(cl), _) => self.call_closure_one(cl, arg.clone()).map(drop),
    (None, Some((defc, midx, cid, this))) => {
        self.call_method_one(defc, midx, cid, this, arg.clone()).map(drop)
    }
    (None, None) => self.call_callable(loader, vec![arg.clone()]).map(drop),
};
```

(sostituisce il `match &fast` esistente; il braccio closure resta primo e
INVARIATO).

## Aperture da verificare all'edit
- `a.iter()` su `PhpArray`: ordine d'inserzione, coppie `(key, &Zval)` — la
  DUAL-REPR (packed) restituisce gli elementi in ordine; il generico usa
  `deref_clone`, qui si ammettono SOLO varianti dirette (Ref ⇒ generico).
- visibilità dei simboli in mod.rs: `resolve_method_runtime`,
  `private_shadow_in_chain`, `Visibility`, `object_class_id` già usati lì.
- dente loc: mod.rs +~45 (push_method_frame_one + blocco ammissione),
  calls.rs +~30 (call_method_one) — PRE-dichiarare in loc_dente.rs.
