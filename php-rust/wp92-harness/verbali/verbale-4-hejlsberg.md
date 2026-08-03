# Verbale sedia 4 — Hejlsberg (Concilio WP-92, su S-90.0)
Perimetro: catene di evidenza (attempts/stamps/matrix/judge), identità toolchain.

## VERDETTO: CONCORDO CON EMENDAMENTI

Tutte le verifiche sono state eseguite A MACCHINA su HEAD=dd6cdce.

## Q1 — Le judge_sha g1/g2 risolvono a blob committati? SÌ (verificato)
- g1: `git show 5d3ec7b:php-rust/wp90-harness/verdict90.sh | shasum -256 | cut -c1-16`
  = `1b1e9e96f5019bc0` == judge_sha della riga g1. ✓
- g2: blob a `564e7ac` = `97a8eff0d9783ee4` == judge_sha g2 == blob a HEAD. ✓
- checker_sha della riga phase=consume: blob `battery-equivalence.sh` a bb4b388
  = `0f62beed576f6298`. ✓  Entrambi i commit sono antenati di HEAD; il
  campaign ledger working copy è IDENTICO a HEAD; `verdict90.a1.g2.out` è
  tracked. Il self-tether A-BG53 ha retto: la catena giudici è chiusa.

## Q2 — La riga ABORT è distinguibile da una riga di script? NO (refutazione)
`battery-90pre.sh` (`att_row`, r.33-34) emette SOLO `esito=REFUSE|PASS|FAIL`.
La riga `esito=ABORT reason=head-moved-mid-battery-…-operator-error`
(epoch 1785745856, rev=8da340c) è quindi PER COSTRUZIONE scritta a mano —
ma con grammatica IDENTICA (`attempt_epoch= battery= rev= esito=`). Nessun
campo `writer=`, nessuna firma: la provenienza è una convenzione non
falsificabile. Un operatore potrebbe scrivere `esito=REFUSE` dove il run fece
FAIL e nessun dente morderebbe — il prefix A-AH54 protegge la storia
COMMITTATA, non autentica l'append. Il `reason=` che si auto-dichiara
"operator-error" è onestà volontaria, non un vincolo.

## Q3 — Il triangolo sha256==DSHA copre anche i FAIL? NO (buco confermato)
Il triangolo A-AH54 chiude SOLO il PASS: verificato al byte
sha256(OUT)=`e81ce60d…` == `.done` == stamps ledger == riga attempts PASS. ✓
Ma le righe FAIL (`fails=1/16 first_fail=measure-cifre`) e REFUSE non portano
ALCUNO sha: il `first_fail` è un'asserzione non riverificabile ex-post (l'OUT
del FAIL vive fuori repo in `/Volumes/…/wp90-battery-out/`, mutabile, non
ancorato). Non tocca la consumabilità (si consuma solo il PASS), ma è
esattamente la classe di forgia che A-AH54 ha chiuso sul PASS.

## Q4 — La finestra 4c99520..bb4b388 era pulita? SÌ (verificato)
`git diff --name-only 4c99520..bb4b388` = esattamente 3 path, tutti
allowlistati A-SK50: stamps ledger, attempts ledger, e il SOLO matrix archive
nominato dal `.done` (`feature-matrix.4c99520.20260803-104804.log`,
matrix_sha256 combacia). Delta attempts in finestra = la sola riga PASS
appesa (diff `11a12`, append a granularità di riga). Finestra pulita.

## Osservazione minore (grammatica ledger campagna)
Le righe `phase=verdict` g1/g2 NON portano `campaign_sha=` (presente su
preflight/start/consume): il verdetto è legato all'attempt solo per numero.

## Emendamenti
- **A-AH58 "writer= sulla riga attempts"**: ogni riga porta
  `writer=script:<sha16(script)>` (emessa da att_row) o `writer=operator`;
  `esito=ABORT` legale SOLO con `writer=operator`; lo script guadagna un trap
  che emette ABORT da sé su segnale/HEAD-move. Il checker rifiuta esiti fuori
  {PASS,FAIL,REFUSE,ABORT} e ABORT senza writer=operator.
- **A-AH59 "triangolo sui FAIL/REFUSE"**: att_row FAIL/REFUSE porta
  `sha256=<sha256(OUT-parziale)>` — ogni esito ancorato al proprio OUT, non
  solo il PASS.
- **A-AH60 "campaign_sha sulle righe verdict"**: verdict90.sh append include
  `campaign_sha=` letto dal ledger di campagna (riga phase=start).

## Kill-switch
- **KS-AH-92-1**: dopo A-AH58, una riga attempts senza `writer=` valido ⇒
  la successiva consumazione same-rev è VOID.
- **KS-AH-92-2**: dopo A-AH59, una riga FAIL/REFUSE senza ancora sha ⇒
  battery VOID (mai "assente con motivo").

## Refutazioni capitali: NESSUNA
La catena consumata (PASS→stamp→OUT→matrix→toolchain→judge g2) regge a
macchina su ogni maglia. I due buchi (provenienza ABORT, FAIL non ancorati)
sono laterali alla consumazione ma vanno chiusi in S-91.0.
