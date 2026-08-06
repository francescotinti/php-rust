# Concilio WP-104 — SINTESI DI CONVERGENZA (su S-102 e programma S-103)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: Gregg (mandato
inverso) dà **AMMESSA** — avanzamento in CONOSCENZA, zero in rapporti.
Nuovo per NOME: banda rumore full-peak PHPR **misurata per la prima volta**
(~1,8%, 34,64 MiB — la gamba È bisecabile); **23 transiti-sorgente/iter**
contati esatti sulla pila operandi (statico confermato); 🔵 **il call-path
ALLOCA ~2/chiamata** (esistenza provata, cifra da fare — RC-LE-104-1);
corpus **1418→1417** per una miglioria di fedeltà non cercata; server
gradato al confine capture. NESSUNA leva perf: i sei rapporti sono fermi
(prop 11,5). È il prezzo ORDINATO dal Concilio WP-103, non una deriva —
ma il limite va nominato: **contatore sessioni-senza-Δ-oggetto = 1**
(A-GR-104-3, nuovo contatore da tenere accanto alla riga ⏱).

**(b) Contatore full/media**: WP-102 = QUESTA sessione (0); coppia nei
2 modi + banda rumore peak nella stessa giornata.

**(c) Rischio d'oggetto più trascurato**: il **21,2% di run_loop resta
SENZA NOME** per la seconda rotazione consecutiva (S-103 bozza non lo
affrontava); secondo: la narrazione «trasversali» di S-101 è CADUTA
(RC-GR-104-1: calls 7,3→7,7 tra-sere la rimangia) — i Δ non-A/B su
categorie non giudicate si declassano a INDIZIO, mai più narrati come
effetto-leva.

## Verdetti di fase 1 (9/9: nessun MI OPPONGO; 8 refutazioni capitali, 1 sanata in sessione)

Verbali VINCOLANTI in `verbali/verbale-*.md`; team in `verbali/team-*.md`.
Capitali:

1. **Hoare RC**: l'audit INV-RECV-1 prova l'invariante SOLO sotto base=2
   (created+slot); il caso **base=1** (ricevitore temporaneo, es. `(new
   C)->x`) tocca i due osservatori `==2` NON esaminati. L'esito si
   RESTRINGE a «invariante su ricevitori slot-held»; la riga base=1 si
   aggiunge alla tavola e si arbitra con la fixture 19b.
