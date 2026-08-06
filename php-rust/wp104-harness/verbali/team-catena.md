# Team «catena» — Concilio WP-104, fase 2

Sedie: 3 Klabnik · 4 Hejlsberg · 6 Pedersen. Relatore: team-catena.
Fonti: verbale-3-klabnik.md, verbale-4-hejlsberg.md, verbale-6-pedersen.md.
Perimetro: igiene dei gate, copertura dei denti, collaudo server.

## 0) NOTA DI FATTO verificata — la capitale Klabnik è SANATA nei fatti

Il ri-giudizio che RC-KL-104-1 chiedeva (A-KL-104-1 punto 4) è stato
ARCHIVIATO in sessione: `wp102-harness/corpus-gate/riverdetto-ref1417.txt`
(2026-08-06 12:40:46; ref `corpus82p2.fails` sha256=7371d3e4e70b20a1,
1417 righe; **off VERDE, on VERDE, set IDENTICO al riferimento aggiornato
nei 2 modi**; delta vs riferimento vecchio = SOLO `nullsafe_operator/
015.phpt` RIMOSSO) + `corpus-gate-riverdetto.done` con **`rc=0` 12:40:47**.
Il relatore ha verificato entrambi i file su disco.

Giudizio del team: l'artifact soddisfa i punti 1 (solo righe rimosse),
2 (set identico ⇒ il test rimosso è assente da entrambe le gambe),
3 (miglioria nominata, commit ee842d0) e 4 (ri-giudizio meccanico, rc=0
in un .done) della regola proposta. **La capitale è sanata; l'emendamento
NON è esaurito**: restano (a) la REGOLA scritta «set che SCENDE» in
NEXT_SESSION/Regole (senza la regola, la prossima assunzione resta a
discrezione), e (b) KS-KL-104-2: la citazione in rotazione deve puntare
per NOME a `riverdetto-ref1417.txt` / `corpus-gate-riverdetto.done` —
nella stessa cartella ora convivono un .done rc=2 (storico) e uno rc=0,
esattamente la configurazione «due fonti di verità» che ha generato la
capitale. Repuntare la citazione è il passo che chiude il caso.

## 1) CONVERGENZE (per NOME)

**CV-1 — Le tre capitali sono UNA classe: «verde dichiarato senza un
artifact che potesse mostrare ROSSO».**
RC-KL-104-1 (gate citato verde con .done rc=2 archiviato) ≈ R-HE-104-1
(«modulo intero» senza provare che il dump stampi i corpi fuori-funnel;
il giudice non ha mai visto rosso) ≈ RC-PE-104-1 (collaudo warning-free
PER DESIGN, cieco al §3.13 che motiva il pin). Le tre sedie convergono
sulla stessa dottrina: copertura dichiarata ≠ copertura controllata; un
verdetto è citabile solo se punta a un artifact discriminante.

**CV-2 — Braccio discriminante obbligatorio in ogni giudice.**
A-HE-104-1 (terzo braccio `=0`: `dump_zero != dump_one` asserito) ≈
A-KL-104-3 (mode-probe nei due fixture-gate) ≈ dottrina già incarnata nel
launcher Pedersen (mode-probe di A-PE-103-2 / KS-PE-103-3). Stesso
principio, tre giudici diversi: le gambe devono DIMOSTRARE di essere in
modi diversi / che il diff sa distinguere.

**CV-3 — Identità del binario: fail-closed e registro, mai prosa.**
A-KL-104-2 (hash+check come gamba 0 dei fixture-gate: oggi un binario
stantio passa in silenzio) ≈ KS-PE-104-2 (mai declassare il fail-closed a
«hash aggiornato a mano») ≈ A-PE-104-3 (riga 49a91e4d nel PIN_REGISTRY
come NON-pin: i rifiutati si registrano, lezione d45b578).

