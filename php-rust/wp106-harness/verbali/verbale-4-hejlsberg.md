# Verbale sedia 4 — HEJLSBERG (compilatori, codegen, layout/inlining) — Concilio WP-106 su S-104

## VERDETTO: CON EMENDAMENTI — la CADUTA di H-C2 è firmata e ben condotta; la conclusione «run_loop è ICACHE-BOUND» è NOMINATA, NON PROVATA.

### R-HE-106-1 — «icache-bound» conflaziona tre meccanismi non separati.
Il dato è: bl 1101→0, run_loop +8.000 B, Δ=−10,33/−11,33. Tre cause lo spiegano ugualmente bene: (i) icache/fetch (più testo caldo); (ii) **pressione di registri**: inlinare il glue in 1101 siti obbliga ogni sito a spill/fill dei vivi che prima la call-clobber ABI gestiva in un punto solo — costo che appare come istruzioni ESEGUITE in più, zero miss; (iii) **layout/BTB**: +8 KB traslano ogni blocco a valle, spostano allineamenti e alias nel branch predictor. Nota aggravante: la banda-layout 0,67 (N=1, KS-HE-105-2) non copre una perturbazione da 8 KB — l'attribuzione a icache è oggi una scelta narrativa tra tre candidati.

### R-HE-106-2 — l'aritmetica del +8.000 B smentisce «inline ovunque» letterale.
8.000 B / 1101 siti ≈ 7,3 B/sito, meno di 2 istruzioni: LLVM ha per forza specializzato per-sito (const-prop del discriminante, dead-drop elimination, tail-merge), non copiato il glue (83 istr). Quindi B potrebbe eseguire MENO istruzioni di A ed essere più lento — il che rafforzerebbe icache/layout — oppure aver gonfiato solo gli arm caldi. Senza il diff per-arm il «dove» dei +8 KB è ignoto.

### R-HE-106-3 — il fingerprint v2 pinna il SORGENTE, non il CODEGEN.
sha256 del blocco enum + align==8 è la metà giusta ma incompleta: il layout/inlining è funzione di (sorgente, rustc/LLVM, profilo). Le 31 copie per-CGU dicono che codegen-units incide; un bump di rustc o un cambio di lto/opt-level rifà il tilt dell'inliner a fingerprint invariato. align==8 è quasi vacuo (Rc/f64 lo forzano).

## Emendamenti

- **A-HE-106-1 (esperimento CHEAP discriminante, ~2h, binari GIÀ in stash phpr-s103/s104)**: contatori HW su A e B via `xctrace` CPU Counters (o equivalente): **INST_RETIRED/iter** e **L1I-miss (fetch stall)/iter** sul giudice prop. Lettura pre-registrata: retired ↑ e miss ≈ ⇒ pressione registri; retired ≈/↓ e miss ↑ ⇒ icache/layout; entrambi ≈ ⇒ BTB/allineamento (terzo braccio: pad-nop N≥3 che ricampiona la banda-layout, salda KS-HE-105-2).
- **A-HE-106-2**: riprodurre il flip SENZA toccare il sorgente: `-C llvm-args=-inline-threshold=…` (o `#[inline(never)]` sul glue) su B — se il Δ segue il solo knob dell'inliner, il meccanismo è confermato indipendente dai 6 siti.
- **A-HE-106-3 (criterio)**: aggiungere al fingerprint: versione rustc/LLVM + tupla di profilo (opt-level, lto, codegen-units, panic). Cambio di uno ⇒ criterio da ri-emettere.
- **A-HE-106-4 (leva, SOLO se A-HE-106-1 firma icache/layout)**: in ordine di costo: (1) **PGO** (`-Cprofile-generate/use` sul giudice+corpus): hot/cold split e layout li fa LLVM, zero cambi al sorgente, A/B pulito; (2) **outlining mirato dei rami freddi** del match (error path, op rare per census) con `#[cold]`/`#[inline(never)]`, verificando la RIDUZIONE in byte di run_loop al disasm; (3) riduzione taglia arm caldi (tail comuni) solo dopo, con cautela: i tail condivisi aggiungono salti.
- **A-HE-106-5 (aggancio del 21,2% senza nome)**: NON subordinato — è il **prefisso di targeting** della leva icache: PC-sampling dentro il range di run_loop (0x100337c30..0x100376a90) bucketizzato per arm via .debug_line nomina il 21,2% E produce la lista hot/cold per (2). Senza, l'outlining sceglie i bersagli a memoria.

## KS
- **KS-HE-106-1**: ogni futura frase «icache-bound» senza contatore fetch/miss a supporto = claim VOID; vale anche per i verbali.
- **KS-HE-106-2**: leva di outlining/PGO promossa senza disasm prima/dopo (bl-count + taglia run_loop + diff per-arm) = VOID (estende il protocollo S-104 §2d).

## Priorità S-105 (mio perimetro)
1. A-HE-106-1 (contatori: discrimina il collo — precede ogni leva di codice).
2. Se firmato: PGO A/B (leva più economica). 3. A-HE-106-5 (21,2% → lista bersagli). 4. A-HE-106-3 nel criterio v3.
