//! M1 — sonda del preambolo di dispatch (S-98.0, ordine Concilio WP-99 punto 1).
//!
//! NON è codice del VM: replica in isolamento le DUE forme del loop di
//! dispatch e misura la differenza D (ns/op) che H-B1 (split-borrow del
//! frame corrente) può rendere. La sonda è fedele all'ASM del binario di
//! parità 0dd98ebbb7eb2d96 (objdump del loop head a run_loop+0x258):
//!
//!   forma A (oggi): a OGNI opcode ricarica frames.len (guardia
//!     MAX_CALL_DEPTH), calcola top, ricarica frames.ptr, madd top*176,
//!     legge ip e func dal frame, bounds check su ops[ip], store ip+1,
//!     jump table. Il reload è FORZATO dal fatto che un ramo del match
//!     chiama &mut self (qui: un ramo freddo #[inline(never)] mai preso
//!     a runtime ma visibile a LLVM).
//!   forma B (H-B1): loop interno che TIENE &mut Frame; il ramo freddo
//!     esce al loop esterno (confine); guardia di profondità solo al
//!     confine. KS rispettate: no unsafe, no raw ptr, no mem::take,
//!     ip resta campo del frame (mai copiato in un locale di loop).
//!
//! Il programma eseguito è il noop-loop VERO dei micro (5 op/iter:
//! LoadVar, CmpJmpConst, IncDecSlot, Pop, Jump — census 500015/100000),
//! così la forma A si VALIDA contro i 6,27 ns/op misurati sul binario
//! di parità: se la sonda non riproduce quell'ordine, non è fedele.

use std::hint::black_box;
use std::time::Instant;

const MAX_CALL_DEPTH: usize = 25_000;
const ITERS: i64 = 200_000_000;

// 48 byte come Op del VM (tag + payload); varianti = i 5 op del noop-loop
// + il ramo freddo che forza il confine del borrow.
#[derive(Clone)]
enum OpS {
    LoadVar(u16, [u64; 4]),
    CmpJmpConst { slot: u16, k: i64, target: u32, pad: [u64; 2] },
    IncDecSlot(u16, [u64; 4]),
    Pop([u64; 5]),
    Jump(u32, [u64; 4]),
    ColdCall([u64; 5]), // mai eseguito a runtime; visto da LLVM
    // Varianti fredde MAI eseguite: esistono per dare al match la
    // PRESSIONE DI REGISTRI del run_loop vero (185 bracci), che nel
    // binario di parità spilla gli invarianti (&frames.len/&frames.ptr
    // stanno in [sp,#0x2c0/0x2c8] e vengono ricaricati a ogni op).
    Cold0([u64; 5]),
    Cold1([u64; 5]),
    Cold2([u64; 5]),
    Cold3([u64; 5]),
    Cold4([u64; 5]),
    Cold5([u64; 5]),
    Cold6([u64; 5]),
    Cold7([u64; 5]),
    Cold8([u64; 5]),
    Cold9([u64; 5]),
    Cold10([u64; 5]),
    Cold11([u64; 5]),
    Cold12([u64; 5]),
    Cold13([u64; 5]),
    Cold14([u64; 5]),
    Cold15([u64; 5]),
}

/// Invarianti vivi attraverso il loop (come adrp/costanti del run_loop
/// vero): forzano lo spill degli indirizzi caldi anche qui.
struct Pressure {
    inv: [u64; 24],
}

#[inline(never)]
#[cold]
fn cold_sink(acc: &mut u64, k: u64) {
    *acc = acc.wrapping_mul(6364136223846793005).wrapping_add(k);
}

struct FuncS {
    ops: Vec<OpS>,
}

// 176 byte come Frame del VM: ip e func agli offset "caldi", il resto slots/stack.
struct FrameS<'m> {
    func: &'m FuncS,
    ip: usize,
    slots: Vec<i64>,
    stack: Vec<i64>,
    _pad: [u64; 14], // porta la taglia verso i 176B del Frame vero
}

