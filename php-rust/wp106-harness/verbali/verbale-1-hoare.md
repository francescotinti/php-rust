# Verbale sedia 1 — HOARE (Concilio WP-106, su S-104)

**VERDETTO: CONCORDO CON EMENDAMENTI.** L'ordine WP-105 è stato eseguito
per intero nella parte vincolante; la leva H-C2 è stata tentata con A/B
DUE volte (KS-GR-105-1 saldato); la caduta ha un meccanismo nominato dal
disasm. Ma tre claim eccedono la loro evidenza.

## Refutazioni

**R-HO-106-1 — «Revert verificato AL BYTE» è un overclaim.** Ciò che è
stato verificato: la taglia di UN simbolo (run_loop 257.632 B) più UN
punto di timing (prop 4,87 s). Uguaglianza di taglia ≠ identità di byte —
e la sessione stessa prova che il codegen è instabile (flip
dell'inliner): due binari possono coincidere in taglia e differire in
layout. Inoltre il binario NON può essere byte-identico al pin S-103:
restano predicato, align-assert, tooth (parity-null dichiarati). Il claim
va rinominato «revert verificato per taglia+timing» oppure completato con
hash dei byte del range di run_loop (2 minuti con lo strumento del §2b).

**R-HO-106-2 — La tesi «icache-bound» è SOTTO-DETERMINATA: una coppia di
binari, zero contatori.** L'esperimento ha girato UNA manopola che ha
cambiato MOLTE cose insieme: inlining globale, allocazione registri (via
call-clobber rimossi), layout/allineamento, pressione BTB. +8 KB su
257 KB è ~3% di testo: attribuirvi ~7% del tempo (11/162 ns) senza un
contatore frontend (L1i-miss/fetch-stall, ottenibili via xctrace sul
PAIO ARCHIVIATO negli stash — costo: due run) è una narrazione
plausibile, non una misura. La coerenza con S-100 non è conferma: più
storie di collo la spiegano. La tesi resta IPOTESI NOMINATA finché non ha
il contatore o una controprova di design (sotto).

**R-HO-106-3 — Il predicato orfano è sano ma il sigillo è incompleto.**
`is_trivial_drop` (zval.rs:248) ha UN solo referente: il tooth. Il match
esaustivo intrappola varianti NUOVE, NON il cambio di payload di una
variante esistente: un `Bool(_)` che diventasse heap-backed compilerebbe
ancora e il predicato mentirebbe `true` ⇒ leak al primo riuso futuro. Il
fingerprint sha256 vive in un `.out`, non nel compilatore.

## Emendamenti

- **A-HO-106-1**: sigillo di TIPO sui bracci trivial — const-check che i
  payload di Bool/Long/Double siano `Copy` (`fn _seal<T: Copy>()` con
  bool, i64, f64, accanto al size/align-assert). Tre righe, safe-only.
- **A-HO-106-2**: annotare nel doc di `is_trivial_drop` il verdetto
  S-104 (leva CADUTA, canale refutato, → hc2-ab-verdetto.out): il codice
  da solo, senza la storia, INVITA a rifare la leva.
- **A-HO-106-3**: sostituire nel registro «al byte» con la formula
  verificata (R-HO-106-1) e, alla prossima occasione di disasm, hashare
  il range run_loop del revert contro il pin.

## Kill-switch

- **KS-HO-106-1**: leva futura che cita «icache-bound» come premessa
  senza contatore frontend NÉ controprova di design = premessa VOID, A/B
  non ammesso.
- **KS-HO-106-2**: nuovo chiamante di produzione di `is_trivial_drop`
  senza criterio nuovo pre-registrato = reject — il predicato è reperto
  sigillato, non invito.

## Priorità S-105 (perimetro Hoare)

1. **H-ICS «cold-out» — la leva che è anche il test della tesi.** Se
   run_loop è icache-bound, la mossa vincente è l'INVERSA di H-C2:
   ridurre i byte del sentiero caldo outlinando in
   `#[cold] #[inline(never)]` i gestori dei op più freddi per census
   (safe-Rust puro). Predizione FIRMATA nel criterio: Δ>0; se Δ≤0 la
   tesi cade e KS-HO-106-1 la archivia. Una leva sola compra guadagno O
   refutazione — mai cieca. Prefisso obbligato: disasm prima/dopo
   (protocollo S-104) + bl-count.
2. **H-D SiteTag** (attribuzione della 1×32 B; ordine 3 residuo) come
   secondo braccio.
3. **Generator-in-cycle** (buco A-HO-103-2, fixture rossa già arbitra):
   debito di fedeltà del mio perimetro — container + descend delle
   catture quando si sceglie il punto-fedeltà.
