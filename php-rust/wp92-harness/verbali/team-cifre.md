# Team CIFRE — relazione di riconciliazione (Concilio WP-92)

Sedie: 3 Klabnik (gate cifre/forge) · 4 Hejlsberg (catena evidenza) · 9 Gregg
(metodologia, attribuzione doc-vs-verdict). Relatore: sedia 3 delega la
relazione; **i tre verbali individuali restano la fonte VINCOLANTE** — questa
relazione non li riassume né li supera, li colloca.

Verdetti: Klabnik CON EMENDAMENTI (4 forge reali passati dal gate vivo) ·
Hejlsberg CONCORDO CON EMENDAMENTI (nessuna refutazione capitale) · Gregg
CONCORDO CON EMENDAMENTI (nessuna refutazione capitale).

---

## CONVERGENZE

1. **Un'asserzione vale quanto il blob a cui si risolve.** Hejlsberg lo
   verifica in positivo (Q1: judge_sha g1=`1b1e9e96f5019bc0`, g2=
   `97a8eff0d9783ee4`, checker_sha=`0f62beed576f6298`, tutti risolti a blob di
   commit antenati di HEAD); Klabnik lo verifica in negativo (Q1: manifest e
   budget letti dal **working tree** mentre il corpus è letto da HEAD ⇒ input
   non autenticato, forge landed). **A-SK-67** è la forma generale della stessa
   legge che **A-BG53/self-tether** ha già fatto reggere sulla catena giudici:
   nessuna tensione, un'unica regola da estendere a manifest/budget/giudice.

2. **Ciò che non è emesso da uno strumento non è autenticato.** Hejlsberg
   Q2: la riga `esito=ABORT reason=…operator-error` ha grammatica identica a
   quelle di `att_row` ma è per costruzione scritta a mano (**A-AH58**
   `writer=`). Klabnik Q3: `--cache/--nonce` sposta il veleno in argv, scelto
   dal chiamante, e il gate non stampa il proprio sha (**A-SK-70**, **A-SK-67**
   parte judge_sha). Stessa classe: *provenienza dell'append/dell'invocazione*.

3. **Le coperture parziali sono buchi, non gradazioni.** Hejlsberg Q3: il
   triangolo `sha256==DSHA` chiude solo il PASS, FAIL/REFUSE non portano alcuno
   sha e il loro OUT vive fuori repo, mutabile (**A-AH59**). Klabnik Q4: `--all`
   guarda solo `wp*-harness/MEASURE*_RESULTS.md`, mentre **43 cifre** in
   NEXT_SESSION (20) e `sessions/WP_SESSION_90.md` (23) restano non giudicate
   contro 28 giudicate (**A-SK-71**). Stessa forma: il perimetro è definito
   dallo strumento, non dal flusso reale dell'evidenza.

4. **Un solo numero pubblicato al posto di una distribuzione mente.**
   Gregg Q2: VCOV 0,576 non esiste in nessun raw (per-W 0,415→0,650; coverage
   marginale 15.781.205 B/worker ≈ 0,80·b_peak) — **A-BG57**, **KS-BG-92-1**.
   Convergenza con Klabnik Q2: la `prov` ammette operandi da campagne diverse
   (live PASS `19.600.000 = 23000000@…axum.83c… − 3400000@…axum.82c…`), cioè
   *pool sbagliato* invece di *aggregato sbagliato*. In entrambi i casi la
   difesa è la stessa: **la cifra pubblicata deve nominare il proprio dominio**.

5. **doc == verdict si controlla per NOME, non per conteggio.** Gregg Q1: il g2
   marca ADVISORY su **7** punti (4 PEAK + 3 WORK), il doc ne nomina 4 e chiama
   BOOT/WORK "lane forti" (**A-BG60**). È lo stesso difetto che Klabnik trova
   in **A-SK66** (la riga «NON e superseded» passa per parola-chiave, non per
   prova) e che **A-SK-72** chiude con ledger committato. La coppia
   A-BG60+A-SK-71 è la stessa disciplina applicata a due perimetri.

