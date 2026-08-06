# Team ARBITRI (Hoare + Matsakis) — Concilio WP-106, fase 2
Relatore: Hoare. Fonti: verbale-1-hoare.md, verbale-2-matsakis.md.

## (a) 19a/19b — posizione unica
Convergenza piena. Matsakis: le due mutazioni S-104 erano specchi della LEVA,
non dell'osservabile delle fixture; bastano a DEMOTERE (fail-closed corretto,
RC-MA-104 aperto), non a condannare. La decisione passa dalla **TERZA
mutazione mirata a OBS-8/holder-esterno** (es. −2→−1 nel confronto, o handle
del braccio non contato come holder): rossa ⇒ 19a/19b restano arbitri del
MOVE (mai di H-C2); non rossa ⇒ riclassifica per NOME nell'header di
`s103-recv-fixtures.sh` come «regressione byte-parity, NON arbitro di
meccanismo». Hoare ritira il braccio-rosso generico WP-105 in favore di
questa forma: è lo stesso principio, ma accoppiato all'osservabile
dichiarato (KS-MA-106-2 lo sigilla). ~30′, in ordine S-105.
Corollario congiunto: i due mutanti sopravvissuti vanno DISPOSIZIONATI
(equivalente / coverage-gap / ridondanza-da-nominare, A-MA-106-2) — «la
sweep compensa» resta ipotesi finché non verificata; inquietante perché il
collector gira MID-statement, prima della sweep.

## (b) fx20 — forma finale del gate RSS
Adottata integralmente A-MA-106-3 (Hoare concorda: un cap fisso da N=1 è una
banda non pre-registrata — stessa lezione S-103). Il gate diventa:
1. cap = BANDA derivata pre-registrata (mediana clean R≥3 + rumore; floor
   dal mutante totale 301);
2. guardia d'erosione: clean ≥ cap/2 ⇒ VOID, mai PASS silenzioso;
3. mutante di sensibilità PARZIALE: leak del solo sentiero Pop (~+25-30 MiB)
   deve scattare — oggi passerebbe sotto 150;
4. nota nel gate: `/usr/bin/time -l` = macOS-only, ru_maxrss in BYTE;
   altrove ⇒ rss vuoto ⇒ VOID esplicito (oggi fail-closed per fortuna).
KS-MA-106-1 congiunto: nessuna fixture pinnata fonda il verdetto su
`memory_get_usage` finché è stub (grep nel gate ⇒ VOID); contatore vero =
backlog per NOME.

## (c) Sigilli is_trivial_drop — ordine vs backlog
**Ordine S-105** (minuti, safe-only): A-HO-106-1 sigillo di TIPO
(`fn _seal<T: Copy>()` su bool/i64/f64 accanto al size/align-assert — il
match esaustivo non copre il cambio di payload); A-HO-106-2 doc col verdetto
S-104 (leva caduta, canale refutato, → hc2-ab-verdetto.out); A-HO-106-3
correzione registro «al byte» → «taglia+timing». Vincolanti subito:
KS-HO-106-2 (nuovo chiamante di produzione senza criterio pre-registrato =
reject) e KS-HO-106-1 (leva che cita «icache-bound» senza contatore né
controprova = premessa VOID).
**Backlog**: hash del range run_loop revert-vs-pin (alla prossima occasione
di disasm); xctrace L1i-miss sul paio archiviato.

## (d) H-ICS «cold-out» vs leva args-Vec (fronte Bak/Leijen/Gregg)
Nessun conflitto di merito, conflitto di SLOT. Posizione del team:
**args-Vec su calls resta leva #1 di S-105** — canale misurato (1×32 B/chiamata,
H-D), nessuna premessa contesa. H-ICS è leva #2 O braccio parallelo se il
budget regge: il suo valore è duale (Δ>0 = guadagno; Δ≤0 = KS-HO-106-1
archivia la tesi icache), quindi non è mai cieca — ma NON deve spostare
args-Vec, e il suo prefisso (disasm prima/dopo + bl-count, protocollo S-104)
non è comprimibile. Se la sessione regge una sola leva: args-Vec; H-ICS
eredita lo slot successivo con criterio già firmato qui.

## Ordine S-105 proposto dal team
1. Terza mutazione 19a/19b + disposizione sopravvissuti (~45′).
2. Sigilli is_trivial_drop (A-HO-106-1/2/3, minuti).
3. Gate fx20 in forma-banda (A-MA-106-3).
4. Leva args-Vec (fronte leva); H-ICS slot successivo, criterio firmato.
