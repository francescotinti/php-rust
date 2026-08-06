# Verbale sedia 6 — PEDERSEN (confine per-richiesta, lifecycle, server) — Concilio WP-107 su S-105

## VERDETTO: CON EMENDAMENTI (nessuna refutazione capitale)

## R-PE-107-n (refutazioni)

**R-PE-107-1 — La lettera «grado PIENO stessa sessione» è VIOLATA, e
l'impossibilità era autoprodotta.** Il lancio della coppia alle 23:03 ha
occupato la macchina che il grado avrebbe dovuto usare: l'ordine di
esecuzione ha reso insoddisfacibile la lettera dell'ordine §S-105-1d.
«Macchina occupata» non è forza maggiore quando sei tu a occuparla.
Mitigazioni VERE: nessuna cifra server emessa su de67cb64; identità
pinnata (stash `php-server-s105`, hash verificato, pin phpr d4d0fa52
INVARIATO dopo la build). Violazione della lettera senza breccia del KS:
sanabile UNA volta sola (A-PE-107-1).

**R-PE-107-2 — La mia stessa lettera KS-PE-106-1 è ambigua e, presa alla
lettera, VOIDerebbe la coppia in volo** («cifre server/WP» con server non
gradato). Assurdo: pair105.sh esercita SOLO phpr-CLI via phpunit; l'hash
php_server in `pair105.identity` è identità d'ambiente, non credenziale.
Risolvo da autore: la coppia CLI è gradata dal pin phpr firmatario
PIN-106 (batteria+fixture+corpus×2 fatti); il grado PIENO custodisce solo
il confine HTTP. Riscrittura in KS-PE-107-1.

**R-PE-107-3 — Il chain non ha pre-flight né gate tra le gambe; il vero
firewall è DENTRO uploads-guard, non nel chain.** Verificato: una gamba
off morta tra backup-wipe e restore lascia STATE pendente ⇒ il
backup-wipe della on ABORTISCE (exit 11 → rc=4): la metrica on NON è
contaminata (viene uccisa) e il backup NON è clobberato. Ma il prezzo:
MySQL giù (banalmente pre-verificabile) brucia la notte intera; uploads
resta AZZERATA con STATE pendente fino a restore MANUALE (nessun
restore-on-failure nel chain); le gambe phpunit girano SENZA watchdog
(regola standing violata) — un hang off = niente `.done` = il fallback
KS-PE-106-2 scatta per silenzio.

**R-PE-107-4 — «Gambe verdi» ≠ saldato.** `pair105.done` porta
rc=$GATE_VOID, che codifica SOLO l'assert conteggi↔nomi: i failnames
DIVERSI per NOME non voidano (solo log in progress), e gli rc phpunit
non confluiscono nel marker. Chi legge il solo `.done` può dichiarare
saldato un debito con la parità rotta.

**(d) VERIFICATO, non refutato**: entrambi i chiamanti di `arity_note`
(`calls.rs:319`, `run.rs:2602`) sotto `#[cfg(feature = "mem-census")]`;
feature opt-in in ogni Cargo.toml della catena; `php-server default =
["cli-server"]`; census in target separato `phpr-census-target`. Il
confine per-richiesta del server NON è toccato. Residuo: la static
GA_ARITY compila incondizionata (inerte, zero chiamanti in parità) ed è
process-global cumulativa → A-PE-107-4.

## A-PE-107-n (emendamenti)

**A-PE-107-1**: grado PIENO = PRIMO ATTO di S-106, PRIMA di ogni cifra
server E di ogni build (la build churna e rompe il same-HEAD). Collaudo:
binario stashato `php-server-s105` (hash de67cb64 riverificato) + pin
phpr d4d0fa52 riverificato; **option 413 + restapi 3508 per NOME, env -i,
2 modi espliciti, mode-probe**. La proroga è SPESA: se S-106 builda prima
del grado, il grado si rifà sul nuovo pair same-HEAD, senza ulteriore
rinvio.

**A-PE-107-2**: chain v2 — pre-flight (ping MySQL + STATE guardia assente
+ spazio disco) PRIMA della off; su rc gamba ≠0 tentare `restore` prima
di proseguire; gambe phpunit sotto watchdog; `.done` arricchito con
l'esito failnames-diff per gamba.

**A-PE-107-3 — protocollo di lettura (chi: apertura S-106, accanto al
grado; bande PRE-REGISTRATE prima di aprire i ratios)**. Precondizioni:
rc_off=rc_on=0 · 8 run rc=0 in progress · failnames.diff VUOTI
(media+full, off+on) · identity=d4d0fa52. Attese: contro WP-102 full
1,89 / media 2,64, direzione ≤ (la leva calls +14% micro deve
comparire); rumore stimato dal confronto off↔on stessa sera; peak =
REGISTRAZIONE per la roadmap footprint (riferimenti S-102 1942/1990
MiB), MAI verdetto (metrica chiusa per sempre).

**A-PE-107-4**: se un census dovesse mai accendersi lato server, i
contatori vanno per-request bracketed (disciplina census-instrumentation);
un cumulativo process-global citato da contesto server = non attribuibile.

## KS-PE-107-n (kill-switch)

**KS-PE-107-1** (riscrive KS-PE-106-1): VOID senza appello (i) ogni cifra
prodotta DAL server su pin non same-HEAD o non gradato PIENO; (ii) ogni
cifra della coppia WP-CLI su pin phpr non firmatario PIN-106. La coppia
CLI è gradata dal pin phpr; il grado server custodisce il confine HTTP.

**KS-PE-107-2**: «debito saldato» dichiarato senza il protocollo
A-PE-107-3 completo = VOID; in tal caso WP-102 decade comunque a storico
(KS-PE-106-2 non si sospende con un rc).

**KS-PE-107-3**: cifre server da de67cb64 emesse prima del grado PIENO =
VOID; seconda proroga del grado = il pin server DECADE e i report tornano
a «server NON misurato».