6. **Il resolver deve verificare l'operazione che è scritta.** Klabnik Q2.3:
   `[derivata: prov X@p:1 diviso Y@p:2]` passa e stampa "provenance-verified"
   perché il test `/−|-/` è soddisfatto dal trattino di `wp89-harness`
   (**A-SK-69**). Gregg Q4 trova il gemello nel giudice: nell'awk `v` non è
   resettato per riga, una riga senza `commit=` eredita la precedente
   (**A-BG59**, 0/20 latente), e VIS usa regex-substring `/win=9/` non match di
   campo. Stessa patologia: **matching lasco al posto di parsing**.

7. **Il ledger deve rigenerare la propria storia da solo.** Gregg Q3:
   `reason=see-verdict-file` / `all-blocks-clean` sono puntatori, non ragioni
   (**A-BG58**, **KS-BG-92-2**); Hejlsberg (osservazione minore): le righe
   `phase=verdict` non portano `campaign_sha=`, il verdetto è legato
   all'attempt solo per numero (**A-AH60**). Due maglie dello stesso anello.

8. **"Attempt=1 PULITO" è legale solo in coppia con le righe sporche.**
   Gregg Q3 lo dichiara esplicitamente (ABORT operatore a `8da340c` nel
   battery-ledger, `esito=FAIL` g1 nel campaign ledger, scope = fasi di misura);
   Hejlsberg Q4 fornisce la prova che rende la coppia leggibile (finestra
   `4c99520..bb4b388` pulita, 3 path allowlistati A-SK50, delta attempts =
   la sola riga PASS appesa). Convergenza piena: la dicitura resta, con obbligo
   di citazione appaiata.

---

## CONFLITTI

**Nessuna contraddizione di fatto tra le tre sedie.** Nessuna cifra affermata
da una sedia è negata da un'altra; nessun emendamento di una sedia rende
illegale un emendamento di un'altra. Restano **cinque tensioni di
collocazione**, che il plenum deve risolvere come ordine, non come merito.

**T1 — "quattro forge capitali" (Klabnik) vs "nessuna refutazione capitale"
(Hejlsberg, Gregg): perimetri disgiunti, non giudizi opposti.**
- *Klabnik*: il gate che **ammette** cifre è forgiabile dal vivo; l'evaluator
  X−Y non è stato abolito ma annotato, e oggi copre il **46,25 %** delle
  differenze nella finestra (contro l'~11,5 % del WP-91: il buco è **cresciuto**,
  non chiuso), su **36.573** operandi indirizzabili contro i 24.042 del corpus.
- *Hejlsberg*: la catena **consumata** in S-90.0 regge a macchina su ogni
  maglia (PASS→stamp→OUT→matrix→toolchain→judge g2); i suoi controlli sono
  stati eseguiti risolvendo blob a HEAD **a mano**, quindi non passano dal
  percorso working-tree che Klabnik ha morso: le due conclusioni sono
  indipendenti e entrambe vere.
- *Gregg*: ogni cifra del doc è stata **ricomputata dai raw** (additività
  19.723.059 = 2.252.800 + 17.276.928 + 193.331 esatta al byte; medie VCOV
  riprodotte al byte dai 20 raw; 34 raw = 20+4+10; 9/9 marginali IN per NOME),
  quindi il *contenuto* di S-90.0 non è in discussione.
- *Collocazione proposta*: S-90.0 **resta consumabile**; i forge di Klabnik
  colpiscono la **ammissibilità futura** e le 43 cifre già pubblicate fuori
  perimetro. Non si riapre il verdetto 90; si chiude il gate prima che il
  prossimo verdetto lo attraversi.

**T2 — A-SK-67 (`working≠HEAD ⇒ FAIL`) vs il flusso reale di scrittura di un
doc.** Un MEASURE/NEXT_SESSION in scrittura non è a HEAD per definizione.
- *Klabnik* chiede HEAD per **manifest, budget, giudice** (gli input di
  autorità), non necessariamente per il doc giudicato.
- *Hejlsberg* (implicito in Q1/Q4) tratta come verdict-grade solo ciò che è
  risolvibile a blob committato.
