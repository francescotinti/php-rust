# Verbale sedia 3 — Klabnik (spec, testabilità, matrici, gate) — Concilio WP-107 su S-105

## VERDETTO

La leva args (forma 2) è promossa su prove solide (co-primari T∧C, admission senza flip, controllo forma 1). Ma il gate di chiusura contiene **tre dichiarazioni al posto di prove**, e una di esse è **falsa alla lettera**: REFUTO il claim «parity-null per costruzione» di 766d3d8 e la capacità del `.done` del chain di arbitrare KS-PE-106-2 così com'è. PIN-106 è eseguito nella sostanza, non alla lettera.

## Refutazioni

**R-KL-107-1 (CAPITALE) — «sigilli const/doc, zero code» è refutato dal diff.** 766d3d8 tocca `crates/php-types/src/array.rs` (+9, un blocco `const` reale) e `zval.rs` (+8, doc-only). Un verbale di gate (`pin106-gate-verdetto.out` r.29-31) che afferma «zero code» smentito da `git show` è il pattern che PIN-106 è nato per uccidere: fiducia in una dichiarazione dove la prova costa un `cargo check`. Il record non stabilisce se il sigillo sia MAI stato compilato: hash₁ è build @ d569a56, la batteria precede il commit (22:50), fx20/corpus/micro non compilano nulla. HEAD porta un item const mai attraversato da rustc. c9df0af (solo .md) e cfa8d3a (solo harness) sono invece parity-null per costruzione legittimi.

**R-KL-107-2 — il sigillo A-HO-106-1 non morde.** `_seal::<bool>(); _seal::<i64>(); _seal::<f64>()` sigilla che i PRIMITIVI sono Copy (tautologia), non che i payload di `Zval` SIANO quei primitivi. Se `Bool(bool)` diventa `Bool(Box<…>)`, il blocco compila identico: fallisce «here, not silently» solo se qualcuno aggiorna a mano la lista — la disciplina che un sigillo dovrebbe rendere superflua. Un dente che non può andare rosso sotto la mutazione che dichiara di coprire non è un dente.

**R-KL-107-3 — PIN-106, incidente batteria: la sequenza regge, la lettera no.** Doppia run ammissibile come recovery, MA: (i) rc=0 viene dalla run 1, il conteggio 1740 dalla run 2 — le due evidenze appartengono formalmente a esecuzioni diverse; (ii) «il binario non cambia tra le due» è ASSERITO, mai misurato (nessun hash intermedio dopo run 1). Salvezza: re-hash₂ dopo run 2 + stash verificato + pin invariato a fine gate coprono la run 2 da sola; la run 1 è evidenza non probante, non contaminante. Il churn hash₁→hash₂ è documentato: quel requisito è soddisfatto.

**R-KL-107-4 — il `.done` del chain non può arbitrare KS-PE-106-2.** Tre buchi: (i) pin-check solo su phpr; l'oracle brew può churnare tra le gambe/sessioni e `pair105.identity` REGISTRA senza ASSERIRE (fx20 ha già stabilito il pin bilaterale fail-closed: qui manca); (ii) php-server hash registrato ma mai asserito; (iii) **rc_off/rc_on catturano l'exit dell'ultimo comando di pair105.sh (blocco ratios/perl), NON `GATE_VOID`**, che finisce solo dentro `pair105.done` per gamba. Un chain.done «rc_off=0» con gamba VOID è possibile per costruzione. La scadenza si arbitra leggendo i tre `.done` più le identity, o non si arbitra.

**R-KL-107-5 — bucket G1: falla di pre-registrazione, non pignoleria.** Il criterio pre-registra «(32,64]», vocabolario che lo strumento non parla (bucket reali le48/le64); la massa è finita in (48,64] e il delta di le48 non è riportato. Il conteggio esatto 19.900.000 su entrambi gli istogrammi + l'aritmetica 4×16=64 salvano il verdetto (non VOID), ma la classe è quella della «banda dal proprio run»: un'attesa scritta fuori dal vocabolario del giudice lascia la mappatura alla discrezione post-hoc.

**R-KL-107-6 — fx21 è collaudo manuale travestito da fixture di gate.** La fixture è tracked, ma il golden PRE-leva vive in `hd-probe-out/` (gitignored: evapora) e non esiste gate-script con pin fail-closed. In più l'attesa giusta NON è «byte-parity» (falsa: 7/8): è 7 righe oracle-identiche ∧ riga 5 ≡ golden phpr pinnato — così il gate va ROSSO quando §3.15 sarà curato e obbliga l'aggiornamento cosciente.

## Emendamenti

- **A-KL-107-1**: primo atto S-106, PRIMA di ogni misura: `cargo check` (o build) a HEAD con esito a verbale — sana R-KL-107-1.
- **A-KL-107-2**: riscrivere il sigillo dove il payload vive (binding nel match di `is_trivial_drop` o assert sul discriminante), non sui primitivi.
- **A-KL-107-3**: promuovere fx21 a `s105-fx21-gate.sh` sul modello fx20 (pin bilaterale, rc 0/1/66, golden riga 5 IN repo).
- **A-KL-107-4**: chain v2 — assert per-gamba di phpr E oracle (e server se usato); `pair105.sh` deve `exit $GATE_VOID`; il chain.done riporta i gate_void, non gli rc dei ratios.
- **A-KL-107-5**: lettura coppia S-105: prima di citare i rapporti, verificare oracle-hash nelle due identity = 07b0df8d.

## Kill-switch

- **KS-KL-107-1**: «parity-null per costruzione» è dicibile SOLO per commit che non toccano file compilati (`git show --stat` a verbale); ogni .rs toccato post-pin esige `cargo check` documentato, pena: prossima sessione apre col check.
- **KS-KL-107-2**: PIN-106 emendato — rc E conteggio della batteria dalla STESSA run; ripetizione ammessa solo con hash dopo OGNI run.
- **KS-KL-107-3**: attese sui census pre-registrate nei bucket ESATTI dello strumento, con TUTTI i delta riportati (zeri attesi inclusi), pena lettura VOID.
- **KS-KL-107-4**: nessuna scadenza (KS-PE-106-2 inclusa) si dichiara saldata da un marker il cui rc non è, per costruzione dello script, il verdetto del gate.
