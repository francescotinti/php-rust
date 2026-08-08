# Verbale sedia Hoare — Concilio S-116→S-117 (lente: safe Rust, soundness, sigilli di tipo)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione (A subito / B regime / D metodo / C riserva) è giusta nell'ordine ma
sbagliata in DUE punti di sostanza dalla mia lente: contiene un componente inesistente
sulla piattaforma (BOLT) e lascia aperta in C una variante che violerebbe il sigillo
SAFE-only (NaN-boxing). Senza gli emendamenti R1 e R4 mi sarei OPPOSTO.

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)

**A′ → B(+D) → C-perimetrata.**

- **A′ (S-117)**: PGO rustc (`-Cprofile-generate`/`-Cprofile-use`) + fat LTO +
  `codegen-units=1` + **order-file ld64** (`-order_file`) per layout deterministico.
  NIENTE BOLT: BOLT riscrive ELF, **non supporta Mach-O su Apple Silicon** — su questo
  Darwin non è una rotta, è una casella vuota. L'order-file ld64 è il sostituto nativo
  e per giunta più affine al problema vero (il metro che boccia le leve è
  l'impaginazione, banda misurata fino a 10 ns/iter).
  Dalla mia lente A′ è **priva di rischi di soundness**: PGO/LTO non alterano la
  semantica del programma safe (garanzia del compilatore, non del profilo) e i sigilli
  VmGate ZST sono fatti di TIPO a compile-time — nessuna passata di codegen li tocca.
  BOLT invece avrebbe bypassato il compilatore riscrivendo il binario: la sua assenza
  su Mach-O ci risparmia l'unico pezzo di A che avrei rifiutato.
- **B come regime, D come selezione dei vagoni**: sì, con R2 (fedeltà per-vagone).
- **C riserva**: sì, ma col perimetro R4 pre-registrato ORA, non quando la si apre.

**Mossa concreta S-117**: spike A′ in un atto solo stile REGOLE §2 — ricetta build
emendata in `scripts/pin-phpr.sh` (profilo raccolto su workload DETERMINISTICO: sei
micro + held-out + smoke WP; `.profdata` CONGELATO e versionato fuori repo, mai
rigenerato a ogni build), poi ripetere ESATTAMENTE la batteria di attribuzione S-114/115:
2 leve nulle → banda N=2 per categoria sul binario A′. Solo dopo, micro R=5 e gate pieni.

## EMENDAMENTI

- **R1 — Depurare A da BOLT.** Cosa: A = PGO+LTO+cgu=1+order-file ld64. Perché: BOLT
  non esiste su Mach-O; inseguirlo brucia la sessione. Misura: il criterio PRE di S-117
  nomina solo strumenti eseguiti con successo su questo host.
- **R2 — Nel treno B, fedeltà PER-VAGONE, perf PER-TRENO.** Cosa: ogni vagone passa da
  solo admission/parità/batteria/corpus 1415 per NOME; solo il cronometro è giudicato
  sulla somma. Perché: una somma promossa può nascondere la regressione semantica di un
  vagone; la parità di output non è additiva. Misura: gate fedeltà eseguiti a ogni
  aggancio di vagone, verbale per NOME.
- **R3 — A′ = «build emendata»: TUTTE le bande decadono.** Cosa: banda micro N=2,
  banda held-out, banda layout si RIMISURANO sul binario A′ prima di giudicare
  qualunque leva (L-A inclusa). Perché: cgu=1+LTO cambia inlining e impaginazione; la
  tassa calls può cambiare segno. Misura: 2 leve nulle sul pin A′, banda pubblicata.
- **R4 — Perimetro safe di C, pre-registrato.** Cosa: se C si apre, la variante ammessa
  è arena per-richiesta + handle indicizzati generazionali + elisione refcount su path
  caldo. **NaN-boxing VIETATO**: impacchettare puntatori in bit di f64 richiede
  transmute/provenance-cast — unsafe per costruzione, rompe il sigillo VmGate.
  L'arena safe converte l'use-after-free in use-after-recycle (bug logico, non UB):
  i generation counter sono parte del design, non un optional. Misura: nessun `unsafe`
  nuovo (grep di gate già esistente), batteria+corpus invariati.

## KILL-SWITCH (pre-registrati)

- **KS-A1**: se dopo A′ la banda globale delle nulle non scende (≤ metà dell'attuale
  10 ns/iter su N=2) E il mediano delle sei micro non migliora ≥2%, A′ decade a fine
  S-118: si tiene solo ciò che ripara il metro, si passa a B su build corrente.
- **KS-A2**: se il `.profdata` non è riproducibile (due raccolte → layout con banda
  diversa oltre N=2), il PGO si sospende e resta solo order-file+LTO.
- **KS-B**: treno di 3 vagoni sotto la soglia di somma pre-registrata sui giudici, o
  un vagone che fallisce fedeltà → il vagone esce, il treno non muore.
- **KS-C**: C si apre SOLO se dopo 3 sessioni di A′+B il peggiore resta >3×.

## APPARATO minimo

Solo l'emendamento della ricetta in `scripts/pin-phpr.sh` (R3 lo esige: pin/stash
nascono già collaudati sull'atto nuovo). Nient'altro blocca l'oggetto.
