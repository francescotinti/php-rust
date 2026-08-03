# Verbale sedia 3 — Klabnik (forge) — Concilio WP-95

## VERDETTO

**S-93.0 p.A è REFUTATO nel merito: A-SK-82 non chiude nulla.** Ho
fabbricato a HEAD, con la sola shell di sistema, **tre** vie che stampano
`PASS gate-measure-cifre --all … judge_sha=2f37f386d153d6ea` con rc=0 —
la firma del blob PRISTINO su codice che non è quello. Il gate cifre v3
+ A-SK-82 non è verdict-grade contro un avversario; è una porta con
tre finestre aperte. p.B/p.C restano in piedi (nessun forge li tocca).

## Forge eseguiti (comando · rc atteso · rc reale)

| # | canale | atteso | reale |
|---|---|---|---|
| F-K1 | `BASH_SOURCE=$SELF bash -c "$(cat $SELF)" $SELF <doc>` (testo pristino) | 1 REFUSE | **64 ADVISORY-PASS** — guardia passata |
| F-K2 | idem, testo patchato (`s/\$all_rc = 1;//` sul dente UNCOMMITTED), `--all`, doc-forge non committato in sessions/ | 1 | **0 + `PASS … --all` firmato**, zero righe A-SK-78/82 |
| F-K3 | symlink `L→dir` + `bash "$REPO/L/../wp81-harness/gate-measure-cifre.sh"` (testo patchato), advisory | 1/NOTE | **64, nessuna NOTE A-SK-78** |
| F-K4 | idem `--all` | 1 | **0 + `PASS … --all` firmato** |
| F-K5 | sourcing annidato (`. $SELF` da altro script) | 1 | 1 REFUSE — **gate tiene** |
| F-K6 | `exec -a <pristino> bash <patchato>` | 1 | bash riscrive `$0` col file: nessun guadagno — **respinto** |
| F-K7 | `BASH_ENV=evil bash $SELF --all`, `evil` definisce `perl(){ echo PASS…; }` — giudice pristino, path pristino | 1 | **0 + riga PASS fabbricata** |
| F-K8 | battery-equivalence: OUT con riga terminale `== BATTERY-88[0-9]PRE PASS …` + basename coerente | disciplina attempts attiva | **A-AH50 NON scatta** — esenzione pre-ledger comprata |
| F-K9 | idem con nome `88pre` coerente: consumo | 1 | 1 — lo **stamp a 5 campi committato** regge (l'identità A-AH68 no) |

Meccanica: `BASH_SOURCE` è iniettabile **dall'ambiente** sotto `bash -c`
(bash 3.2: diventa scalare esportato, `declare -x`, non array). `$HERE`
usa `cd`/`pwd` **logici**: `L/..` collassa lessicalmente sul giudice
pristino mentre il kernel legge il file patchato — `SELF_ABS` firma un
omonimo, esattamente il vizio che A-SK-78 doveva chiudere, un livello
più in là. `BASH_ENV` è il colpo di grazia: **nessun tether sul TESTO
può salvare un giudice il cui AMBIENTE è scelto dal chiamante**.

## Emendamenti

- **A-SK-88** — `declare -p BASH_SOURCE` deve iniziare con `declare -a`:
  uno scalare esportato è un'iniezione, REFUSE. (verificato)
- **A-SK-89** — `HERE`/`SELF_ABS` **fisici**: `cd -P … && pwd -P`; il
  forge F-K3/K4 muore lì (verificato: HERE fisico ≠ logico).
- **A-SK-90** — re-exec sanificante come primo atto: `exec env -u
  BASH_ENV -u ENV -u SHELLOPTS bash -p "$SELF_PHYS" "$@"` con marker
  anti-loop. `bash -p` neutralizza BASH_ENV **e** le funzioni esportate
  (verificato: HIJACK → REAL).
- **A-SK-91** — il tether va provato sul **comportamento**, non sul
  testo: dente che esegue il giudice con `perl`/`git` dirottati e pretende
  REFUSE.
- **A-AH-69** — esenzione pre-ledger ancorata: `battery-8[0-8]pre` (oggi
  `battery-8[0-8]*` regala l'esenzione a ogni batteria a tre cifre 8x0-8x8).
- **A-AH-70** — `--selftest-identity` estenda i casi al **consumo**
  (scope), non solo al predicato del nome.

## KS
- **KS-SK-95-1**: PASS verdict-grade prodotto per canale non sanificato ⇒ campagna VOID.
- **KS-SK-95-2**: path del giudice risolto logicamente ⇒ tether vacuo.
- **KS-SK-95-3**: dente che copre un canale e non la sua variante d'ambiente ⇒ dente sotto-portata.
- **KS-SK-95-4**: esenzione di scope da glob non ancorato ⇒ disciplina saltata in silenzio.

## Risposte

1. **T23 non è vacuo ma è sotto-portata da entrambi i lati**: arm-a prova
   il `-c` NUDO (la variante con env passa, F-K1); arm-b prova che il
   forge riproduce solo a rc=64 — non asserisce mai l'escalation a rc=0
   firmato, che è l'unica cosa che conta (F-K2/K4). Lo strip a marker
   fallisce rumorosamente se la guardia viene riscritta: quello va bene.
2. **Cifre di p.B**: B1/B2 (sei siti nominati dal backtrace, sei dealloc)
   sono **dimostrative**, non statistiche — consumabili come ADVISORY
   piene. B3 no: un run per braccio, due punti W, e il **rumore
   build-to-build dichiarato nello stesso .out è maggiore del delta
   rivendicato** — «delta nullo» va riscritto «indistinguibile dal
   rumore». Serve un grado **PROBE sotto ADVISORY** (rc=65, coerente con
   A-SK-79) per misure senza dispersione; il rapporto CLI phpr/oracle sta
   lì, magnitudine.
3. **S-94.0, FONDAMENTALI-first**: (a) **leva arene per-file del
   preludio** con gate parità completi — è l'oggetto, quantificato; (b)
   **battery61 riproducibile** (criterio 5); (c) apparato SOLO A-SK-88/89/90
   in un'ora, perché senza quelli ogni PASS futuro è firmabile da chiunque
   — poi congelato.

**Refutazioni capitali: SÌ (3).**
