# Team FORMA-MOTORE (Hoare · Hejlsberg · Bak) — nota di fase 2, S-146

Fonte VINCOLANTE: i verbali individuali. Questa nota registra, non sostituisce.

## 1) CONVERGENZE
- **3/3 (refutazione centrale)**: TakeSlot NON rimuove il pavimento memcpy (69,5%) — un take è ancora un movimento (copia 16 B + store Undef); elide solo inc-dec+nota (≤0,46 s teorici). Il borrow/through-borrow è l'unica classe coerente con KS-B4: elimina il movimento intero. **Ordine invertito: borrow PRIMA, take poi/retrocesso** (Hoare R4, Hejlsberg d, Bak R2).
- **3/3 forma-flag ammissibile**: bit/campo dentro il braccio LoadSlot esistente = zero corpi caldi nuovi ⇒ **O1 NON è prerequisito** di questa forma; restano dovuti nm -S PREDETTA prima e disasm bl-count (lezione H-C2). Bak: il vincolo si trasforma in vincolo di TAGLIA del braccio (+16–48 B stimati, rischio inliner).
- **3/3 perimetro (b)**: nucleo senza identità (Stogov); guard di tipo su Ref obbligatorio e MAI superfluo (safe_ref 0,013%, economico via BTB).
- **3/3 arena-conteggi**: ARCHIVIARE salvo definizione scritta (≤1 pagina per Hoare) che riduca movimenti e superi i veti alloc.
- **3/3 (e)**: B3 compra ≤1,52 s modellati; nessun claim su parità né sui ~4,4 s di glue.

## 2) CONFLITTI NON LEVIGATI
- **Primo censimento ORM**: Hoare = siti borrow-abili (classe FR1, zero liveness); Hejlsberg = **MI OPPONGO a F1** com'è posta: prima i DIGRAMMI (LoadSlot;CallArg/…); Bak = **esige F1-ORM** con classi allineate alla partizione sonda prima di aprire QUALSIASI forma. Tre oggetti diversi.
- **Bak, posizione secca non condivisa dagli altri**: TakeSlot realistico ~0,2 s < banda giudice (~0,3 s) ⇒ non istruibile come scommessa suite; e «2,88 ns/movimento comprimibile SOLO per-conteggio».
- **Sigilli di tipo (solo Hoare)**: SlotMode enum non-bool, token ZST per Take, mutation-check del guard, sentinella read-after-take (R1–R3, R5).
- **Solo Hejlsberg**: ordine per-tipo inattuabile a compilazione (R3); scommessa KS-B1 da RI-REGISTRARE alla scala del perimetro.

## 3) PRIORITÀ per S-147
1. Censimento consumatori su ORM (conciliare i tre oggetti: per sito/opcode/digramma, guadagno in SECONDI = conteggio×prezzo firmato).
2. FR1-ext borrow ai siti consumatori, protocollo L-FR1 per nome (criterio ≤10 righe, R=5 ABAB, giudici dentro lo script).
3. TakeSlot forma-flag SOLO se residuo take-eligible non-borrowable ≥ banda, coi sigilli Hoare R1–R3.
4. Arena: archiviata salvo definizione.

## 4) KILL-SWITCH
- **Unificato 3/3**: bl-count run_loop o taglia nm -S fuori predizione ⇒ STOP fetta (KS-H3/H1/BAK-1).
- **Coincidenti nello spirito, esiti diversi**: censimento sotto soglia ⇒ Hoare: FR1-ext non si apre; Hejlsberg: B3 chiusa SENZA codice; Bak: micro-only, niente scommessa suite.
- **Vigente citato**: fail NUOVO per NOME weakrefs/destructor ⇒ STOP.
- **Solo Hoare**: read-after-take>0 ⇒ STOP take; mutante-guard sopravvive ⇒ STOP fetta. **Solo Bak**: pair zcell/arr0 fuori gate 5% ⇒ leva in istruttoria.
