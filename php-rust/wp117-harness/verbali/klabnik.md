# Verbale Klabnik — S-116 concilio di rotta (lente: chiarezza, spec dei gate, testabilità)

## VERDETTO: CONCORDO CON EMENDAMENTI

La raccomandazione (A subito / B regime / D metodo / C riserva) è ordinabile ma sotto-specificata in due punti che, dalla mia lente, la invaliderebbero così com'è: (1) **BOLT non esiste su Mach-O/Darwin** — l'ambiente è macOS ARM; la voce va sostituita con strumenti reali (PGO + LTO fat + codegen-units=1 + eventuale `-order_file` di ld64) o la rotta A nasce non testabile; (2) **A vuota TUTTE le bande pre-registrate** (banda micro N=2, banda held-out N=1, famiglia calls −5,50): cambiata la pipeline, i binari conservati (s114-la ecc.) e le nulle misurate non sono più applicabili. «A subito» senza ri-misura delle bande ricrea esattamente il vizio già vietato: gate a soglia fissa su giudice senza banda.

## ROTTA DALLA MIA LENTE (orizzonte 3 sessioni)
Ordine: **A′ (A emendata) → ri-banda → B(treno con vagoni D)**; C riserva con trigger nominato.
- **S-117 (mossa concreta)**: spike A′ = `scripts/pgo-build.sh` (profilo pinnato, `.profdata` hashata e stashata nell'atto); prova di determinismo (build ×2 stessa ricetta ⇒ hash identico, altrimenti la pipeline non può fare da pin); gate pieni sul binario PGO (batteria rc da file, corpus per NOME ×2, parità); micro R=5 per la taglia del guadagno; PRIMA leva-nulla post-A per il primo campione di banda nuova.
- **S-118**: bande complete (≥2 nulle micro, ≥3 campioni held-out) + manifest del treno.
- **S-119**: giudizio del treno.

## EMENDAMENTI
- **R1 (piattaforma)**: la spec di A nomina solo strumenti esistenti su Darwin/aarch64; BOLT espunto. Misura: build ×2 ⇒ hash identico; PGO on/off A/B interleaved sui sei micro.
- **R2 (bande azzerate)**: dopo A, NESSUN gate sul treno finché banda micro (N≥2) e banda held-out (N≥3) non sono ri-misurate sulla pipeline nuova. Misura: verdetti-nulla committati prima del manifest.
- **R3 (circolarità del profilo)**: il workload di profiling è NOMINATO e non coincide coi giudici (preferito: WP+held-out profilano, i micro giudicano); se coincide, la circolarità si dichiara nel verbale. Misura: lista dei file di profilo nel criterio PRE.
- **R4 (spec del treno — il cuore)**: (i) manifest pre-registrato: vagoni per NOME e ordine, cap 5; (ii) admission PER-vagone (parità output, dump, batteria, corpus) PRIMA dell'imbarco; (iii) l'A/B di promozione è UNO SOLO: binario-treno vs pin — mai somma di delta da A/B distinti (già vietato); (iv) guardie con banda da **nulla-treno di taglia comparabile** (byte-delta ±30%): la banda delle nulle piccole non si estende per fede a +15 KB; (v) tie su valori a 2 decimali con le regole S-116(c) (uguale⇒PASS promozione, uguale⇒tiene guardia); rc SOLO da file S-116(d); (vi) **bisezione pre-registrata**: treno bocciato ⇒ si stacca l'ultimo vagone e si rigiudica, max 2 iterazioni, poi verdetto secco. Senza (vi) la domanda «quale vagone incolpo» ricrea S-115.
- **R5 (formula di promozione)**: UNA formula committata prima dello smoke — proposta: promozione se la categoria peggiore migliora ≥ soglia E nessuna categoria peggiora oltre max(2×spread_dep; banda(cat)) E held-out entro banda misurata. La scelta (i)/(ii) del verdetto S-116 (ridurre la tassa calls vs gate a beneficio netto pesato) è **decisione utente pre-registrata**, non deroga in corsa.
- **R6 (leggibilità)**: `treno-manifest.md` ≤20 righe; verdetto `.out` appeso dagli script; scoreboard con voce «vagoni imbarcati: N».

## KILL-SWITCH
- **KS-A**: build non riproducibile ×2 O (guadagno micro globale <2% E banda nulla nuova non più stretta della vecchia, globale ≥10,00) ⇒ A si archivia in 1 sessione, si tiene solo LTO se gratis.
- **KS-B**: treno bocciato 2 volte DOPO bisezione ⇒ B si sospende, si apre C.
- **KS-D**: vagone senza direzione firmata (smoke R=2, segni concordi) non si imbarca — nessun «forse» a bordo.
- **Trigger C**: dopo A′+un treno giudicato, se prop resta >6× ⇒ C diventa cantiere nominato, non riserva.

## APPARATO minimo
Solo `scripts/pgo-build.sh` (profilo+build+hash+stash in UN atto, REGOLE §2) e lo script nulla-treno. Nient'altro: il resto esiste.
