# s143-criterio-istruttoria — REGOLA PRE-REGISTRATA (firmata PRIMA di leggere i dati; deliberato concilio S-143)

1. Oggetto: census CH_* per CLASSE e per TAGLIA dei ~471M alloc/free su suite ORM — monobinario census (grade CENSUS, MAI tempo), ×2 repliche, r1==r2 al singolo evento; parità per NOME vs baseline16 rc=0; sentinelle pgrep pre/post stampate (non-gate).
2. Classi enumerate: obj (Props/payload oggetto) · arr (PhpArray box) · str (PhpStr buffer) · vecargs (Vec argomenti call) · rczval (box Rc di Zval condivisi) · other (residuo = galloc_n − Σ classi, dichiarato). Denominatore dal sorgente del giudice (galloc_n/gfree_n della stessa run).
3. REGOLA DI DECISIONE (sintesi concilio; dissensi a verbale nel fascicolo): quota coppie obj(+props Rc) ≥40% ⇒ A-poi-B (A RICONDIZIONATA pool+refcount, mai arena-sweep) · <25% ⇒ B sola / B-poi-A · 25–40% ⇒ RICONVOCA il concilio su terza via · clausola Klabnik: churn memcpy-dominato ≥60% ⇒ B-prima.
4. Questa tornata emette SOLO conteggi e quote di conteggio: nessun secondo, nessun prezzo (i prezzi arrivano dalla sonda monobinaria classe S-138, voce c dell'istruttoria).
5. Bilancio bytes: la run stampa anche i GB per classe; lo scarto free>alloc (33,8 vs 29,4 GB S-141) va o chiuso o attribuito per NOME (Leijen R3) prima di ogni prezzo sui GB.
6. Esiti pre-registrati: (i) quota in una banda della regola p.3 ⇒ si applica la regola; (ii) r1≠r2 oltre 1% su una chiave ⇒ si dichiara e si replica; (iii) probe muto sui CH_* allo smoke ⇒ STOP (rc=8), niente run.
7. Strumentazione cfg-gated (feature mem-census/zval-census) su HEAD post-pin; il pin resta lo STASH immutabile s142 (lezione S-142: niente gate di byte-identità su edit .rs).
8. Il profilo ORACLE per famiglia (voce b) ha criterio proprio alla sua apertura; questa regola non lo pregiudica.
