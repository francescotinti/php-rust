# s144-criterio-profilo-oracle — criterio proprio della voce b (istruttoria S-143; firmato PRIMA della run)

1. Oggetto: profilo campionario per FAMIGLIE della SUITE ORM sul PHP ORACLE
   8.5.7 (brew, `/opt/homebrew/opt/php/bin/php`, `memory_limit=-1` §3.14) —
   la lente SPECCHIO di s140-criterio-profilo (stesso strumento `sample`,
   stessa aggregazione top-of-stack, 2 repliche). Scopo: budget = phpr−oracle
   per canale (Bak R2, Stogov R4, Gregg K3); la promozione della prima fetta
   B ASPETTA questo verdetto.
2. FAMIGLIE oracle (lista CHIUSA, primo match vince — mappa dichiarata):
   vm_inline=`execute_ex|ZEND_.*_SPEC|zend_execute` ·
   churn_zval=`zval_ptr_dtor|rc_dtor|zend_assign_to_variable|zval_copy` ·
   gc=`zend_gc|gc_|collect_cycles` · alloc=`_emalloc|_efree|zend_mm|malloc|\bfree\b` ·
   map=`zend_hash` · prop_dim=`_property|zend_fetch_dimension|zend_std_|obj_prop` ·
   calls=`zend_call|execute_internal|zend_vm_stack|init_fcall|leave_helper` ·
   memops=`memcpy|memmove|memset|memcmp|platform_mem` · str=`zend_string|smart_str|concat` ·
   compile=`zend_compile|zend_ast|zendparse|lex_scan|opcache` · refl=`reflection` · resto=other.
3. LIMITE DICHIARATO (non emendabile a valle): Zend inlina churn/alloc dentro
   gli handler (`execute_ex` cumula ciò che in phpr è simbolo separato) ⇒ il
   confronto per famiglia è ASIMMETRICO per inlining; grade INDIZIO
   bilaterale — decide DIREZIONI e ordini di grandezza, MAI cifre di tempo,
   MAI sottrazioni puntuali phpr−oracle in secondi.
4. Finestre: la run oracle dura ~ una manciata di secondi ⇒ UNA finestra
   `sample PID` per replica avviata subito (sleep 0,5), che copre la vita
   utile del processo (il pavimento di avvio phpunit resta dentro, DICHIARATO
   — stesso trattamento del profilo phpr S-140).
5. Gate/igiene: sentinelle rustc/cargo/rust-analyzer pre/post per finestra
   (stampate); lock misura PRESENTE (verify-only); summary phpunit e
   fail-names REGISTRATI (prima fotografia oracle: nessuna baseline congelata
   esiste ⇒ osservativo, non gate).
6. Esiti pre-registrati: (i) K3-Gregg/Klabnik: una famiglia bersaglio di B
   (churn_zval+memops) in quota oracle ≥50% della quota phpr omologa ⇒ il
   canale esce dal budget e la fetta B che lo bersaglia NON si promuove;
   (ii) quota oracle <50% della phpr ⇒ canale IN budget, differenziale
   firmato in DIREZIONE; (iii) <100 campioni totali in una replica ⇒
   replica NON valida, si ripete con finestra doppia.
7. Confronto: le quote phpr di riferimento sono quelle del verdetto
   s140-profilo (stessa lente, stesso strumento); il paragone è
   quota-vs-quota (percentuali dello stesso denominatore per-binario), mai
   tempo-vs-tempo.