**CV-4 — Cella worker/cross-fixture scoperta (equivalenza nominata dal
mandato).** A-KL-104-4 (cb su worker DIVERSO: workers=2, marker
per-worker; + cella errore-poi-successo) ≈ A-PE-104-2 (interleaving
cross-fixture su workers=1: E-cb1-E-cb1-cb1). Entrambe attaccano lo
stesso buco: lo stato residuo (RetainSet/unit-cache/reset boundary) tra
fixture o tra worker non è mai esercitato dal dente cb1, che è l'UNICA
unità servita.

**CV-5 — Attese ESATTE, mai lasche.**
A-HE-104-3 (`== 1`, mai `>= 1`; funnel derivato per identità
`all_funcs − prop_inits`, mai lista a mano) ≈ KS-KL-104-1 (righe AGGIUNTE
al riferimento mai assorbibili come miglioria) ≈ KS-HE-104-1 / KS-HE-103-1
(pin numerici `Op==48` + `size_of::<Zval>()`): un'attesa lasca è un falso
verde in incubazione.

## 2) CONFLITTI (posizione di ciascuna sedia)

**CF-1 — Perimetro del collaudo del pin nuovo: quante celle entrano nel
«minimo»?**
- *Pedersen* (A-PE-104-4): il grado resta MINIMO e BASTA per l'ordine
  S-103 (le gambe A/B peak, H-C2, H-D sono tutte CLI); il grado pieno
  (option 413 + restapi 3508) NON è dovuto — non anticiparlo. Ma esige
  cb2 warning-line (KS-PE-104-1: senza, il grado è nullo) e l'interleave
  su workers=1.
- *Klabnik* (A-KL-104-4): le celle vuote della matrice (cb su worker
  DIVERSO con workers=2 + marker per-worker; errore-poi-successo) vanno
  riempite AL collaudo del pin nuovo, non dopo.
- *Hejlsberg*: nessuna posizione (fuori perimetro).
**Composizione del team**: non è un conflitto di grado ma di celle — le
aggiunte Klabnik NON sono il «grado pieno» che Pedersen vieta di
anticipare (nessuna suite option/restapi), sono celle della matrice
minima. Si adottano entrambe, fuse dove possibile (v. §3-P1).
Klabnik workers=2 e Pedersen workers=1 NON sono la stessa cella: la prima
prova il capture-boundary su worker FRESCO dopo che l'altro ha servito,
la seconda prova il worker che ha GIÀ servito altro. La fusione è
legittima solo se l'attribuzione per-worker resta a costo minuti;
altrimenti due bracci separati.

**CF-2 — «Prima si estende il dump, poi il claim» vs timebox/fondamentali.**
- *Hejlsberg* (A-HE-104-2): il claim «modulo intero» resta FALSO finché
  il controllo positivo fuori-funnel (sezione prop_init nel dump, marker
  `Binary(Add)` residuo) non passa; se il dump non stampa, PRIMA lo si
  estende.
- *Klabnik* (perimetro d, e dottrina fondamentali-first): i criteri si
  fissano PRIMA e l'igiene sta al punto 5 con timebox; un'estensione del
  dump non deve scavalcare le gambe di misura di S-103.
**Composizione del team**: si separa il CLAIM dal LAVORO. A costo zero e
subito: declassare il commento del dente da «modulo intero» a
«funnel-covered» finché il controllo positivo non passa (questo scioglie
la capitale R-HE-104-1 sul piano evidenziale). L'eventuale estensione del
dump entra al punto 5 SOTTO timebox; se il timebox scade, il claim resta
declassato — non si blocca nessuna fondamentale.

