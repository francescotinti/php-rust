# TEAM CIFRE — relazione di fase 2 (Concilio WP-94)

Sedie: Klabnik (v-3), Hejlsberg (v-4), Gregg (v-9). I verbali individuali restano
la fonte VINCOLANTE; questa relazione non li riscrive.

## CONVERGENZE

1. **Denti dichiarati armati che non mordono** (Klabnik F-K1..F-K6; Hejlsberg
   f1..f6 + matcher + checker vuoto). Dodici forge ESEGUITE, tutte benedette.
   Il difetto non è nei singoli emendamenti: è fail-open per costruzione.
2. **L'identità dell'artefatto presa da un nome scelto dal CHIAMANTE** — Klabnik:
   A-SK-78 firma `$0`, aggirato con `bash -c` (rc=0, `judge_sha` corretto in
   stampa); Hejlsberg: lo scope dei denti viene dal BASENAME di `OUT`
   (`b91`→`battery-88pre.out` salta l'intera disciplina attempts).
   KS-SK-94-1 e KS-AH-94-2 sono la stessa legge vista da due lati.
3. **Match di PREFISSO/SOTTOSTRINGA al posto del campo** — Hejlsberg f3
   (`campaign_sha=<start>deadbeef`), f4 (`judge_sha` 16hex non ancorati),
   matcher A-AH64 a sottostringhe; Klabnik F-K1 (prefisso decimale di `rev=` e
   di uno sha di commit legalizza una MISURA). Stessa classe che A-AH61 ha
   appena chiuso sul sha256: il checker ripete il peccato del gemello.
4. **Il tokenizer del corpus è una superficie, non una difesa** — Klabnik F-K2
   (colla a sinistra: `W16-20.999.999` invisibile); Gregg A-BG67 (le virgole US
   di `dl59-join.out` entrano come token-scheggia legali, dentro il budget
   24.329 che oggi ha headroom 0).
5. **✓ manuali mai sweeppati a macchina** — Gregg: `WP_SESSION_91:45` e
   `GAP_TREND:101` (judge=yes) citano `20.289.946 B` senza token di grado;
   Klabnik: 894 occorrenze ≥3 cifre fuori perimetro (`.MD` maiuscolo, ROOT
   README/COVERAGE/TODO, `WP_SESSION_75`). KS-BG-94-2 ⟂ A-SK-86.
6. **Asserzioni che non possono fallire** — T16 asserisce solo `rc==1` da
   `--all`, e la baseline è già 1 in ogni sessione di concilio (untracked);
   `chk.sh` VUOTO ⇒ rc=0 ⇒ ledger «conforme» da giudice vuoto.
7. **Denti per NOME** (A-SK-87, A-AH67, KS-BG-94-2): un dente vale solo se
   asserisce la riga FAIL che NOMINA il forge, previa misura della baseline.

## CONFLITTI (registrati, non appianati)

- **CF-1 — gravità della classe «nome dal chiamante».** Hejlsberg: Q1 **NON**
  capitale, emendabile con A-AH68. Klabnik: la stessa classe su `$0` è
  **capitale** (KS-SK-94-1). Compatibili nei fatti, incompatibili nella tassa.
  Posizione del team: la gravità la decide il bite-test, non l'emendabilità —
  ma il dissenso resta di Hejlsberg e va portato in plenaria.
- **CF-2 — cosa fare del token malformato.** A-SK-83 (Klabnik): il run adiacente
  a un identificatore è **RIFIUTATO, mai cancellato**. A-BG67 (Gregg): «fusione
  **o** REFUSE». La fusione è una terza semantica silenziosa: due regole di
  corpus con esiti diversi sullo stesso byte. Da unificare su REFUSE.
- **CF-3 — dove sta il pericolo pubblicato.** Klabnik lo mette FUORI
  `php-rust/` (i file che GitHub pubblica, 11+4 cifre mai giudicate); Gregg lo
  mette DENTRO (schegge vive nel corpus a HEAD). Cumulativi, non alternativi.

## PRIORITÀ PROPOSTE per S-93.0

1. **Tether reale + identità dal contenuto** (A-SK-82 `BASH_SOURCE[0]` con
   REFUSE se vuota o ≠ `$0`; A-AH68 `BATTERY_NAME` dalla riga PASS/campo
   `battery=`). È la falla di autorità più grave: oggi il giudice non sa né
   cosa gira né cosa giudica. Denti T23 + tabella scope.
2. **Ancoraggi di campo ovunque** (A-AH65, A-AH66, A-SK-85): campi parsati con
   `( |$)`, 16/64 hex esatti, `esito` come campo unico, matcher solo su
   `phase=verdict`, A-SK57 esteso ai campaign ledger.
3. **Wiring fail-closed + bite-test end-to-end** (A-AH67): sha(chk.sh)==blob
   HEAD, `--selftest` della COPIA, canary malformato alla prima campagna m91;
   T16 riscritto per NOME (A-SK-87).
4. **Perimetro vero** (A-SK-86): ogni `.md` del REPO, regex case-insensitive,
   le 22 esclusioni per NOME nel manifest. T27 = F-K5.
5. **Corpus**: A-SK-83 + A-BG66/A-BG67 unificati su REFUSE; sanatoria
   `dl59-join.out`; **ricomputo del budget 24.329** (headroom 0 non può
   restare un caso).
6. **A-SK-84** (istante): operandi prov dalla STESSA RIGA — chiude F-K3.
7. Sanatorie doc: A-BG69 (sweep a macchina), A-BG68, A-BG70; `$DSHA` nudo a
   r.322 sotto `set -u`; divergenza `requalify` verdict/supersede dichiarata.

## S-92.0 È CONSUMABILE?

**NO come contenuto verificato — SÌ solo in ADVISORY.** Motivazione a gambe:

- **Would-have-allowed, non forge avvenuto.** Tutte e dodici le forge sono
  girate in scratch fuori repo o su fixture; nessun commit; albero porcelain.
  Nessuna sedia porta evidenza di una cifra fabbricata nel pubblicato.
- **Ma una superficie di fabbricazione è VIVA a HEAD**: le schegge-virgola di
  `dl59-join.out`, contate dentro il budget con headroom 0 (KS-BG-94-1). Non è
  una cifra falsa: è il materiale per coniarla, dentro il perimetro che
  dovrebbe chiuderlo.
- **Verifica al byte parziale.** Gregg verifica al byte ~25 cifre contro le
  righe macchina (incluse `−45.875` col segno e il «+87 esclusioni» esatto al
  suo commit); ma 2 siti judge=yes sono muti sul grado e `56 unit / 39
  message-asserting` non ha `.out` committato. Klabnik: 894 token mai giudicati
  fuori perimetro. Nessuno ha verificato l'INSIEME.

**CONDIZIONI** (tutte falsificabili a macchina):
C1 sweep per NOME dei token di grado su tutto il repo → 0 siti muti.
C2 A-BG67 morde, `dl59-join.out` sanato, budget ricomputato.
C3 le 11+4 cifre di ROOT giudicate o esentate per NOME nel manifest.
C4 T16 riscritto, baseline misurata, ogni dente nomina il proprio forge.
C5 la cifra lsp_check ottiene un `.out` o è declassata ad ADVISORY.
C6 le 6 forge di Hejlsberg falliscono per nome sul checker emendato e il canary
m91 fa FAIL end-to-end.
