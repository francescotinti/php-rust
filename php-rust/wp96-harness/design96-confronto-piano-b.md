# design96 — Il confronto esplicito col piano B (A-TH-97-3), al netto del corpo caldo

**Che cos'è questo documento**: il passo 2 dell'ordine vincolante del Concilio
WP-97. La regola a tre bande di `design95-liveness.md` §P1 impone, in banda
MEDIA, un **confronto esplicito fra i due piani a parità di conto**. Il
riconteggio di S-96.0 mette il perimetro fedele (nucleo stringhe) esattamente
in quella banda, quindi il confronto non è facoltativo: è la condizione che
apre o chiude F3.

Le cifre vengono da `wp96-harness/zvalcensus-recount.out` (autorità macchina).

## 1. Che cosa dice il riconteggio

Il fix di soundness A-TH-97-1 **non cambia nessun conteggio F1** su questo
corpus (`delta_would_take_vs_f2` e `delta_sites_movable_vs_f2` nel raw). Il
difetto era reale — la fixture `fixtures/t4-first-op-def.php` lo fa mordere a
macchina fra binario pre-fix e post-fix — ma la forma che lo espone non ricorre
nel media group di WordPress. Le bande di S-95.0 erano dunque **corrette per
fortuna del corpus, non per correttezza dell'analisi**, e la distinzione va
tenuta: un corpus diverso non deve la stessa fortuna a nessuno.

P2 resta soddisfatta, quindi **KS-TH-97-3 non scatta**: non si arriva al
confronto per caduta di P2, ci si arriva per la banda del perimetro FEDELE.

Il perimetro fedele è il nucleo stringhe, non il perimetro F2 intero: Stogov ha
refutato quest'ultimo (in Zend i CV non si consumano mai, e la morte anticipata
è osservabile anche senza `__destruct` — `spl_object_id`, `WeakReference`,
chiusura di risorse). Il nucleo stringhe sta in **banda MEDIA**
(`guadagno_cpu_atteso_str_pct_min/_max`), e la banda ALTA del perimetro intero
**non è disponibile a un F3 fedele**.

## 2. Il fatto nuovo: il piano B del confronto NON ESISTE nella forma assunta

`design95-liveness.md` apre dicendo che «la superistruzione `LoadSlot+Binary`
resta PIANO B, documentata in `design95-leva-zval.md` §Correzione».

**Quel riferimento è pendente.** In `design95-leva-zval.md` non c'è nessuna
sezione «Correzione», e la parola «superistruzione» non compare in nessun altro
documento del repo: esiste solo nella frase che la cita. Il piano B che sta
davvero su disco è **A-ZV1**, e A-ZV1 non è una superistruzione:

> «Fast path per riferimento in `binary_value_ab`. […] Esporre
> `binary_value_ref(b, &Zval, &Zval) -> Option<Zval>` e chiamarlo al sito caldo
> PRIMA di materializzare gli operandi; si clona solo se ritorna `None`
> (percorso generico invariato).»

Cioè: **un fast path DENTRO un braccio che già esiste**, non un braccio nuovo.

## 3. Perché questo ribalta la regola di spareggio di §P1

La banda MEDIA di §P1 è scritta così:

> «si confrontano i due piani a parità di conto (il piano B ha guadagno simile
> ma rischio medio, **perché aggiunge un corpo caldo**): a parità di guadagno si
> preferisce la strada lunga, **che non aggiunge opcode** e resta riusabile per
> tutti i siti di lettura»

Entrambe le premesse dello spareggio sono oggi refutate, e in direzioni opposte:

| premessa di §P1 | stato | fonte |
|---|---|---|
| «il piano B aggiunge un corpo caldo» | **FALSA** per il piano B che esiste: A-ZV1 è un fast path dentro il braccio `Binary` | `design95-leva-zval.md` §La leva, punto 1 |
| «la strada lunga non aggiunge opcode» | **FALSA**: `TakeSlot` è un braccio nuovo e paga il tetto WP-39..44 | Bak, A-LB-97-1 (Concilio WP-97) |

Lo spareggio di §P1, applicato agli artefatti che esistono davvero, **punta
dalla parte opposta a quella in cui è scritto**. Non è un dettaglio di
redazione: è la regola che avrebbe deciso F3.

## 4. Il conto, al netto del corpo caldo

**Strada lunga, perimetro fedele (nucleo stringhe)** — guadagno LORDO
`guadagno_cpu_atteso_str_pct_min/_max` del raw. Da questo va sottratto il costo
del braccio nuovo. Il costo storico, misurato in casa:

