# Revisione S-143 — lente MISURA (revisore singolo adversariale)

## Reperto principale — il numero 1,38% NON è il numeratore della regola
La regola p.3 decide su «quota coppie **obj(+props Rc)**». Il criterio p.2 enumera `rczval` (box Rc di Zval condivisi) come classe propria; la run NON l'ha attribuita: è finita dentro other=61,66% insieme a vecargs. Il census pubblica quindi un **minorante** del numeratore pre-registrato e lo chiama quota. Con le scale note (propget 29,9M + recv_clone 14,8M dal dossier, +19,5M realloc-eventi fuori denominatore — `realloccensus n=19497739` in census-zval-r1) il maggiorante additivo onesto è ~8–15%: **il verdetto B sopravvive, ma passa da «misurato 1,38%» a «minorante 1,38%, maggiorante NON misurato sotto ipotesi»**. Nulla nella run esclude per misura una popolazione obj-attribuibile >111M eventi dentro i 290M non attribuiti — serve l'attribuzione, non l'assunzione. Via: tranche-2 del census con funnel `rczval` marcato alla nascita (cammino proprietà sì/no) + contatore vecargs; pubblicare `quota_obj_max`.

## Reperti secondari
- `copy_with_id(0)` (object.rs:395, vm/mod.rs:10049): il box `Rc::new(RefCell::new(..))` sintetico NON passa dal mint (funnel unico a vm/mod.rs:3681) → raw event nel denominatore, mai in obj. Probabile piccolo (serializzazione), non contato.
- dyn_entries: contato UNA volta col propsbuf (banda dichiarata nel codice, ≤0,69pp). Ok ma la banda va nel verdetto, non solo nel sorgente.
- Convenzione realloc dichiarata per i BYTES (voce d) ma non per i CONTEGGI: 19,5M eventi (~4% del denominatore) invisibili alle quote.
- Emenda v2 post-dati: quote invarianti per costruzione (num e den entrambi ×2), soglie intatte — accettabile, ma è un precedente: il parser va collaudato su fixture PRIMA della run.

## Vagliate e respinte
- **Figli multipli**: 1 sola riga `tag=exit` per raw (pid=63551 in r1), file aperto in append (memcensus.rs:167) → un processo, nessuna somma spuria.
- **Pavimento avvio**: per flippare a ≥25% servirebbe floor ≥445M su 471M — assurdo (str da sola 129,9M distribuita nella suite). Smoke ~184k = 0,04%.
- **r1==r2 esatto**: contatori di eventi su suite deterministica; galloc ±1 e bytes ±649 tra run provano indipendenza delle repliche.
- **Figli heap in str/arr**: corretto — l'A rifondata dal concilio (pool per blocchi oggetto, refcount conservato) non ospiterebbe stringhe/array condivisi.
- **Clausola Klabnik**: se mordesse imporrebbe B-prima — può solo riordinare dentro la famiglia B, mai restaurare A. Esito intatto.

## Azioni S-144
1. Census tranche-2: funnel `rczval` (nato su cammino proprietà?) + vecargs; pubblicare `quota_obj_max` e chiudere il numeratore della regola per misura.
2. Attribuire per NOME ≥80% di other 61,66% o dichiararlo fuori-budget (specchio del Gregg R3 già a verbale).
3. Emendare il verdetto: convenzione conteggi realloc (19,5M fuori denominatore) dichiarata come per i bytes; banda dyn_entries nel verdetto.
4. Golden-test del parser (fixture exit+exit_mi) committato INSIEME al criterio delle prossime tornate.
5. (Se tranche-2 economica) contare `copy_with_id(0)` con un tick dedicato.
