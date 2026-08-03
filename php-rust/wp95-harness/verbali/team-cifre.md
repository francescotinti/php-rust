# team-cifre — fase 2, Concilio WP-95 (relatore)

Sedie del team: Klabnik (3), Hejlsberg (4), Hoare (1, parte identità/gate).
Mandato: riconciliare o registrare i dissensi. Nessuna benedizione.

---

## 1. Convergenze

**C1 — La classe comune delle refutazioni di identità.** Le tre vie di
Klabnik (F-K1/K2 `BASH_SOURCE` da env; F-K3/K4 symlink + risoluzione
LOGICA; F-K7 `BASH_ENV`) e il capitale di Hejlsberg (Q4c, append
`writer=script:<h16>` verificato in FORMA) sono lo **stesso vizio**:

> **il giudice autentica STRINGHE che il chiamante sceglie, mai gli
> ARTEFATTI che il kernel legge ed esegue.**

Declinato:
- `$0` è un **nome fornito dal chiamante**, non il file aperto (F-K3/K4:
  `HERE="$(cd "$(dirname "$0")" && pwd)"` collassa `L/..` lessicalmente);
- `BASH_SOURCE[0]` sotto bash 3.2 è **dato d'ambiente**, non il registro
  di bash (verificato: `declare -x BASH_SOURCE="…"`, scalare esportato);
- l'**ambiente** che dà significato ai comandi del testo non è legato da
  nulla (F-K7: `perl` diventa una funzione del chiamante);
- lato ledger, `writer=script:<h16>` è una **forma di stringa**, non
  un'origine (A-AH-71).

Corollario condiviso: A-SK-78 → A-SK-82 hanno inseguito il vizio di un
livello alla volta (prima il nome, poi il registro del nome) senza mai
cambiare **classe** di ancoraggio. Ogni patch che resta sul TESTO o sul
NOME sarà aggirata dal livello successivo (KS-SK-95-2).

**C2 — FONDAMENTALI-first, l'oggetto è la leva.** Tutte e tre le sedie
mettono al primo posto la leva arene per-file del preludio con gate di
parità COMPLETI, e nessuna propone gate nuovi. Hoare 2), Hejlsberg 1),
Klabnik (a).

**C3 — battery61 riproducibile in modo nativo** (criterio 5): Hoare 1),
Klabnik (b), Hejlsberg 3). Concorde.

**C4 — vietato adattare i gate alla leva** (KS-TH-95-2): nessun dissenso.

**C5 — Un dente che prova il TESTO non prova il COMPORTAMENTO.**
A-SK-91 (giudice eseguito con `perl`/`git` dirottati) e A-AH-70/K
(`--selftest-identity` esteso al CONSUMO, non al predicato del nome)
sono la stessa forma su due oggetti diversi: nessuno dei due è ridondante.

---

## 2. Conflitti (posizione per sedia)

**K1 — L'apparato entra o no nell'ordine S-94.0?** *(conflitto reale)*
- **Klabnik**: SÌ, terzo posto, «SOLO A-SK-88/89/90 in un'ora, perché
  senza quelli ogni PASS futuro è firmabile da chiunque — poi congelato».
- **Hejlsberg**: NO — «apparato congelato (condizione 4), gli emendamenti
  restano A VERBALE, si attuano nella prossima finestra apparato».
- **Hoare**: «SOLO se blocca (condizione 4), **nessun gate nuovo**».
- **Composizione del relatore**: entra, per il criterio di Hoare, non
  malgrado esso. (i) Non è un gate NUOVO: è la riparazione di un gate
  esistente, misurata in poche righe. (ii) **Blocca l'oggetto**: le cifre
  che S-94.0 produrrà sulla leva sono consumate a rc=0 di
  `gate-measure-cifre --all`; se quel rc=0 è producibile da un canale
  scelto dal chiamante, la misura della leva **non ha autorità** —
  non è apparato per l'apparato, è l'autorità del numeratore. Il dissenso
  di Hejlsberg è registrato e resta valido come **vincolo di timebox**,
  non come veto.

