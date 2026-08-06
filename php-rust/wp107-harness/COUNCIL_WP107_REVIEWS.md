# Concilio WP-107 — INDICE (su S-105 e programma S-106)

> Formato indice (decisione utente 2026-08-06): link + ricevute ≤80
> parole; i testi integrali vivono SOLO in `verbali/` e nei file di team.
> **STATO: CONCILIO CONSUMATO** — fase 1 COMPLETA 9/9 (Leijen e Stogov
> recuperate con rilancio a quota liberata, indipendenza preservata);
> fase 2 (3 team) + SYNTHESIS tenute in S-106 (2026-08-07 notte).
> **[COUNCIL_WP107_SYNTHESIS.md](COUNCIL_WP107_SYNTHESIS.md) = documento
> VINCOLANTE finale**: 21 direttive S-106-D-1..21 + 6 raccomandazioni,
> ordine S-106 RATIFICATO (sequenza confermata, testi emendati).

| Sedia | Verbale | Ricevuta |
|---|---|---|
| 1 Hoare | [verbali/verbale-1-hoare.md](verbali/verbale-1-hoare.md) | CON EMENDAMENTI — A-HO-107-1 sigillo Copy via costruttori di variante, -2 doc icache come ipotesi N=1, -3 dente VM sul braccio direct-bind (negativi hinted/by-ref/variadic/generator), -4 predicato fast unificato, -5 perimetro census solo Op::Call — KS: -1 allargare simple_call senza dente+fx21 = VOID; -2 decay_arg con effetti invalida la concessione d'ordine — refutazione capitale: sì — il sigillo Copy è VACUO («Bool→boxed» passa tutti e tre i sigilli). |
| 2 Matsakis | [verbali/verbale-2-matsakis.md](verbali/verbale-2-matsakis.md) | APPROVO CON EMENDAMENTI — A-MA-107-1 dente ordine-di-drop (decay `(0..n).rev()` vs sorgente: «pure-read» non provato); -2 backstop ArgPlace rumoroso; -3 rinvio OBS-8 legittimo, 19a/19b PASS = solo byte-parity, rientro primo atto S-106; -4 §3.15 confermato in expr.rs:1615 — KS: claim d'ordine solo con prova/dente; niente direct-bind su op dinamiche senza check ArgPlace — capitali: no; ownership forma 2 sana (pool by-move, D-R11 preservato). |
| 3 Klabnik | [verbali/verbale-3-klabnik.md](verbali/verbale-3-klabnik.md) | PIN-106 eseguito nella sostanza, NON alla lettera — A-KL-107-1 cargo check a HEAD primo atto S-106; -2 sigillo Copy riscritto (tautologico); -3 fx21 a gate fail-closed con golden riga 5; -4/5 chain v2 con assert bilaterale ed exit GATE_VOID — KS-KL-107-1..4 — capitale: sì — «zero code» di 766d3d8 è FALSO al diff (2 .rs, const mai compilato post-pin). |
| 4 Hejlsberg | [verbali/verbale-4-hejlsberg.md](verbali/verbale-4-hejlsberg.md) | PROMOZIONE forma 2 NON contestata, sovrastruttura REFUTATA — A-HE-107-1 diff per-target completo nell'admission; -2 «37 ns» declassato a divario direzionale; -3 micro su hash₁ E hash₂ (banda-layout gratis dal churn); -4 text-budget run_loop (+4 KB ⇒ PGO/outlining) — KS: -1 niente «37 ns» nei criteri; -2 icache solo targeting; -3 componenti non citabili senza diff per-target — capitale: sì — il «~37 ns» è estimatore post-hoc tra binari a layout diversi (stessa falla di R=7). |
| 5 Bak | [verbali/verbale-5-bak.md](verbali/verbale-5-bak.md) | LEVA PROMOSSA NON TOCCATA, REFUTATE Scoperta-4 e perimetro S-106 — A-BA-107-1 contatore hit/miss+per-chiamante prima del prossimo sito; -2 srotolamento per-arità respinto; -3 prop solo dopo braccio contatori, altrimenti arith — KS: copertura≠istogramma arità; specializzazioni solo con run_loop non-crescente; verdetti contenitori per forma-e-confine — capitale: sì — il 73,1% è arità a bind_params, NON copertura del predicato fast (builtin/metodi/default mai censiti). |
| 6 Pedersen | [verbali/verbale-6-pedersen.md](verbali/verbale-6-pedersen.md) | CON EMENDAMENTI — A-PE-107-1 grado PIENO primo atto S-106 prima di cifre e build (proroga SPESA); -2 chain v2 pre-flight+restore-on-fail+watchdog; -3 lettura ratios con precondizioni failnames-VUOTI e bande su WP-102; -4 census server solo per-request — KS: 107-1 riscrive 106-1; 107-2 saldato-senza-protocollo VOID; 107-3 seconda proroga ⇒ pin server decade — capitali: no; firewall inter-gamba esiste solo dentro uploads-guard (verificato). |
| 7 Leijen | [verbali/verbale-7-leijen.md](verbali/verbale-7-leijen.md) | CON EMENDAMENTI — sostituzione di lettera-gate = emendamento dichiarato col test soppresso ri-collocato; bande H-C3 mai da componenti prezzate (solo segno+soglie); rerun arità su binario pulito ∘ A-BA-107-1; attese negli spigoli reali dell'istogramma; «TL quasi gratis» = ipotesi, non cifra — KS: KS-LE-107-1 manifest-binario per ogni lettura census; KS-LE-107-2 niente STOP/bisect da bande a componenti — capitali: no; la promozione forma 2 regge sui co-primari, convenzione S-102 verificata a sorgente. |
| 8 Stogov | [verbali/verbale-8-stogov.md](verbali/verbale-8-stogov.md) | CON EMENDAMENTI — forma 2 = metà del modello Zend (resta il doppio transito push→pop, debito nominato); cura §3.15 Zend-esatta (verificata su zend_check_arg_send_type) ma da estendere a Error/MakeRef ≥ vslot + gamba dinamica fx21; ordine fedeltà: §3.15 > get_gc > §3.13 > §3.12-i > §3.14 — KS: KS-ST-107-1 voci nuove cercate nel fail-set congelato, fix cita i fail da flippare (attesa 1417→1415); KS-ST-107-2 doppia copia fast-path ⇒ fx21 VOID se se ne tocca una sola — capitali: no; ma R-ST-107-4 refuta la Scoperta 3: by_ref.phpt era GIÀ rosso nei 1417, il congelamento l'aveva sepolta. |
| 9 Gregg | [verbali/verbale-9-gregg.md](verbali/verbale-9-gregg.md) | PROMOSSO CON EMENDAMENTI (co-primari T∧C reggono, admission senza flip) — A-GR-107-1 «~37» = contrasto post-hoc CON confondenti (reverse+bind_params); -2 quota-calls WP solo da formula pre-registrata — KS: -1 early-stop smoke 2/2 opposto; -2 contrasti tra A/B distinti mai cifra; -3 banda coppia PRE-registrata full∈[1,84;1,89] media∈[2,57;2,64] f̂=(1−r/1,89)/0,14 — capitali: no; oggetto avanzato (calls 6,3) ma prop/arith fermi: S-106 morda lì. |