**CF-3 — Stato della capitale Klabnik dopo la NOTA DI FATTO.**
- *Klabnik* (verbale, scritto prima dell'archiviazione): gate non
  citabile verde finché non esiste un artifact rc=0.
- *Fatto verificato dal relatore*: l'artifact esiste (§0).
**Composizione del team**: capitale SANATA nei fatti; l'ordine S-103
eredita solo il residuo di codifica (regola scritta + repuntamento
citazione, KS-KL-104-2). Il «primo atto retroattivo» di A-KL-104-1(4) è
GIÀ eseguito e non va rifatto.

## 3) PRIORITÀ PROPOSTE per l'ordine S-103

**P1 — Punto 1: collaudo del pin server NUOVO — forma minima che GRADA
dopo gli emendamenti.**
Il launcher S-102 invariato NON grada (RC-PE-104-1; KS-PE-104-1 vige).
Forma minima gradante = launcher S-102 (sentinella bimodale + mode-probe
+ fail-closed hash + cb1×3 stesso worker + cross-mode + oracle sanity),
PIÙ:
1. **Braccio warning-line cb2** (A-PE-104-1): fixture che emette il
   warning §3.13 (lettura proprietà undef) sia nel corpo sia in
   `__destruct`/shutdown; byte-id vs oracle `php -S` nei 2 modi, RIGHE
   dei warning comprese. Senza questo braccio il grado è NULLO
   (KS-PE-104-1).
2. **Braccio cross-fixture/worker** (fusione A-PE-104-2 + A-KL-104-4):
   workers=2, marker per-worker, sequenza interleaved E-cb2-E-cb2-cb2;
   risposte cb byte-id in OGNI posizione e attribuzione worker
   verificata (deve esistere almeno una cb servita da worker che ha già
   servito E, e una da worker fresco). Fallback se l'attribuzione
   per-worker non è a costo minuti: due bracci separati (interleave
   workers=1 + cb ripetuta workers=2).
3. **Cella errore-poi-successo** (A-KL-104-4): richiesta in errore
   seguita da cb2; la risposta successiva byte-id (reset boundary
   post-errore).
4. **Riga 49a91e4d nel PIN_REGISTRY come NON-pin** (A-PE-104-3),
   contestuale al collaudo.
Vincoli confermati: grado resta MINIMO (A-PE-104-4 — niente option/
restapi); KS-PE-104-2 al pre-flight (binario ≠ pin ⇒ ricostruire con la
ricetta, mai hash a mano); KS-PE-104-3 (nessuna cifra server su grado
minimo, peak compreso).

**P2 — Igiene dei gate (costo minuti, PRIMA delle gambe di misura).**
a) Repuntare la citazione in rotazione a `riverdetto-ref1417.txt` +
   `corpus-gate-riverdetto.done` rc=0 (chiude RC-KL-104-1; KS-KL-104-2).
b) Scrivere la regola «set che SCENDE» A-KL-104-1(1-4) in
   NEXT_SESSION/Regole, con KS-KL-104-1 (righe aggiunte = rosso, punto).
c) Fixture-gate: gamba 0 hash fail-closed (A-KL-104-2) + mode-probe per
   braccio (A-KL-104-3) + assert nomi senza spazi/glob (KS-KL-104-3).
d) Pre-registrare `wp103-harness/hc2-criterio.out` PRIMA della misura
   H-C2 (A-KL-104-5) e pinnare `size_of::<Zval>()` accanto a Op==48
   PRIMA di aprire H-C2 (KS-HE-104-1). Per H-D: controllo positivo del
   census pre-registrato (~2 alloc/chiamata già note, pena strumento
   rotto).

**P3 — Copertura dei denti (contestuale ai punti 3 e 5 dell'ordine).**
a) Dente sottoprocesso: terzo braccio `=0` con `dump_zero != dump_one`
   asserito (A-HE-104-1) + controllo positivo fuori-funnel su prop_init
   (A-HE-104-2); SUBITO e a costo zero: claim declassato a
   «funnel-covered» finché il controllo non passa (v. CF-2); estensione
   dump solo sotto timebox al punto 5.
b) Body-zoo: `== 1` esatto; funnel derivato per identità
   `all_funcs − prop_inits`; const-thunk con add pinnato fuori perimetro
   per NOME o zoo esteso (A-HE-104-3).
c) `cargo check --release --features zval-census` (lista chiusa
   KS-HE-103-3) PRIMA o contestuale al punto 3 (estensione stackcensus a
   Call/Ret — la classe liveness.rs ricorre lì) (A-HE-104-4);
   A-HE-103-2 (budget call-site `enabled()`) al punto 5 con timebox.
