# s126-leva-nominata.md — L-OL1 «ciclo-di-vita oggetto» (da istruttoria ORM, criterio p.6+p.8)

**Fatto** (verdetti s126-orm-micro{,2}-verdetto.out, pin s125): il churn oggetti è 10,3× e il suo
segmento dominante è **objalloc = new+ctor+drop: 1220 ns/oggetto phpr vs 123,3 oracle (9,9×, ~67%
del churn)**; objmap (insert/overwrite identity-map) 17,3× ma pesa ~10%. Nel profilo ORM la
macchineria è visibile multi-%: Zval clone/drop · PropsLayout::slot_of · gc_note_slow/sweep/
collect_cycles · mi_malloc/free. Coerente con ORM 8,5× (UoW/mock = churn di oggetti).

**Leva L-OL1** (esecuzione S-127): ridurre il costo di alloc+init+drop per oggetto.
Istruttoria d'ammissione PRIMA della forma: census ±zval e conteggio malloc/free per `new En` (quante
allocazioni per oggetto: header, Vec props, default per-prop, registrazione GC?) + disasm del sentiero
new/ctor/drop (bl-count prima/dopo, protocollo S-104). Forme candidate DA CENSIRE, non predette:
meno allocazioni per oggetto (layout inline dei default) · gc_note fuori dal sentiero caldo · drop
non ricorsivo per oggetti senza cicli.

**Criterio A/B pre-registrato per la forma** (da ri-committare con la forma scelta):
- Giudice: `wp126-harness/micro-orm/objalloc.php` (N=3.000.000 emesso dal sorgente), R=5 ABAB,
  user CPU netto-pavimento per-binario.
- Segno: phpr objalloc_ns_iter DIMINUISCE. Soglia: max(4 ns/iter, rumore R=5, banda-layout
  max(SL s123, SL s125) della categoria più affine = prop). Smoke R=2 early-stop a segno opposto,
  lettore proprio (MINFAM rispettato: famiglia=1 categoria).
- Guardie NON-bersaglio a SOLO-REGRESSIONE: le sei micro storiche + objchurn/objmap.
- Gate promozione: batteria · corpus congelato per NOME ×2 modi · fixture bilaterali · ORM 16 nomi
  == baseline · hk 0E/0F · pin via scripts/pin-phpr.sh (collaudo-nell'atto).

**Aperture per NOME registrate** (non nominate, cliff reali ma peso ORM ≤~1% leaf):
- `evalcls` **316,9×** (2,38 ms/classe eval'd vs 7,5 µs oracle — sentiero compile-per-classe; prima
  di ogni leva serve lo strumento di DENSITÀ: quante classi eval'd genera una suite reale).
- `refl` **42,4×** (32,9 µs vs 0,78 µs per giro ReflectionClass+getMethods+getProperties+invoke).
