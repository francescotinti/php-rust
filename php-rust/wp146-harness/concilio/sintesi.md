# CONCILIO a 9 — S-146 — sintesi di convergenza (B3/filone conteggi, KS-B4)

Convocato per cambio di rotta da sonda (REGOLE §7). Fascicolo:
`s146-concilio-fascicolo.md`; verbali integrali VINCOLANTI in
`COUNCIL_S146_REVIEWS.md` (9 sedie + 3 note di team). Protocollo a due fasi
(bozze indipendenti → team forma-motore/semantica-confini/misura-giudici).

## §FONDAMENTALI

- **Oggetto**: gap ORM 8,59–8,71× (37,6 s). KS-B4 scattato per misura
  (memcpy 69,5% ≥ 60%): B1/B2 chiusi; il concilio ordina il filone conteggi.
- **Mandato inverso (Gregg)** — cosa sappiamo oggi che ieri non sapevamo:
  (1) il ciclo per-movimento è PAVIMENTO-dominato — può pagare solo muovere
  MENO; (2) prezzi per-movimento firmati (2,88–3,85 ns/coppia per tipo) +
  conteggi ORM (367,6M): ogni conteggio futuro si converte in SECONDI, lo
  SCREEN 4,5–6,5% è pensionato; (3) il perimetro modellato ha un tetto
  ASSOLUTO di 1,52 s su 37,6 s; (4) churn IN budget vs oracle, memops FUORI.
  NON sappiamo: la liveness su ORM e il ponte fra le convenzioni di conteggio.
- **Sessioni-senza-misura**: 0 (S-145 ha misurato; S-146 ha la coppia WP in
  finestra). Leve spedite S-146 al momento della sintesi: 0, dichiarato.
- Rischio d'oggetto più trascurato: residuo `other` 57,9% fuori-budget —
  tranche-3 growth-alloc nominata CONCORRENTE per la leva successiva (Leijen).

## Deliberato

