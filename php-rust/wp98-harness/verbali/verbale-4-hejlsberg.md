# Verbale — Sedia 4 (Hejlsberg) — WP-98
Perimetro: compilatori incrementali, interning/dedup, emissione.

## VERDETTO

S-96.0 regge nel mio perimetro: il fix di soundness, i match esaustivi e il
riconteggio sono lavoro di compilatore fatto bene, e il verdetto del passo 2 è
argomentato invece che subìto. Il **§WP-97 punto 1 no**: è scritto su una
premessa di layout che è irrilevante e su una premessa di costo che è un errore
di categoria. Come formulato **non riapre A-ZV2 e non va messo per primo**.

## Il conto sul flag `take` (la domanda che mi è stata posta)

**La taglia di `Op` non c'entra, in nessuna delle due forme.** `Op` è
`#[derive(Clone, PartialEq)]`, allineamento 8, e la sua stride è fissata dalle
varianti **fredde** con `Rc<[u8]>` fat-pointer (`CallHostBuiltinOut` ≈ 44 B di
payload; `StaticCall` con `ClassTarget`+`MethodIc` è l'altro candidato).
`LoadSlot(Slot)` occupa **4 byte su ~40 disponibili**: quello slack è già pagato
oggi, su ogni op di ogni `Vec<Op>`. Quindi `LoadSlot { slot, take: bool }` →
`size_of::<Op>()` **INVARIATO**, stride invariata, streaming del bytecode
invariato, footprint per worker invariato. E le varianti sono ~180 < 256: il tag
resta 1 byte, **anche un `TakeSlot` nuovo è layout-free**. La premessa «un flag
cambia la taglia di `Op`» è **falsa**; ma è falsa anche per il piano opposto,
quindi il layout **non arbitra nulla**. Ritratto qui la parte di A-AH-97-4 che
lasciava credere il contrario: l'assert è una guardia di regressione, non un
argomento.

**Ciò che il flag sposta è dove cresce il codice caldo, non quanto.** Il braccio
`LoadSlot` (`run.rs:585`) oggi è tre righe. Col flag diventa due percorsi + il
guard di tipo su `Zval::Ref` dentro **il braccio più eseguito del `run_loop`**:
60.598.093 letture di slot sul media group, per servirne 25.826.594 safe (42%) e
9.989.963 stringhe (16,5%). Si tassano **tutte** le letture per pagarne un sesto.
Un opcode separato lascia il braccio caldo **intatto** e tassa solo i siti take.
Perciò «un `LoadSlot` con flag non è un corpo caldo in più» è **vero alla
lettera e falso nella moneta**: WP-39..44 non contava i simboli, contava il
working-set I-cache/BTB del `run_loop`, e il flag lo fa crescere nel punto di
massima frequenza. Stogov ha ragione.

Ordine di grandezza: guadagno lordo nucleo stringhe 0,84–1,21% (SCREEN, R=1);
un branch ben predetto + il guard su 60,6 M esecuzioni non è quotabile a mente,
ma è **dello stesso ordine**. Il netto resta indistinguibile da zero, ed è
esattamente il numero che nessuno ha intenzione di produrre.

## Emendamenti

**A-AH-98-1.** Il punto 1 si istruisce **solo** con l'A/A di A-AH-97-5 (build
gemella col percorso compilato e mai emesso) + `nm -S` di `run_loop`
prima/dopo, coppia adiacente. `nm -S` da solo misura la taglia, non il pedaggio
per esecuzione: non basta. Senza A/A, il punto 1 **non è una voce di sessione**.
**A-AH-98-2 (precedenza).** O1 (outlining) prima: è l'unica leva che abbassa i
corpi caldi, è misurabile con lo stesso strumento e **produce il giudice** che a
tutte e tre le candidate manca. Il punto 1 senza O1 rifà WP-39..44.
**A-AH-98-3 (debito non evaporato).** A-AH-97-1/3 e KS-AH-97-1/3 vivono SOLO
dentro `wp97-harness/`: non sono in NEXT_SESSION, non in TODO.md, non come
marker nel codice. Vanno iscritti al backlog PER NOME **e** come commento
`TODO(port)`-grade sopra la chiave di `zvalcensus.rs:81-85`, che oggi si
autogiustifica con «accettabile in una build di sola misura» — una condizione
che nulla presidia.
**A-AH-98-4 (archiviazione falsificabile).** Un perimetro è archiviato, e non
abbandonato, **solo se** il documento nomina l'artefatto datato il cui valore
ribalterebbe il verdetto. design96 §5 nomina tre voci ma nessun artefatto: va
aggiunta la riga «A/A + O1 ⇒ riapertura», altrimenti la distinzione è una
formula.

## Kill-switch

**KS-AH-98-1**: si scrive `take`/`TakeSlot` senza A/A a monte → reject.
**KS-AH-98-2**: cache d'analisi keyed-by-pointer in un percorso di emissione
(non di sola misura) → reject senza discussione (riaffermo KS-AH-97-3).
**KS-AH-98-3**: `size_of::<Op>()` non fissato da un test nello stesso commit
del primo cambio a `Op` → reject.

## Refutazioni capitali

**RC-1.** «Un `LoadSlot` col flag `take` non è un corpo caldo in più» —
**refutata**. Il layout è neutro (lo slack c'è già), quindi il flag non compra
nulla sul fronte che invoca; e sul fronte che conta sposta la crescita dal
numero dei corpi alla **taglia del corpo più caldo**, tassando 60,6 M letture
per servirne 9,99 M. Non è la terza forma: è la peggiore delle due.

**RC-2.** «Il verdetto ha sospeso il debito» — **falso**. Il codice
pointer-keyed è ancora nell'albero e la sua unica difesa è una condizione
(«sola misura») che il punto 1 stesso violerebbe riusando `analyze`
dall'emissione. Un debito che vive solo nel verbale che l'ha nominato non è
registrato: è **evaporato**. Prima voce di S-97.0 nel mio perimetro, prima di
qualunque leva.
