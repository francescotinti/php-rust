//! S-145 sonda-B, lato PREZZI (modello wp145-harness/s145-sonda-b-modello.md,
//! regola madre s144-criterio-B.md p.2–3): loop di prezzo per-movimento
//! dentro il binario `--features sonda-price`. La feature gate SOLO questo
//! modulo e il suo braccio di dispatch: nessun contatore census è attivo,
//! quindi i cammini clone/drop/gc_note prezzati sono la forma di parità.
//! Ogni iterazione dei segmenti mv_* è una COPPIA clone+drop (il drop del
//! clone chiude l'iterazione): il segmento prezza il ciclo di vita di UN
//! movimento. Tutti i contrasti (seg−cal, seg_classe−seg_scalar) vivono in
//! QUESTO binario. MAI una cifra verdict-grade da una build col builtin ma
//! con una census accesa.

use super::*;

const N_MV: u64 = 200_000_000;
const N_PAIR: u64 = 20_000_000;

fn bench(n: u64, mut f: impl FnMut(u64)) -> f64 {
    let t = std::time::Instant::now();
    for i in 0..n {
        f(i);
    }
    t.elapsed().as_nanos() as f64 / n as f64
}

impl<'m> Vm<'m> {
    /// `__phpr_sonda_b($str, $arr, $obj)` (builtin nascosto, solo probe):
    /// scrive i prezzi grezzi (cal NON sottratto: la sottrazione è del
    /// parser, così il raw resta auditabile) sul file `PHPR_SONDA_OUT`.
    /// Ritorna `true` se ha scritto tutte le chiavi, `false` su argomenti
    /// della specie sbagliata (lo smoke pretende `true` + chiavi presenti).
    pub(super) fn ho_sonda_b(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        use std::io::Write;
        let mut it = args.into_iter();
        let (Some(vstr), Some(varr), Some(vobj)) = (
            it.next().map(|v| v.deref_clone()),
            it.next().map(|v| v.deref_clone()),
            it.next().map(|v| v.deref_clone()),
        ) else {
            return Ok(Zval::Bool(false));
        };
        if !matches!(vstr, Zval::Str(_))
            || !matches!(varr, Zval::Array(_))
            || !matches!(vobj, Zval::Object(_))
        {
            return Ok(Zval::Bool(false));
        }
        let Some(path) = std::env::var_os("PHPR_SONDA_OUT") else {
            return Ok(Zval::Bool(false));
        };
        let Ok(mut f) = std::fs::File::create(path) else {
            return Ok(Zval::Bool(false));
        };

        let cal = bench(N_MV, |i| {
            std::hint::black_box(i);
        });
        let scalar_src = Zval::Long(42);
        let mv_scalar = bench(N_MV, |_| {
            let c = std::hint::black_box(&scalar_src).clone();
            std::hint::black_box(&c);
        });
        let mv_str = bench(N_MV, |_| {
            let c = std::hint::black_box(&vstr).clone();
            std::hint::black_box(&c);
        });
        let mv_arr = bench(N_MV, |_| {
            let c = std::hint::black_box(&varr).clone();
            std::hint::black_box(&c);
        });
        let mv_obj = bench(N_MV, |_| {
            let c = std::hint::black_box(&vobj).clone();
            std::hint::black_box(&c);
        });
        let note_scalar = {
            let v = Zval::Long(7);
            bench(N_MV, |_| {
                self.gc_note(std::hint::black_box(&v));
            })
        };
        // Una nota fuori misura bufferizza l'oggetto: il loop prezza il
        // braccio REPEAT (borrow + flag, early-out), quello che i conteggi
        // gcnote_cont pagano in stragrande maggioranza (il sovrapprezzo
        // first-note resta NON prezzato, dichiarato nel modello).
        self.gc_note(&vobj);
        let note_cont_repeat = bench(N_MV, |_| {
            self.gc_note(std::hint::black_box(&vobj));
        });
        let pair_zcell = bench(N_PAIR, |i| {
            let b = php_types::zcell(Zval::Long(i as i64));
            std::hint::black_box(&b);
        });
        let pair_arr0 = bench(N_PAIR, |_| {
            let b = Rc::new(php_types::PhpArray::new());
            std::hint::black_box(&b);
        });

        let mut out = String::new();
        for (k, v) in [
            ("cal", cal),
            ("mv_scalar", mv_scalar),
            ("mv_str", mv_str),
            ("mv_arr", mv_arr),
            ("mv_obj", mv_obj),
            ("note_scalar", note_scalar),
            ("note_cont_repeat", note_cont_repeat),
            ("pair_zcell", pair_zcell),
            ("pair_arr0", pair_arr0),
        ] {
            out.push_str(&format!("s145.price.{k}_ns={v:.4}\n"));
        }
        out.push_str(&format!("s145.price.n_mv={N_MV}\ns145.price.n_pair={N_PAIR}\n"));
        if f.write_all(out.as_bytes()).is_err() {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(true))
    }
}
