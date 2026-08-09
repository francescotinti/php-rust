# s122-istruttoria-re.md — re residuo «10→8» SENZA swap (NEXT_SESSION §S-122 p.3)

## Correzione del bersaglio (a verbale)

L'args-Vec di CallHostBuiltinOut vale **1 alloc/iter, NON 2**: «10→8 via
args-Vec» era una lettura errata (il census conta le alloc, non le coppie
alloc+free). Decomposizione post-L-RE1 che SOMMA a 10 (preg_match argc=3,
3 gruppi): 1 args-Vec (run.rs:3504 → pop_keys/split_off mod.rs:8187) +
1 `Caps.groups` collect (preg.rs:175-183) + 3 `to_vec` gruppo (preg.rs:180) +
3 `Rc<PhpStr>` gruppo (preg.rs:2229) + 1 buffer PhpArray + 1 `Rc::new(arr)`
(preg.rs:2209-2220).

## Perché lo scratch su Vm non passa (e non deve)

Non è solo E0502 (il receiver `&mut self` non si spezza attraverso una
chiamata): è RE-ENTRANCY REALE — `flush_diags` (mod.rs:5504-5538) a
run.rs:3521/3524 può invocare l'error-handler UTENTE ⇒ run_loop annidato ⇒
stessa preg ⇒ scratch di Vm clobberato. Il take/restore era la cura di questo,
e il suo bookkeeping è la causa indiziata della refutazione L-ST1.
L'alternativa a indici `(top, base)` borrow-checka ma riscrive ~20 arm +
la macro `host_builtins!` (centinaia di arm): fuori scala per 1 alloc.

## Prossimo scalino SANO: 10→9 via scratch PER-ENGINE (non per-Vm)

`Caps.groups` (collect per chiamata) è riusabile come scratch dentro
`CachedRegex` — PRECEDENTE GIÀ IN CODICE: `CachedRegex::scratch`
(`CaptureLocations`, preg.rs:262) fa esattamente questo per le locations.
Nessun borrow di Vm attraversato, re-entrancy confinata alla singola regex
cache-entry (stesso perimetro del precedente). Runtime-only, php-types
INTOCCATO.

## Vincoli per il criterio (quando si tenta)

- Attesa ~1 alloc mimalloc/iter ⇒ pochi ns: bersaglio re con soglia
  max(4,00; BANDA_LAYOUT_re dalla misura S-122; 2×spread_A; 2×quanto) —
  se BANDA_LAYOUT_re risulta ≥ 10 la leva singola è INGIUDICABILE sulle
  micro: dichiararlo prima, non tentarla.
- Census di meccanismo: re 10,00→9,00 ESATTO, guardie alloc invariate.
- I 6 alloc «gruppi» (3 to_vec + 3 Rc) = blocco grosso ma tocca php-types:
  resta NOMINATO fuori perimetro (stessa radice del pavimento PhpStr 2-alloc
  di str/arr — vedi s122-istruttoria-arr.md).
