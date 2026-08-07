# Team PIN-CENSUS-SERVER — Concilio WP-108, fase 2

**Membri**: Klabnik (relatore), Pedersen, Leijen · **Data**: 2026-08-07
**Fonti**: verbale-3-klabnik.md · verbale-6-pedersen.md · verbale-7-leijen.md · NEXT_SESSION §S-107

## Convergenze (ID canonico)

- **C-PC-1** — Nessuna refutazione capitale: pin **eb555106** regge; il grado **dde2a64d** è VALIDO al suo tempo (rc=0 voids=0 ×2, rito D-16); decadenza de67cb64 gestita alla lettera di A-PE-107-1. (KL/PE/LE unanimi.)
- **C-PC-2** — Ordine S-107 1-5 confermato da tutte e tre le sedie, con innesti (sotto).
- **C-PC-3** — Principio comune «dente = positivo NOMINATO + lettore»: un controllo ri-collocato può cedere il sito (KS-KL-108-1), un contatore senza dump+gate non è un dente (KS-LE-108-1), un fail-set senza file per NOME è un precedente elastico (KS-PE-108-2). Tre facce dello stesso teorema.
- **C-PC-4** — Registro/identity fail-closed: coppia commit + parity-null nel registro (A-PE-108-2), tre hash da file inclusa l'oracle (KS-PE-108-3), dicitura churn corretta (R-KL-108-3).

## Conflitti risolti

1. **Regrade server: prima del pair o prima delle cifre?** Risolto **dalle stesse parole di Pedersen**: il suo giudizio d'ordine gata il punto 4 SOLO sul launcher tre-pin (A-PE-108-3), mentre KS-PE-108-1 gata «qualunque cifra server». Il pair misura phpr-CLI; il server vi è identità d'ambiente, non oggetto misurato. **Decisione**: regrade obbligatorio PRIMA di ogni cifra ATTRIBUITA al server; il pair può correre con dde2a64d purché l'identity lo dichiari «PRE-H-A1» e nessuna cifra server venga letta dal pair. Klabnik e Leijen concordano: evita un rebuild speculativo che brucerebbe la finestra della leva (regola di ritmo).
2. **Etichetta churn** — non conflitto ma mislabel (R-KL-108-3): la sostanza (ri-validazione su hash₂, micro su entrambi Δ≤0,4) regge; si corregge la dicitura, non il verdetto.
3. **Carico del punto 1** (KL riserva b vs innesti LE): i due denti funnel sono test-only ~10′ e la ri-registrazione census viaggia nello STESSO run D-5 (nessuna finestra nuova) → si imbarcano entrambi, ma con timebox esplicito che protegge la leva.

## Direttive T-PC-108-n

**VINCOLANTI (1-8):**

