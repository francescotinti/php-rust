# Revisione S-177 — lente SEMANTICA

## Verdetto: REGGE CON RILIEVI

## Ciò che regge
- **L-CM1 (a)**: `PropIc/MethodIc::get/fill` (bytecode.rs 222-236, 280-292) confrontano l'epoch passata; l'unico bump a runtime è mod.rs:689 in `vm_new` (653), letto a 785; costruzioni solo per-richiesta (worker_pool.rs 918/1178/1787, `run_*` di mod.rs); nessun altro chiamante di `ic_epoch()` fuori dai test (22162, 22432). Sotto A-DS15 identica; se violata (Vm annidato) è PIÙ restrittiva e chiude una falla latente: col TLS il Vm esterno accettava celle riempite dall'interno con id numerici omonimi. «Identica per costruzione» vale solo sotto A-DS15 — da precisare.
- **(b)** run.rs 279-281: `return l.checked_add(r)` ≡ arm `Add => l.checked_add(r)?` (arm ora morto).
- **Census**: psm_* nello stesso ordine delle guardie dell'hit (ramo run.rs 1027-1054 vs 957-968), PSM_OTHER=0, Σpsm=miss; con un solo processo e L-CM1 `epoch≠` è impossibile ⇒ ic_empty = mai riempita in questo Vm, nessun polimorfismo mascherato.
- **Fetta 4**: `class C { public $x=0; public $y=1; }` (wp172-harness/prop-dq.php) è un fatto di sorgente, non del binario census.

## Rilievi
1. **Perimetro «4,2M private = 47 %» è un TETTO**: psr non esclusivi, Σpsr=159 % ⇒ private∩readonly ≥ 44 % del miss. Nel tarball ORM PHPUnit 13 ha 523 `readonly class` e 142 promossi `private readonly` (Doctrine src: 0 readonly class): il «55 % di Doctrine» è in gran parte il sistema di eventi PHPUnit, write-once in costruttore. Un fill privato senza bit RO (NEXT p.1b) può avere perimetro ≈0.
2. **psm_ic_empty è per esecuzione, non per sito**: la lettura p.4 del criterio («ic_empty ⇒ siti freddi») è contraddetta per costruzione (un sito caldo mai riempibile conta milioni di ic_empty); la lettura usata è post hoc, va emendata dichiarando.
3. **cm1 incoerente**: il .out dice «né cifra né direzione», ma WP_SESSION_177 p.5 usa D_flag +3,60/+2,93 come «replica S-176» dalla stessa finestra. Il +60 % è scheduling su E-core (P-core ai Chrome): micro-architettura diversa, non rumore gonfiato — giusto scartare, ma per TUTTI i D.
4. **E1/E2 post hoc**: PREV same-binary nasce in S-176 (az. 5), cm1 era la prima corsa; soglia 4 fissata dopo aver visto 18,47. E2 (<150 %) attende da 02:00 a 526-838 % (lancio-cm1b.log): tetto 90 min ⇒ rc=8 senza misura. Il lanciatore cita `s177-criterio-cm1b.md` (riga 2) che non esiste.
5. **Tree mai collaudato**: 30 siti + test riscritti senza `cargo test` né corpus (CI_FEED potata a 710d823c, runner in attesa del lock s177 ancora vivo dalle 00:57); parità del braccio C = 5 fixture.
6. **Hit con chiave mangled**: `write_property_at` (oop.rs 83-101) nel fallback scrive `name` (la prop pubblica) non la key; `resolve_prop_access` (oop.rs 426-437) dà `slot: None` se obj_class≠scope (figlio con layout spostato). «Impossibile a class_id combaciante» regge solo con vincolo obj_class==scope al fill.

## Azioni S-178
1. Contatori private∩readonly / private∩¬readonly (+ top-10 classi) PRIMA della leva; se PHPUnit domina, dichiararlo nel giudice ORM.
2. Emendare criterio census p.4 (ic_empty per esecuzione) dichiarando.
3. Degradare D_flag in WP_SESSION_177 p.5: nessuna replica da finestra contaminata.
4. Rimuovere il lock s177, leggere batteria+corpus del tree prima di ogni promozione; correggere il riferimento a criterio-cm1b.
5. Leva privata: vincolo obj_class==scope, fallback per key, mutante «slot stantio su figlio».
