# Backlog scan data-driven (fine step 37, 2026-06-15)

> Generato con assistenza AI (Claude Opus 4.8). Scan statico di frequenza sul
> corpus `.phpt` completo (`Zend/tests` + `tests` + `ext`, **21.201 file**) per
> decidere data-driven il prossimo step (metodo [[legacy-port]]: priorità per
> frequenza). Il capability-scan *dinamico* (phpt-runner) NON completa in un
> colpo perché un test in `Zend/tests` manda il tree-walker in **stack overflow**
> (ricorsione profonda) → `SIGABRT` che aborta l'intero run (vedi "Finding"
> sotto). Quindi conteggio statico via `grep` (file-count per marker di feature).

## Comando

```sh
find /tmp/php-src/Zend/tests /tmp/php-src/tests /tmp/php-src/ext \
  -name '*.phpt' > /tmp/phptlist.txt          # 21.201 file
xargs grep -lE "<pattern>" < /tmp/phptlist.txt | wc -l   # file-count
# NB macOS: `xargs -a file` NON esiste (è GNU) → leggere da stdin.
# NB `grep -r --include` rende 0 in questo ambiente → usare find+xargs.
```

## Risultati (file-count)

Calibrazione con feature **già fatte**: `enum`=211, `preg_*`=269 → danno la scala.

| Feature non implementata | File | Valutazione |
|---|---:|---|
| attributes `#[Attr]` | 307 | count più alto ma **ingannevole**: metadata inerte; il valore vero richiede Reflection → unblock reale << 307 |
| **`yield` (generators)** | 284 | **alto valore** (controllo di flusso vero); **alto rischio/sforzo**: esecuzione sospendibile in tree-walker (state-machine o thread) → design pass + sessione dedicata |
| intl (Collator/numfmt/IntlDateFormatter) | 160 | ext ICU, niche → **skip** |
| `mb_*` | 150 | **BLOCCATO** (oracle PHP senza mbstring) |
| **DateTimeZone** | 148 | medio; **serve design pass** (embedding tz-database) — chiude l'arco DateTime |
| Fiber | 145 | come generators ma più niche, stessa difficoltà |
| **nullsafe `?->`** | 111 | **piccolo, contenuto, niente design pass**, riusa la machinery di accesso prop/metodo |
| goto | 49 | controllo di flusso contenuto |
| sodium_ | 28 | crypto ext → skip |
| readonly prop | ~5+ (regex stretta) | piccolo |

## Decisione (con utente, 2026-06-15)

**SCELTO = nullsafe `?->` + argomenti nominati** (miglior valore/sforzo/rischio:
~111+ file, comunissimi nel PHP 8 moderno, nessun design pass, riusano la call
machinery esistente). Named-args non conteggiabili in modo affidabile via grep
ma notoriamente frequenti in PHP 8.

**Big swing rimandato = generators (`yield`, 284)**: massimo valore di
linguaggio, sessione dedicata con design pass ampio quando si vuole.

## Finding collaterale — stack overflow del runner

Un test in `Zend/tests`/`tests` provoca ricorsione nativa profonda nel
valutatore → `thread '<unknown>' has overflowed its stack / fatal runtime error:
stack overflow, aborting` (`SIGABRT`, exit 134). Aborta l'INTERO run del
phpt-runner (non isolato per-file). PHP stesso crasha su ricorsione infinita
senza Xdebug, quindi il comportamento di runtime non è lontano; il problema è di
**tooling** (un test cattivo uccide il batch). Possibile mitigazione futura:
limite di profondità nel valutatore con fatal PHP-like, o esecuzione per-file
isolata nel runner. Non blocca lo sviluppo (lo scan statico aggira il problema).

## Backlog ordinato residuo (post nullsafe/named-args)

1. **generators `yield`** (284) — big, design pass dedicato.
2. **DateTimeZone + tz-db** (148) — medio, design pass.
3. **goto** (49) — contenuto.
4. **trailing-capture-trimming PREG** (`preg_match_non_capture.phpt`) — piccolo.
5. **attributes** (307) — parse-and-ignore prima, Reflection poi (basso ROI).
6. BLOCCATI/skip: `mb_*` (no oracle), intl/sodium (ext niche).
