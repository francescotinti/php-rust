# Concilio WP-106 — INDICE (su S-104 e programma S-105)

⚖️ Convocato 2026-08-06 sera a valle della chiusura S-104. **Questo file è
un INDICE: i testi integrali vivono SOLO in `verbali/`** (regola utente
2026-08-06). Sintesi: `verbali/SYNTHESIS.md`. Esito: 9/9 CON EMENDAMENTI,
nessun MI OPPONGO; **1 refutazione capitale (Klabnik: PIN-105
insoddisfacibile per costruzione)**; convergenza trasversale: «icache-bound»
declassato a ipotesi non firmata (servono contatori o inlining pinnato).

## Fase 1 — verbali individuali (VINCOLANTI)

| Sedia | File | Ricevuta |
|---|---|---|
| 1 Hoare | [verbali/verbale-1-hoare.md](verbali/verbale-1-hoare.md) | CONCORDO CON EMENDAMENTI — A-HO-106-1 sigillo Copy sui payload trivial; A-HO-106-2 verdetto S-104 nel doc del predicato; A-HO-106-3 rinominare «al byte» — KS-HO-106-1 tesi icache senza contatore/controprova = VOID; KS-HO-106-2 chiamante nuovo di is_trivial_drop senza criterio = reject — capitali: no; tesi icache N=1; propone H-ICS «cold-out» (leva che testa e sfrutta l'ipotesi insieme). |
| 2 Matsakis | [verbali/verbale-2-matsakis.md](verbali/verbale-2-matsakis.md) | APPROVO CON EMENDAMENTI — A-MA-106-1 terza mutazione su OBS-8 (holder-esterno) o riclassifica 19a/19b per NOME; A-MA-106-2 disposizione mutanti sopravvissuti; A-MA-106-3 cap RSS = banda derivata + guardia erosione + mutante leak-parziale — KS-MA-106-1 niente verdetti su memory_get_usage stub (VOID); KS-MA-106-2 il rosso vale solo su mutante dell'osservabile dell'arbitro — capitali: no. |
| 3 Klabnik | [verbali/verbale-3-klabnik.md](verbali/verbale-3-klabnik.md) | CON EMENDAMENTI — **PIN-106** (build→hash₁→batteria→re-hash₂→STASH→gate: la batteria certifica il SORGENTE, non il binario); parity-null sempre col PERIMETRO nominato (server 31aa7c2e = inferito; banda ±0,2 provvisoria); coppia WP evento-O-timeout entro S-105 — KS-KL-106-1 parity-null senza perimetro = declassato; KS-KL-106-2 S-105 senza coppia = anomalia; KS-KL-106-3 build post-stash = gate VOID — **capitale: sì — PIN-105 insoddisfacibile (la batteria relinka SEMPRE)**. |
| 4 Hejlsberg | [verbali/verbale-4-hejlsberg.md](verbali/verbale-4-hejlsberg.md) | CON EMENDAMENTI — «icache-bound» nominato non provato (conflazione icache/registri/BTB; +8.000 B / 1101 siti ≈ 7,3 B/sito ⇒ «inline ovunque» letterale smentito) — A-HE-106-1 contatori INST_RETIRED+L1I-miss sull'A/B in stash; A-HE-106-2 flip via solo inline-threshold; A-HE-106-3 fingerprint+rustc/profilo; A-HE-106-4 PGO poi outlining #[cold]; A-HE-106-5 il 21,2% = prefisso di TARGETING — KS: claim icache senza counter = VOID; leva senza disasm prima/dopo = VOID — capitali: no (la caduta regge; cade l'attribuzione). |
| 5 Bak | [verbali/verbale-5-bak.md](verbali/verbale-5-bak.md) | CON EMENDAMENTI — A-BA-106-1 leva S-105 = H-D inline-storage 2 slot con criterio SiteTag+arità+audit-fuga+disasm; A-BA-106-2 braccio contatori L1I/mispredict PRIMA di ristrutturare run_loop; A-BA-106-3 superistruzioni pila = H-C3 gated, threaded-dispatch vietato — KS-BA-106-1 mai componenti di costo da A/B che flippa il codegen — capitali: no; «chiamate quasi gratis»/«icache-bound» declassati a non-firmati (l'A/B misurò leva+inliner). |
| 6 Pedersen | [verbali/verbale-6-pedersen.md](verbali/verbale-6-pedersen.md) | CON EMENDAMENTI — A-PE-106-1 cifre server solo da pin same-HEAD gradato PIENO; A-PE-106-2 debito coppia WP con SCADENZA (prima leva O chiusura S-106); A-PE-106-3 retention backup uploads per NOME; A-PE-106-4 controllo d'ambiente nel launcher per-fase — KS-PE-106-1 cifra su pin difforme = VOID; KS-PE-106-2 senza coppia entro S-106 il riferimento WP-102 decade a storico — capitali: no. |
| 7 Leijen | [verbali/verbale-7-leijen.md](verbali/verbale-7-leijen.md) | CON EMENDAMENTI — 4/4 confermate ma la «simmetria byte» esclude le TAGLIE non i SITI (bound ~50 ppm); SiteTag pieno sostituito da **probe cap-bump 2→4** come gate d'apertura; leva = SmallVec inline-2 (pool REFUTATO: duplica la freelist TL mimalloc), attesa Δ∈[6,14] ns/iter; lettura R=7: estimatore scambiato post-hoc ⇒ magnitudine «INDETERMINATA», non «sotto banda» — KS: promozione args esige timing E census alloc/chiamata→0,0000; estimatore accoppiato pre-registrato o lettura VOID — capitali: no (il verdetto R=7 sopravvive solo grazie allo STOP §3). |
| 8 Stogov | [verbali/verbale-8-stogov.md](verbali/verbale-8-stogov.md) | CON EMENDAMENTI — A-ST-106-1 cura memory_get_usage a DUE GRADINI (contatore per-thread o mi_* on-demand; functional-parity dichiarata, MAI byte-parity; KS-ST-106-1 promozione solo con A/B+disasm); A-ST-106-2 fedeltà: generator > §3.13 unit > §3.12 regimi; A-ST-106-3 fusioni: prop RMW (ASSIGN_OBJ_OP-omologo) + forme registro arith, KS-ST-106-2 protocollo disasm/A/B — capitali: no; REFUTATA la cura «atomics process-global a release» (conflaziona i worker, tassa calls). |
| 9 Gregg | [verbali/verbale-9-gregg.md](verbali/verbale-9-gregg.md) | KS-GR-105-1 SALDATO questa volta (A/B ×2, meccanismo nominato, revert al byte) ma i rapporti sono FERMI da 3 sessioni — A-GR-106-1 contatore SDOPPIATO (sessioni-senza-Δ-rapporti = 3); A-GR-106-2 admission-disasm + smoke R=2 + hash output + seconda canna — KS-GR-106-1 (=4 ⇒ riallocazione di categoria); KS-GR-106-2 (leva senza admission/smoke = VOID) — capitali: no; icache declassato a ipotesi forte N=1; priorità S-105 = SiteTag→leva calls. |

## Fase 2 — note di team (alimentano la sintesi; individuali restano vincolanti)

| Team | File | Ricevuta |
|---|---|---|
| leva (Bak+Leijen+Hejlsberg) | [verbali/team-leva.md](verbali/team-leva.md) | Convergenze: SmallVec inline-2 per gli args, pool escluso senza gara, audit-fuga con fixture, disasm obbligatorio, «icache-bound» declassato a ipotesi. Conflitti: gate d'apertura (SiteTag+arità Bak vs probe cap-bump Leijen — componibili); contatori prerequisito (Hejlsberg) vs braccio parallelo (Bak/Leijen: sì per leve icache, no per la leva args). Priorità: atto zero → probe+arità → fughe → SmallVec → A/B → gate doppio co-primario+PIN; contatori in parallelo breve. |
| metodo (Gregg+Klabnik) | [verbali/team-metodo.md](verbali/team-metodo.md) | PIN-106 ingloba l'invariante run_loop; admission-disasm+smoke R=2 e perimetro parity-null complementari (sezione unica «cosa copre la misura»). Contatori: ritmo utente = pavimento, sessioni-senza-Δ-rapporti=3 = tetto d'esito (KS a 4 ⇒ riallocazione). Coppia WP: scadenza S-105, fallback Pedersen S-106 non prorogabile. |
| arbitri (Hoare+Matsakis) | [verbali/team-arbitri.md](verbali/team-arbitri.md) | Convergenza piena: (a) 19a/19b — la terza mutazione mirata OBS-8 decide (rossa=arbitri del MOVE, verde=riclassifica per NOME; Hoare ritira il braccio-rosso generico); (b) fx20 — cap→banda pre-registrata + guardia erosione cap/2 + mutante leak-parziale Pop; (c) in S-105: sigillo Copy, doc verdetto, fix «al byte»; KS su chiamanti nuovi e premessa icache; (d) args-Vec resta leva #1, H-ICS slot successivo con criterio firmato. |
| fedeltà (Stogov+Pedersen) | [verbali/team-fedelta.md](verbali/team-fedelta.md) | mem_get_usage: voce 🔴 subito + gradino TLS/mi_* in S-105; promozione solo con A/B+disasm (KS-ST-106-1), atomics process-global refutati. In S-105: generator get_gc + §3.13; §3.12-i viaggia nella fusione. Scadenza: leva = trigger → stessa sessione rebuild@HEAD + grado PIENO + coppia WP; fallback S-106 o WP-102 decade a storico. Propone prop-RMW davanti a calls (conflitto composto in sintesi: args-calls #1). |

## Sintesi

→ [verbali/SYNTHESIS.md](verbali/SYNTHESIS.md) — §FONDAMENTALI in testa;
ordine DEFINITIVO S-105 recepito in `NEXT_SESSION_WORDPRESS.md` §S-105.

## Esito in sessione

Convocato e consumato nella chiusura S-104 (stesso giorno). L'ordine S-105
è scritto nella rotazione; il primo atto di S-105 è il pre-flight, non la
riconvocazione.
