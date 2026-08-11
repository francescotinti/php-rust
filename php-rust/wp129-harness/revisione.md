# Revisione S-129 — lente SEMANTICA

## Reperto principale
**«Modello del tempo CHIUSO» è chiuso come CONTABILITÀ, non come identificazione: il segmento dominante E1 (155 ns, 52%) non è mai stato sondato.** Le sonde misurano solo TOT, A, B, C, D, E, E2 (s129-tempo-passi-verdetto.out:14-20): E1 = E−E2 (202,6−47,8), un **residuo per sottrazione** la cui etichetta «resolve-per-NOME» viene dal solo disasm statico (conteggio bl), non da una sonda. La «chiusura 96%» (riga 21) valida la somma, non l'attribuzione. Il modello stesso ammette «quota obbligata ignota» (s129-modello-tempo.md:25-27), ma il claim di testa e la roadmap promuovono «resolve-per-NOME 155 (52%)» a cifra: la leva regina di S-130 (resolve-once) eredita così un UB che include dispatch, key-coercion e traffico borrow non toccati dal caching. Nota asimmetrica: B+C+D, gli unici segmenti sondati direttamente, sono stati corroborati dall'A/B F4 (D=+66,7/+71,7 vs UB 73) — il modello trasferisce dove è misurato, non dove è dedotto.

## Reperti secondari
1. **Gate ictx/s (claim 4): soglia semanticamente incoerente.** La mediana 908 è POOLED sui due motori (s129-pair-legoff-verdetto.out:4-9): l'oracle vive a 1271-2624, phpr a 189-545 ⇒ una gamba phpr contesa non può MAI essere segnalata (servirebbe 6× il suo regime; leg1-off phpr=545 = 2,6× tipico passa il gate da sola). Inoltre le due escluse hanno oracle-CPU più BASSA (423,6/408,8 vs 441-445, righe 11-15): il meccanismo «contesa⇒misura corrotta» predice CPU gonfiata, non sgonfiata — l'esclusione delle due peggiori (1,909/1,947) resta correlazionale. Infine 1,758 è un accoppiamento INCROCIATO (riga 17-19); le coppie proprie sono 1,765-1,805, e «full» usa user+sys contro il canone user-only (riga 3).
2. **Lettura A/B F4:** «il gate vi costa un solo matches! fallito» (s129-ab-f4-lettura.md:20) è impreciso: il loop objmap non contiene FieldAssign (wp127-harness/micro-orm/objmap.php), quindi il gate non esegue affatto nel loop — costo zero, refuso innocuo che però corrobora la diagnosi layout.

## Vagliate e respinte
- **F4 non è no-op:** respinta. `has_prop_hooks` è flattened parent-inclusive (bytecode.rs:2023, class.rs:467) e `prop_hook` vi si corto-circuita (mod.rs:13975); `indirect_hook_target` guarda solo il primo passo sulla classe base (mod.rs:14113); il fast-path di `lazy_prop_access` (mod.rs:12931-12933) coincide col gate; Ref/PropDyn/lazy/proxy cadono in via lenta (f4143a6).
- **2 alloc ≠ cloni:** respinta — mod.rs:14334 e 12877 (il clone a 12900 corre solo su lazy); census 22/22.
- **PropSet mono-passo:** respinta — expr.rs:2524-2559 copre This E Local; caveat innocuo: `$GLOBALS['x']->p` resta FieldAssign.

## Azioni S-130
1. Aggiungere allo script s129-tempo-passi.sh un segmento E1a che sonda le sole chiamate `resolve_prop_access` dentro `field_write_prop_step`: UB di resolve-once misurato, non residuale.
2. Rinominare nei file di rotazione E1 in «E−E2 (dispatch+prop_step)» finché E1a non esiste.
3. Rientro F4: committare PRIMA formula di rumore trimmed drop-1 simmetrica e bande objmap/objalloc/objchurn da R≥5 sul pin.
4. Riderivare (dai .time esistenti, zero run nuove) il gate ictx/s con mediana PER MOTORE; pubblicare intervallo coppie proprie accanto alla matrice e ratio full anche user-only.
