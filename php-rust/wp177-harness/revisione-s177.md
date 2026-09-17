# Revisione S-177 — lente SEMANTICA

## Verdetto: REGGE CON RILIEVI

## Ciò che regge
- L-CM1 (a): get/fill con epoch esplicita (bytecode.rs 222-236, 280-292); unico bump mod.rs:689 in `vm_new`, letto a 785; costruzioni solo per-richiesta (worker_pool.rs 918/1178/1787). Identica sotto A-DS15; se violata è PIÙ restrittiva (col TLS il Vm esterno accettava celle di un Vm annidato). Da precisare.
- (b) run.rs 279-281: `return l.checked_add(r)` ≡ arm `Add` (ora morto).
- Census: psm_* seguono l'ordine delle guardie dell'hit (run.rs 1027-1054 vs 957-968), PSM_OTHER=0, Σpsm=miss; un solo processo ⇒ ic_empty = mai riempita, nessun polimorfismo mascherato.
- Fetta 4: `class C { public $x=0; public $y=1; }` (wp172-harness/prop-dq.php): fatto di sorgente.

## Rilievi
1. **Perimetro «4,2M private = 47 %» è un TETTO**: Σpsr=159 % ⇒ private∩readonly ≥ 44 % del miss. PHPUnit 13 nel tarball ORM ha 523 `readonly class` e 142 promossi `private readonly` (Doctrine src: 0): il «55 % di Doctrine» è il sistema eventi PHPUnit, write-once in costruttore. Fill privato senza bit RO ⇒ perimetro ≈0.
2. **psm_ic_empty è per esecuzione, non per sito**: la lettura p.4 del criterio («ic_empty ⇒ siti freddi») è contraddetta per costruzione; quella usata è post hoc.
3. **cm1 incoerente**: il .out dice «né cifra né direzione», ma WP_SESSION_177 p.5 usa D_flag +3,60/+2,93 come «replica S-176». Il +60 % è scheduling su E-core: micro-architettura diversa, vale per TUTTI i D.
4. **E1/E2 post hoc**: PREV same-binary nasce in S-176 (az. 5); soglia 4 fissata dopo il 18,47. E2 (<150 %) attende a 526-838 % (lancio-cm1b.log): tetto 90 min ⇒ rc=8. Il lanciatore cita `s177-criterio-cm1b.md` (riga 2), inesistente.
5. **Tree mai collaudato**: 30 siti + test riscritti senza `cargo test` né corpus (CI_FEED potata a 710d823c, lock s177 vivo dalle 00:57); parità di C = 5 fixture.
6. **Hit con chiave mangled**: `write_property_at` (oop.rs 83-101) nel fallback scrive `name`, non la key; `resolve_prop_access` (oop.rs 426-437) dà `slot: None` se obj_class≠scope. «Impossibile» regge solo con vincolo obj_class==scope al fill.

## Azioni S-178
1. Contatori private∩readonly / private∩¬readonly (+ top-10 classi) PRIMA della leva; se PHPUnit domina, dichiararlo nel giudice ORM.
2. Emendare criterio census p.4 (ic_empty per esecuzione) dichiarando.
3. Degradare D_flag in WP_SESSION_177 p.5: nessuna replica da finestra contaminata.
4. Rimuovere il lock s177, leggere batteria+corpus del tree prima di ogni promozione; correggere il riferimento a criterio-cm1b.
5. Leva privata: vincolo obj_class==scope, fallback per key, mutante «slot stantio su figlio».
