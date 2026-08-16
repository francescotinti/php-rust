# Team SEMANTICA-CONFINI — nota di relazione S-146 (Stogov · Pedersen · Matsakis)
I verbali individuali restano la fonte VINCOLANTE.

## 1) CONVERGENZE
- **3/3 — Borrow prima di take**: l'ordine fedele è through-borrow ai siti consumatori (precedenti HC1/L-FR1), TakeSlot subordinato (Stogov d; Pedersen R4; Matsakis verdetto+R1). Matsakis dà il perché di conto: il take NON elimina il memcpy (69,5%), solo l'inc-dec (14,1%).
- **3/3 — Perimetro fedele = nucleo senza identità**: scalari/stringhe consumabili (stringhe solo con analisi sound); array/oggetti MAI con drop anticipato — `__destruct`, spl_object_id, WeakReference, risorse osservabili anche senza destructor (Stogov b vincolante; Pedersen b; Matsakis b).
- **3/3 — Arena-conteggi si ARCHIVIA** salvo definizione su carta che conservi morte refcount-driven (veto Pedersen su morte-a-confine; veto costo-sostitutivo Stogov/Matsakis).
- **3/3 — Nessun claim oltre risoluzione** (e): perimetro modellato 1,52 s su 37,6 s; niente parità.
- **2/3 — Fail rumoroso obbligatorio**: sentinella/Poison-Undef come build di collaudo, MAI nel pin (Pedersen R2; Matsakis R4).
- **2/3 — Guard `Ref` runtime mai superfluo**: safe_ref 0,013%, la correttezza non si misura in frequenza (Pedersen b; Matsakis a).

## 2) CONFLITTI NON LEVIGATI
- **Quale censimento**: Stogov R1 = movimenti per SITO D'ORIGINE×categoria; Matsakis R2 = borrow-census per sito CONSUMATORE (F1-liveness serve SOLO se si istruisce TakeSlot); Pedersen c = F1-ORM obbligatorio con tetto aritmetico. Tre oggetti diversi, non fusi.
- **Quanto compra il take su str**: Pedersen ~0,4 s (clone str evitato); Matsakis ≤0,21 s (solo inc-dec). Divergenza di MODELLO su cosa il take evita — va sciolta col censimento, non a tavolino.
- **Fondamento dell'analisi**: Matsakis R3 = allowlist SAFE chiusa, renounce-list vietata per nome; Stogov accetta le rinunce S-96 intere. Pedersen intermedio (soundness residua).
- **Soglie fetta**: Stogov R2 ≥100M movimenti per giudice-coppia vs Matsakis KS-M2 quota <20% — soglie diverse, non riconciliate.

## 3) PRIORITÀ S-147 — fixture/gate per NOME = PRE-condizione di ogni riga
1. **Fixture bilaterali Pedersen R1** (byte-id vs oracle): fx-destructor-order, fx-generator-suspend, fx-a-append-a, fx-compact-after-last-use, fx-weakref-slot, fx-ref-to-str, fx-resource-close-order.
2. **Gate STOP allargato**: fail NUOVO per NOME in weakrefs/destructor + generators/references (Pedersen R3; Matsakis KS-M4).
3. **Censimento ORM monobinario** (×2, r1==r2, parità per NOME) sciogliendo il conflitto 2a.
4. Solo poi fette FR1-ext; TakeSlot dietro KS-M3/KS-P3/R3-Stogov.

## 4) KILL-SWITCH DEL TEMA
- KS-ST-146-3 / KS-P4: take su container senza deferral, o morte a confine ⇒ veto immediato senza misura.
- KS-P2: sentinelle >0 dopo riparazione ⇒ perimetro falsificato.
- KS-M3: TakeSlot chiuso finché inc-dec ≤20% del churn ripartito.
- KS-P1/KS-M4/KS-ST-146-1: fail NUOVO per NOME ⇒ STOP fetta + revert.
- KS-ST-146-2: siti aggredibili <100M ⇒ niente giudice-coppia, o si chiude B3.
