# Verbale Sedia 5 — Bak (microarchitettura) — Concilio WP-100 su S-98.0 / programma S-99.0

**VERDETTO: i due esiti (H-B1 a tavolino; H-B2 spedita) SOPRAVVIVONO, ma NON
per le ragioni scritte nei `.out`** — tre refutazioni capitali sul
ragionamento microarchitetturale, e il punto 3 dell'ordine S-99.0 (criterio
rollout «derivato da D=6,07») è **NON DERIVABILE** così com'è.

## Refutazioni capitali

**R1 — La sonda non misura Δ(A_reale, B).** Nell'ASM della sonda
(`m1-probe-loops.asm`, forma A) `frames.len` vive in **x23 (registro)** e ptr
sta dietro UN load da `[x21,#8]`; nel run_loop vero
(`m1-runloop-preamble.asm`) len e ptr arrivano da **catene di DUE load
dipendenti** (`[sp,#0x2c0]→[x8]`, `[sp,#0x2c8]→[x8]`) che alimentano il madd
dell'indirizzo frame → ldr ip → madd op → ldrb tag. La sonda ha misurato
Δ(A_sonda, B), sistematicamente più piccolo del vero. Il verdetto P<10% regge
SOLO grazie al tetto anti-hiding — vedi R2.

**R2 — Il tetto anti-hiding è auto-contraddittorio.** D_tetto = 1,4 × 11/28 è
una ripartizione per CONTEGGIO di istruzioni: esattamente il metodo che la
STESSA sessione mette nei NON-riproporre. Sopravvive per fortuna aritmetica
(6,7% < 10%); con soglia 8% il verdetto sarebbe stato deciso da un numero
costruito col metodo vietato. Ogni futura P deve venire da un argomento di
CATENA DI DIPENDENZE: il preambolo è fuori dal cammino del VALORE perché la
jump table è servita dal predittore indiretto (il noop a 5 bigrammi fissi è
pasto per un TAGE; i load servono solo alla verifica del retire) — non da
quote di istruzioni.

**R3 — D=6,07 è 2,2–4,3× il SUO tetto statico** ([1,4–2,8] ns pre-registrato
nello stesso `.out`): il meccanismo NON è quantitativamente spiegato.
Candidati non separati: (i) prologo/epilogo di `binary_value_ab`
(callee-saved, ~10 store+load per chiamata); (ii) marshalling — su arm64
AAPCS un composito >16B passa per **puntatore a copia**: ogni Zval è store
24B + load nel callee, e il ritorno `Result` è sret ⇒ 3–4 salti
store→load-forwarding SERIALI (4–5 cicli l'uno) sul cammino del VALORE — è
per questo che il plumbing di chiamata paga dove il preambolo era gratis;
(iii) pop/push del `Vec` (load len, branch, store len, store/load elemento).
6 ns ≈ 19 cicli: plausibile come somma, MAI decomposto. Senza decomposizione
il criterio del rollout è un numero copiato, non derivato.

**R4 (minore) — Il −0,53 della forma B (build 2) è un artefatto della
sonda**: testa di bounds-check DUPLICATA (ingresso + back-edge, righe 39–51
dell'ASM), allineamento/BTB. La lezione corretta è «la sonda ha un pavimento
di rumore ~0,5 ns/op», NON «split-borrow perde». Il NON-riproporre di H-B1
resta valido sul solo P<10%.

## Punto (c): quota di D trasferibile alle forme registro

BinarySS/SC/Dst NON hanno pop/push: la quota (iii) sparisce per costruzione.
Restano (i)+(ii) se anche le forme registro chiamano `binary_value_ab`.
Quota trasferibile: plausibile 50–70%, ma è stima da tetto, **non misura** —
ignota finché R3 non è decomposta.

## Emendamenti

- **A-BA-100-1**: PRIMA del rollout flag-on, UNA build intermedia che
  decompone D (es. `#[inline(always)]` del solo ramo Add nel funnel, tenendo
  pop/push): separa (i)+(ii) da (iii); il criterio delle forme registro si
  pre-registra dalla quota MISURATA.
- **A-BA-100-2**: annotare in `m1-preamble.out` che il tetto anti-hiding è
  count-derived (R2) e che la sonda sotto-misura Δ (R1); la lezione «reload
  gratis» si riscrive «non misurabile sotto ~0,5 ns con questa sonda su
  questo core».
- **A-BA-100-3**: nel collaudo arith flag-off di S-99, profilo a campioni sul
  loop-head: se il preambolo pesa >7% del tempo, M1 si riapre (il noop
  generalizza all'arith solo se i port L1 non saturano — i corpi Zval li
  contendono).

## Kill-switch

- **KS-BA-100-1**: VOID ogni criterio di rollout copiato da D=6,07 senza la
  decomposizione A-BA-100-1; ogni occorrenza (Sub/Mul int-int, forme
  registro) porta il SUO controfattuale statico con la quota pop/push
  sottratta.
- **KS-BA-100-2**: VOID ogni claim della sonda M1 sotto 2× il suo pavimento
  (1,0 ns/op): si pubblica come banda, mai come verso.
