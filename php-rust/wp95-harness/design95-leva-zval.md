# design95 — LEVA A-ZV1: il clone che muore subito

**Regola del progetto applicata**: predizione scritta PRIMA della misura
(WP-48), contatore del MECCANISMO prima dell'orologio (Bak, consulenza
S-95.0). Questo documento è firmato **prima** di toccare il codice.

## Il fatto misurato (profilo `prof95-media.out`, workload reale)

- `Zval::drop` = **7,20% della CPU userland**; `Zval::clone` = **2,85%**.
  Insieme **10,05%**, contro il 4,33% aggredibile dell'intero dispatch.
- `binary_value_ab` = **2,64% della CPU**, e **1,77% è il drop dei suoi
  operandi**: due terzi del costo di un'operazione binaria è distruggere
  gli operandi, non calcolare il risultato.
- Il chiamante più caldo (91,7% dei campioni di `binary_value_ab`) è il
  percorso `run_loop` che passa da `read_slot`/costanti, non il percorso
  `pop` dallo stack.

## Il meccanismo, dal codice

```rust
pub(super) fn read_slot(cell: &Zval) -> Zval {
    match cell {
        Zval::Undef => Zval::Null,
        Zval::Ref(r) => r.borrow().clone(),
        other => other.clone(),      // <-- CLONE incondizionato
    }
}
```

e al sito caldo (`run.rs:240`):

```rust
let lhs = read_slot(&self.frames[top].slots[s as usize]);
let v = self.binary_value_ab(BinOp::Concat, lhs, rhs)?;   // consuma e DROPPA
```

Lo slot **resta vivo** (è un registro della funzione): il valore clonato
muore subito dopo. Per `Zval::Str`/`Array`/`Object` — cioè le varianti che
portano un `Rc` — questo è **incremento del refcount seguito da decremento**:
lavoro netto ZERO, pagato a ogni esecuzione. Per Int/Bool/Float il clone è
una copia di parola e il costo è trascurabile: **la leva vale sulle varianti
con Rc**, ed è lì che va misurata.

`read_slot` ha 15 siti di chiamata (`run.rs` ×9, `mod.rs` ×5, `session.rs` ×1):
la leva non è "riscrivere read_slot", è **non materializzare un valore owned
quando il consumatore può leggerlo per riferimento**.

## La leva (A-ZV1), in ordine di rischio crescente

1. **Fast path per riferimento in `binary_value_ab`**. `binary_fast(b, &lhs,
   &rhs)` prende GIÀ riferimenti ed è puro. Esporre
   `binary_value_ref(b, &Zval, &Zval) -> Option<Zval>` e chiamarlo al sito
   caldo PRIMA di materializzare gli operandi; si clona solo se ritorna
   `None` (percorso generico invariato). Semantica identica per
   costruzione: è lo stesso predicato che gira oggi, solo prima del clone.
2. Estensione agli altri siti `read_slot` dove il consumatore è a sola
   lettura (uno per uno, ognuno con il suo test).

**Non in questa leva** (dichiarato): l'analisi di liveness/last-use in HIR
proposta da Stogov (operandi `Take|Borrow|Copy`). È la generalizzazione
corretta e va fatta dopo, quando questa avrà mostrato il segno.

## PREDIZIONE EX-ANTE (firmata prima della misura)

**Contatore del meccanismo** (dietro feature, conta prima e dopo):
`clone_rc` = numero di `Zval::clone` su varianti che portano un `Rc`
eseguiti nel workload `--group media`.

- **P1 (meccanismo)**: `clone_rc` deve calare di **almeno il 15%**. Se cala
  meno del 5%, la leva NON ha agito e qualunque Δ tempo viene da altro:
  run VOID.
- **P2 (tempo)**: user CPU di phpr sul media group **−1,0%…−2,5%**.
  Falsificata se il Δ è positivo o se supera −5% (un guadagno più grande di
  così non può venire da questo canale: sarebbe un altro effetto, da
  nominare prima di rivendicarlo).
- **P3 (parità)**: corpus Zend per NOME **1418 invariato** + refl 290;
  `battery61` rc=0 con gli stessi sei esiti; media group 762 test / 1912
  assertions **identici**.
- **P4 (nessun effetto collaterale sul footprint)**: peak footprint entro
  ±1,5% (questa leva non tocca l'allocazione, solo il refcount).

**Giudice**: coppia oracle-vs-phpr della stessa sera, con l'oracle
rimisurato (mai il denominatore congelato — sanatoria WP-96), più il
ri-profilo con la stessa ricetta samply: la quota di `Zval::drop` deve
scendere dal 7,20% al valore predetto.

**Coppia A/A obbligatoria prima del verdetto**: ricompilazione della
sorgente immutata, per conoscere lo spread inter-build. Un Δ sotto lo
spread è SCREEN, non verdetto (lezione LEVER-2, WP-93).

## Trappole nominate (Stogov)

- I rientri nella VM da `__destruct` e `set_error_handler`: un valore non
  più clonato potrebbe essere osservato in uno stato diverso da un
  distruttore che rientra. Il percorso generico resta invariato, ma il test
  va scritto.
- L'ordine di iterazione delle hash è osservabile nell'output PHP: la leva
  non deve toccare alcun ordine.
- `Zval::Ref`: `read_slot` fa `r.borrow().clone()`. Il fast path per
  riferimento deve prendere il `borrow()` per la durata dell'operazione e
  non oltre — attenzione al `RefCell` già preso in prestito da un operando
  che è lo stesso slot (`$a .= $a`): caso di test obbligatorio.