#[inline(never)]
#[cold]
fn cold_grow(frames: &mut Vec<FrameS<'_>>) {
    // Il ramo che rende &mut self irraggiungibile dal borrow tenuto:
    // esiste per LLVM, non per il runtime.
    frames.pop();
}

macro_rules! cold_arms {
    ($acc:ident, $d:ident, $op:ident, $frames:ident) => {
        match $op {
            OpS::Cold0(_) => cold_sink($acc, $d[0]),
            OpS::Cold1(_) => cold_sink($acc, $d[1]),
            OpS::Cold2(_) => cold_sink($acc, $d[2]),
            OpS::Cold3(_) => cold_sink($acc, $d[3]),
            OpS::Cold4(_) => cold_sink($acc, $d[4]),
            OpS::Cold5(_) => cold_sink($acc, $d[5]),
            OpS::Cold6(_) => cold_sink($acc, $d[6]),
            OpS::Cold7(_) => cold_sink($acc, $d[7]),
            OpS::Cold8(_) => cold_sink($acc, $d[8]),
            OpS::Cold9(_) => cold_sink($acc, $d[9]),
            OpS::Cold10(_) => cold_sink($acc, $d[10]),
            OpS::Cold11(_) => cold_sink($acc, $d[11]),
            OpS::Cold12(_) => cold_sink($acc, $d[12]),
            OpS::Cold13(_) => cold_sink($acc, $d[13]),
            OpS::Cold14(_) => cold_sink($acc, $d[14]),
            OpS::Cold15(_) => cold_sink($acc, $d[15]),
            _ => unreachable!(),
        }
    };
}

/// Forma A — il preambolo di OGGI, ricaricato a ogni opcode.
fn run_a(frames: &mut Vec<FrameS<'_>>, p: &Pressure, acc: &mut u64) -> i64 {
    // invarianti vivi attraverso il loop, come nel run_loop vero
    let mut d = [0u64; 16];
    for i in 0..16 {
        d[i] = black_box(p.inv[i]) ^ black_box(p.inv[i + 8]);
    }
    loop {
        if frames.len() > MAX_CALL_DEPTH {
            return -1;
        }
        let top = frames.len() - 1;
        let ip = frames[top].ip;
        let func = frames[top].func;
        let op = &func.ops[ip];
        frames[top].ip = ip + 1;
        match op {
            OpS::LoadVar(s, _) => {
                let v = frames[top].slots[*s as usize];
                frames[top].stack.push(v);
            }
            OpS::CmpJmpConst { slot, k, target, .. } => {
                let v = frames[top].slots[*slot as usize];
                if !(v < *k) {
                    frames[top].ip = *target as usize;
                }
            }
            OpS::IncDecSlot(s, _) => {
                frames[top].slots[*s as usize] += 1;
            }
            OpS::Pop(_) => {
                let v = frames[top].stack.pop();
                if v.is_none() {
                    return -2;
                }
            }
            OpS::Jump(t, _) => {
                if *t == u32::MAX {
                    // uscita del programma sintetico
                    return frames[top].slots[0];
                }
                frames[top].ip = *t as usize;
            }
            OpS::ColdCall(_) => {
                cold_grow(frames);
                if frames.is_empty() {
                    return -3;
                }
            }
            other => cold_arms!(acc, d, other, frames),
        }
    }
}

