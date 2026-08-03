# Verbale SEDIA 9 — Gregg (metodologia di misura, doc==macchina per NOME, leggibilità) — Concilio WP-94

**VERDETTO: CON EMENDAMENTI. Due refutazioni capitali** (token di grado mancante su citazioni vive; schegge-virgola nel corpus del gate v3).

## Q1 — doc==macchina AL BYTE

Verificate contro le righe macchina: b_boot 2.252.800/se 6.345,5 (repair90 r37) · b_work 17.276.928/se 501.347,7→501.348 (r52) · somma 19.529.728 (r88) · delta **−45.875 col segno** (r89 `delta=-45875` — A-BG63 attuata) · b_peak(med) 20.289.946/se 1.084.655/banda [18.120.635, 22.459.256] (r87) · robustezza 0,972/0,999 (r86) · marginale 15.777.004=0,778 (r101) · invisibile 4.480.174+71.934.811 (r102) · dl59 ratio 0,090/1,887 (r109/r102 del .out) · 51 fixture "=18 v1+19 v2+8 v3+6 v4" (ds35-verify4.out r830) · budget 24329 (`gate-cifre-corpus.budget`) · rev 6910767 (`gate-cifre-revs.txt` r8) · 2.8/46.25 revocate + 110 espunta (gate r476-478, T19) · ADVISORY-PASS=64 (gate r106/1146) · **+87 esclusioni ESATTO al suo commit**: judge=no 141→228 a 2ab2780 (il mio conteggio a fine sessione dà 89: i 2 extra sono dei commit successivi — la cifra è corretta per NOME al commit A-SK-80) · A-BG65 verificato: `bite-test-historic.log` dichiara head=c71bf9e, atterraggio ce1ee27; `git rev-parse ce1ee27^`==c71bf9e ✓. **Gap dichiarato**: «56 unit / 39 message-asserting» (lsp_check) non ha riga macchina committata — autorità = `cargo test` rieseguibile, ma è l'unica cifra di WP_SESSION_92 (judge=yes) senza .out.

## Q2 — dl59-join.out

Autosufficiente e leggibile SÌ: grade=ADVISORY in testa (r2), input per path, formule SLACK per NOME (r36-46), regola d'attribuzione dichiarata, colonne di validazione (thrSumMatch, d1/d2mispl), refutazione con esempio numerico, identity check, verdetto motivato. **MA formato MISTO**: le righe machine-truth (r7-9) stampano interi nudi, le derivate stampano virgole US (`8,455,362.5`, `-32,963,974.4`, `401,467.5`, `4,049,975,808`). Il tokenizer del corpus (`(\d[\d.]*\d|\d)`) spezza alle virgole: (a) le cifre assolute derivate NON sono citabili nei judge=yes (morso in sessione: il doc cita solo i ratio); (b) **peggio: le schegge ENTRANO nel corpus come token legali** («455», «963», «974.4», «808», «528»…) e possono legalizzare cifre estranee — superficie di fabbricazione viva a HEAD, contata dentro il budget 24329. Serve la regola di formato: SÌ.

## Q3 — token di grado

MEASURE90 ✓ ovunque; NEXT_SESSION:46 ✓ («ADVISORY-RIPUBBLICATA»). **NON scala**: `sessions/WP_SESSION_91.md:45` (judge=yes!) cita «b_peak(mediana) = 20.289.946 B» SENZA token; `gaps/GAP_TREND.md:101` idem (ADVISORY solo sulla somma); riga WP-92 di GAP_TREND e WP_SESSION_92 §6 citano il verdetto dl59 (grade=ADVISORY nel .out) senza token. Il ✓ su «A-BG64 ovunque» (p4) è un ✓ MANUALE mai sweeppato a macchina.

## Q4 — lezioni con evento-prova

L1 ✓ (Scoperta 2: tether vacuo, f9bf110, «PRIMO morso di T17»). L2 ✓ (A-DL-59: naive 1,887 vs fisico 0,090, .out committato). **L3 NO**: «un if bash non distingueva un FAIL da un pass» è un controfattuale argomentato — nessun morso/commit/dente nominato in sessione lo prova.

## Emendamenti

- **A-BG66**: formato degli out macchina: token numerici SOLO cifre ASCII nude (punto decimale ammesso), separatori di migliaia US BANDITI; formato misto nello stesso .out bandito; forma umana in campo companion.
- **A-BG67**: tokenizer corpus fail-closed sulle sequenze `\d,\d` (fusione o REFUSE, estensione A-SK-76 al lato corpus) — mai schegge nel corpus; sanatoria dl59-join.out.
- **A-BG68**: A-BG62 col discriminatore per NOME (half-up ovunque; decimale citato SOLO al tie esatto .5 — oggi 501.347,7→501.348 e 6.345,5 convivono senza regola che li distingua).
- **A-BG69**: sanatoria token di grado su WP_SESSION_91:45 + GAP_TREND:101/102 + WP_SESSION_92 §6; norma estesa alle lane ADVISORY nuove.
- **A-BG70**: L3 → nominare l'evento-prova o declassarla da lezione a razionale.

## Kill-switch

- **KS-BG-94-1**: un .out sorgente-corpus contenente `\d,\d` = gate FAIL finché A-BG67 non morde.
- **KS-BG-94-2**: nessuna condizione di consumabilità spuntata ✓ senza sweep macchina per NOME delle citazioni.

## Refutazioni capitali

1. **SÌ** — «A-BG64 ovunque ✓» (condizione di consumabilità WP-93 n.2) è FALSA per NOME su 2 siti vivi (WP_SESSION_91:45, GAP_TREND:101).
2. **SÌ** — il corpus del gate v3 conia token-scheggia dalle virgole di dl59-join.out: superficie di fabbricazione dentro il perimetro che dovrebbe chiuderla.
