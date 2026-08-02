# BRIEF Concilio WP-91 — revisione di S-89.0 (sessione WP-89) + programma WP-90(sessione)

MANDATO: REFUTARE, mai benedire. Verificare contro CODICE e RAW, non
contro i verbali. Ogni claim del report va considerato falso finché non
verificato. Se una premessa del report è sbagliata, dirlo con la prova.

REPO: /Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust

LETTURE BASE (tutte le sedie):
- sessions/WP_SESSION_89.md (report di sessione)
- wp89-harness/MEASURE89_RESULTS.md + verdict89.a1.g3.out (e g1/g2 per la storia)
- NEXT_SESSION_WORDPRESS.md (stato pre-rotazione; il §WP-89 eseguito)
- wp89-harness/design89.md
Poi il CODICE/RAW del proprio perimetro (v. prompt).
NOTA STRUMENTI: per i file .rs usare il tool Read (righe mirate) o
mcp__serena__* — il grep bash sui .rs è bloccato da hook. I raw stanno
in wp78-harness/measure-out/m89.* (committati).

CONTESTO S-89.0 in 10 righe:
1. Gate cifre: corpus COMMITTED-only (A-SK55), [derivata] a scope di
   FIGURA con verifica aritmetica X−Y su operandi in corpus (A-SK56),
   ± a finestra allowlist (A-SK53-bis), prefix ledger per riga (A-SK57).
2. Strumenti: failpath subshell+teardown ledgerati (A-PP46), pid-echo
   x-phpr-pid (A-BG51), spans giudicato (A-SK58), presence-guard
   estrattori (A-BG50, pin ==2 sulla coppia pre/post-collect),
   judge_sha + governo generazioni gG (A-AH51/A-SK59), commit_calls
   bandito (A-DL46), recorder rustc (A-AH53).
3. Catena: battery-attempts.ledger per OGNI esito (A-AH50≡A-BG49) +
   allowlist finestra emendata in corsa (quasi-morso sul proprio stamp).
4. Sigilli v7: flag probe PRIVATO (A-MS43), #[must_use] (A-MS44), belt
   grafie (A-MS45), noprobe --selftest in catena (A-TH48), 5 grafie
   (A-TH49), putord non-decrescente in a_ds36 (A-TH51 riqualificata dal
   morso: la coppia main_evicted/evict-fp condivide il putord),
   publish a contatore due-lane (A-PP47 riqualificata: F8c = il
   link-fatal non si pubblica MAI; lane pulita su file reale),
   NPASS pinnato ==13 (A-PP48).
5. Campagna m89: battery 4 tentativi tutti ledgerati (FAIL/REFUSE/PASS/
   PASS); attempt=1 pulito; giudice g1 FAIL (regime clamped NUOVO ai dt
   1-5 ms), g2 FAIL (pin forma-canale), g3 PASS.
6. Cifre g3: b_base=19.575.603 B/worker se=584.723 2σ=[18.406.157,
   20.745.049] grade verdict-grade-candidate; b_ret0=19.329.843
   (ADVISORY, un marginale fuori fascia δ=0,15) ⇒ VATTR:
   NOT-attributed-to-retention (P-RET0 REFUTATA, KL-90-4 con census
   per-theap + braccio ord36 in-band).
7. VSWEEP: cal 7.801.102 B al byte (4ª campagna); P-DT20 4/4 al byte;
   clamped ai dt 1-2 (entrambi ordini) e dt5 afirst; discriminatore
   UNSTABLE 2/3 (timing-attached).
8. A-DS35 fase 1: 18 fixture + pin oracle committati PRIMA del codice
   (ds35-verify); implementazione = residuo per NOME (sede
   lower/class.rs, appendice nel session file).
9. KL-85-2 ritirata a registro; A-BB60/A-PP49 design (design89.md).
10. Binari: phpr 64e9e51c281de6d1 (stash phpr-wp89); mem-census
    e2f27c9c671b737a; campagna a ed427f4.

PROGRAMMA WP-90(sessione) DA GIUDICARE (bozza da confermare/rifiutare/
riordinare): 1) A-DS35 fase 1 IMPLEMENTAZIONE (primo item, gate di
merge KS-DS-88-3/KS-DS-89-3); 2) attribuzione b iterazione 2 (candidati
residui: pagine parzialmente usate per bin, abandoned con blocchi vivi,
ord 44, census a worker VIVI); 3) attuazioni design (A-BB50 net
per-thread; A-BB60; A-PP49); 4) sanatorie/emendamenti dal concilio.

FORMATO VERBALE (file, ≤600 parole): intestazione `# VERBALE — <Nome>,
sedia N, Concilio WP-91`; VERDETTO (CONCORDO / CON EMENDAMENTI /
MI OPPONGO); Q1..Q4 con le prove; **Emendamenti** numerati nella PROPRIA
serie (es. A-TH52, A-MS46, A-SK60, A-AH54, A-BB61, A-PP50, A-DL48,
A-DS48, A-BG53 in poi); **Kill-switch** serie WP-91 (KH91-n, KS-MS-91-n,
KS-SK-91-n, KS-AH-91-n, KB-91-n, KS-PP-91-n, KL-91-n, KS-DS-91-n,
KG-91-n); firma.

SCRIVERE il verbale in wp91-harness/verbali/verbale-<sedia>-<nome>.md
(es. verbale-3-klabnik.md). Il messaggio FINALE dell'agente deve essere
SOLO la ricevuta ≤80 parole:
`sedia N <nome>: VERDETTO — emendamenti: <sigle+titoli brevi> — KS:
<sigle+condizioni brevi> — refutazioni capitali: <sì/no + una frase>`.
