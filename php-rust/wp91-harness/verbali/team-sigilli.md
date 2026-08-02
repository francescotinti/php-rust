# TEAM-SIGILLI — Concilio WP-91 (relatore su verbali 1-Hoare, 2-Matsakis)

Fonti vincolanti: `verbale-1-hoare.md`, `verbale-2-matsakis.md`.

## CONVERGENZE
1. **Single-source dei rilevatori.** Stesso vizio da due lati: Hoare (A-TH52) — TH49RE/TH44RE ri-digitate tra self-test e sweep ⇒ positivo che non esercita il rilevatore di produzione (classe A-PP48); Matsakis (A-MS48) — belt con regex a binder letterale (`|f| f.set(`) che un binder diverso, `take/swap`, UFCS o split multilinea eludono. Regola comune: UNA variabile-regex condivisa self-test↔sweep (modello TH33RE), decoy che passano per la regex COMPOSTA di produzione, same-commit.
2. **Il limite lessicale va dichiarato e recintato.** Hoare: `paste!`/`concat_idents` fuori portata di ogni sigillo lessicale ⇒ bando per nome nei Cargo.toml (A-TH53) finché A-MS27 (rustc giudice) non chiude. Matsakis converge sul principio col sigillo di TIPO/visibilità: dove il compilatore può morire a compilazione, la regex è solo belt.
3. **Restringere il perimetro di privacy, non fidarsi del file.** Matsakis A-MS46: `mod probe` annidato ~40 righe con `pub(super)` — il perimetro attuale (`mod implementation`, ~1880 righe test inclusi) lascia ogni riga futura libera di scrivere il flag, giudicata solo dalla belt bucata. Coerente con la linea Hoare: il sigillo di forma (finestra awk chiusa, pin ==1) batte il pattern-match aperto (A-TH56).
4. **Gate igienici fail-closed.** A-MS49 (`rm -f .done` in testa + confronto git=HEAD imposto) e A-TH56 (pin `^probe_in()` ==1, finestra chiusa, tainted2 declassato a forward-guard): stato stantio e doppia definizione devono morire prima del verdetto.

## CONFLITTI
Nessun conflitto frontale. Tensione di metodo: Hoare estende la belt (branch nuovi per 6 grafie Q1); Matsakis preferisce spostare il giudizio sul compilatore (privacy/mod annidato) e tenere la belt come cintura. Composizione naturale: A-MS46 riduce la superficie che A-TH52/A-MS48 devono coprire — prima il recinto di tipo, poi la belt single-source sul residuo.

## PRIORITÀ (proposta del team; i verbali restano vincolanti)
**Sigilli v8 SUBITO (meccanici, chiudono kill-switch attivi):**
- A-TH52 + A-MS48 (single-source + grafie estese + decoy che mordono la regex composta) — sblocca KH91-2 e KS-MS-91-1.
- A-TH56 (noprobe: pin ==1, finestra chiusa, pin di forma) — sblocca KH91-3.
- A-MS49 (.done igiene) e A-MS47 (`let <ident> = ProbeWindow::arm()` ==1, `-D clippy::let_underscore_must_use`).
- A-TH54 (riscrittura A-TH50 in forma order-dependent; fix "a_ds36"→a_ds26) — doc, costo zero, KH91-1 già attivo sui claim.

**DESIGN (sessione dedicata):**
- A-MS46 (mod probe annidato) — refactor piccolo ma di forma, va col codice non con gli script.
- A-TH53 (bando paste/concat_idents) — decisione di policy da ratificare.
- A-TH55 (ordine canonico KIND intra-putord in a_ds26/a_ds38).
- A-MS50 — vedi sotto.

## RINVIO AL TEAM-MISURA (non deciso qui)
La refutazione capitale Matsakis Q4 — i visitor census ALLOCANO (Vec::push, BTreeMap::entry) dentro la visita mimalloc del subproc visitato, violando l'invariante dichiarato (righe 698-700) — tocca il CANALE DI MISURA: KS-MS-91-3 rende ADVISORY ogni riga mi_theap_* pre-A-MS50. Se e come declassare le figure m89 spetta al team-misura; A-MS50 (capacità pre-riservata, bins array-fisso classe BinTab, `acc` riderivato da arg) è il prerequisito per tornare verdict-grade.

Nessuna delle refutazioni tocca il PASS di VERDICT89 (dichiarato da entrambe le sedie nei rispettivi ambiti).
