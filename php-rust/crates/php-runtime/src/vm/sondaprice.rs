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
        // S-149 p.2 (sonda-prezzo pair, wp149-harness/s149-criterio-pair.md):
        // coppia malloc+free alla TAGLIA del churn reale della testa hostcall
        // (shape s148: ≤16 B 98,8M · ≤48 B 107,9M) via Vec::with_capacity
        // esatta; il drop a fine iterazione chiude la coppia.
        let pair16 = bench(N_PAIR, |_| {
            let v: Vec<Zval> = Vec::with_capacity(1);
            std::hint::black_box(&v);
        });
        let pair32 = bench(N_PAIR, |_| {
            let v: Vec<Zval> = Vec::with_capacity(2);
            std::hint::black_box(&v);
        });
        let pair48 = bench(N_PAIR, |_| {
            let v: Vec<Zval> = Vec::with_capacity(3);
            std::hint::black_box(&v);
        });
        // Pattern pop_keys ESATTO (run.rs): push×3 sullo stack sorgente poi
        // `split_off` NON in testa (un elemento di fondo tiene at=1: il ramo
        // at==0 di std fa mem::replace e NON riprodurrebbe la coppia) =
        // malloc(48)+memcpy(3×Zval)+drop-glue+free per iterazione.
        let splitoff3 = {
            let mut src: Vec<Zval> = Vec::with_capacity(8);
            src.push(Zval::Long(0));
            bench(N_PAIR, |i| {
                src.push(Zval::Long(i as i64));
                src.push(Zval::Long(1));
                src.push(Zval::Long(2));
                let args = src.split_off(1);
                std::hint::black_box(&args);
            })
        };

        // S-152 p.1 (sonde-prezzo canali, wp152-harness/s152-criterio-sonde.md):
        // prezzi CORRENTI dei canali census C1/C2/C3 sull'handle Object REALE
        // + prezzi del SOSTITUTIVO mock store-indicizzato (forma A3 ratificata:
        // bucket+free-list, incref sullo slot, gen-check). Il mock è su slot
        // CALDO = lower bound OTTIMISTICO del sostitutivo, DICHIARATO nel
        // criterio (§2/§6): niente modello cache/working-set qui.
        let Zval::Object(orc) = &vobj else {
            return Ok(Zval::Bool(false));
        };
        let c1_pair = bench(N_MV, |_| {
            let c = Rc::clone(std::hint::black_box(orc));
            std::hint::black_box(&c);
        });
        let c2_borrow = bench(N_MV, |_| {
            let g = std::hint::black_box(orc).borrow();
            std::hint::black_box(&*g);
        });
        let c2_borrow_mut = bench(N_MV, |_| {
            let g = std::hint::black_box(orc).borrow_mut();
            std::hint::black_box(&*g);
        });
        // C3: coppia malloc+free alla taglia della cella condivisa
        // RefCell<Object> (il payload dell'Rc). Header Rc (2×usize) NON
        // incluso: proxy per DIFETTO dichiarato; init campi/props INVARIANTI
        // tra i mondi, fuori dal netto per costruzione.
        let obj_size = std::mem::size_of::<RefCell<Object>>();
        let c3_size_pair = bench(N_PAIR, |_| {
            let v: Vec<u8> = Vec::with_capacity(std::hint::black_box(obj_size));
            std::hint::black_box(&v);
        });
        struct MockSlot {
            gen: u32,
            rc: std::cell::Cell<u32>,
            payload: [u64; 4],
        }
        let store: Vec<MockSlot> = (0..1024u32)
            .map(|i| MockSlot { gen: i ^ 0x5a, rc: std::cell::Cell::new(1), payload: [0; 4] })
            .collect();
        let (idx, gen) = (512usize, store[512].gen);
        let mock_deref = bench(N_MV, |_| {
            let s = &store[std::hint::black_box(idx)];
            if s.gen != std::hint::black_box(gen) {
                unreachable!("gen-check fallito nel mock");
            }
            std::hint::black_box(&s.payload);
        });
        let mock_dup_rel = bench(N_MV, |_| {
            let s = &store[std::hint::black_box(idx)];
            s.rc.set(s.rc.get() + 1);
            std::hint::black_box(&s.rc);
            s.rc.set(s.rc.get() - 1);
        });
        // Drop sostitutivo (coda decrementi, Matsakis): push dell'id sulla
        // coda riusata; il clear ammortizzato resta DENTRO la misura. Il
        // decref applicato al drenaggio è già prezzato da mock_dup_rel.
        let mock_decq = {
            let mut q: Vec<u32> = Vec::with_capacity(1024);
            bench(N_MV, |i| {
                q.push(std::hint::black_box(i as u32));
                if q.len() == 1024 {
                    q.clear();
                }
            })
        };
        // Alloc sostitutivo di C3: pop free-list + gen-bump + push di ritorno
        // (slot riusato, NESSUN malloc sul cammino).
        let (mock_alloc, miheap_pair) = {
            let mut free_ids: Vec<u32> = (0..1024).collect();
            let mut store2: Vec<(u32, [u64; 4])> = vec![(0, [0; 4]); 1024];
            let ma = bench(N_PAIR, |_| {
                let id = free_ids.pop().unwrap();
                let slot = &mut store2[std::hint::black_box(id as usize)];
                slot.0 = slot.0.wrapping_add(1);
                std::hint::black_box(&slot.1);
                free_ids.push(id);
            });
            // Braccio mi_heap (Leijen R2): coppia alloc+free a taglia Object
            // su heap mimalloc DEDICATO (il per-richiesta di A3); new/destroy
            // dell'heap fuori misura = ammortizzati sul blocco, dichiarato.
            let mh = unsafe {
                let heap = libmimalloc_sys::mi_heap_new();
                let v = bench(N_PAIR, |_| {
                    let p = libmimalloc_sys::mi_heap_malloc(heap, std::hint::black_box(obj_size));
                    std::hint::black_box(p);
                    libmimalloc_sys::mi_free(p);
                });
                libmimalloc_sys::mi_heap_destroy(heap);
                v
            };
            (ma, mh)
        };

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
        for (k, v) in [
            ("pair16", pair16),
            ("pair32", pair32),
            ("pair48", pair48),
            ("splitoff3", splitoff3),
        ] {
            out.push_str(&format!("s149.price.{k}_ns={v:.4}\n"));
        }
        out.push_str(&format!(
            "s149.price.zval_size={}\ns149.price.n_pair={N_PAIR}\n",
            std::mem::size_of::<Zval>(),
        ));
        for (k, v) in [
            ("c1_pair", c1_pair),
            ("c2_borrow", c2_borrow),
            ("c2_borrow_mut", c2_borrow_mut),
            ("c3_size_pair", c3_size_pair),
            ("mock_deref", mock_deref),
            ("mock_dup_rel", mock_dup_rel),
            ("mock_decq", mock_decq),
            ("mock_alloc", mock_alloc),
            ("miheap_pair", miheap_pair),
        ] {
            out.push_str(&format!("s152.price.{k}_ns={v:.4}\n"));
        }
        out.push_str(&format!(
            "s152.price.obj_size={obj_size}\ns152.price.n_mv={N_MV}\ns152.price.n_pair={N_PAIR}\n"
        ));
        if f.write_all(out.as_bytes()).is_err() {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(true))
    }
}
