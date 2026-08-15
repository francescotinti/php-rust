# s144-criterio-tranche2 — REGOLA PRE-REGISTRATA (firmata PRIMA di leggere i dati; az.rev. S-143 #1/#3/#4/#5)

1. Oggetto: tranche-2 del census CH_* su SUITE ORM — attribuzione di `rczval`
   (box `Rc<RefCell<Zval>>`) e `vecargs` dentro other 61,7%; pubblica
   `quota_obj_max`. Monobinario census (grade CENSUS, MAI tempo), ×2 repliche,
   parità per NOME vs baseline16, sentinelle pgrep pre/post stampate (non-gate).
2. Funnel (siti per NOME, enumerazione CHIUSA DAL COMPILATORE — il parametro
   `Zval` di `zcell` rifiuta Object/Resource/GenState):
   - `zcell` = ogni nascita `Rc<RefCell<Zval>>` (41 siti; ex ricerca esaustiva
     `Rc::new(RefCell::new` su crates/, residui NON-Zval dichiarati:
     Object ×8 · Resource ×9 · GenState ×1 · test ×2).
   - `zcell_prop` = SOLI siti inequivocabilmente proprietà: `prop_ref_cell`
     (oop.rs) · makeref-magic ×2 (run.rs) · radici lazy field_set ×4 (run.rs).
     DICHIARATO: le promozioni `&$o->prop` passano da `make_cell` (misto
     prop/dim) e stanno nel TOTALE, non nel flag ⇒ strict è un MINORANTE del
     vero prop-share; loose (totale rczval) è il MAGGIORANTE.
   - `vecargs` = Vec argomenti con `capacity>0` ai funnel `bind_params` ∪
     `decay_args` (disgiunti); path diretto stack→slot non alloca (fuori per
     costruzione); altri path nativi FUORI perimetro, dichiarato.
   - `objsynth` = tick del box sintetico `copy_with_id(0)` (az.rev. #5).
3. Convenzione CONTEGGI (az.rev. #3, specchio della voce-bytes): le quote sono
   su `galloc_n` della stessa run; i realloc-EVENTI (`s144.grealloc_n`,
   ~19,5M S-141) sono FUORI dal denominatore per costruzione (A-LE-104-1) e
   stampati accanto. Banda dyn_entries: il propsbuf conta slots+dyn UNA volta
   ⇒ quota_obj ha banda +0/-0,69pp (dichiarata NEL verdetto, non solo nel
   sorgente).
4. Numeri emessi: quota_rczval · quota_rczval_prop · quota_vecargs ·
   **quota_obj_max_strict = quota_obj + rczval_prop/g** ·
   **quota_obj_max_loose = quota_obj + rczval/g** · quota_attribuita
   (obj+arr+str+rczval+vecargs) e residuo-other.
5. REGOLA DI DECISIONE (arbitra resta la regola S-143 p.3, applicata al
   MAGGIORANTE): loose <25% ⇒ **B sola / B-poi-A CONFERMATA PER MISURA**
   (numeratore chiuso, revisione S-143 az.1 sanata) · strict ≥40% ⇒ A-poi-B ·
   strict in 25–40% ⇒ RICONVOCA · loose ≥25% con strict <25% ⇒ RICONVOCA con
   tranche-3 (flag di contesto su make_cell) — nessun altro esito ammesso.
6. Esiti pre-registrati: (i) probe MUTO sulle chiavi s144.* allo smoke ⇒ STOP
   rc=8, niente run; (ii) r1≠r2 oltre 1% su una chiave ⇒ dichiara e replica;
   (iii) parità per NOME diversa da baseline ⇒ verdetto NON valido per la
   regola (solo osservativo).
7. Parser ESTERNO `s144-census-parse.py`, collaudato dal golden-test
   (fixture exit+exit_mi, az.rev. #4) COMMITTATO PRIMA della run; match del
   tag ESATTO (lezione S-143).
8. Strumentazione cfg-gated (`mem-census`) su HEAD post-pin; il pin resta lo
   stash s142 (phpr-s142 bba8a734); a fine tranche il binario di parità in
   `release/` si RIPRISTINA dallo stash e si ri-hasha.