**K2 — Grado PROBE (rc=65) per misure senza dispersione.** *(conflitto)*
- **Klabnik**: serve un grado sotto ADVISORY per B3 e per il rapporto CLI.
- **Hoare**: «nessun gate nuovo».
- **Composizione**: il PROBE è **espansione** d'apparato, non riparazione
  → **BACKLOG per NOME (A-SK-92-PROBE)**. In S-94.0 la sostanza di
  Klabnik si ottiene a costo zero per via **lessicale**: una misura a un
  run per braccio si scrive «indistinguibile dal rumore», mai «delta
  nullo». Questo è vincolante da subito e non tocca una riga di codice.

**K3 — Numeratore della leva: touched o capacità?** *(coppia non risolta)*
- **Hoare**: A-TH-75 pinna il numeratore al **touched ≈25,8 MB**;
  KS-TH-95-3 annulla ogni predizione firmata con 39.534.144.
- **Hejlsberg**: emenda con `Bump::with_capacity` per unit **proprio per
  recuperare i 13.738.592 B di coda mai toccata** — cioè esattamente la
  differenza capacità−touched.
- **Composizione**: non è contraddizione ma **accoppiamento non
  dichiarato**: con il pre-size, una parte del risparmio viene da capacità
  mai toccata e NON è predicibile dal touched. Vincolo: **A-AH-72 deve
  emettere entrambe le grandezze per-unità** (`allocated=` capacità E
  touched), e la predizione-misurata WP-48 va firmata come **DUE
  predizioni separate** (touched; coda di capacità), mai una sola.
  Altrimenti scatta KS-TH-95-3 sul residuo.

**K4 — Collisione di NUMERAZIONE (registrata come difetto di verbale).**
`A-AH-69` e `A-AH-70` esistono in DUE significati diversi:
- Klabnik: A-AH-69 = ancorare la glob dell'esenzione pre-ledger
  (`battery-8[0-8]pre`); A-AH-70 = `--selftest-identity` esteso al consumo.
- Hejlsberg: A-AH-69 = `.done` parsato per-RIGA; A-AH-70 = ancora
  `sha256=…` sul triangolo + grammar sulle righe PASS; A-AH-71 = writer
  autenticato.
- **Disambiguazione proposta al plenario**: il blocco di Hejlsberg
  (69→73, contiguo e già esteso a 72/73 senza collisione) **conserva** i
  numeri; i due di Klabnik diventano **A-AH-74** (ancoraggio glob) e
  **A-AH-75** (selftest-identity sul consumo). Nessun ID riusato.
  *Un registro con due significati per lo stesso ID è un registro rotto:
  questa non è pedanteria, è la stessa classe di C1 (l'etichetta non è
  la cosa).*

**Non-conflitti** (registrati per completezza): la refutazione aritmetica
di Hoare (Q2, commento `lower/mod.rs:1013-1015`) non è contestata da
nessuno; i due hazard di Hoare (`var_os` nell'alloc-path, `thread::current()`
nel `GlobalAlloc`) non sono toccati dalle altre due sedie e sono
riparazioni di UB/deadlock, non apparato.

---

## 3. Cura minima ordinata

**Esiste UNA cura che chiude tutti e tre i canali di Klabnik insieme.**
Non è la somma delle tre toppe: è il **cambio di classe di ancoraggio**
richiesto da C1 — smettere di legare il nome, ristabilire il contesto
d'esecuzione sotto controllo del giudice PRIMA di qualunque lavoro.

**Nucleo: A-SK-90 (re-exec sanificante) su path FISICO (A-SK-89).**
Primo atto eseguibile del giudice:

```
exec env -u BASH_ENV -u ENV -u SHELLOPTS bash -p "$SELF_PHYS" "$@"
```

