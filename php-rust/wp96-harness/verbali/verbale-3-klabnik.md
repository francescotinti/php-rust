# Verbale sedia 3 — Steve Klabnik (Concilio WP-96)

## VERDETTO: **A-SK-88/89/90 AGGIRATE — due forge riprodotte a macchina, una terza primitiva verificata.**

La cura di S-94.0 ha sanificato l'ambiente per **lista di negazione**
(`env -u BASH_ENV -u ENV -u SHELLOPTS -u BASHOPTS -u CDPATH -u GLOBIGNORE
$UNSET_FN`). Una blacklist non può enumerare ciò che non conosce. Tutte le
autorità di questo giudice sono **sottoprocessi `git`** e il suo valutatore è
un **processo `perl`**: né `GIT_*` né `PERL5*` sono toccati dal re-exec. La
classe del difetto è di nuovo UNA: *il giudice si fida di programmi il cui
comportamento è definito dall'ambiente del chiamante.*

Controllo positivo (albero invariato, verbali WP-96 untracked):
`bash wp81-harness/gate-measure-cifre.sh --all` → **FAIL**, 7 righe
`UNCOMMITTED … perimeter class`.

## Refutazioni capitali

**F-K10 — perimetro cieco per `core.excludesFile` (nessun byte del giudice
toccato).** Doc-forge piantato: `php-rust/wp96-harness/zzk96-forge.md`
(«il picco era 123458 B e la media 987654 B»), cifre in NESSUN corpus.

```
printf 'wp96-harness/\n' > /tmp/ex
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.excludesFile GIT_CONFIG_VALUE_0=/tmp/ex \
  bash wp81-harness/gate-measure-cifre.sh --all
```
→ **rc=0**, **zero FAIL**, `PASS gate-measure-cifre --all …
[judge_sha=27e5b75fb593d86e … head=93733821404d]`. Il giudice è il blob di
HEAD, verbatim, firmato: il perimetro A-SK-71/A-SK-80 poggia su
`ls-files --others --exclude-standard`, e l'ignore è **politica
dell'ambiente**, non autorità.

**F-K11 — self-tether A-SK-78 sconfitto da un clean filter iniettato.**
`git hash-object` applica i filtri per default; la config si inietta per
sola env:
```
perl -pe 's/\$all_rc = 1;//g if /UNCOMMITTED/' wp81-harness/gate-measure-cifre.sh \
  > wp81-harness/zzk96-judge.sh          # sha reale 08a573e4e0e8a687
GIT_CONFIG_COUNT=2 GIT_CONFIG_KEY_0=core.attributesFile GIT_CONFIG_VALUE_0=/tmp/attrs \
  GIT_CONFIG_KEY_1=filter.pristine.clean GIT_CONFIG_VALUE_1="cat …/gate-measure-cifre.sh" \
  bash wp81-harness/zzk96-judge.sh --all
```
(`/tmp/attrs`: `zzk96-judge.sh filter=pristine`) → **rc=0**, nessun REFUSE
A-SK-78, `PASS --all` con **judge_sha pristino 27e5b75fb593d86e** mentre gira
testo PATCHATO. Primitiva isolata: `hash-object` del file patchato restituisce
`27e5b75f…` invece di `a42354d9…`.

**F-K12 (primitiva verificata) — `PERL5OPT`/`PERL5LIB` dentro il valutatore.**
`PERL5LIB=… PERL5OPT=-MHack perl - a b <<'P'` esegue il `BEGIN` del modulo
ostile nel processo che COSTRUISCE il corpus, e sopravvive intatto a
`/usr/bin/env -u BASH_ENV … /bin/bash -p`. Da lì `CORE::GLOBAL::exit`, `qx`,
`%corpus` sono del chiamante.

**Canale CHIUSO (refutazione mia, riportata):** `BASH_FUNC_x%%` con corpo non
`() {` (subshell, newline) — il bash 3.2 Apple **rifiuta** l'import: la regex
di `FN_ENV_NAMES` non è aggirabile per questa via su questa macchina.

## Emendamenti

- **A-SK-93** — l'ambiente si **costruisce**, non si sottrae: re-exec via
  `env -i` con lista CHIUSA (PATH fisso, HOME, LANG, TMPDIR). Una blacklist è
  vacua per costruzione.
- **A-SK-94** — ogni `git` del giudice passa da un wrapper unico con
  `GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null`.
- **A-SK-95** — `hash-object --no-filters` ovunque si tetheri (shell **e**
  `work_blob_sha`); meglio: sha calcolato sui byte letti, mai delegato.
- **A-SK-96** — gli untracked del perimetro si elencano **senza**
  `--exclude-standard`, sottraendo solo i `.gitignore` COMMITTATI a HEAD.
- **A-SK-97** — `perl` invocato con `-T` o con `PERL5OPT/PERL5LIB/PERLLIB/
  PERL5DB` provati assenti nel predicato di stato.

## Kill-switch

- **KS-SK-96-1 (T27)** — F-K10: rc **esatto 1**, mai `PASS --all`; morso sul
  giudice pre-cura (rc=0 + PASS firmato).
- **KS-SK-96-2 (T28)** — F-K11: REFUSE per nome rc=1; morso pre-cura rc=0 con
  judge_sha pristino.
- **KS-SK-96-3 (T29)** — F-K12: REFUSE per nome; morso pre-cura.
- **KS-SK-96-4 (T30, il solo che non invecchia)** — dopo il re-exec il giudice
  confronta il proprio `env` con la lista chiusa: **una variabile in più =
  REFUSE**. Senza T30, WP-97 mi ritrova qui con un prefisso nuovo.

*Residui rimossi (`zzk96-judge.sh`, `zzk96-forge.md`); nulla committato.*
