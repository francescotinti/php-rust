# Verbale Sedia 8 — Stogov (Zend/opcache, semantica engine) — Concilio WP-104

## VERDETTO

**APPROVATO CON RISERVA FORTE.** I miei tre emendamenti WP-103 sono stati
eseguiti bene: le fixture 14-16 esistono, con attese scritte prima, byte-id
oracle nei 2 modi (verificato sui file `out/`); il fix §3.13 timbra la riga
all'accodamento (KS-ST-103-3 rispettato: carve-out cancellata nello stesso
commit) e ha perfino morso il corpus in positivo (1418→1417). Ma il claim
«§3.13 FEDELE» è SOVRADIMENSIONATO, e `is_gc_container` porta una
contraddizione doc/codice sulla `Ref`.

## Perimetro

**(a) §3.13.** La marca è fedele alla semantica Zend della RIGA
(`EG(current_execute_data)->opline->lineno` = l'opline della lettura): i 5
siti PropGet timbrano (mod.rs:11691; run.rs:524, 3617, 4698, 5810) e
`flush_diags` preferisce la marca. Però: (1) Zend NON ha una coda — chiama
`zend_error` sincrono all'atto della lettura; phpr conserva coda+flush, quindi
il **TIMING** di un `set_error_handler` che echoa resta al flush, non alla
lettura: in `echo "a", $o->undef, "b";` l'interleaving può divergere. La
marca cura la riga, non il tempo — non fixturato. (2) ~435 siti accodano in
`self.diags` senza timbrare: i builtin (host.rs, 192) flushano alla riga
della call e sono fedeli di fatto; la famiglia **fetch** (array undef key,
string offset, undef var in espressioni multi-riga) = §3.11/§3.12 resta
infedele a riga-del-flush e va CATALOGATA per NOME.

**(b) is_gc_container.** Str/Resource esclusi al secondo livello ✓
(GC_MAY_LEAK, mai nel root buffer — A-ST-103-5 eseguito). Ma il commento
dice «a Ref is unwrapped by the caller» mentre il match ritorna
`Ref(_) => true`: in Zend `gc_check_possible_root` scartoccia GC_REFERENCE e
controlla `Z_COLLECTABLE` del valore INTERNO — una Ref(Long/Str) non si
bufferizza MAI; in phpr, ogni chiamante che non scartoccia la nota = churn
che Zend non ha. **Generator=false è INFEDELE**: in Zend il generator È un
zend_object collectable con `zend_generator_get_gc` — un generator in ciclo
non verrà mai raccolto (dtor/finally mancati). S-103 lo relega a backlog
igiene: inaccettabile senza almeno la fixture che dica se morde.

**(c) Fixture 14-16.** Semantica confermata: 14 garbage-poi-dtor con
rientranza annidata (dtor(A) vede obj(B), dtor(B) vede il write di A) ✓;
15 coercion riuscita/fallita, vecchio valore intatto ✓; 16 CoW separata alla
scrittura in `__clone`, oggetti shallow condivisi ✓. Corni mancanti: `unset`
della stessa prop dentro il dtor (la mia A-ST-103-1 lo nominava); float
frazionario su `int $prop` (deprecation 8.5 + troncamento — incrocia §3.13);
il corno typed-REF di §3.12.

**(d) H-C1c.** L'ordine §S-103 (H-C2, H-D, leva-nulla, igiene) NON apre
H-C1c: KS-ST-103-2 rispettato ✓. H-C2 è ammissibile: KS-ST-103-1 è sanato
dalle fixture 14-18.

## Emendamenti

- **A-ST-104-1**: fixture handler-timing (`set_error_handler` che echoa
  dentro un multi-echo con lettura undef a metà): se l'interleaving diverge
  dall'oracle → catalogare come §3.14 o fissare.
- **A-ST-104-2**: censimento dei percorsi non-timbrati che SOPRAVVIVONO
  alla riga (famiglia fetch §3.11/§3.12): un micro-test per famiglia,
  riga oracle vs phpr, catalogo per NOME.
- **A-ST-104-3**: fixture 19 generator-in-cycle + `gc_collect_cycles()`:
  se Generator=false morde → fix o riga in PHPR_DIVERGENCES; il buco non
  resta senza dato.
- **A-ST-104-4**: sanare Ref in `is_gc_container`: o il predicato scartoccia
  (come `gc_check_possible_root`) o l'assert A-MA-103-2 diventa NON
  opzionale nel punto 5 S-103.
- **A-ST-104-5**: fixture 15-bis (float frazionario su typed int + corno
  typed-REF §3.12, attese dall'oracle).

## Kill-switch

- **KS-ST-104-1**: H-C1c resta GATED (KS-ST-103-2 invariato).
- **KS-ST-104-2**: nessuna nuova leva sul canale gc_note si promuove finché
  A-ST-104-4 non è sanato o assertito.
- **KS-ST-104-3**: «§3.13 CHIUSA» vale SOLO per la famiglia PropGet
  timbrata; vietato estendere «FEDELE» ad altri produttori di diag senza il
  censimento A-ST-104-2 (un verdetto vale solo sul giudice che l'ha prodotto).

## Refutazioni capitali

**NO.** Nessuna misura promossa cade. Riserva forte, non capitale: il fix
§3.13 è line-faithful su 5 siti, non mechanism-faithful (Zend non ha coda);
e la contraddizione Ref è un difetto latente di churn, non di correttezza
osservata.