Perché chiude i tre canali con un colpo solo:
- **F-K1/K2** (`bash -c` + `BASH_SOURCE` iniettato): dopo il re-exec il
  testo **non viene più dal chiamante** ma dal file a `$SELF_PHYS` — il
  testo patchato muore all'exec, la corsa prosegue sul giudice pristino;
- **F-K3/K4** (symlink logico): `SELF_PHYS` risolto con `cd -P … && pwd -P`
  nomina il file che il kernel apre davvero → il tether A-SK-78 confronta
  lo sha del file PATCHATO, REFUSE;
- **F-K7** (`BASH_ENV`, funzioni esportate): `env -u` + `bash -p`
  neutralizzano sia il file di startup sia le funzioni ereditate
  (**verificato a macchina**, §5).

**Ordine di atterraggio — UN SOLO COMMIT, in questa sequenza:**

1. **A-SK-89** *(indispensabile, precondizione)* — `HERE`/`SELF_ABS`/
   `SELF_PHYS` **fisici** (`cd -P`, `pwd -P`). Senza questo il re-exec
   ri-esegue un omonimo e il tether continua a firmare un nome.
2. **A-SK-90** *(indispensabile, il nucleo)* — re-exec sanificante come
   **primo atto**, prima di `git`, `perl`, `HERE`, di qualunque lettura.
   ⚠️ **Hazard nominato dal relatore, da progettare nello stesso commit**:
   il marker anti-loop non può essere una env var che il chiamante
   pre-imposta per saltare il re-exec — sarebbe C1 di nuovo, un livello
   più in là. Il marker va **validato**, non solo letto: se il marker è
   presente ma `BASH_SOURCE` non è un array che nomina `SELF_PHYS` con
   sha == blob HEAD, **REFUSE**.
3. **A-SK-88** *(non ridondante, ma declassato: da blocco a asserzione)* —
   `declare -p BASH_SOURCE` deve iniziare con `declare -a`. Dopo il
   re-exec `BASH_SOURCE` è un array genuino, quindi 88 **non blocca più
   nulla da solo**: serve (a) a rendere SANO il marker del punto 2, (b) a
   morire con un NOME anziché per effetto collaterale. Verificato che
   discrimina esattamente (`declare -a` normale vs `declare -x` iniettato).
