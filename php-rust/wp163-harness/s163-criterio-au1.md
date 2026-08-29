# Criterio S-163 p.3 — leva L-AU1 «autoload array-callable [obj,metodo] k=1 senza args-Vec» — scritto PRIMA di edit e run

1. **Leva**: in `try_autoload` (mod.rs), ammissione PER-LOADER (lista LIVE
   S-71.2, non hoistabile) della forma `Zval::Array` a 2 elementi
   `[Object, Str metodo]` (elementi DIRETTI, non-Ref): risoluzione
   `resolve_method_runtime` + predicato IC-fill (vincitore PUBLIC, non-static,
   `!private_shadow_in_chain`) + `func.simple_call && n_params==1` ⇒ dispatch
   per-miss via `call_method_one`/`push_method_frame_one` NUOVI (specchio di
   `call_fn_one`/`push_fn_frame_one` + braccio usable di
   `dispatch_instance_call` con deref=false, meno le 3 alloc). Ogni altra
   forma (statica per stringa, static via obj, privata/protetta, __call,
   n_params≠1, Ref-wrapped, ombra private) resta su `call_callable` INVARIATO.
2. **Segno**: braccio B (leva) PIÙ VELOCE del braccio A (gemello==pin s162)
   su m-arrload; giudice `m-arrload.php` N=10M letterale (gemello di
   m-missload, sola forma del loader cambiata), ns/miss al netto dei
   pavimenti per-binario.
3. **Bundle CONTATO (mai per analogia)**: 3 alloc/miss rimosse — (i) args-Vec
   `vec![arg]` in try_autoload, (ii) elems-Vec `iter().collect()` in
   invoke_array_callable, (iii) `to_vec()` del nome metodo. Coeff sito-autoload
   7,0±3,0 ns/passaggio (tabella PER-SITO, rimisura s162) ⇒ **banda smoke
   VINCOLANTE D ∈ [3×4; 3×10] = [12; 30] ns/miss**; fuori banda ⇒ ARBITRATO
   census (s163-census-au1.sh, conteggio alloc per miss su ENTRAMBI i bracci)
   PRIMA di R=5.
4. **Soglia record**: max(4 ns/miss, rumore drop-1, banda-layout); R=5 ABAB;
   smoke R=2 con early-stop a segno opposto.
5. **Guardie** (solo-regressione, set s162 EREDITATO): micro 6 categorie R=2
   sui due bracci + m-missload (closure NON toccata) + m-strmap/m-arrfilter;
   fixture bilaterali fx-au (BYTE-ID pin==oracle, collaudata sul pin) +
   fx-au-div (INVARIANZA pin==gemello, §3.27) + fx-sm/fx-sm-div/fx-am (L-AM2
   intatta); batteria `cargo test --release`; corpus 1412×2 per NOME;
   disasm bl run_loop A==B; dente loc PRE-dichiarato in loc_dente.rs.
6. **Catena artefatti**: gemello A dal tree s162 PULITO nel target CANONICO,
   hash==pin 20c63af44bfd077a PENA STOP; edit SOLO dopo lo stash di A; build B
   e stash via script s163-ab-au1.sh (COPIA DICHIARATA di s162-ab-am2.sh,
   manifest verificato); promozione SOLO via scripts/pin-*.sh con catena
   batteria→re-hash→stash→corpus/fixture→micro. NESSUN build prima della
   chiusura della coppia t13+ORM (il binario canonico è il pin sotto misura).
7. **Attesa a valle** (si dichiara ORA): quota Composer/ORM del sito
   array-callable NON censita in ns; l'effetto su ORM si giudica alla PROSSIMA
   coppia col SUO criterio (nessun claim da questa leva sui workload).
