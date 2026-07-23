# Analisi Post-WP-42: Fine dello Stack e Inizio dell'Era a Registri (23 Luglio 2026)

Ho letto i report della sessione WP-42. Questa non è solo una fine sessione, è la chiusura di un'intera era architetturale per `phpr`. Claude ha dimostrato, dati alla mano, che le inefficienze locali sono state spremute fino all'ultima goccia. 

Ecco le mie riflessioni e raccomandazioni da leggere *prima* di avviare il cantiere WP-43.

## 🚨 1. EMERGENZA ASSOLUTA: Spazio su Disco
Prima di scrivere o misurare anche solo una linea di codice per la WP-43, c'è un'emergenza infrastrutturale da risolvere. Avere saturato il volume root a 0 byte ed essere rimasti con soli ~5GB liberi su macOS è un rischio inaccettabile per la stabilità del progetto.
L'harness genera log massicci, la cache di cargo si gonfia a ogni build e il SO ha bisogno di swap. **Se il disco si riempie durante un full-run, rischiate la corruzione del database SQLite/MySQL o un kernel panic.**
**Azione Obbligatoria (per l'utente):** Fate pulizia manuale prima di ripartire. Valutate l'eliminazione della macchina virtuale di Parallels (4.9GB), lo svuotamento rigoroso delle cache di Chrome/Google (8.9GB) e un purge dei vecchi `vm_bundles` di Claude (8.5GB). Ripartite solo con almeno 20GB di ossigeno.

## 2. La Pistola Fumante: Il Census del Bytecode
Il censimento degli opcode condotto da Claude è il dato più importante estratto da mesi di lavoro. Scoprire che il **30,77%** degli opcode emessi (su oltre 743 milioni) è puro *data-movement* (`PushConst`, `LoadVar`, `DerefTop`, `Pop`, `Dup`) decreta formalmente la fine del potenziale della Virtual Machine a Stack.
Stiamo letteralmente sacrificando un terzo del tempo di esecuzione dell'interprete per fare il "gioco delle tre carte" spostando temporanei sulla memoria. Il pattern `Dup → StoreSlot → StoreSlot → Pop` misurato 9 milioni di volte è l'esatta traduzione meccanica di un'architettura che non mappa le variabili su registri fissi.

## 3. Il Cantiere Registri (Leva B)
Il piano tracciato in `REGISTER_BYTECODE_PLAN.md` (5 stadi, dual-mode opt-in per-funzione, parità totale a ogni commit) è magistrale. Questo è il modo in cui i team di alto livello compiono grandi refactoring senza mai spaccare il ramo `main`.
*   **Aspettative:** Un tetto teorico stimato dell'8-15% di CPU è un obiettivo realistico e massiccio. Basterebbe a far crollare il gap dal 2.68x a 2.3x o meno in un solo colpo.
*   **Esecuzione:** WP-43 (stadio 1) deve essere solo infrastruttura a zero delta codice. Niente fretta, ogni stadio richiederà un gate impeccabile.

## 4. Rettifiche Tecniche e Rulebook
Claude ha fatto benissimo a smentire i miei dubbi sull'AST-leak: se `mago` muore a fine lowering e ciò che sopravvive è l'HIR unit-cache, allora il footprint di memoria è fisiologico e coerente con la logica di una *opcache* (sacrificare RAM per salvare la compilazione ai cicli successivi).
Inoltre, prendo atto in via definitiva del divieto assoluto da parte del RULEBOOK §0 sull'uso dell'unsafe nel value core (e che `PhpArray` è già custom dual-repr). Mantenere la purezza architetturale in Safe Rust è il grande vanto di `phpr`.

**Verdetto finale:**
Le leve locali per mitigare il churn sono esaurite (dimostrato dal flat corretto di `silent_get_path`).
Sistemate il disco, mettetevi il casco protettivo e aprite il cantiere della **WP-43**. Il Bytecode a Registri cambierà tutto.
