# Verbale Leijen (lente: allocatore mimalloc, footprint fisico) — S-116, concilio di rotta

## VERDETTO: CONCORDO CON EMENDAMENTI

A subito sì, B come regime sì, D come selezione sì. MI OPPONGO al solo punto «C riserva»: dalla mia lente C non è una riserva, è aritmeticamente l'unica rotta che chiude, e la sua istruttoria deve partire subito.

## ROTTA DALLA MIA LENTE (3 sessioni)

**A′ (S-117) → B+D (S-118) → istruttoria C in parallelo da S-118.**

L'aritmetica: prop è ~107 ns/iter lato pin contro ~14 dell'oracle; ≤3× vuol dire ~42. L-A vale ~27; A, se rende il 5-15%, altri ~8-11. Restano ~70: ancora ~5×. Il fatto S-103 (costo/op 9-10 ns INVARIANTE tra categorie) dice che il pavimento è il ciclo di vita Zval — nascita/morte/refcount — cioè territorio dell'allocatore, non dell'impaginazione. PGO non toglie un solo incremento di refcount né una alloc: A ripara il METRO (ed è per questo che va fatta subito), non il collo. Chi vende A come rotta di chiusura si illude.

**Mossa concreta S-117**: pipeline PGO rustc (`-Cprofile-generate` sulle sei micro + media WP, merge, `-Cprofile-use` + LTO fat + codegen-units=1), profdata CONGELATO come artefatto versionato; poi RI-MISURARE la banda leve-nulle (nulla-1/nulla-2 ricostruite sotto la nuova pipeline) PRIMA di qualunque verdetto su leve. La banda N=2 attuale muore con la pipeline nuova: non si eredita.

## EMENDAMENTI

**R1 — BOLT non esiste su questa piattaforma.** BOLT è ELF/Linux-centrico; su macOS arm64 Mach-O il layout deterministico si fa con PGO rustc + `ld64 -order_file` (o linker order equivalente). Pre-registrare la toolchain REALE prima di promettere «BOLT»; apparato timebox ½ sessione (REGOLE §1). Misura: la pipeline esiste se produce due build byte-stabili a sorgente invariato.

**R2 — Profilo congelato o A peggiora il metro.** PGO rende il layout FUNZIONE del profilo: se il profdata cambia tra i bracci, ogni A/B confronta impaginazioni diverse e la banda esplode. Regola: stesso profdata pinnato per ENTRAMBI i bracci di ogni A/B; ri-profilare solo a promozione avvenuta, con ri-misura banda. Misura di successo di A come riparazione del metro: banda nulla N=2 sotto PGO+order_file ≤ 5 ns/iter (oggi 10). Sotto quella soglia le leve da 3-30 ns tornano giudicabili.

**R3 — C non è riserva: istruttoria da S-118.** Il censimento alloc/op e refcount-op/op per categoria, su ENTRAMBI i motori (apparato free-hist H-D già esistente, S-103: 1 alloc×32 B/chiamata), si fa dentro la finestra leva di S-118. Decide QUALE variante C: se il pavimento è refcount/drop e non malloc, la variante giusta ELIMINA allocazioni (scalari inline/NaN-box, niente Rc sui scalari), non le sposta.

**R4 — Arena per-richiesta SOPRA mimalloc: presunzione di colpevolezza.** WP-59 ha misurato frag mimalloc 2% al picco: mimalloc non spreca e il suo fast path è già a pochi ns. Un secondo livello (arena) = doppia contabilità + high-water ritenuto fino a request_end ⇒ rischio diretto sul vincolo peak WP 1842 MiB. Gate pre-registrato per OGNI variante C: peak WP ≤ 1842 MiB +2%, pena bocciatura della variante (non della rotta).

**R5 — B pretende la banda dei TRENI.** Le tasse sistematiche si sommano come i guadagni: tre vagoni da −1..−1,5 su calls fanno −4 veri. Prima di giudicare un treno: treno-NULLO (3-5 commit vuoti) per la banda multi-commit sotto la pipeline nuova.

## KILL-SWITCH

- **A**: se dopo PGO+order_file la banda nulla N=2 resta >5 ns O il geomean micro non migliora ≥3% ⇒ A declassata a solo-layout-freeze, si passa a B/D.
- **A-apparato**: toolchain non in piedi in ½ sessione ⇒ ripiego a PGO-solo.
- **C-arena**: peak WP > 1842+2% in qualunque variante arena ⇒ variante morta.
- **C-tutta**: se il censimento R3 mostra <1 alloc/op e refcount <30% del budget ciclo sulle categorie calde ⇒ l'ipotesi Zval-lifecycle è refutata, C si ridisegna prima di spendere sessioni.

## APPARATO minimo
Solo R1 (pipeline PGO/order_file) e ri-misura bande: tutto il resto usa strumenti già esistenti (free-hist, leve nulle, vmmap physical footprint).
