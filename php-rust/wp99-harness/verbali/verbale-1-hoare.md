# Verbale Sedia 1 — Hoare (design linguaggio/runtime Rust, safe-only) — Concilio WP-99

## VERDETTO: CON EMENDAMENTI

Su S-97.1 in sé non ho refutazioni: la disciplina del criterio scritto ha morso
(−30,7% < −40% → abbandono senza negoziare), e la rimozione del fold commutativo
const-lhs è la decisione giusta presa per la ragione giusta. Le verifiche del mio
perimetro, sul codice:

1. **`reg_store_slot` vs `Op::StoreSlot` (run.rs 655–672 vs 244–257): replica
   esatta.** Il pop/push del ramo typed_refs di StoreSlot è un puro transito del
   valore verso `typed_ref_assign`; tra push e secondo pop non accade nulla, e
   entrambi i path ri-indicizzano `slots[i]` FRESCO dopo la coercizione. Flusso
   del valore, ordine `store_slot`→`gc_note`, stato pila al momento di un
   eventuale `Err` (profondità d'ancora identica): equivalenti. Non refutabile.
2. **`reg_load_slot` su Undef: momento del warning corretto.** Stessa coda
   `self.diags` di `Op::LoadVar` (anch'esso «queued; flushed at the next emit
   point»), stesso punto di flush a valle, stesso ordine l-poi-r nel funnel
   (anche `$u+$u` → due warning, come la sequenza originale).
3. **Bitrot: parzialmente presidiato.** La batteria v3 ESEGUE il codice lowerato
   (`assert_eq!(run(&m), run(&lm))`, via `lower_func` diretto: gira a ogni
   `cargo test` senza dipendere dall'env). Ma vedi A-HO-99-2/3: due rami sono
   scoperti.

## Refutazione capitale (una): R-HO-99-CAP — il confine di H-B1 come scritto è FALSO

«Frame in registro, ricaricato SOLO ai confini call/ret/throw»: l'insieme dei
confini è sottostimato di un ordine. Sono archi di ri-entrata nel VM, presenti
dentro quasi ogni handler: `gc_note` (→ `__destruct`), `flush_diags`
(→ `set_error_handler`), `binary_value_ab` (→ `__toString`, overload GMP/BcMath,
init_trigger lazy), `typed_ref_assign`, `to_bool` su oggetti. In Rust SAFE un
`&mut Frame` cache-ato attraverso questi archi NON COMPILA — quindi niente UB di
aliasing, ma la premessa «ricarico solo a call/ret/throw» è irrealizzabile come
scritta: o si ricarica a OGNI arco (e il risparmio va ristimato su quella
frequenza), o si estrae il frame da `self.frames` (`mem::take`) accecando GC
root-scan e backtrace durante la ri-entrata. La forma safe onesta è lo
split-borrow strutturale (`Vm` = pila frame + resto; handler su
`(&mut Frame, &mut VmRest)`), dove il borrow checker FORZA la resa del frame a
ogni arco: la refutazione colpisce il testo dell'ipotesi, non l'obiettivo.

## Emendamenti

- **A-HO-99-1**: in `reg_load_slot`, `debug_assert` che `unit_slot_name` sia
  byte-uguale al name-const del LoadVar foldato (oggi il contratto WP-65 è
  fidato, non verificato; il ramo Undef è freddo, costo nullo).
- **A-HO-99-2**: snippet in batteria con typed property legata by-ref a un
  locale scritto da una forma `*Dst`: il ramo typed_refs di `reg_store_slot` è
  oggi NON esercitato da alcun test (messaggio di coercizione e TypeError
  byte-identici all'oracle).
- **A-HO-99-3**: test flag-on di attribuzione di RIGA su espressione multilinea
  che lancia TypeError (`$a\n+ $b`): l'op fuso siede alla posizione della
  finestra; il corpus flag-on non è mai stato diffato, la riga può slittare.
- **A-HO-99-4**: il criterio di H-B1 va scritto come NUMERO con predizione di
  canale PRIMA del codice (ns attesi da 4 indicizzazioni + 2 `len()`): «sotto
  il rumore della coppia R=3» è ~0,5% su 7,83 s — un predicato quasi vacuo
  contro un fattore 8, della famiglia già vietata («soddisfatto dal proprio
  testo»).
- **A-HO-99-5**: prima del design, ENUMERARE l'insieme vero degli archi di
  ri-entrata (lista sopra) e adottarlo come confine; realizzazione via
  split-borrow, mai via indice/puntatore cache-ato.

## Kill-switch

- **KS-HO-99-1**: se H-B1 introduce `unsafe`, raw pointer o cache di
  `slots.as_mut_ptr()`/`NonNull<Frame>`, la sessione si ferma (safe-only).
- **KS-HO-99-2**: se richiede `mem::take` del frame fuori da `self.frames`
  attraverso un arco di ri-entrata, H-B1 cade (GC/backtrace ciechi).
- **KS-HO-99-3**: nessuna promozione futura del flag-on a strada di parità
  senza corpus 1418 per NOME eseguito flag-ON (il verde attuale è flag-off).