1. **T-PC-108-1** — Funnel: ripristinare UN positivo BinaryDst nel `{main}` (`$s = $s + $i + $i;`) + negativo «nessun LoadSlot/Swap residuo nel `{main}` ON». Denti test-only al punto 1. [assorbe R-KL-108-1, R-KL-108-2, KS-KL-108-1, KS-KL-108-2]
2. **T-PC-108-2** — Dicitura di registro: «**BUILD EMENDATA post-A/B** — file enumerati: <lista>; ri-validata sul pin (batteria+corpus+micro su hash₂; micro hash₁≡hash₂ Δ≤0,4)». «Churn» riservato al relink puro a sorgente identico. [assorbe R-KL-108-3, KS-KL-108-3]
3. **T-PC-108-3** — Il grado lega un QUADRUPLO (hash server, phpr, oracle, HEAD): dde2a64d è PRE-H-A1; ogni CIFRA server S-107 pretende rebuild ricetta @ HEAD S-106 + regrade D-16 PRIMA della lettura; citarlo accanto a eb555106 senza regrade = VOID. [assorbe KS-PE-108-1]
4. **T-PC-108-4** — Coppia WP (punto 4): NON richiede regrade preventivo (server = identità d'ambiente); richiede launcher chain-v2 con gamba 0 TRE-PIN fail-closed da file (phpr+server+oracle, PIN_ORACLE_ATTESO.txt) e identity che dichiara «dde2a64d PRE-H-A1»; nessuna cifra server letta dal pair. [assorbe A-PE-108-3, KS-PE-108-3, R-PE-108-2]
5. **T-PC-108-5** — Pre-flight NEXT_SESSION emendato: «server dde2a64d GRADATO @ c7b6eb2 (**PRE-H-A1**), stash php-server-s106; primo atto di qualunque fronte server con cifre = rebuild+regrade». [assorbe A-PE-108-1]
6. **T-PC-108-6** — Registro dde2a64d: aggiungere coppia commit (server @ c7b6eb2 vs pin phpr @ d569a56) + enumerazione parity-null per NOME; ratificare D-6 emendata: fail-set baseline pinnato PER NOME su file, «vuoti» = diff vuoto contro QUEL file, eccezione congelata {wp_is_stream #2}, ogni crescita = voce nuova. [assorbe A-PE-108-2, A-PE-108-4, KS-PE-108-2, R-PE-108-3, R-PE-108-5]
7. **T-PC-108-7** — Attese stack-census S-102 DECADUTE dalla fusione (−5 transiti/iter): ri-registrazione per NOME (arith E prop) sul census-build del pin corrente, nello STESSO run census del punto 1 (D-5); ogni confronto col 23/iter storico = VOID. [assorbe R-LE-108-2, A-LE-108-2]
8. **T-PC-108-8** — Contatori con lettore, stesso commit: `argplace_decay_hits` → riga dump zvalcensus + gate «atteso 0»; contratto GA_ARITY corretto a DUE siti (bind_params + direct-bind) in docstring/etichetta o famiglie separate; manifest del rerun D-5 dichiara la composizione; dichiarare che il pin di parità resta silenzioso sul degrade. [assorbe R-LE-108-3, R-LE-108-4, A-LE-108-1, KS-LE-108-1]

**RACCOMANDAZIONI (9-11):**

9. **T-PC-108-9** — Punto 5: feature-gate statics GA_ARITY/GA_ARGPLACE_DECAY (R-5 ESTESO) senza churn del pin; se il pin churna, si applica la dicitura T-PC-108-2. [assorbe A-LE-108-3]
10. **T-PC-108-10** — Timebox esplicito sul punto 1 (denti ora 7): se sfora, la leva del punto 3 NON salta (regola di ritmo utente); matrice ST per NOME (`-=` su prop, `.=` vs ConcatAssignSlot, jump-target nella finestra, fixture diagnostic-safe) = backlog NOMINATO, non blocco. [assorbe A-KL-108-1, riserva (b) KL]
11. **T-PC-108-11** — Igiene: rc di `cargo check` archiviato in .out (A-KL-108-2); retro-verifiche lettura-only dichiarate come tali nel .out (R-PE-108-4); riscrivere «parity-null provato dalla compilazione» → il non-clobber dell'hash phpr non prova il codegen-null del runtime imbarcato (R-PE-108-1); alla riapertura footprint, primo atto = re-baseline peak (1942/1990 = storiche non citabili, R-LE-108-5).

## Modifiche all'ordine §S-107

- **Punto 1** (+innesti, CON timebox T-PC-108-10): entra T-PC-108-1 (due denti funnel, test-only); il run census hit/miss imbarca ANCHE il manifest GA_ARITY due-siti e la ri-registrazione stackcensus arith+prop (T-PC-108-7/8) — un solo run, nessuna finestra nuova.
- **Punto 2** (§3.15): invariato; PE conferma che 1417→1415 NON tocca i denominatori server 413/3508 — nessun aggiornamento dei conteggi pinnati.
- **Punto 3** (leva): invariato, protetto dal timebox.
- **Punto 4** (coppia): condizionato al launcher tre-pin T-PC-108-4; il **regrade server NON è prerequisito del pair** — è prerequisito di qualunque CIFRA server (T-PC-108-3), e si esegue solo SE un fronte server con cifre si apre (nessun rebuild speculativo).
- **Punto 5**: allargato a GA_ARGPLACE_DECAY (T-PC-108-9).
- **Pre-flight**: riga server riscritta secondo T-PC-108-5.

*Relatore: Klabnik — nessun dissenso residuo a verbale.*