**9/9 CONCORDO CON EMENDAMENTI** (Klabnik quasi-opposizione su TakeSlot;
Hejlsberg si oppone a F1-liveness «com'è posta»; opposizioni ASSORBITE
nell'ordine sotto). **Rovesciamento unanime dell'ordine di B3**: un take è
ancora un movimento — la copia di 16 B resta, si elidono solo inc-dec
(0,21 s) e al più la nota; la sola classe coerente con KS-B4 è
**borrow-first/through-borrow (famiglia FR1-ext)**, che elimina il movimento
intero. **TakeSlot RETROCESSO** dietro censimento e condizioni. 
**Arena-conteggi ARCHIVIATA per nome** (rientra solo con definizione ≤1
pagina e giudice proprio; com'è nominata è una leva di prezzo travestita).
**Forma-flag** (take/mode dentro LoadSlot): ammissibile SENZA O1 — il
vincolo diventa di TAGLIA (nm -S predetta prima + disasm bl-count, lezione
H-C2). **Perimetro fedele**: nucleo senza identità — scalari sempre,
stringhe solo con analisi sound, container MAI con drop anticipato (veto di
sedia Stogov/Pedersen, senza misura); guard runtime su `Ref` mai superfluo;
la scelta FINALE del perimetro si fa DOPO i conteggi ORM (trasferire le
quote WP a ORM = denominatore a memoria, Gregg b).

## Ordine per S-147 (armonizzato dai 3 team)

1. **CENSUS UNICO ORM monobinario** (criterio ≤10 righe firmato PRIMA; kill
   aritmetico pre-registrato PRIMA dei dati): nella STESSA run emette
   (i) movimenti per SITO/digramma × categoria (ranking dei bersagli
   borrow-first — soddisfa Hoare/Hejlsberg/Klabnik/Stogov/Gregg-R5);
   (ii) contatore-PONTE slot_reads↔movimenti + F1-liveness su ORM
   (Gregg R1, Bak, Pedersen, Leijen); (iii) separazione per tipo di ciò che
   un take eviterebbe (scioglie il conflitto 0,21 vs 0,4 s sul take-str).
   ×2 repliche, r1==r2 ≤1% per chiave, parità per NOME, denominatori dal
   sorgente. VIETATO mescolare le due convenzioni fuori dal ponte.
2. **FR1-ext borrow-first**: l'istruzione PROCEDE senza attendere il census
   sui bersagli GIÀ nominati per NOME (chiave da SLOT `$o->d[$k]` pattern
   LoadSlot; famiglia FieldRead/isset); il census ordina i successivi.
   Protocollo L-FR1 per nome: criterio ≤10 righe, R=5 ABAB, guardie con
   giudice DENTRO lo script (az.rev. S-145 #3), disasm bl-count.
3. **Fixture bilaterali (Pedersen R1) + gate STOP allargato**: fx-destructor-
   order · fx-generator-suspend · fx-a-append-a · fx-compact-after-last-use ·
   fx-weakref-slot · fx-ref-to-str · fx-resource-close-order (byte-id vs
   oracle); STOP esteso a generators/references. PRE-condizione di ogni riga
   di FETTA che consuma (non del census, non delle fette borrow-only).
4. **TakeSlot forma-flag: CHIUSO finché** (tutte e tre): residuo
   take-eligible NON-borrowable × prezzo ≥ soglia (banda sotto) · fixture
   p.3 verdi · sigilli Hoare R1–R3 (SlotMode enum, token ZST, mutation-check
   del guard). KS-M3: chiuso comunque finché inc-dec ≤20% del churn ripartito.

**Soglia arbitrata** (dissensi a verbale, si applica la PIÙ SEVERA che
scatta): banda alta d'attesa < 1× risoluzione del giudice della scommessa
(±0,7% coppia ORM ≈ 0,26–0,30 s) ⇒ ZERO codice sul bersaglio (KS-G1/K1);
1×–2× ⇒ SOLO fette micro-judged (soglie REGOLE §3), NESSUNA scommessa
suite; ≥2× (≥~0,6 s) ⇒ scommessa suite ammessa, ma KS-B1 va RI-REGISTRATO
alla scala del perimetro (Hejlsberg R4: il −25% churn+memops di s144 non si
applica a B3 com'è). Dissenso registrato: Klabnik chiude la scommessa suite
sotto 2× senza la banda intermedia; Gregg apre a 1×.

**Allowlist vs rinunce** (arbitrato): per ogni eventuale fetta che CONSUMA
si adotta l'**allowlist SAFE chiusa** (Matsakis R3 — più severa; lezione
S-96 «cura enumerabile vs attacco non enumerabile»); dissenso Stogov
(rinunce S-96 intere) registrato.

## Kill-switch unificati (pre-registrati; i KS di sedia restano vincolanti)

- **KS-146-1** aritmetico: banda alta < soglia (regola sopra) ⇒ zero codice.
- **KS-146-2** ponte indefinibile dal sorgente o r1≠r2 >1% ⇒ riconvoca.
- **KS-146-3** taglia nm -S o bl-count run_loop fuori predizione ⇒ STOP fetta.
- **KS-146-4** semantico (senza misura): take su container senza deferral /
  morte a confine ⇒ veto immediato.
- **KS-146-5** parità: fail NUOVO per NOME in weakrefs/destructor/
  generators/references ⇒ STOP fetta + revert.
- **KS-146-6** giudici: delta galloc_n dalla fetta (B3 è alloc-neutro,
  KS-L1) ⇒ STOP; fetta giudicata su quota memops (KS-G3) o su prezzi pair
  (gate 5% mai ricollaudato, KS-L2) ⇒ criterio invalido. Memops resta VOCE
  PROPRIA (Gregg R4): si riapre solo con attribuzione Zval-move dedicata.
- **KS-146-7** orizzonte: 4 sessioni di fette con coppia ORM ferma in banda
  ⇒ revert + riconvoca (eredita KS-B2/KS-B3-K4).

## Che cosa B3 NON è (9/9)

Non è la scommessa di parità: al tetto irraggiungibile compra ≤1,52 s
(~2,8% del gap); i ~4,4 s di glue churn restano fuori modello, i 471,3M
alloc-pair non sono toccati (alloc-neutro), memops resta fuori budget.
Il secondo atto (other 57,9%, tranche-3) va dichiarato nella rotta.
