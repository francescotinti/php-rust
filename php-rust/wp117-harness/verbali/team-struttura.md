# Verbale TEAM-STRUTTURA — Concilio S-116→S-117 (Hoare · Matsakis · Klabnik)

Verdetti sedie: 3× CONCORDO CON EMENDAMENTI.

## CONVERGENZE
1. **BOLT espunto (unanimità)**: non esiste su Mach-O/aarch64. A′ = PGO rustc (`-Cprofile-generate/use`) + LTO fat + `codegen-units=1` + `-order_file` ld64. Il criterio PRE nomina solo strumenti eseguiti con successo su questo host.
2. **A′ vuota TUTTE le bande (unanimità)**: banda micro N=2, held-out, layout e binari conservati (052ea417, nulla2) DECADONO; nessun gate finché le bande non sono ri-misurate sul binario A′ (≥2 leve nulle). L-A si rigiudica ricompilando 2c18b2e sotto la pipeline nuova.
3. **Treno B**: manifest pre-registrato per NOME (cap 5); fedeltà/admission PER-VAGONE (parità, batteria, corpus per NOME), cronometro PER-TRENO con UN solo A/B binario-treno vs pin; anti-tassa: netto per OGNI categoria ≥ −banda(cat) del metro nuovo; guardie da nulla-treno di taglia comparabile (byte-delta ±30%); tie a 2 decimali regole S-116(c); rc SOLO da file; bisezione pre-registrata (stacca ultimo vagone, max 2 iterazioni).
4. **C safe-only**: NaN-boxing su PUNTATORI vietato (unsafe per costruzione, rompe VmGate); variante ammessa = arena per-richiesta + indici generazionali + elisione refcount (use-after-free → use-after-recycle logico); compatibile col binding output-capture.
5. **Apparato minimo**: solo script build PGO in UN atto (profilo pinnato, `.profdata` hashata/stashata, REGOLE §2) + prova determinismo build ×2 ⇒ hash identico.
6. **D**: ogni vagone dichiara PRIMA la strategia di ownership (indice/borrow/RefCell; RefCell su path caldo solo con A/B del costo) e serve direzione firmata (smoke R=2 segni concordi) per imbarcarsi.

## CONFLITTI (registrati, non appianati)
- **Posizione di C** — Hoare: riserva, KS-C solo se dopo 3 sessioni A′+B il peggiore resta >3×. Klabnik: riserva ma trigger anticipato (dopo A′+UN treno, prop >6× ⇒ cantiere nominato). Matsakis: «C in riserva» è REFUTATO dall'aritmetica (−65 ns/iter necessari; A+L-A rendono −31..−45; il resto è lifecycle Zval che solo C tocca): C1 borrow-non-clone entra come vagone già in S-118, C2 arena subito dopo.
- **Workload di profiling** — Hoare e Matsakis includono i sei micro nel profilo; Klabnik R3: il profilo NON deve coincidere coi giudici (WP+held-out profilano, micro giudicano), altrimenti circolarità da dichiarare.
- **Soglia KS-A** — Matsakis: banda ≤5 ns/iter E uplift ≥2% o chiusa; Hoare: banda ≤ metà di 10 E mediano ≥2%, decade a fine S-118; Klabnik: archiviazione in 1 sessione, si tiene solo LTO se gratis.
- **NaN-boxing su indici** — Hoare: vietato tout court (R4). Matsakis: C3 su soli indici resta ultima carta, previa decisione utente esplicita.

## PRIORITÀ PER L'ORDINE S-117 (max 3)
1. **Spike A′ in un atto**: script PGO+LTO+cgu=1+order_file; misura: build ×2 ⇒ hash identico, gate pieni (batteria rc da file, corpus per NOME ×2, parità) sul binario A′.
2. **Ri-banda sul binario A′**: ≥2 leve nulle per il primo campione N=2; misura: verdetti-nulla committati; meter-riparato se max(banda) ≤5 ns/iter (soglia Matsakis; Hoare: ≤ metà dell'attuale).
3. **Taglia del guadagno + rigiudizio L-A**: micro R=5 su A′ e ricompila 2c18b2e; misura: uplift mediano globale ≥2%, altrimenti scatta KS-A.

## KILL-SWITCH CONSOLIDATI
- **KS-A**: build non riproducibile ×2 (o `.profdata` non riproducibile ⇒ resta solo order-file+LTO) O (uplift <2% E banda non più stretta) ⇒ A′ si archivia; si tiene solo ciò che è gratis.
- **KS-B**: vagone che fallisce fedeltà esce, il treno non muore; treno bocciato 2 volte DOPO bisezione ⇒ B sospeso, si apre C (Matsakis: due bocciature per accumulo tasse ⇒ C2 diretto).
- **KS-D**: vagone senza direzione firmata non si imbarca.
- **KS-C/trigger C**: divergente per sedia (vedi CONFLITTI); da sciogliere con decisione utente pre-registrata, come la formula di promozione R5-Klabnik.
