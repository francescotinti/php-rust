# Team coda-stack (Bak 5 · Stogov 8) — Concilio WP-101

Tema: coda H-B2 sul percorso pila (Sub/Mul/cmp int-int); criteri, perimetri, ordine S-100.

## Convergenze

1. **Criterio 0,7 ns non aggiudicabile — le due sedie chiudono la stessa porta da due lati.**
   Bak (R2, KS-BA-101-1): un criterio SOTTO il pavimento dichiarato della sonda (1,0 ns)
   non può né mordere né assolvere; ogni sonda futura pubblica BANDA + pavimento, e se
   criterio < pavimento la sonda è VOID e il criterio si rialza al pavimento — quindi
   D_registro ≥ 1,0 ns/occ finché la sonda non migliora. Stogov (KS-ST-101-3) tiene chiuso
   il rollout registro salvo MISURA che superi il criterio: nessun argomento statico lo
   riapre. Regola composta: soglia = max(0,7; pavimento sonda), scritta nel .out.
2. **La decomposizione va pubblicata come BANDA hit-only, non come quanto fisico.**
   Bak (R1, A-BA-101-1): 57/43 è uno split dipendente dall'ordine di rimozione; banda
   call/marshalling 52–62% col termine d'interazione NOMINATO; INT2 solo se decisionale.
   Stogov ri-perimetra il controfattuale (A-ST-101-1): banda valida solo guard-always-hit,
   rimovibile per-FORMA (BinarySC/SCDst pagano `consts[cidx].to_zval()` sul hit — non ~0
   per ZStr, refcount bump), conclusione D≈0 ristretta ad Add int-int. Le due restrizioni
   si sommano: «banda 52–62%, hit-only, per-forma» è l'enunciato da mettere in NEXT.
3. **Census delle frequenze = precondizione della scelta dell'occorrenza.** Bak
   (R5, A-BA-101-3, KS-BA-101-2): census sull'emissione POST-FLIP (frequenze su emissione
   in pensione = void), forme fuse CmpJmp/CmpJmpConst (run.rs:982-1004) contate A PARTE
   — sono il consumatore più caldo del plumbing; atteso scalato per frequenza PRIMA di
   scegliere; se atteso < risoluzione della coppia, l'occorrenza cade a tavolino.
4. **cmp non è fungibile a Sub.** Stogov R2/A-ST-101-2 (perimetro: __toString/ordine dei
   side effect, smart_streq «10»=="1e1", NaN/-0.0 nelle forme swapped, segno di Spaceship
   sotto swap, const-lhs; dichiarato PRIMA del controfattuale + una fixture per classe,
   pena VOID per KS-ST-101-2) e Bak R5b (le forme fuse vanno censite come classe propria)
   sono lo stesso vincolo visto da semantica e da microarchitettura.

## Conflitti

Nessun conflitto sostanziale. Una tensione d'ordine: per Bak il primo atto empirico di
S-100 include il profilo a campioni co-equale (A-BA-101-2); per Stogov nulla si promuove
finché le sette trappole AssignOp (A-ST-99-3) non sono gate PER NOME (R1, A-ST-101-3,
KS-ST-101-1: flip VIETATO senza fixture byte-identiche nei due modi). Compatibili: gate
prima del flip, profilo dentro la prima misura H-C.

## Priorità per l'ordine S-100

1. Sette trappole AssignOp PER NOME nel punto 1 come gate della promozione (bloccante,
   non backlog) — KS-ST-101-1.
2. Criterio D_registro rialzato al pavimento della sonda; banda+pavimento obbligatori
   (KS-BA-101-1) prima di ogni riapertura registro.
3. Census post-flip per-op con CmpJmp* a parte; atteso×frequenza decide l'occorrenza,
   soglia di caduta a tavolino (KS-BA-101-2).
4. Se il census indica cmp: perimetro compare SCRITTO prima del controfattuale + fixture
   per classe (KS-ST-101-2).
5. Ri-enunciato in NEXT: banda 52–62% hit-only per-forma; profilo a campioni co-equale
   alla tavola conteggio×costo nella prima misura H-C (A-BA-101-2); enumerazione classi
   flag-on (A-ST-101-4).
