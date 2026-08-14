# S-136 — istruttoria leva «dim-write su proprietà» (scritta PRIMA di ogni misura/codice)

## Reperto d'ingresso (fonti agli atti)
- Sonda S-135 (`s135-sonda-verdetto.out`): objdatains su s134 = **2 resolve/iter
  RESIDUE** (i 2 set del ctor sono NPhit; le resolve restano sul dim-write
  `$e->data['k']=$i`), fuori perimetro sia della IC non-plain (S-134) sia di AP1 (S-135).
- Lowering VERIFICATO sul pin s135 (dump `PHPR_DUMP_OPS`, probe dimprop.php):
  `$e->data['k']=1` → **`Op::FieldAssign { base: Local, steps: [Prop("data"), Index] }`**
  — famiglia Field* del modello S-129 (statement ≈300–340 ns; E−E2 dispatch+prop_step
  ~155 = 52%), modello prop_step S-131 (interno 130,7 · dispatch 36,3).
- Direzione (sottrazione stesso-binario, SOLO indizio): objdatains net 1060,0 −
  objalloc 820,0 ≈ **240 ns/iter** per il dim-write; oracle ≈ 163,1 − 126,2 ≈ 37.

## Ordine post-finestra (la coppia WP ha la precedenza sulla finestra)
1. **Modello del tempo FieldAssign** (metodo = s135-tempo, build emendata MAI
   pinnabile, probe a segmenti su bench m-dimwrite dedicato): arm totale ·
   resolve (i 2 siti, PER NOME dal codice) · prop_step/guardie · walk/Index ·
   set+gc_note. Chiusura ≥90% o modello INCOMPLETO dichiarato.
2. **Criterio leva** con UB FALSIFICABILE = somma dei prezzi MISURATI dei soli
   canali che la leva rimuove (modello AP1 = precedente) + banda giudice;
   guardie SOLO-REGRESSIONE a formula del giudice (max(4, banda fondata,
   rumore drop-1 del run per categoria)).
3. **Forma candidata** (da confermare col codice, Serena POST-finestra): cella
   IC sul passo `Prop` di FieldAssign (pattern NP/TY di S-134, fatti di classe
   provati al fill) o hoist resolve-once (pattern S-131/S-133) se i 2 siti
   condividono lo stesso nome. Perimetro: SOLO passo Prop con fatti provati;
   hooked/readonly/`__set`/mangled restano al cammino pieno.
4. **Giudice**: objdatains (N=3e6 dal sorgente), R=5 ABAB vs stash phpr-s135;
   guardie objalloc/objmap/objchurn/allocni. Fedeltà: fixture dim-write su prop
   (sezioni nuove, bilaterali) PRIMA dell'A/B.

## Vincoli di finestra (a verbale)
- Coppia WP in volo (6 gambe): NESSUN build/misura/LSP fino al `pair136-t1.done`.
- Incidente dichiarato: rust-analyzer acceso ~40 s in finestra (attivazione
  Serena, mio errore) e ucciso 2 volte (02:54–02:55, dentro gamba off-1, nessun
  gate in corso); da pesare sulla firma della gamba off-1 nel verdetto.
- Probe leggere eseguite in finestra (dichiarate): 3× dump compile-only
  `PHPR_DUMP_OPS` (~50 ms l'una, 02:57–03:02, lontano dai gate).