## Fase 2 — team tematici (S-106, 2026-08-07)

| Team | Nota | Ricevuta |
|---|---|---|
| METODO-MISURA (Hejlsberg rel., Gregg, Leijen) | [team-metodo-misura.md](team-metodo-misura.md) | 10 direttive T-MM-107-1..10 (7 vincolanti). Nessuna modifica di sequenza; 3 integrazioni (punto 5: criterio H-C3+early-stop+admission per-target; punto 6: census su binario pulito con manifest; chiusura: micro su hash₁+hash₂ e taglia run_loop). Conflitti risolti 3; equivalenza canonica KS-HE-107-1. |
| VM-SEMANTICA (Stogov rel., Hoare, Matsakis) | [team-vm-semantica.md](team-vm-semantica.md) | 10 direttive T-VM-107-1..10 (7 vincolanti). Ordine-di-drop: vince Matsakis (claim senza prova/dente = VOID); §3.15 composta Stogov∘Matsakis (veicolo compiler-side + semantica Zend-esatta, attesa 1417→1415, resta punto 7 testa non negoziabile); punto 4 +backstop ArgPlace, punto 6 +dente VM direct-bind. |
| PROCESSO-PIN-SERVER (Klabnik rel., Pedersen, Bak) | [team-processo-pin-server.md](team-processo-pin-server.md) | 12 direttive T-PS-107-1..12 (9 vincolanti). Grado: conforme nella sostanza, 4 difformità puntuali (sanate nel launcher v2 al rilancio forzato dalla decadenza del pin). Punto 1 «saldato condizionato»; punto 4 clausola sequenza; punto 5 default arith esplicito. |

**SYNTHESIS**: [COUNCIL_WP107_SYNTHESIS.md](COUNCIL_WP107_SYNTHESIS.md)
— RATIFICATA 2026-08-07: 21 vincolanti S-106-D-1..21 + 6 raccomandazioni
da 29 di team (fusioni: hit/miss D-5; retro-verifica coppia D-6; fx21
D-14). Recepisce i fatti nuovi (de67cb64 DECADUTO, regrade v2 su
dde2a64d). Nuova lezione ⭐⭐ a veto: l'hash pinna l'identità, non la
ricetta. Contatore concili-arretrati = 0.

**Esito in sessione (S-105, stessa notte)**: recepiti subito in
NEXT_SESSION: KS-HE-107-1 ≡ KS-GR-107-2 (niente «37 ns» come cifra),
KS-GR-107-1 (early-stop smoke), KS-GR-107-3 (banda coppia
pre-registrata), A-BA-107-1 (hit/miss prima del prossimo sito calls),
A-KL-107-1 (cargo check a HEAD), A-HO-107-1 ≡ A-KL-107-2 (sigillo Copy
da riscrivere), A-PE-107-1/KS-PE-107-3 (grado server primo atto, seconda
proroga = decadenza).