2. **Matsakis RC**: le fixture 17-18 girano intra-arm ma con lo slot vivo
   (distanza ≥2 dalla soglia): il **−1 del MOVE non è mai arbitrato da un
   conteggio a soglia esatta**. Fixture 19 a DUE CORNI (19a: soglia esatta
   mid-arm su slot-held; 19b: base=1) PRIMA di ogni estensione MOVE/H-C1c
   (KS-MA-104-2 ∘ KS-ST-104-1). Le promozioni H-C1a/b restano intatte
   (nessun osservatore mid-arm nel giudice; corpus/fixture coprono l'uso).
3. **Klabnik RC — SANATA in sessione**: il gate corpus citato «1417×2
   verde» aveva rc=2 ARCHIVIATO (giudicato contro il riferimento vecchio);
   il ri-giudizio ora è ARCHIVIATO (`wp102-harness/corpus-gate/
   riverdetto-ref1417.txt`, verde ×2 modi). Resta la regola scritta «set
   che SCENDE» (A-KL-104-1) e KS-KL-104-2 (un .done rosso non è MAI
   citabile verde senza artifact di ri-giudizio).
4. **Hejlsberg RC**: il dente sottoprocesso dichiara «modulo intero» senza
   PROVARE che il dump stampi i corpi fuori-funnel — serve il controllo
   positivo che `prop_init` compaia nel dump (A-HE-104-2) + il braccio
   `=0` discriminante (A-HE-104-1: due bracci uguali-per-costruzione non
   possono fallire per modo).
5. **Bak RC ×3**: (i) il «denominatore 23» è un AGGREGATO — i Δ_A/B si
   dividono per sito×primitiva, mai ÷23 (KS-BA-104-1); (ii) i **~11
   drop/iter di H-C2 non sono MAI stati contati** (stima da profilo):
   drop-census PRIMA della banda (KS-BA-104-3 ≡ RC-LE-104-1: canale mai
   contato = fuori dai criteri); (iii) ABAB è cieco al **code-layout**:
   la leva-nulla (A-BA-103-4) diventa PREFISSO obbligato di H-C2
   (KS-BA-104-2: nessun A/B micro sotto la banda-layout misurata).
6. **Pedersen RC**: il launcher di collaudo S-102 invariato è **CIECO al
   §3.13** che motiva il pin nuovo — il collaudo del server S-103 esige il
   braccio warning-line (cb2: fixture servita che emette il warning
   undef-prop, riga giudicata al confine HTTP) o NON grada (KS-PE-104-1).
7. **Leijen RC**: «2 alloc/chiamata, ~35 B» è **ESISTENZA, non cifra** —
   realloc conta doppio anche in-place (netto ~32,0 B), linearità di calls
   mai misurata: la cifra si rifà con realloc disaggregato (A-LE-104-1) +
   due punti (calls_small) + istogramma size-class + tag per-sito con
   residuo≡0 (A-LE-104-3 ≡ A-BA-104-4; indiziato: `ret_cell` Rc).
8. **Gregg RC**: i «trasversali S-101» (arith −0,5, calls −0,6) sono
   TRA-SERE — declassati a indizio (KS-GR-104-1: nessuna narrazione
   cross-sessione senza A/B). Banda tra-sere del giudice da NOMINARE come
   numero (≥3 sere, A-GR-104-1).

Non-capitali che entrano: audit finestra A/B peak (A-GR-104-2 ∘
A-LE-104-5: coppie adiacenti pubblicate, tetto spread ≤1,5× fase 1
pena VOID, zona marginale Δ∈(banda,2×banda] ⇒ R≥7); Stogov: «FEDELE»
§3.13 ridimensionato a «famiglia PropGet timbrata» (5 siti su ~435) +
censimento §3.11/§3.12 (A-ST-104-2) + assert contraddizione Ref in
is_gc_container (A-ST-104-4, composto col «ok-così» di Hoare: assert
obbligatorio, semantica invariata) + Generator senza terza deroga
(A-HO-104-1 ≡ A-ST-104-3: fixture di morso o birth-track in S-103);
KS-HE-104-1 (pin `size_of::<Zval>()` prima di H-C2); H-C2 SOLO via
predicato unico (A-HO-104-5 ≡ A-MA-104-4, KS-MA-104-1: un fast-out che
salti drop/gc_note su un container = reject).

## Ordine DEFINITIVO S-103 (regola di ammissione applicata)

1. **Verdetto A/B peak** (regola pre-registrata + audit finestra: delta
   per coppia adiacente, ≥3/5 segni opposti ⇒ si ripete; spread >1,5×
   fase 1 ⇒ VOID; zona marginale ⇒ R≥7) + **collaudo pin server NUOVO**
   (build ricetta axum-server + launcher EMENDATO: braccio warning-line
   cb2 §3.13, interleaving cross-fixture workers=2, cella errore-poi-
   successo, PIN_SRV_ATTESO aggiornato; riga NON-pin 49a91e4d nel
   registro; grado MINIMO basta, cifre server = zero finché pieno).
2. **Pacchetto ricevitore** (blocca estensioni, non le promozioni):
   fixture 19a+19b (soglia esatta mid-arm; base=1) con attese PRIMA +
   tavola INV-RECV-1 emendata (esito ristretto a slot-held + riga
   base=1) + marcatori stabili ai 12 osservatori + assert Ref in
   is_gc_container.
3. **H-C2 in sequenza vincolata**: leva-nulla (banda-layout) →
   drop-census (contare gli ~11) → `hc2-criterio.out` (banda [8,22]
   ripesata sul CONTATO, pavimento ½ prudenziale, pin size_of::<Zval>) →
   A/B da sola → gate pieno (fixture 13+5+19 + batteria + corpus 1417×2
   + coppia WP). Fast-out SOLO via `is_gc_container`.
4. **H-D cifra netta**: realloc disaggregato + istogramma size-class +
   tag per-sito TL RAII (residuo≡0) + calls_small (linearità) → cifra
   netta alloc/chiamata → SOLO POI leva.
5. **Igiene/denti (timebox ½ sessione)**: regola «set che scende» scritta;
   braccio `=0` + controllo positivo prop_init nel dente sottoprocesso;
   body-zoo `==1` esatto; hash fail-closed + mode-probe nei fixture-gate;
   banda tra-sere giudice (≥3 sere, può chiudersi su sere successive);
   generator-in-cycle (morso o birth-track — terza deroga VIETATA);
   §3.13 claim ridimensionato + censimento §3.11/3.12; gh-status-sync
   (corpus 1417).

**BACKLOG per NOME**: A-MA-104-2 (audit get_mut/try_unwrap/make_mut),
A-HO-104-4 (assert anti-sovrapposizione marche), A-LE-104-4 (dump atexit
senza note zval), A-BA-104-5 (sito Other + grow della pila), A-HE-103-2
(budget enabled()), A-ST-104-1 (fixture handler-timing), A-ST-104-5
(fixture 15-bis float→int + typed-REF), 21,2% run_loop senza nome
(SECONDA rotazione — candidata a voce d'ordine in S-104 se S-103 non
apre spiragli).

## Conflitti registrati

- **Ref in is_gc_container**: Hoare «coerente così» vs Stogov «contraddice
  gc_check_possible_root» — composto: ASSERT obbligatorio (il chiamante
  scartoccia sempre), semantica invariata.
- **Celle collaudo workers=2** (Klabnik) vs **grado MINIMO** (Pedersen) —
  composto: le celle nuove entrano nel MINIMO che grada, il grado PIENO
  resta option+restapi.
- **Leva-nulla**: Bak (taratura layout) vs Gregg (calibro profilo) —
  UNA build, due letture.
- **«2 alloc/35B»**: Gregg (conoscenza) vs Leijen (esistenza-non-cifra) —
  adottata Leijen; la conoscenza è l'ESISTENZA del canale.
- **Onere base=1**: Matsakis (fixture) vs Hoare (riga di tavola) — chiude
  la fixture 19b che serve entrambi.
