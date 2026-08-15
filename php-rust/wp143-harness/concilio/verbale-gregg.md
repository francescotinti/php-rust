# Verbale sedia GREGG — lente: metodologia di misura e attribuzione (S-143)

VERDETTO: CONCORDO CON EMENDAMENTI
DELIBERA: ISTRUTTORIA-PRIMA — 1 sessione timeboxed (census CH_* per classe + profilo oracle per famiglia), con regola di decisione A-vs-B PRE-REGISTRATA prima di leggere i dati; la direzione «ciclo di vita» è già firmata bilateralmente, la RIPARTIZIONE A/B no.

## §Analisi (mandato inverso: cosa sappiamo oggi che ieri non sapevamo)
1. Sappiamo, con quattro falsificazioni pre-registrate consecutive (dim-write Δ≈0, HC1 0,13%, L-RD1 0,24–0,53%, teardown=canale intero ~2%), che il divario ORM è DIFFUSO: nessun sito supera la risoluzione della coppia (±0,7%). È conoscenza positiva di grado A/B: legittima da sola l'abbandono delle micro-leve.
2. Sappiamo (S-129, bilaterale, chiusura 96%) che la tassa per-statement è ~10× QUASI INVARIANTE per forma, e (S-103) che il costo/op del loop è fermo a 9–10 ns. Questa è l'unica evidenza BILATERALE che punta al ciclo di vita per-valore — ed è la vera base della scommessa, non il profilo.
3. Il resto NON regge da solo una scelta tra A e B: il profilo §2 è campionario grade=INDIZIO a un lato solo; i «26–28 s cumulati» sono una SOMMA di magnitudini non ripartite — REGOLE §4 vieta di trattarla come cifra; i prezzi §3 sono «plausibili» con banda ~2× (alloc 3,8–7,1 s); «other» 26,6% (11,3 s) è la voce più grande del profilo e non ha nome.
4. Quindi: la scommessa strutturale in sé è deliberabile OGGI (direzione+meccanismo firmati). La scelta A-vs-B no: A compra una QUOTA IGNOTA dei 471M (census per-classe assente); B bersaglia memops+churn (9,8 s) che sono INDIZIO unilaterale — se anche Zend paga memcpy in proporzione simile, B compra meno del previsto (feedback-one-sided-profile: prerequisito, non rifinitura, per B; per A il census CH_* è prerequisito per definizione — «quota da censire» lo dice il dossier stesso).

## §Emendamenti
- **R1 — Regola di decisione pre-registrata** (≤10 righe, PRIMA del census): se quota oggetti+Rc dei 471M ≥ soglia dichiarata (proposta: ≥35% coppie o ≥10 s ai prezzi firmati) → A-poi-B; sotto → B-poi-A. Niente lettura dei dati prima della firma della regola.
- **R2 — Prezzi firmati, non plausibili**: sonda monobinaria classe S-138 su alloc/free e gc_note (§7.4) DENTRO l'istruttoria; senza prezzo firmato il budget di A resta una banda 2× e il kill-switch non ha giudice.
- **R3 — «other» 26,6%**: la riquantificazione S-141 va chiusa o la voce dichiarata fuori-budget della scommessa; una struttura che promette 26–28 s con 11,3 s senza nome ha il denominatore scoperto.
- **R4 — Vertical slice come primo atto della via scelta**: arena/layout su un perimetro nominato (oggetti senza __destruct/weakref), giudicato sulla SUITE ORM, mai solo sulle micro (le micro hanno già ingannato quattro volte in direzione opposta).

## §Veti (Q3)
- NaN-boxing: CONFERMA (B = Option/niche, safe-only; NaN-boxing non necessario né ammesso).
- Contenitori sul call path: CONFERMA; l'handle di A è un'indirezione sul cammino caldo — cade sotto lo spirito del veto: va prezzata nel modello sostitutivo.
- Alloc-removal senza modello del costo SOSTITUTIVO: CONFERMA e RAFFORZO — è il cuore di A: bump+deref+sweep per-request vanno prezzati PRIMA dell'A/B, col binding output-capture intatto.
- SSO inline: CONFERMA (fuori perimetro A/B).
- Leva GC note-time (WP-21): CONFERMA — la famiglia gc è sweep-dominata (§3); A può contare la nota obj 56,5M solo come collaterale, mai come canale giustificante.
- Notti su PhpStr-full: CONFERMA.

## §Kill-switch (Q4)
- **K1 (istruttoria, 1 sessione)**: census CH_* r1==r2 <1%; se quota oggetti sotto soglia R1 → A decade a seconda via, senza appello.
- **K2 (prezzi)**: se la sonda R2 firma alloc/free tale che il budget A < 2 s → il canale alloc di A è falsificato; giudice: sonda monobinaria, stessa sessione.
- **K3 (bilaterale)**: se il profilo oracle mostra memops/churn in proporzione comparabile a phpr → i bersagli di B non sono divario → B decade; giudice: stessa lente sui due motori, net-pavimenti per-binario.
- **K4 (slice)**: la via scelta deve muovere il rapporto ORM di suite ≥5% (fuori banda ±0,7%, 2 gambe/lato) entro 5 sessioni dalla prima promozione, o si dichiara falsificata e si torna al concilio. Niente proroghe implicite: la quinta sessione emette verdetto.
