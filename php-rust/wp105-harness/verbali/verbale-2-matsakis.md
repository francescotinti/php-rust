# Verbale sedia 2 — MATSAKIS (ownership/aliasing/borrow) — Concilio WP-105

## VERDETTO: CONCORDO CON EMENDAMENTI
Nessuna refutazione capitale. S-103 è una sessione di misure oneste; ma due
dei suoi sigilli valgono meno di quanto la rotazione li fa pesare.

## Refutazioni

**R1 (punto a) — Un arbitro che non ha mai prodotto un rosso non è un
arbitro.** 19a/19b PASS al primo colpo provano la parità del pin corrente,
non la capacità delle fixture di mordere il −1 del MOVE. Il claim «la
fixture MORDE» (commento di 19a) è oggi una deduzione statica su OBS-8,
non un fatto dinamico: nessuna build in cui l'handle mosso NON conta è mai
stata vista fallirle. In più l'«attesa scritta PRIMA» è stata emendata
post-oracle (ordine dell'echo multi-argomento): registrata come cosmetica
e l'ordine semantico regge, ma un'attesa non eseguibile alla lettera è
mezza attesa — il rito non va ripetuto così.

**R2 (punto a-bis) — RC-MA-104 è chiusa per la magia, non per i hook.**
Il MOVE H-C1b serve anche il sentiero property-hook (`push_hook`, run.rs
~3641, stesso `target.clone()` + `continue` che scarica l'handle del
braccio): lì gira codice utente col collector invocabile mid-arm, stessa
leva già spedita, zero fixture. Le zone `==2` (OBS-4/6) e l'aritmetica
OBS-8 restano non arbitrate su quel sentiero.

**R3 (punto b) — L'enumerazione per annotazione non può vedere ciò che non
annota.** `dcn!` conta i bracci annotati, non le fine-vita: la prova è nel
census stesso («2 fallback IC di warmup NON annotati», emersi solo come
coda). L'attesa v2 «a zero scarti» conferma i siti annotati contro se
stessi; un drop per-iter su un sentiero non annotato sarebbe invisibile e
l'attribuzione per sito×specie di H-C2 ne erediterebbe l'errore in
silenzio. Manca un totale indipendente (ledger).

**R4 (punto c) — Il confine DropC regge solo se è dinamico nel punto di
morte.** I 3 DropC/iter (2 handle PropGet + 1 PropSet) sono fuori canale
per DICHIARAZIONE; il confine è tracciato solo se il fast-out è espresso
come chiamata a `is_gc_container` sul valore che muore, mai come
specializzazione statica per-sito («qui muore sempre un Long»). `Ref`
porta la cella Rc: classificarlo scalare perde un decremento — l'esaustività
S-102 del predicato copre la classificazione, non l'USO. Un «reject in
review» non è un dente; e il debug_assert nested-Ref vive solo nelle build
debug/census (annotato onestamente, ma il release resta senza denti lì).

## Emendamenti

- **A-MA-105-1 (mutation-check, pre-atto della leva)**: build sabotata
  feature-gated (clone + drop anticipato dell'handle al posto del move);
  19a/19b DEVONO andare rosse; output rosso archiviato in wp105-harness.
  Solo dopo, le fixture sono arbitri nei gate.
- **A-MA-105-2 (fixture 19c)**: hook `get` con self-cycle e
  `gc_collect_cycles` mid-arm, varianti base=1 e soglia esatta; attesa
  prima, oracle ×2 modi.
- **A-MA-105-3 (ledger fine-vita)**: nella build census, contatore totale
  di fine-vita Zval indipendente dai `dcn!` (Drop census-gated o wrapper
  al punto unico di discard); vincolo: totale ≡ Σ siti annotati + code
  nominate.
- **A-MA-105-4 (dente fast-out)**: nelle build census/debug il ramo
  fast-out asserisce `!is_gc_container(v)` — violazione = panic, non
  review.

## Kill-switch

- **KS-MA-105-1**: se la build sabotata NON fa fallire 19a/19b, le fixture
  non arbitrano RC-MA-104 ⇒ la voce si RIAPRE e la tavola emendata perde
  il verdetto dinamico.
- **KS-MA-105-2**: se il ledger diverge dai conteggi enumerati oltre le
  code nominate, l'attribuzione per sito×specie di H-C2 è VOID (la
  promozione può ancora passare sull'aggregato Δ_A/B; l'attribuzione no).

## Priorità S-104

Ordine §1 (verdetto A/B) invariato. A-MA-105-1 e A-MA-105-4 sono piccoli e
vanno DENTRO il punto 2 come pre-atto e dente della leva stessa — non
nuovi prefissi (i prefissi sono consumati; sessioni-senza-Δ-oggetto = 2:
niente che ritardi l'apertura oltre mezza giornata). A-MA-105-2/3 nel
timebox igiene o contestuali al gate pieno della leva.