/// Forma B — H-B1: split-borrow, frame tenuto attraverso il loop interno,
/// ricaricato SOLO al confine (ramo freddo); guardia solo al confine.
fn run_b(frames: &mut Vec<FrameS<'_>>, p: &Pressure, acc: &mut u64) -> i64 {
    let mut d = [0u64; 16];
    for i in 0..16 {
        d[i] = black_box(p.inv[i]) ^ black_box(p.inv[i + 8]);
    }
    'outer: loop {
        if frames.len() > MAX_CALL_DEPTH {
            return -1;
        }
        let top = frames.len() - 1;
        let frame = &mut frames[top];
        loop {
            let ip = frame.ip;
            let func = frame.func;
            let op = &func.ops[ip];
            frame.ip = ip + 1;
            match op {
                OpS::LoadVar(s, _) => {
                    let v = frame.slots[*s as usize];
                    frame.stack.push(v);
                }
                OpS::CmpJmpConst { slot, k, target, .. } => {
                    let v = frame.slots[*slot as usize];
                    if !(v < *k) {
                        frame.ip = *target as usize;
                    }
                }
                OpS::IncDecSlot(s, _) => {
                    frame.slots[*s as usize] += 1;
                }
                OpS::Pop(_) => {
                    let v = frame.stack.pop();
                    if v.is_none() {
                        return -2;
                    }
                }
                OpS::Jump(t, _) => {
                    if *t == u32::MAX {
                        return frame.slots[0];
                    }
                    frame.ip = *t as usize;
                }
                OpS::ColdCall(_) => {
                    // confine: il borrow di `frame` termina qui
                    cold_grow(frames);
                    if frames.is_empty() {
                        return -3;
                    }
                    continue 'outer;
                }
                other => {
                    cold_arms!(acc, d, other, frames);
                    continue 'outer; // anche i freddi sono confini
                }
            }
        }
    }
}

fn program() -> FuncS {
    // il noop-loop dei micro: for($i=0;$i<ITERS;$i++){}
    //   0: LoadVar $i          (push)
    //   1: CmpJmpConst $i<K -> se falso salta a 5
    //   2: IncDecSlot $i
    //   3: Pop
    //   4: Jump 0
    //   5: Jump uscita (u32::MAX)
    // NB: ordine leggermente compresso rispetto al bytecode vero, ma
    // stessa POPOLAZIONE di 5 op caldi per iterazione e stessi bigrammi
    // dominanti (LoadVar->CmpJmp, CmpJmp->IncDec, IncDec->Pop, Pop->Jump,
    // Jump->LoadVar).
    FuncS {
        ops: vec![
            OpS::LoadVar(0, [0; 4]),
            OpS::CmpJmpConst { slot: 0, k: ITERS, target: 5, pad: [0; 2] },
            OpS::IncDecSlot(0, [0; 4]),
            OpS::Pop([0; 5]),
            OpS::Jump(0, [0; 4]),
            OpS::Jump(u32::MAX, [0; 4]),
            OpS::ColdCall([0; 5]), // irraggiungibile: esiste per il borrow
        ],
    }
}

fn time_run(which: char, func: &FuncS, p: &Pressure, acc: &mut u64) -> (f64, i64) {
    let mut frames = vec![FrameS {
        func,
        ip: 0,
        slots: vec![0; 8],
        stack: Vec::with_capacity(64),
        _pad: [0; 14],
    }];
    let t0 = Instant::now();
    let r = match which {
        'a' => run_a(black_box(&mut frames), p, acc),
        _ => run_b(black_box(&mut frames), p, acc),
    };
    (t0.elapsed().as_secs_f64(), r)
}

fn main() {
    assert_eq!(std::mem::size_of::<OpS>(), 48, "OpS deve pesare 48B come Op");
    eprintln!("sizeof(FrameS)={}", std::mem::size_of::<FrameS>());
    let func = program();
    let mut p = Pressure { inv: [0; 24] };
    for (i, v) in p.inv.iter_mut().enumerate() {
        *v = black_box(0x9e3779b97f4a7c15u64).wrapping_mul(i as u64 + 1);
    }
    let mut acc = 0u64;
    let ops_per_iter = 5.0;
    let total_ops = ITERS as f64 * ops_per_iter;
    // alternanza A/B per non regalare a nessuna forma la coda termica
    for round in 0..3 {
        for w in ['a', 'b'] {
            let (t, r) = time_run(w, &func, &p, &mut acc);
            println!(
                "round={} forma={} tempo_s={:.3} ns_per_op={:.3} check={}",
                round, w, t, t * 1e9 / total_ops, r
            );
        }
    }
    eprintln!("acc={acc}"); // tiene vivo il canale dei rami freddi
}