- *Risoluzione proposta*: due modi espliciti nel gate — `advisory`
  (pre-commit, doc dal working tree, autorità comunque da HEAD, **mai**
  verdict-grade) e `verdict` (tutto da HEAD, PASS che stampa
  judge_sha+manifest_sha+budget_sha). Senza questa distinzione A-SK-67
  rende il gate ineseguibile durante la sessione.

**T3 — A-SK-70 (cache abolita) vs A-SK-71 (perimetro da 28 a ~71 cifre).**
Abolire la cache mentre il perimetro quasi triplica è un costo che nessuna
sedia ha misurato. Klabnik offre già la via che non paga il conflitto (**un
solo processo perl per `--all`**, invece della cache creata dal parent):
adottare quel ramo, non il ramo "cache rifiutata se non creata dal parent",
che lascia in piedi un'autorità da argv.

**T4 — A-SK-68 (abolizione `legacy_frozen`) vs i doc storici congelati.**
Nessuna sedia difende `legacy_frozen` (Klabnik: "doc STORICO CONGELATO" è un
**commento**, non un predicato — nessun test di committed-ness, di data, di
appartenenza a M84-M88; il forge è passato con una **riga di manifest mai
committata**). Ma l'abolizione secca, applicata prima di aver collocato i 5
doc, li spinge in un limbo: rigiudicati con le regole nuove **fallirebbero**
(usano l'evaluator libero pre-WP-91), e con essi diventerebbero non
verdict-grade cifre già citate a valle (NET_H/NET_P di WP-85, slope WP-87,
b_base WP-88/89). Vedi P2 nelle priorità: l'ordine risolve, il merito no.

**T5 — chi scrive per primo sulla stessa riga.** A-AH58 (`writer=` su
attempts), A-AH59 (sha su FAIL/REFUSE), A-AH60 (`campaign_sha` su verdict),
A-BG58 (`reason=` autosufficiente) e A-SK-72 (supersessione da ledger
committato) toccano **le stesse due grammatiche di ledger**. Applicati in
delibere separate produrrebbero tre revisioni di formato in una sessione, e
ogni cambio di grammatica invalida i checker che la leggono. Vanno emessi
come **una sola delibera di formato**, con i checker aggiornati nello stesso
commit.

---

## COMPATIBILITÀ DELLE ABOLIZIONI (richiesta esplicita del mandato)

**A-SK-68 (abolire `legacy_frozen`) è compatibile con Hejlsberg** — anzi ne è
il corollario: sostituire un predicato "sha in una riga di manifest del
working tree" con "verdetto d'epoca citato per blob committato" *aumenta* la
risolvibilità a blob, che è esattamente il criterio con cui Hejlsberg ha
chiuso Q1. **È compatibile con Gregg** a una condizione: `judge=no` non
significa "cifra libera". Gregg pretende doc==verdict **per NOME**; per un doc
storico il verdetto è quello d'epoca, quindi la riga deve nominare
`verdict_blob=<sha16>` e le cifre del doc devono corrispondere a quel blob per
nome — altrimenti `judge=no` diventa la nuova `legacy_frozen`.

**A-SK-70 (abolire la cache) è compatibile con entrambi senza condizioni**:
nessun emendamento di Hejlsberg o Gregg dipende dalla cache; l'unico effetto è
il costo di runtime di T3.

**Ordine che non rompe i doc storici congelati** (questo è il punto delicato):
non abolire prima e collocare poi. Prima si esegue un **bite-test dei 5 doc**
con il resolver `prov` emendato (A-SK-69) e si assegna a ciascuno l'esito:
`judge=yes` per quelli che reggono con operandi dello stesso file, `judge=no +
verdict_blob` per gli altri, **con la riga di manifest committata nello stesso
commit dell'abolizione**. Solo dopo si rimuove il ramo `legacy_frozen` dal
codice. Così nessun doc attraversa uno stato in cui non è né giudicato né
collocato, e nessuna cifra a valle perde il proprio ancoraggio.

---

## PRIORITÀ PROPOSTE per l'ordine S-91.0
*(ordinate per potere di blocco: cosa deve atterrare PRIMA perché il resto sia
consumabile)*

**P1 — A-SK-67 + KS-SK-92-1 + KS-SK-92-4: autenticare gli input di autorità
del gate (manifest, budget, giudice) da HEAD; PASS stampa
judge_sha+manifest_sha+budget_sha; modi `advisory`/`verdict` (T2).**
Potere di blocco massimo: finché manifest e budget vengono dal working tree e
il giudice non si nomina, **ogni** altro emendamento al gate è aggirabile
dalla stessa porta, e i risultati di P3/P5 non sarebbero credibili. Include
l'abolizione della cache (A-SK-70) nel ramo "un solo processo per `--all`",
perché è la stessa superficie di autorità-da-argv. Controllo positivo
obbligatorio: **i quattro forge di Klabnik devono essere ripetuti e devono
FALLIRE**, con l'output del fallimento ledgerato (il dente nuovo morde sul
proprio harness — lezione WP-88/89).

