# TEAM-SEMANTICA — verbale di riconciliazione (S-151 fase 2)
Sedie: STOGOV · PEDERSEN · MATSAKIS — 3× CONCORDO CON EMENDAMENTI.
I verbali individuali restano la fonte VINCOLANTE; qui si cita, non si riscrive.

## §Convergenze
1. **Census-first, chiavato per CANALE semantico, mai per sito/file:riga** —
   sopravvive ad A2 (Stogov Q1; Matsakis Q1.i; Pedersen R5 lo eleva a GATE
   delle tranche: identità esatta dei conteggi per canale = sostituto della
   byte-identità vietata).
2. **MAI ObjectId Copy-senza-refcount; weak = handle generazionale
   (index+gen) contro ABA su slot riusati; spl_object_id espone il solo
   indice con riuso alla Zend** (Stogov Q3.3+R2; Matsakis Q3c; Pedersen Q3.5:
   la generazione NON trapela nell'id PHP-visibile).
3. **Forma store: bucket di puntatori STABILI + free-list alla
   `zend_objects_store`; arena contigua inline (Gemini §2.2-2) REFUTATA**
   (Stogov R2 dal C di Zend; Matsakis: realloc invalida i riferimenti nei
   cammini re-entranti ⇒ re-fetch-by-index; PropIc a indici = sinergia).
   **Residenza: store = campo della Vm, muore con la Vm** (Pedersen R1 +
   KS-P1; «reset arena» come sostituto del teardown INAMMISSIBILE).
4. **Rischio capitale di A3 = refcount MANUALE, non il layout** (dec mancato
   = dtor mai eseguito, bug semantico). Cura convergente: handle non-Copy con
   clone/drop obbligati via store (Stogov R3) + coda decrementi thread-local
   drenata dallo sweep di statement esistente (Matsakis: senza la coda nel
   criterio, A3 irricevibile) + shadow-mode Rc↔rc + live-count==0 a fine
   richiesta (Stogov R3; Pedersen KS-P2, R6: used_n=0 ri-puntato allo store).
5. **Vittime nominate da riprovare**: `Drop for PhpArray`/L-RD1 non vede lo
   store (Stogov Q3.6 ≡ Matsakis); invariante ricevitore WP-102 (Matsakis);
   soglie osservatore Fase A `2+extra` RI-DERIVATE (Pedersen Q3.1).
6. **Gate tranche A2**: batteria + corpus 1412×2 per NOME + fixture
   bilaterali + micro R=5 + disasm bl-count run_loop (tutti) + conteggi
   census identici (Pedersen R5); commit move-only, `#[inline]` preservati
   (Matsakis); freddi prima, run_loop ULTIMO e NON spezzato — veto sullo
   spacchettamento exec/ops_* di Gemini §4.3 (Stogov Q2; Matsakis conforme).
7. **Coppia WP = legge utente**: deroga si CHIEDE all'utente (Stogov R6,
   Pedersen R7, Matsakis Q2); proposta comune: tranche accorpate, un
   pin/sessione, una coppia per pin.
8. **Tensione tetto 1,27 s ≈ 3,4%**: unanime — NON è contraddizione: il
   tetto cappa il solo canale movimenti; A3 deve reggersi sui canali
   NON-movimento, ciascuno prezzato; cifre Gemini NON firmate.
9. **A4 dente**: sede BATTERIA (CI backlog ~3 gg non morde); nuovi ≤2.000;
   esistenti ratchet decrescente per file; conteggio RIGHE, mai pattern.
10. **Mandato BT1-pesca dentro A1** prima di congelare 4–6 sessioni senza
    leve (Stogov Q5; Matsakis Q5.4): ogni BT1 ricalibra il gap residuo di A3.

## §Conflitti-e-dissensi (registrati, NON appianati)
- **C1 — integrale vs interleaving**: Stogov Q1 + Pedersen Q1: A1→A2→A3
  regge, nessun sottoinsieme piccolo. Matsakis Q1 emenda: A2 limitato al
  perimetro che A3 toccherà (il census invecchia in 4–6 sessioni; inliner
  flippa, WP-104). Lo scioglie l'utente/plenaria, non il team.
- **C2 — timing distruttori (pregiudiziale A3.0)**: Stogov R1 tiene DUE
  progetti legittimi — (a) sweep-preserving, (b) Zend-immediate come atto di
  FEDELTÀ (golden rifondato, flip per NOME, sentinelle WP-39 riscritte).
  Pedersen R8: ordine teardown INVARIANTE per tutta A2+A3. Matsakis: la coda
  preserva il timing collaudato; K-M4 ferma A3 se un dtor-order cambia.
  Maggioranza di fatto per (a), ma l'opzione (b) resta agli atti: NON appianata.
- **D1 — perimetro**: Stogov Q3.7 mette Resource FUORI (resta Rc); Matsakis
  Q5.1-2 esige Zval::Ref e catture Closure DENTRO il criterio. Non
  contraddittori, ma il perimetro va DICHIARATO per nome prima di A3.
- **D2 — soglia kill-switch**: KS-ST-1 «<15% del gap ORM netto» vs K-M1
  «<15% del tempo ORM»: denominatori diversi, da armonizzare in pre-registrazione.

## §Esigenze-verso-A1/A2 (per la DECIDIBILITÀ di A3)
Numeri dal census: (1) churn 32% spaccato per specie Zval (Object/Array/
ZStr/Ref), clone e drop, su ORM (Stogov i; Matsakis 1); (2) borrow/borrow_mut
per canale, Object vs Ref, ns/op da A/B (Stogov ii; Matsakis 2); (3) dec-a-
zero vs dec-a-non-zero (Matsakis 3); (4) drop container-con-oggetti (Stogov
iii); (5) quota borrow/fughe re-entranti per 1k op ORM (Stogov iv; Matsakis
4); (6) distribuzione #props per istanza ORM — mediana >8 ⇒ inline-8 fuori o
rifondato (Matsakis 5/K-M5); (7) gamba SERVER multi-richiesta col teardown
prezzato, canali movimento/non-movimento separati (Pedersen R4: il census
ordinato è CLI-only); (8) riconciliazione esplicita col tetto 1,27 s.
Fixture/gate PRIMA di A3: fx-objstore bilaterale (spl_object_id-riuso,
serialize, WeakMap-weakness, risurrezione — Stogov R5; weakrefs 18/35 nel
fail-set); gate server due-richieste con `__destruct` che STAMPA, parità al
byte, capture prima di request_end (Pedersen R2 — verificato ASSENTE);
fixture riuso-id server + «dtor libera peer non walkato» (Pedersen R3);
risorsa-in-prop-ciclica per il busy-cell K-M72.2 (Pedersen Q3.7).
Kill-switch da pre-registrare: KS-ST-1/K-M1 armonizzati (D2), KS-ST-3/4,
KS-P2/P3/P5, K-M4.

## §Priorità-per-l'ordine-S-151+
1. Decidere A3.0 (Stogov R1) col dissenso C2 agli atti.
2. Emendare il piano census coi numeri (1)–(8) + mandato BT1-pesca.
3. Pre-registrare: residenza store (Pedersen R1) · bucket (Stogov R2) ·
   weak=(id,gen) · perimetro A3 per nome (D1) · soglie armonizzate (D2).
4. Sciogliere C1 in plenaria/utente.
5. Fixture/gate (Stogov R5; Pedersen R2/R3) SUL PIN, bilaterali, prima di
   ogni riga di chirurgia; deroga coppia-WP chiesta all'utente.
6. Dente A4 in batteria subito (unanime).