4. **A-SK-91** *(indispensabile — è il falsificatore)* — dente che esegue
   il giudice con `perl`/`git` dirottati e con i tre canali, e pretende
   il **rc ESATTO**. Senza 91 la cura è una promessa: KS-SK-95-3 (dente
   che copre un canale e non la sua variante d'ambiente = sotto-portata).
   Assorbe la sotto-portata di **T23** denunciata da Klabnik: arm-b deve
   asserire l'assenza dell'**escalation a rc=0 firmato**, non solo il 64.

**Ridondanze/sovrapposizioni dichiarate:**
- A-SK-88 è **ridondante come blocco**, necessario come guardia del marker
  e come nome dell'errore → resta, ma non è quello che chiude i canali.
- **A-AH-74** (ex Klabnik A-AH-69, ancoraggio glob) e **A-AH-75** (ex
  A-AH-70, selftest sul consumo) NON si sovrappongono a nulla di
  Hejlsberg: oggetti diversi (esenzione di scope; predicato di consumo).
- Sul lato Hejlsberg: **A-AH-71** (autenticare `writer=` contro lo sha del
  battery a HEAD) **sussume il rischio** che A-AH-70/H (ancore `sha256=`
  + grammar sulle righe PASS) mitiga soltanto → 71 prima, 70 dopo.
- **A-AH-69/H** (`.done` per-RIGA) non è ridondante con 71: verificato che
  oggi i `.done` reali sono **a riga singola** (`m90.done`, 97 B, 1 riga)
  MA il parse è `grep -q "^rev=$BREV "` accoppiato a `sed … | head -1`:
  con un `.done` a due righe i 4 campi nascono da righe diverse. Il
  `.done` è scritto dal battery, cioè interamente dal forgiatore.

---

## 4. Priorità S-94.0 (FONDAMENTALI-first, timebox mezza sessione d'apparato)

**Regola applicata** (utente 2026-08-03): un emendamento d'apparato entra
SOLO se blocca il prossimo passo sull'OGGETTO. Discriminante usata:
*la cifra che S-94.0 produrrà passa da questo percorso?*

### DENTRO la mezza sessione d'apparato (tetto duro; se sfora, si taglia dal fondo)

| # | Emendamento | Perché BLOCCA l'oggetto | Costo |
|---|---|---|---|
| A1 | **A-SK-89 + A-SK-90 + A-SK-88 + A-SK-91** (blocco unico, ordine §3) | Le cifre della leva sono consumate a rc=0 di `--all`; oggi quel rc=0 è **fabbricabile** (verificato §5) ⇒ ogni numero di S-94.0 nascerebbe senza autorità | ~30 righe + 1 dente |
| A2 | **A-AH-71** (writer autenticato contro sha del battery a HEAD) | battery-91pre / battery61 sono l'oggetto di S-94.0 e si consumano per quel percorso; è **una** riga: forma → origine | 1 riga |
| A3 | **A-AH-69/H** (`.done` per-RIGA: i 4 campi dalla riga che porta `rev=$BREV`; più righe `rev=` ⇒ REFUSE) | stesso percorso di consumo di A2; ancora `sed` già presente, cambia solo il pattern | 2 righe |
| A4 | **A-TH-73 + A-TH-74** (env-read fuori dall'allocatore; via `thread::current()` dal `GlobalAlloc`) | **non è apparato**: è un deadlock latente e un panic-path che è UB da contratto nel percorso che genera le misure | 2 righe |

*Se il timebox si esaurisce, l'ordine di taglio è A3 → A2: il blocco A1
non si taglia. Motivo: A1 chiude canali che un operatore può imboccare
**senza volerlo** (un `bash -c`, un symlink nel path); A2/A3 richiedono
una fabbricazione deliberata.*

### FUORI dalla mezza sessione — BACKLOG PER NOME (non «più avanti»: per nome)
- **A-SK-92-PROBE** — grado PROBE rc=65 sotto ADVISORY (K2). *Sostituito
  in S-94.0 dalla regola lessicale «indistinguibile dal rumore».*
- **A-AH-70/H** — ancore `sha256=[0-9a-f]{64}( |$)` sul grep del triangolo
  e grammar-anchor esteso alle righe PASS.
- **A-AH-74** (ex Klabnik A-AH-69) — `battery-8[0-8]pre` ancorato.
  *Diventa bloccante se S-94.0 gira una batteria con nome a tre cifre 8x0-8x8.*
- **A-AH-75** (ex Klabnik A-AH-70) — `--selftest-identity` esteso al CONSUMO.
- **A-AH-73** — dente HIR plain-data (precondizione della via precompilata,
  che è leva #2, non #1).

### L'OGGETTO (ciò per cui si fa la sessione — non conta nel timebox)
1. **Leva arene per-file del preludio + pre-size**, con i 7 obblighi di
   prova di Hoare (A-TH-76), il contatore per-unità **A-AH-72 PRIMA**
   (KS-AH-95-1) emesso **su due grandezze** (K3), gate parità COMPLETI e
   corpus per NOME nello **stesso commit**. Numeratore pinnato da
   **A-TH-75** (touched ≈25,8 MB) — la predizione a 39.534.144 è NULLA.
2. **Misura CLI hello/refl post-leva vs oracle**: il 4,42× deve muoversi.
3. **battery61 riproducibile in modo nativo** (criterio 5).

---

## 5. Verifica eseguita (a macchina, HEAD `a9a1b364`, nulla committato)

Scelta: **la catena F-K2 di Klabnik** — la più grave, perché è l'unica che
produce un `PASS … --all` **verdict-grade, rc=0, firmato col judge_sha
pristino, mentre gira testo patchato**. Le altre due degradano o
fabbricano una riga; questa **firma**.

**Baseline pristina.**
```
$ bash wp81-harness/gate-measure-cifre.sh --all      # 25,5 s
rc=1   (9 × FAIL … UNCOMMITTED php-rust/wp95-harness/verbali/verbale-*.md)
$ git rev-parse -q --verify HEAD:php-rust/wp81-harness/gate-measure-cifre.sh
2f37f386d153d6ea6fe4f86b2d26e85b953ac2e3     (== git hash-object del working tree)
```

**Primitiva del canale** (bash 3.2.57 arm64-apple-darwin25):
```
$ BASH_SOURCE=/tmp/pristine bash -c 'declare -p BASH_SOURCE' /tmp/pristine
declare -x BASH_SOURCE="/tmp/pristine"          ← scalare ESPORTATO
```
⇒ la guardia A-SK-82 `[ "$SELF_SRC" != "$0" ]` è **soddisfatta**: entrambi
valgono la stringa scelta dal chiamante.

**Forge.**
```
$ perl -pe 's/\$all_rc = 1;//g if /UNCOMMITTED/' "$SELF" > $SCR/patched.sh   # 2 righe cambiate
$ BASH_SOURCE="$SELF" bash -c "$(cat $SCR/patched.sh)" "$SELF" --all
FORGE rc=0
```
Output riga 265:
```
PASS gate-measure-cifre --all (A-SK64/A-SK-67): manifest perimeter,
bidirectional, authorities from HEAD [judge_sha=2f37f386d153d6ea
manifest_sha=f2ebfa986614710a budget_sha=34dda74c63b87eba
revs_sha=7797b9d7ebd5b880 head=a9a1b3646e6f]
```
Nessuna riga `REFUSE`, nessuna `NOTE` A-SK-78/A-SK-82.

**ESITO: forge CONFERMATO, e più grave di come è riportato nel verbale 3.**
Il rc passa da 1 a 0 e la riga PASS firmata **coesiste nello stesso output
con le nove righe FAIL** che il patch ha reso non-fatali: chi consuma il
rc, o la riga terminale, legge PASS verdict-grade su una campagna che il
giudice pristino aveva bocciato. Il tether A-SK-78 **firma il blob
pristino di codice che non è quello che è girato** — precisamente
KS-SK-95-1.

**Controprova delle cure proposte** (stesse invocazioni, oggetti-giocattolo):
```
A-SK-88  normale:   declare -a BASH_SOURCE='([0]="…")'     ← discrimina
         iniettato: declare -x BASH_SOURCE="…"                   REFUSE
A-SK-90  BASH_ENV=evil bash  v.sh  → HIJACK     (canale F-K7 aperto)
         BASH_ENV=evil bash -p v.sh → REAL      (canale F-K7 CHIUSO)
```
A-SK-89 non è stato riprodotto end-to-end (servirebbe un symlink nel
repo): resta **verificato da Klabnik**, non dal relatore — lo dichiaro.

Nessun commit, nessun file toccato nel repo; tutti i temporanei
(`patched.sh`, `p.sh`, `evil.sh`, `v.sh`, `L`, `realdir`, `*.out`)
cancellati dallo scratchpad a fine verifica.

---

## Riepilogo del relatore

Non c'è nulla da benedire. Il gate cifre v3 + A-SK-82 **non è
verdict-grade**: l'ho rifatto io a HEAD e ho ottenuto rc=0 firmato. La
cura non è un'altra toppa sul nome — è il **re-exec sanificante su path
fisico**, che sposta l'ancoraggio dal nome all'artefatto; A-SK-88 e
A-SK-91 la rendono rispettivamente sana e falsificabile. Entra in S-94.0
non perché sia bello avere apparato, ma perché **senza di essa nessuna
cifra prodotta da S-94.0 sulla leva sarà un'autorità**: mezza sessione,
tetto duro, poi congelato. Tutto il resto va a backlog per nome.