- WP-33: **un branch mai preso in testa al `run_loop`** è costato una
  regressione di parecchi punti percentuali (la cifra sta in
  `sessions/WP_SESSION_41.md`, citata lì e in WP-44);
- WP-39..44: il canale registri è stato **bocciato** proprio così — «da 2 corpi
  caldi del canale si passa a 4 o 9 e il working-set I-cache/BTB cresce
  comunque; l'elisione dei `LoadVar` non lo ripaga»;
- Bak, consulenza S-95.0, voce O3 (superistruzioni): guadagno atteso dello
  stesso ordine di questa leva, ma «**aggiunge corpi caldi, il modo esatto in
  cui è fallito WP-39..44**. Ammissibile solo DOPO O1 e con tetto: un corpo
  outlineato per ogni corpo aggiunto».

Il guadagno lordo del nucleo stringhe è **dello stesso ordine di grandezza del
costo storico di un corpo caldo in più**. Non si può quindi affermare che il
netto sia positivo, e nemmeno che sia negativo: si può affermare che **il netto
non è distinguibile da zero con quello che sappiamo oggi**, e che l'unico modo
di saperlo è misurarlo — cioè costruire la leva prima di sapere se conviene
costruirla.

**Piano B (A-ZV1, quello che esiste)** — nessun braccio nuovo, quindi nessun
tetto WP-39..44 da pagare. La sua predizione ex ante è nel suo design
(`design95-leva-zval.md` §PREDIZIONE, P2). **Attenzione al grado**: quella è una
PREDIZIONE, non una misura; confrontarla con una banda derivata da conteggi
esatti non è un confronto alla pari, ed è scorretto trattare i due numeri come
omogenei. Ciò che il confronto stabilisce con solidità non è «B vale più di
A», ma questo:

> a parità di canale aggredito, il piano B **non paga il pedaggio** che la
> strada lunga paga, e il pedaggio della strada lunga è dell'ordine del suo
> guadagno.

## 5. Verdetto

**La strada lunga NON vince il confronto sul perimetro fedele.** Per l'ordine
del Concilio WP-97 — «solo se la strada lunga vince il confronto» si passa a
`TakeSlot` — il passo 3 **non si apre**. `TakeSlot` non va scritto in S-96.0.

Questo non archivia A-ZV2. Archivia **una scelta di perimetro**: la strada lunga
sul solo nucleo stringhe, con un braccio nuovo, non ha un netto difendibile.
Restano aperte, per NOME:

1. **La forma dell'emissione non è decisa, e cambia il conto.** Hejlsberg
   (RC-1): «o corpo handler nuovo (WP-43: il costo è il NUMERO di corpi caldi) o
   branch in un arm esistente (WP-38)». Le due forme hanno pedaggi diversi: un
   branch in testa al `run_loop` si paga a OGNI opcode, un branch dentro il
   braccio `LoadSlot`/`LoadVar` si paga solo sulle letture di slot ed è
   per-sito, quindi ben predetto dal BTB. **Un `LoadSlot` che porta un flag
   `take` deciso a compilazione non è un corpo caldo in più.** Questa forma non
   è stata valutata da nessuno e potrebbe cambiare il verdetto: va istruita con
   la taglia `nm -S` PREDETTA prima, come chiede A-LB-97-1.
2. **O1 di Bak (outlining dei bracci freddi) è il prerequisito dichiarato.** È
   l'unica leva che *abbassa* il numero di corpi caldi, ed è ciò che pagherebbe
   il pedaggio di qualunque leva che ne aggiunge uno. Finché O1 non è fatta, il
   tetto A-LB-97-1 («Δ netto bracci caldi ≤ 0») non è soddisfacibile da
   costruzione.
3. **Il moltiplicatore del canale è SCREEN.** Il valore 4,5%…6,5% di §P1 viene
   da un profilo campionato R=1. Ogni banda derivata da lì — comprese quelle di
   questo documento — eredita quel grado. Nessuna decisione presa qui è
   verdict-grade, e nessuna va rivendicata come tale.

## 6. Che cosa NON è stato fatto, dichiarato

- `TakeSlot` non è stato scritto (è il punto 5).
- La coppia F4 non è stata eseguita: F4 era il giudice di una leva che non
  esiste. Cronometrare oggi misurerebbe il binario di ieri.
- La taglia `nm -S` predetta non è stata calcolata: si predice la taglia di un
  braccio che si ha intenzione di scrivere, non di uno che si è deciso di non
  scrivere.
