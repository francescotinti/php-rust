# s158-criterio-refl2 — leva «L-RF2: tranche-2 slice __reflect_*» (fetta ALLOC canale H-D, estensione HD2-hostcall S-156; PRE-REGISTRATO prima di edit/misura)

1. **Edit** (mod.rs tabella `host_builtins!` + host_reflect.rs firme): i SEI nomi
   `__reflect_method_info` · `__reflect_prop_details` · `__reflect_prop_attr_new`
   · `__reflect_class_real_name` · `__reflect_method_names` · `__reflect_class_loc`
   passano dalla sezione `vec:` alla sezione `slice:` (stessa tabella, nessuna
   doppia lista); firme `Vec<Zval>` → `&[Zval]` e `&args`→`args` verso gli
   helper GIÀ a `&[Zval]` (resolve_named_class_with_autoload, named_trait,
   trait_loc, prop_attr_thunk). Corpi verificati SOLO-LETTURA (first/get,
   nessun consumo), UN chiamante ciascuno (la tabella `vec:`), arità ≤3.
   Semantica INVARIATA (stessi valori/ordine, contratto visibilità storico HD2).
2. **Attesa fondata** (denominatore: census s156 dà ALLOC, non chiamate —
   famiglia 11,66M alloc/run ORM: method_info 3,37M · prop_details 2,15M ·
   prop_attr_new 2,05M · class_real_name 1,97M · method_names 1,20M ·
   class_loc 0,93M): a scala ORM ~7 ns × N_chiamate < 0,293 s ⇒ leva
   **MICRO-JUDGED** (veto S-155); prezzo HD2 ~7 ns/chiamata (S-125) ×
   2 chiamate/iter del driver ⇒ attesa ≈ **+14 ns/iter**, segno POSITIVO.
3. **Giudice**: `m-refl.php` (wp158-harness, NUOVO dichiarato), N=10.000.000
   letterale, 2 chiamate convertite/iter (method_info cache-hit +
   class_real_name su classe esistente, niente autoload). Parità: RF-OK.
4. **Soglia**: max(4 ns/iter; rumore drop-1). **UB-alloc falsificabile**:
   2 × miheap 6,9 = **13,8 ns/iter**; D > 13,8+rumore = canale non-alloc —
   a verbale con sonda dovuta (non blocca il verdetto).
5. **R**: smoke R=2 early-stop a segno opposto → R=5 ABAB. **RAMO «MORSO ALLO
   SMOKE» PRE-REGISTRATO (az.rev. S-157 #5)**: smoke rc∈{2,5,9} ⇒ STOP; per
   rc=5 l'UNICA prosecuzione ammessa è l'arbitrato DEDICATO del morso
   (ri-risoluzione a N con tick ≤ soglia/4, copia-gate PRIMA del run) PRIMA
   del R=5; altrimenti la leva si ferma lì e si dichiara.
6. **Bracci**: A = GEMELLO ricostruito dal tree s157 (ricetta canonica);
   identità: byte == pin 76787303716acd4e a cache calda; a freddo arbitrato a
   CONTENUTO con REGIONI PRE-REGISTRATE (az.rev. S-157 #3): ammessi SOLO i
   cluster LC_UUID (16 B) + stringa data build/banner mimalloc (≤32 B) +
   firma code-sign (2×32 B); QUALUNQUE byte fuori regione ⇒ STOP (verbale
   s158-gemelloA-identita.out). B = tree s157 + SOLI edit p.1, hash dichiarato
   al run. Stash SOLO via `pin-phpr.sh --braccio`. Header dei verdetti con
   hash MISURATI dei bracci, mai stringhe fisse (az.rev. S-157 #3).
7. **Guardie SOLO-REGRESSIONE** (comparatore STRETTO: morde sse D < −soglia):
   missload (m-missload N=10M, presidio L-AL1) + hostargs (N=10M, tranche-1)
   + backtrace24 (N=2,4M) + obj* (bande fondate 13,3/6,7/10,0/3,3) + le sei
   (SL storiche). Giudici/guardie = BYTE-COPIE dichiarate da wp157-harness.
8. **Disasm DOVUTO** (protocollo S-104): bl-count di `run_loop` A vs B
   registrato PRIMA del giudizio; delta a verbale (reperto, non gate).
9. **Parità/fedeltà**: m-refl stampa RF-OK identico A==B pena STOP;
   `fx-refl.php` A==B BYTE-ID con **RUOLI DISTINTI per ogni nome a 2+ arg
   (az.rev. S-156 #4, dovuta qui)**: method_info/prop_details/prop_attr_new
   anche a argomenti SCAMBIATI (esiti visibilmente diversi dal caso dritto);
   NESSUNA gamba oracle (nomi interni phpr, fuori dal perimetro PHP: il
   contratto è A==B). Dente loc: salita mod.rs PRE-DICHIARATA (sei righe
   spostate + commento tranche-2: 25778 → 25779 atteso, conteggio esatto in
   promo); churn host_reflect.rs dichiarato (solo firme).
10. **Igiene**: lock di sessione presente, quiescenza rc=0, attesi smoke BLIND
    (`s158-smoke-atteso-refl2.md`) verificati da SECONDO ATTORE prima del run,
    rc autoritativi da file, sentinella language-server nel verdetto; misure
    SOLO a coppia t8+ORM concluse (run pesanti sequenziali).