**P2 — A-SK-69 + A-SK-73, poi bite-test dei 5 doc storici, poi A-SK-68.**
Secondo per potere di blocco: la `prov` è oggi *più larga* del buco che doveva
chiudere (46,25 % di chiusura, 36.573 operandi contro 24.042), quindi ogni
cifra ammessa dopo P1 ma prima di P2 resta forgiabile per composizione.
L'ordine interno è vincolante e non negoziabile: **resolver emendato →
bite-test → collocazione committata dei 5 doc (`judge=yes` | `judge=no +
verdict_blob`) → rimozione del ramo `legacy_frozen`**, tutto con la riga di
manifest nello stesso commit. Invertire questi passi mette i doc storici in
limbo (T4).

**P3 — delibera unica di formato ledger: A-AH58 + A-AH59 + A-AH60 + A-BG58 +
A-SK-72, checker aggiornati nello stesso commit (T5).**
Blocca la *leggibilità* di tutto ciò che S-91.0 produrrà: senza `writer=` un
ABORT resta indistinguibile da un esito di script; senza sha su FAIL/REFUSE
la classe di forgia chiusa sul PASS resta aperta su ogni altro esito; senza
`reason=` autosufficiente e `campaign_sha=` la storia g(n)→g(n+1) non è
ricostruibile dal solo ledger. Emettere queste cinque regole insieme costa
una migrazione di grammatica; emetterle separate ne costa tre.

**P4 — A-SK-71 + A-BG60 + A-SK-72 (lato regex `.out` opzionale): perimetro
reale delle cifre pubblicate.**
Deve venire **dopo** P1-P2, altrimenti estende un gate forgiabile a 43 cifre
in più invece di proteggerle. Include l'emendamento del doc S-90.0 sul
conteggio ADVISORY (**7 = 4 PEAK + 3 WORK**, "lane forti" scopata alla sola
scala Δcommit): è il caso di prova che dimostra che la disciplina doc==verdict
per NOME morde anche dentro il perimetro già coperto.

**P5 — A-BG57 + A-BG59 + A-BG61 + KS-BG-92-1: igiene del giudice e della
pubblicazione delle metriche.**
Non blocca il consumo di S-90.0 (Gregg ha ricomputato tutto dai raw), ma
blocca l'**iterazione 3** dell'attribuzione: VCOV va pubblicata per-W con la
pendenza marginale (15.781.205 B/worker) e mai come solo pooled; `v` va
resettato per riga (0/20 latente oggi, ma è un ereditamento silenzioso); la
legge **clamped ⇔ spans=OVERLAP 10/10** va dichiarata e dotata del suo
negativo controllato (pd=1000 **con** overlap forzato), perché oggi l'unico run
senza clamp è anche l'unico NO-OVERLAP.

**Nota di sequenza.** P1 e P2 sono l'unico blocco veramente seriale: toccano
lo stesso file di gate e la stessa autorità. P3 è indipendente da P1-P2 e può
procedere in parallelo se lo tocca un'altra mano. P4 dipende da P1+P2+P3
(serve il manifest autenticato *e* la grammatica nuova). P5 è indipendente da
tutti e può chiudere la sessione.
