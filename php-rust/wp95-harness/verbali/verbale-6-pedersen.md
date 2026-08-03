# Verbale sedia 6 — Pedersen (Concilio WP-95) — confine per-richiesta, lifecycle, igiene probe

**VERDETTO: PASS CON RISERVE — S-93.0 consumabile in ADVISORY; DUE refutazioni capitali sul lato B3/igiene.**
B1/B2 sono lavoro pulito: la coppia alloc/dealloc vista dal canale VIVO (huge-sites.out:13-18, 43-48) con falsificazione per NOME dei sospetti è il metodo giusto.

## Q1 — Igiene del probe
Le identità dei build strumentati sono dichiarate BENE (huge-sites.out:6-7: tre hash, parent 2859c81, feature mem-census) — non scambiabili per la release. **Ma il ripristino del pin è solo DICHIARATO** (huge-sites.out:8-9 «RIPRISTINATO d45b57843eeb1375», WP_SESSION_93.md:19): nessuna ricevuta in-band (shasum post-ripristino nel .out, né ri-run del gate lever-pins DOPO il probe). Il «PASS gate lever-pins v10» in NEXT_SESSION è pre-S-93.0. Porta 8199 e log grezzi del trace: smaltimento non dichiarato per NOME. **REFUTAZIONE CAPITALE n.1**: il criterio 3 («binari pinnati INVARIATI», GIUDIZIO_C_AXUM.md:62-63) poggia su una dichiarazione, non su una ricevuta.

## Q2 — Arm senza readback (A-DL41)
L'arm legge l'env (worker_pool.rs:521-523) e scatta in silenzio (worker_pool.rs:630-633): **nessuna eco in banda**. Un delta NULLO da arm muto è indistinguibile da un arm mai scattato (var mal digitata, feature spenta, worker senza richieste); su w=4 il conteggio atteso fired==4 è inverificabile. **REFUTAZIONE CAPITALE n.2**: «LEVER-2 REFUTATA CON MISURA» (WP_SESSION_93.md:31) è insostenibile come scritta dal SOLO probe. La conclusione sopravvive in ADVISORY sulla gamba m90 (arena committed invariata a exit_collect_mi, huge-sites.out:72-74) — gamba indipendente e già committata — ma non è promuovibile senza eco.

## Q3 — Punto di chiamata post-send
Nessun confine di richiesta violato (risposta già inviata, zero latenza aggiunta). **Però** il collect gira DOPO il dec Release di OUTSTANDING (worker_pool.rs:619-620) — è lavoro heap-mutante del worker FUORI dalla finestra del testimone: un censimento preso a outstanding==0 può sovrapporsi al collect in corso. Stessa classe del morso WP-94 «la coppia non prova la finestra». Fix a costo zero: ordine send → collect → dec (worker_pool.rs:607, 630-633 prima di 617-627).

## Q4 — huge_note e disciplina del canale census
Guardia di rientranza IN_TRACE corretta (main.rs:71-73, 100); l'early-return su size<HUGE_MIN (main.rs:76-78) protegge dalla ricorsione dell'env-read. Due vizi: (a) **realloc asimmetrico** (main.rs:120-124): nota `realloc` sul size nuovo ma NESSUN `dealloc` del size vecchio huge ⇒ il saldo alloc−dealloc del canale huge deriva sui path realloc (i sei chunk bumpalo passano da alloc/dealloc, quindi B1/B2 non ne soffrono — ma il canale resta storto); (b) le allocazioni del trace stesso (backtrace, format) passano dal GlobalAlloc e vengono CONTATE da galloc_note/gfree_note: con PHPR_HUGE_TRACE=1 i contatori census sono contaminati e vanno dichiarati VOID.

## Q5 — Priorità S-94.0 (FONDAMENTALI-first)
1. **battery61 riproducibile, modo nativo** (criterio 5, mezza sessione) — in cima, debito di 31 sessioni.
2. **Campagna m91 con battery-91pre** (MAI girata): certifica ANCHE le modifiche sorgente env-gated dormienti di S-93.0 — nessuna cifra nuova prima della battery.
3. **Attribuzione slope ~18,8 MB/worker per NOME** (criterio 1, canale m91) — con A-PP-75 attuato prima di fidarsi del testimone.
4. Leva per-file del preludio: DOPO, sessione dedicata, gate parità COMPLETI. Apparato solo se blocca (timebox).

## Emendamenti
- **A-PP-74**: ogni arm env-gated emette al fire una riga ascii-nuda `arm=<nome> fired=<n> thr=<id>`; raw senza fired==W ⇒ run VOID (legge A-DL41).
- **A-PP-75**: lavoro post-risposta DENTRO la finestra del testimone (send → lavoro → dec OUTSTANDING), o dichiarato fuori-testimone in banda.
- **A-PP-76**: huge_note simmetrico su realloc — emettere anche `dealloc` del layout vecchio (main.rs:120-124).
- **A-PP-77**: dopo OGNI probe con build strumentata: ricevuta di ripristino (ri-run gate lever-pins o shasum in-band nel .out) + smaltimento porta/log dichiarato per NOME.
- **A-PP-78**: PHPR_HUGE_TRACE=1 ⇒ contatori galloc/gfree del run dichiarati VOID (auto-contaminazione del trace).

## Kill-switch
- **KS-PP-95-1**: nessun esito di leva da arm senza eco in banda è promuovibile oltre ADVISORY.
- **KS-PP-95-2**: lavoro del worker schedulato dopo il dec di OUTSTANDING è fuori testimone — censimenti a outstanding==0 con tale lavoro pendente sono VOID.
- **KS-PP-95-3**: pin «ripristinato» senza ricevuta in-band = pin DICHIARATO — ogni claim di parità resta ADVISORY finché il gate non rimorde.

**Refutazioni capitali: SÌ (2)** — ripristino pin non provato; refutazione LEVER-2 muta come scritta (salva in ADVISORY via m90).
