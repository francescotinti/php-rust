# s148-anatomia-hostcall — naming statico della testa del ranking (grado INDIZIO: lettura sorgente, NESSUNA misura nuova; cifre dal verdetto s148-attrib-verdetto.out)

1. Densità: hostcall.other 165,6M su ~24,8M dispatch builtin (21,6M
   CallHostBuiltin da digramma s147 + 3,2M CallBuiltin) ≈ **6,7 alloc
   non-attribuite per chiamata** ⇒ il grosso vive nei CORPI dei builtin
   (temp String/Vec/collect), non nel solo plumbing.
2. Plumbing NOMINATO: `pop_keys` = `Vec::split_off` ⇒ **1 Vec allocata a ogni
   CallHostBuiltin** (run.rs:3650; upper bound ≈21,6M/run). L-HD2 forma-2
   (S-125) eliminò la coppia malloc+free SOLO su CallBuiltin ad arità ≤4 —
   CallHostBuiltin è rimasto al split_off.
3. Stessa firma su 10 siti dei cammini di chiamata UTENTE (run.rs 3725, 3766,
   5214, 5286, 5321, 5442, 5507, 5521, 5541, 5589): quei Vec cadono nel tag
   `none` ⇒ quota candidata di none.other 94,6M — **l'indiziato «args-Vec»
   della free-hist S-104 ora ha canale e ordine di grandezza**.
4. Shape (hist sul tag INTERO, non other-only, dichiarato): hostcall dominata
   da ≤48 B (107,9M) e ≤16 B (98,8M) — coerente con Vec argomenti a 2–3 slot
   (32–48 B) e box/temp da 16 B.
5. Prossimo atto (proposta per l'ordine S-149; nessuna leva ora, criterio
   p.8): (i) **tranche-4 = census per-NOME-builtin** (tag dinamico sul nome nel
   dispatch, apparato s148 riusabile) per nominare le teste DENTRO
   hostcall.other; (ii) **sonda-prezzo pair COLLAUDATA** sul pattern reale
   (churn 16–48 B) per la conversione conteggi→secondi (il prezzo zcell/arr0
   resta INDIZIO); (iii) leva (es. pop-diretti su CallHostBuiltin, forma-2
   estesa) SOLO dopo (i)+(ii).
