# Team «catena» — fase 2 Concilio WP-103

Sedie: 3 Klabnik · 4 Hejlsberg · 6 Pedersen. Relatore: team-catena.
Fonti: SOLO verbale-3-klabnik.md, verbale-4-hejlsberg.md, verbale-6-pedersen.md.
Tema: catena dei gate e dei denti per S-102.

## 1. Convergenze

1. **Criterio scritto-PRIMA per ogni punto S-102** — tutte e tre le sedie,
   indipendentemente: Klabnik A-KL-103-4 (criterio-file stile hc1a/b con banda,
   pavimento e CLAUSOLA DI RINUNCIA — oggi il punto 3 non ce l'ha); Hejlsberg
   A-HE-103-1/5 («attesa scritta PRIMA» sul tripwire e sul dump-diff);
   Pedersen A-PE-103-2 (il collaudo minimo che grada è definito PRIMA di
   eseguirlo, con i suoi bracci nominati).
2. **L'evidenza si prova, non si presume** — stessa lezione su tre oggetti
   diversi: un dente che finge copertura è peggio dell'assenza (A-HE-103-4);
   un gate che passa con N≠13 è VOID e si rifà (KS-KL-103-2); un collaudo
   senza mode-probe nel log del server NON grada (KS-PE-103-3).
3. **Insiemi pinnati per NOME, mai conteggi o «non vuoto»**: fixture-set a 13
   basename (A-KL-103-2), dump-diff sul MODULO intero e mai solo `main.ops`
   (R-HE-103-1 vizio minore + A-HE-103-3), option 413 + restapi 3508 per NOME
   nel collaudo GRADUATO pieno (A-PE-103-2).
4. **Emissione toccata ⇒ coppia WP bimodale + corpus 1418×2 per NOME, non
   derogabile** (regola 2): KS-HE-103-2 e A-KL-103-4 la nominano entrambe per
   i punti 2-3 di §S-102.
5. **BODY_ZOO come fixture condivisa dei due denti Hejlsberg**: la stessa
   fixture con classe/prop-init serve sia il tripwire ON fuori-funnel
   (A-HE-103-1) sia la coppia assente↔`=1` in sottoprocesso (A-HE-103-3) —
   un solo artefatto, due giudici. Coerente con l'estensione di matrice
   Klabnik (fixture 14 `clone`, finestra GC automatica): assi mancanti si
   NOMINANO, non si dichiarano coperti.
6. **I debiti hanno scadenza e conseguenza NOMINATA**: fix §3.13 ⇒ rimozione
   `09-*.expected-divergence.diff` + chiusura a catalogo (A-KL-103-4);
   collaudo 2c4242b6 = debito NON condizionato con divieti attivi finché non
   gradato (A-PE-103-1/3). Nessuna finestra aperta a durata illimitata.

## 2. Conflitti (posizione per sedia, per NOME)

1. **Tenuta di S-101 — verdetti divergenti.**
   - *Klabnik*: «S-101 REGGE — nessuna refutazione capitale» (4 emendamenti
     d'igiene).
   - *Hejlsberg*: **R-HE-103-1, refutazione CAPITALE** sul dente
     A-KL-102-3 (`absent_env_is_identical_to_explicit_one`): la metà
     «stessa emissione» è `f(x)==f(x)` — pinna il determinismo, non
     assente≡`=1`; falso verde per costruzione in-process; la coppia
     assente↔`=1` end-to-end oggi non la esercita NESSUNO.
   - *Pedersen*: S-101 «ammessa con riserva» (lettera del registro rispettata,
     sostanza no).
   - **Posizione del team**: i tre verdetti NON sono incompatibili nei fatti —
     Klabnik ha verificato ordine/rc/insiemi dei gate (e lì regge), non la
     semantica del confronto in-process del dente. Il team ADOTTA R-HE-103-1:
     il dente si rifà in sottoprocesso (A-HE-103-3) e la metà in-process si
     ridichiara come determinismo o si elimina (A-HE-103-4). Nota di metodo:
     la refutazione colpisce un dente di provenienza Klabnik (A-KL-102-3) —
     nessuna obiezione di Klabnik in verbale, si registra come adottata senza
     opposizione.
2. **Che cosa apre S-102.**
   - *Pedersen*: il collaudo di 2c4242b6 è PRIMO ATTO, non condizionato
     (RC-PE-103-1 + A-PE-103-1): la clausola «se si tocca il server» della
     bozza confonde toccare il CODICE del server con toccare ciò che il
     server ESEGUE — H-C1a/b vivono già in ogni richiesta.
   - *Hejlsberg*: precondizioni dei punti 2-3 (dump-diff, budget, criteri) —
     non nomina il server.
   - *Klabnik*: igiene dei gate (A-KL-103-1/2/3) prima di riusarli.
   - **Posizione del team**: conflitto solo apparente d'ordinamento, si
     compone (vedi §3). Sulla regola di ammissione («apparato solo se blocca
     l'oggetto») il team accoglie l'argomento Pedersen: il collaudo server
     NON è apparato — è debito di collaudo il cui rinvio lascia nel registro
     un pin non graduato con runtime cambiato, e mette la prossima build in
     rotta di collisione col divieto del registro (violazione che scatterebbe
     senza decisione). BLOCCA l'oggetto: si ammette come primo atto.
3. **Nessun conflitto** su: budget Op 48 (KS-HE-103-1), path normalizzato
   (A-KL-103-3), staging gate-evidence senza `*.rs` (A-KL-103-1),
   `cargo check --release --features zval-census` a lista chiusa
   (A-HE-103-7, KS-HE-103-3), KS-KL-103-1 su H-C1c.

## 3. Priorità proposte per l'ordine S-102

Regola di ammissione applicata: apparato solo se blocca l'oggetto; l'atto 1
è ammesso perché blocca (argomento Pedersen accolto in §2.2).

1. **Collaudo 2c4242b6 — primo atto** (A-PE-103-1/2): sentinella estesa
   bimodale (16 interleaved + 4 concorrenti, workers=2) CON mode-probe
   (A-PE-102-1) + **dente capture-boundary NUOVO** (output da
   `__destruct`/shutdown, ≥2 richieste consecutive STESSO worker, byte-id
   tra richieste e tra modi) — l'unico braccio che vede il MOVE al confine.
   Finché non gradato: nessuna cifra server, nessuna nuova build
   (A-PE-103-3; ultima collaudata f2ab0636). KS-PE-103-1/2 armati.
2. **Cura di R-HE-103-1** (A-HE-103-3/4): coppia assente↔`=1` in
   SOTTOPROCESSO (`env -i`, apparato A-SK-93..97) con `PHPR_DUMP_OPS`,
   dump-diff BYTE su BODY_ZOO, modulo intero; la metà in-process si
   ridichiara «determinismo» o si elimina.
3. **Tripwire ON fuori-funnel** (A-HE-103-1, stessa fixture BODY_ZOO,
   attesa scritta prima) + budget call-site `enabled()` (A-HE-103-2).
4. **Igiene gate prima di riusarli sui punti 2-3**: fixture-set pinnato a
   13 per NOME o VOID (A-KL-103-2 / KS-KL-103-2); expected-diff §3.13 a
   path NORMALIZZATO (A-KL-103-3); regola meccanica di staging — commit
   gate-evidence senza `*.rs` in `--cached` (A-KL-103-1).
5. **Punti 2-3 §S-102 (pila operandi; PropGet/PropSet a slot) SOLO dopo
   1-4**, ciascuno con: criterio-file scritto-prima CON clausola di
   rinuncia (A-KL-103-4); dump-diff 2-modi per FORMA di ricevitore, fuori
   dalle forme nominate diff ZERO (A-HE-103-5); controfattuale CONTATO dal
   census a banda LARGA (A-HE-103-6); Op size 48 progettato prima
   (KS-HE-103-1); coppia WP bimodale + corpus 1418×2 per NOME
   (KS-HE-103-2, regola 2).
6. **Punto 6 (§3.13)** con conseguenza NOMINATA nell'ordine: fix ⇒ rimozione
   di `09-*.expected-divergence.diff` + fixture 09 byte-identica + chiusura
   a catalogo (A-KL-103-4).
7. **Backlog nominati (non bloccanti S-102, ma vincolanti)**: fixture 14
   `clone` + finestra cycle-collector automatico (matrice Klabnik §1);
   giudici di MISURA per specie prima di qualunque riga H-C1c
   (KS-KL-103-1); `cargo check --release --features zval-census` in
   batteria con costo warm registrato (A-HE-103-7, lista feature chiusa
   KS-HE-103-3).
