# Revisione S-141 — revisore singolo, lente SEMANTICA

## Reperto principale (scopre la prima metà del claim)
«SEMANTICAMENTE INVARIANTE» oggi è un teorema da lettura del codice, non un fatto misurato: l'unica parità eseguita in sessione sono gli output dei 10 micro (file da 8–18 byte), e il giudice m-arrdrop esercita SOLO Packed con scalari+str. Il ramo Hashed (Key::Str, tombstoni), gli array annidati e le otto varianti portatrici di rd1_drop_val non sono attraversati da nessuna esecuzione di collaudo; batteria, corpus 1414×2, fixture e ORM sono rinviati alla CI — e il «parità per NOME rc=0» del census NON copre la leva (census_phpr=b41f762e è la probe PRE-acb5e7d). Ho rifatto la lettura e il teorema regge (Vec droppa in avanti; tupla .0→.1 ⇒ Key prima del valore; drain prima della glue preserva entries→index; tombstoni gestiti; cur_holds è un cursore, nessun aliasing; drop prende &mut self), ma il claim va declassato a «invarianza argomentata, non ancora verificata».

## Reperti secondari
1. **Controllo positivo senza artefatto**: il criterio p.4 rende il disasm OBBLIGATORIO («senza questa firma la leva non si promuove, qualunque D»); agli atti c'è solo il messaggio di commit («204 istr, 1 bl residuo verso glue Zval»), nessun file in wp141-harness/ab-out. E se il bl residuo sta sul cammino Str (3 elementi su 6 del giudice), la firma è solo parziale.
2. **«Profondità INVARIATA» conta i LIVELLI, non i byte**: drop_bounded copre davvero solo Props/Captures/Val (object.rs:288-292; verificato), ma le catene array→array ricorrono nativamente in A e in B, e i frame per livello sono cambiati (glue snella → drop() da 204 istr). La soglia di SIGSEGV (lezione WP-25: ~45k) può essersi mossa; nessuna sonda di nesting eseguita.
3. **Bordo esatto**: D=+5,0 == soglia 5,0, con soglia dettata dal drop-1 A'=5,0 di UNA coppia (rawA=2,81); la strettezza (> vs ≥) non era pre-registrata e D è sotto il modello 6–40. La qualifica in bb18476 è corretta: evidenza portante = segni 7/7, non il margine.

## Vagliate e respinte (con la prova)
- **Panic⇒leak**: reale differenza (la glue droppava il resto in unwind), ma dichiarata; nessun cammino di panico realistico nel teardown (Rc dec non panica; __destruct è allo sweep eager).
- **mem-census**: free(CH_ARR) precedeva già il drop dei figli anche in A (era l'unico corpo del Drop); totali e ordine invariati.
- **Emenda giudice d6b77b7**: solo forma ($i<N per l'awk); smoke↔R5 riconciliati in banda.

## Azioni S-142
1. Catena completa sul commit CON la leva prima di ogni uso del pin: batteria, corpus per NOME, fixture bilaterali, ORM 3E/13F per NOME.
2. Disasm A/B agli atti (bl-count e destinazione del residuo); se il residuo è sul cammino Str, dichiararlo e rivalutare la firma.
3. Sonda profondità: $a annidato ×N (Packed e Hashed) su A e B fino alla soglia di sfondamento; documentare o rientrare.
4. Micro di parità Hashed: chiavi stringa + tombstoni + annidati, confronto al byte con l'oracle.
5. Pre-registrare la strettezza della soglia; a D==soglia il verdetto sia «AL BORDO ⇒ replica», mai «SOPRA».
