# WP_SESSION_77.4.2 — 2026-07-30 (sera)

**Mandato**: binding steps 4-5 del programma WP-77.4 (deferiti da WP-77.4.1):
KM-77-2 concurrent gate + KS-M1-Gate-Upgraded byte-snapshot ×3 workload +
metriche M4. Baseline: phpr 7bf53854 @ 45d3bc5. **Esito: ENTRAMBI I
KILL-SWITCH CHIUSI PASS**, verdetti da runner COMMITTATI. Commit: 927e66f.

## Verdetti

| Gate | Verdetto | Evidenza |
|---|---|---|
| **KM-77-2** | ✅ PASS | 15 round × 2 curl parallele vs php-server (cli-mode): 30/30 PASS, 30 marker unici, 0 leak; **controllo positivo MORDE** (stale $GLOBALS iniettato ⇒ FAIL) |
| **KS-M1-Gate-Upgraded** | ✅ PASS | A: axum hello 7/7 byte-id (42 B) · B: WP front 57 428 B + post 71 093 B + feed 1 675 B, tutte 7/7 byte-id (5 seq + 2 par) · C: ORM 3484 **3E/13F, 16 nomi IDENTICI** alle famiglie catalogate |

Verdict-file: `/Volumes/Extreme Pro/Claude/wp77-harness/gate-out/
{km77_2,ks_m1_upgraded}.verdict` — emessi da `wp77-harness/
run_gate_km77_2.sh` e `run_gate_ks_m1_upgraded.sh` (committati nel repo).
Pin nomi ORM: `wp77-harness/gate-baseline/orm7742-fails.txt` (16 righe;
provenance: conteggio = stato gate WP-72, nomi = famiglie recipe 2026-07-15).

## Metriche M4 (PULITE — prima passata scartata: contaminata dall'ORM
phpunit in parallelo; rimisurate a macchina scarica)

| Endpoint | Mediana (n=10) | Footprint fisico (vmmap) |
|---|---|---|
| Axum hello-world (placeholder) | 0,3 ms | 4,3 MB |
| cli-server (fixture km77) | 6,8 ms | 18,4 MB |
| WP front (phpr -S, DB wp) | 352 ms | **350–367 MB STABILE su ~46 req (no growth)** |

Abort-times: N/A (M3 non costruito, nulla è abortito).

## Caveat onesto di perimetro

Il handler Axum è ancora **placeholder** (non esegue PHP): il workload A
valida plumbing+stabilità byte della risposta; l'isolamento $GLOBALS reale
è validato sul percorso **cli-server** (dov'è il PHP per-richiesta oggi).
Il KM-77-2 "task async simultanei nello stesso processo" in senso pieno
richiede il Vm nel handler ⇒ WP-77.5.

## ⭐ Lezioni

- ⭐⭐ **Un fixture che non può fallire non è un gate**: il
  `gate_km77_2_concurrent.php` di WP-77.4.1 settava il marker a 'initial'
  e verificava 'initial' — un leak sarebbe passato VERDE. La forma giusta:
  stamp UNICO per richiesta + controllo positivo che dimostri il morso
  (stale iniettato ⇒ FAIL). Superseded, non emendato: il vecchio resta
  come storia.
- ⭐⭐ **7 hash identici su corpi VUOTI = controllo positivo fallito**
  (variante della lezione istogramma-tutto-zero WP-72): `/?p=1` è una 301
  body-vuoto e lo snapshot dava "byte-id" su 0 byte. Ogni parity-check di
  snapshot vuole la guardia `size>0`.
- ⭐ **`wait` nudo con un server backgrounded nello stesso shell non torna
  mai**: nel runner il `wait` del round aspettava anche il server ⇒ hang
  al primo run (nell'esplorativo funzionava perché il server era
  daemonizzato a parte). `wait $PID1 $PID2` espliciti.
- ⭐ **Latenze misurate con un phpunit concorrente sono spazzatura**: la
  prima passata M4 è stata scartata e rifatta post-ORM (variante del
  "wall inquinato → solo user CPU": qui l'inquinamento era la CPU di un
  altro run nostro).
- ⭐ I file WP-77 vivono in DUE posti (harness esterno + repo-interno
  `wp77-harness/`): il pre-flight che guarda solo l'esterno dichiara
  "fixture mancanti" che invece sono committati nel repo. Fonte di verità
  = repo-interno (committato).

## Stato binario

phpr **7bf53854** INVARIATO (unico build: php-server con feature
axum-server — phpr ri-verificato dopo). Stash: `phpr-wp77.4` (già =
7bf53854, nessun ri-stash). php-server con axum: 17,8 MB.

## ⚖️ Concilio (chiusura sessione — verbali `wp77-harness/COUNCIL_WP77.5_REVIEWS.md`)

**4 MI OPPONGO (Matsakis, Pedersen, Leijen, web-runtime) + 6 CON
EMENDAMENTI ⇒ programma "Vm in task_local" RESPINTO; verdetti di questa
sessione RIETICHETTATI**: Pedersen ha verificato sul tree che
`request_end()` ha ZERO chiamanti e che il cli-server usa Vm fresco +
mass-teardown WP-72 ⇒ i PASS valgono per il confine cli-server/fresh-Vm,
NON per il reset-e-riusa (kill-switch M1 reale APERTO). Metriche M4
retrocesse a indicative (wall≠user CPU, niente oracle, "no growth" non
falsificato a 46 req). Architettura convergente per WP-77.5: worker-attore
stile php-fpm (thread dedicati proprietari della Vm, mpsc bounded +
oneshot); request_shutdown() in ordine Zend prima di request_end().

## ⭐ Lezione aggiunta dal concilio

- ⭐⭐ **Un kill-switch va chiuso sul MECCANISMO che deve gateare, non su un
  meccanismo omologo**: KM-77-2 doveva gateare il confine reset-e-riusa;
  è stato chiuso sul confine fresh-Vm (che WP-72 aveva già provato). Il
  gate era vero, l'etichetta no — gate-inflation. Prima di dichiarare un
  kill-switch chiuso: verificare CHI CHIAMA il codice sotto gate.

## Prossimo (WP-77.5)

Recepire il concilio (VINCOLANTE) PRIMA del codice: rinegoziare M9 →
worker pool php-fpm-style; G1 spike Vm-storage compila; G2 gate
due-richieste-stesso-Vm; G3 tripla+amp sul path reuse; G4 coppia oracle.
M3/M4 vincolati agli emendamenti S/W/G/B/L.
