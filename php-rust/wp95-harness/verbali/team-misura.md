# team-misura — verbale di team (fase 2, Concilio WP-95)

**Sedie**: Bak (5) · Pedersen (6) · Leijen (7) · Gregg (9). **Relatore**: team-misura.
**Fonti vincolanti**: `wp95-harness/verbali/verbale-{5-bak,6-pedersen,7-leijen,9-gregg}.md`;
oggetto giudicato: `wp93-harness/huge-sites.out` (letto per intero).
**Mandato**: riconciliare o registrare i dissensi, MAI benedire.

---

## 0. FONDAMENTALI (in testa, per direttiva)

Il team NON ha misurato nulla di nuovo. Ha giudicato cifre già pubblicate e ha trovato
**un errore aritmetico agli atti** (§4, atto A2) oltre ai difetti di grado. Il contatore
delle sessioni-senza-misura-full è fermo a **WP-85 (8 sessioni)**: nessuna leva di questo
filone ha oggi un giudice, perché la predizione-misurata (WP-48) richiede un «prima» fresco.

---

## 1. Convergenze (4/4 sedie, nessun dissenso)

1. **Il probe slope di B3 è uno SCREEN, non una misura.** Un run per punto, due punti W,
   varianza mai stimata nel probe. La risoluzione dichiarabile è lo **spread inter-build
   38229 B/worker** (baseline b048b697 18852538 vs ctrl 8e966efd 18814309), che **supera** il
   delta della leva **21837 B/worker**. Verdetto legale: «|effetto| < ~0,2% alla risoluzione
   del probe», mai «delta = 21837». (Bak Q1 · Gregg Q1/KS-BG-95-1 · Pedersen Q2 · Leijen Q2)
2. **La qualifica «REFUTATA CON MISURA» va retrocessa nel GRADO.** Tutte e quattro le sedie
   la rifiutano come scritta; nessuna sostiene che LEVER-2 funzioni.
3. **B1 e B2 reggono nella sostanza**: sei chunk di raddoppio (x2+16, firma bumpalo) di UNA
   arena — il parse del preludio stdlib in `lower_prelude_uncached` — con falsificazione per
   NOME dei sospetti (UNIT_CACHE, STUBS, arene dei Module) e coppia alloc/dealloc vista dal
   canale in-band Rust. Questo è il metodo giusto e il canale di fede (Leijen KS-DL-95-2:
   in caso di divergenza, fede alla coppia in-band, la stat mimalloc è solo testimone).
4. **`malloc_huge` dei dump m90 è CUMULATIVO** e va declassato ovunque; ogni derivata di m90
   che l'ha consumato come *retained* è sospetta e da censire (Gregg A-BG-72).
5. **La leva «arene per-file» è la candidata giusta**, ma NON in questa sessione e non prima
   che esistano baseline e giudice (Leijen A-DL-68 dà il via libera dal perimetro allocatore;
   Bak la mette al 3° posto; Gregg al 4°; Pedersen «dopo, sessione dedicata»).
6. **Nessuna cifra di questo filone è verdict-grade.** Il file si autodichiara ADVISORY
   (riga 2) e il team conferma che quello è il **tetto**, non il pavimento: alcune righe
   stanno sotto.

---

## 2. Conflitti registrati (posizione di ciascuna sedia)

### CONFLITTO M-1 — che cosa resta in piedi di LEVER-2 (il più importante)
- **Pedersen**: la conclusione «sopravvive in ADVISORY sulla **gamba m90**» (arena committed
  invariata a exit_collect_mi, huge-sites.out:72-74) — gamba indipendente e già committata.
- **Leijen**: **quella gamba non esiste**. In release `_mi_prim_decommit` su macOS fa
  `madvise(MADV_FREE_REUSABLE)` ma pone `*needs_recommit=false` (prim.c:495-504, gate
  prim.c:503) e `mi_os_decommit_ex` decrementa `committed` SOLO se `needs_recommit`
  (os.c:590-591): **`committed` non scende MAI su purge in release, per disegno**. Inferire
  «committed piatta ⇒ mi_collect non decommitta» NON SEGUE. Resta il solo braccio A/B fisico.
- **Bak / Gregg**: il braccio A/B fisico è **sotto il pavimento di rumore** del probe stesso.
- **Composizione del relatore** (Pedersen non aveva letto Leijen; Leijen non aveva letto
  Bak/Gregg sul pavimento): **la gamba m90 cade per il meccanismo (Leijen), il braccio A/B
  cade per la risoluzione (Bak/Gregg) ⇒ non resta nulla che regga il verbo «refutata».**
  Grado finale di B3: **SCREEN — «effetto non rilevato sotto la risoluzione del probe»**.
  L'ADVISORY concesso da Pedersen è **ritirato** perché poggia sull'unica premessa che Leijen
  refuta per file:riga. Nessuna sedia ha argomenti contro questa composizione; è comunque
  registrata come dissenso di partenza, non come unanimità originaria.

### CONFLITTO M-2 — «transiente» al livello malloc vs «riserva» al livello OS
- **B2 agli atti / Leijen**: spike transiente, liberato e purgabile; nessuna controindicazione.
- **Bak (KS-BB-95-2)**: la dicotomia riserva/spike è falsa al livello provisioning — al
  restart il thundering herd sincronizza gli spike (picco ~W×39,5 MB) che il capacity planning
  DEVE coprire; «freed ≠ decommitted ≠ non-provisioned».
- **Attrito reale**: Bak porta a sostegno l'arena committed 151 MB dei raw m90 — **la stessa
  cifra che Leijen dichiara illeggibile** (os.c:591 ⊣ prim.c:504).
- **Composizione**: la tesi di Bak **regge sul picco sincrono** (argomento di scheduling, non
  di allocatore) e **cade sull'evidenza citata**. Nel testo emendato: tenere la clausola
  provisioning, **sostituire** l'appoggio a `committed` con il picco ×W misurato.

### CONFLITTO M-3 — che cosa si misura per PRIMO in S-94.0
- **Bak**: battery61 riproducibile → probe slope v2.
- **Pedersen**: battery61 (debito 31 sessioni) → campagna m91 con battery-91pre (MAI girata).
- **Leijen**: **sanare il canale stat** (MI_STAT=1 + coppia in-band) PRIMA di m91.
- **Gregg**: sanatoria di carta → **COPPIA FULL stessa-sera** (contatore fermo a WP-85) → m91.
- **Composizione**: §5. Il dissenso è ordinale, non sostanziale — tutte le sedie mettono la
  **leva per-file DOPO**, e nessuna accetta cifre nuove prima dell'apparato di battery.

### CONFLITTO M-4 — sorte del canale stat mimalloc
- **Leijen (A-DL-65)**: census m91 con `MI_STAT=1` esplicito e livello dichiarato nel banner.
- **Pedersen (A-PP-78)**: con `PHPR_HUGE_TRACE=1` i contatori galloc/gfree sono
  auto-contaminati (backtrace+format passano dal GlobalAlloc) ⇒ VOID.
- **Non contraddittori, ma incompatibili nello stesso run.** Regola composta: **MI_STAT=1 e
  TRACE=1 mai nella stessa esecuzione**; il run TRACE serve a NOMINARE, il run MI_STAT a
  CONTARE; ogni cifra da stat gated senza livello MI_STAT nel banner è invalida.

### CONFLITTO M-5 — grado del 4,42× e la sua forza di priorità
- **Gregg (KS-BG-95-2)**: magnitudine di canale su UN workload, non priorità di roadmap
  finché la media non è rimisurata; pin solo dopo R≥3 (A-BG-74).
- **Bak**: il 4,42× non compare come priorità; la sua leva n.1 è lo snapshot del preludio.
- **Composizione**: nessun conflitto duro — il 4,42× **motiva** la leva per-file ma **non la
  autorizza**; l'autorizzazione viene dalla coppia full.

### Punti senza dissenso ma con effetto sul testo
- **Pedersen A-PP-76**: `huge_note` è **asimmetrico su realloc** (main.rs:120-124: nota
  `realloc` sul size nuovo, nessun `dealloc` del size vecchio huge) ⇒ il saldo del canale huge
  deriva sui path realloc. Pedersen stesso esclude l'impatto su B1/B2 (i sei chunk bumpalo
  passano da alloc/dealloc), ma **il canale resta storto** e va sanato prima di riusarlo.
- **Pedersen A-PP-75**: il collect gira DOPO il dec Release di OUTSTANDING
  (worker_pool.rs:619-620 vs 630-633) ⇒ lavoro heap-mutante **fuori dalla finestra del
  testimone**; un censimento a outstanding==0 può sovrapporsi al collect. Fix a costo zero:
  ordine send → collect → dec.

---

## 3. Tabella dei gradi per cifra — `wp93-harness/huge-sites.out`, riga per riga

Legenda: **VERDICT** = pinnabile/ledger · **ADVISORY** = consumabile con provenienza, non pin
· **SCREEN** = indica direzione, vietato il pin e vietato il verbo «refutata/confermata» ·
**VOID** = non consumabile, va rimossa o marcata refutata.

### Intestazione
| riga | cifra / clausola | grado deciso | motivo |
|---|---|---|---|
| 2 | `grade=ADVISORY` (globale) | **da sostituire** con grado PER SEZIONE | il globale è il *tetto*: B3 sta sotto |
| 6-7 | tre hash di build + parent + feature | **FATTUALE** (regge) | identità dichiarata bene, non scambiabile per la release (Pedersen Q1) |
| 8 | `phpr d5ce86e3 INVARIATO (mai ricompilato)` | **ADVISORY** | plausibile ma non ricevutato in-band |
| 9 | `php-server RIPRISTINATO d45b578 dopo il probe` | **DICHIARATO — non ricevuta** (≈ VOID come prova) | nessun shasum in-band né ri-run del gate lever-pins DOPO il probe; il PASS in NEXT_SESSION è pre-S-93.0 (Pedersen KS-PP-95-3) |
| 10 | carico/W/porta 8199 | **FATTUALE**; smaltimento porta e log grezzi **non dichiarato** | A-PP-77 |

### B1 — i sei siti (righe 13-40)
| riga | cifra | grado | motivo |
|---|---|---|---|
| 13-18 | le sei `size` (622576 … 19922928) | **ADVISORY forte** (autoverificante) | catena x2+16 verificata su tutti e 5 i passi; canale in-band, non stat C. Promuovibile a VERDICT nel census m91 con MI_STAT dichiarato |
| 19 | `somma=39423200` | **VOID — ERRATA** | la somma vera è **39223200** (delta +200000 agli atti). Vedi atto A2 |
| 20 | relazione di raddoppio | **VERDICT-grade** (identità aritmetica) | verificata: 2·s+16 esatta 5/5 |
| 21-22 | 6 huge dopo req1 e dopo req3 (numero FISSO) | **ADVISORY** | riproduzione intra-run, non inter-run |
| 23-25 | `nota-somma` residuo os-good-size | **VOID come scritta** | dipende dalla somma errata: il residuo corretto è 39911424−39223200 = **688224**, non 488224 (cifra ripresa anche da Leijen Q4) |
| 28-38 | sito/via/radice/identità (backtrace) | **ADVISORY** | attribuzione per NOME dal vivo; il canale TRACE non falsifica sé stesso |
| 39-40 | falsificati per NOME (UNIT_CACHE, STUBS, Module) | **ADVISORY forte** | è la parte metodologicamente migliore del file (Pedersen) |

### B2 — la natura (righe 43-59)
| riga | cifra / clausola | grado | motivo |
|---|---|---|---|
| 43-48 | le sei `dealloc` (ordine inverso) | **ADVISORY forte** | coppia in-band = canale di fede (KS-DL-95-2); promuovibile a VERDICT in m91 |
| 49-50 | «tutti e sei liberati dentro la prima richiesta» | **ADVISORY** | regge |
| 51-54 | conseguenza-1: «riserva fissa mai liberata» REFUTATA; stat cumulativa | **ADVISORY, ma da RIFORMULARE** | la conclusione sopravvive, la *ragione* agli atti è imprecisa (§ sotto) |
| 55-57 | conseguenza-2: «il decremento NON morde sul canale stat» | **VOID come scritta** | non «non morde»: **non è compilato** (free.c:612 / stub 635-637 a MI_STAT=0) |
| 58-59 | `natura=SPIKE TRANSIENTE … non riserva né leak` | **ADVISORY con riserva** | vero al livello malloc; **falso al livello provisioning** per lo spike sincrono ×W (Bak KS-BB-95-2) |

### B3 — LEVER-2 (righe 61-76)
| riga | cifra | grado | motivo |
|---|---|---|---|
| 61 | titolo «REFUTATA con misura» | **SCREEN** (retrocessione) | 4/4 sedie |
| 62-64 | protocollo (1 run per W, W∈{1,4}) | **FATTUALE** — ed è la prova che è uno screen | zero repliche, zero varianza |
| 65-68 | 4 coppie `max_rss`/`peak_footprint` | **SCREEN** | R=1 per punto; non pinnabili |
| 69-70 | slope ctrl 18814309 / lever 18792472 | **SCREEN** | «mai nel ledger come slope» (Gregg Q1) |
| 71 | `delta slope=21837 (0.12%)` | **VOID come pin**, SCREEN come bound | sotto il pavimento 38229; il bound legale è «|effetto| < ~0,2%» |
| 72-74 | `coerenza=` arena committed invariata ⇒ mi_collect non decommitta | **VOID — inferenza REFUTATA** | prim.c:503-504 (`needs_recommit=false`) + os.c:590-591: `committed` non scende mai su purge in release (Leijen ref. capitale 2) |
| 75-76 | baseline build precedente b048b697 (slope 18852538) | **SCREEN come cifra, ma PROMOSSA DI RUOLO** | è la **risoluzione del probe**: da nota a piè di pagina a parametro dichiarato in testa a B3 |

### B3 — canale della leva vera (righe 79-92)
| riga | cifra | grado | motivo |
|---|---|---|---|
| 79-80 | `allocated_bytes=39534144` | **MAGNITUDINE (ADVISORY)** — pin vietato | contatore in-band deterministico ma R=1 e legato all'identità build (A-BG-74: pin dopo R≥3) |
| 80 | `chunk_capacity=13738592` | **SCREEN** | il nome del campo non prova «coda MAI usata»: capacità residua ≠ non toccata; serve il campo *used* in banda |
| 81-83 | lettura «13738592 di coda MAI usata» | **SCREEN** | discende dalla riga sopra |
| 84-88 | costo CLI: 44630520 / 44597728 / 10093048 e **4.42** | **MAGNITUDINE (SCREEN-ADVISORY)** — pin vietato | 1 run per lato (Gregg Q1/A-BG-74); rapporto ricalcolato 4,4219 ✓ |
| 89-92 | leva nominata + obbligo gate parità completi | **non è una cifra**: dichiarazione d'intento, **valida e rafforzata** | Leijen A-DL-68 dà via libera dal perimetro allocatore |

### Verdetto finale del file (righe 95-101)
| riga | clausola | grado |
|---|---|---|
| 95-96 | «B1 CHIUSO» | **regge in ADVISORY** (con la somma corretta) |
| 97 | «B2 CHIUSO: transiente, liberata; il "mai liberata" era la statistica monca» | **regge in ADVISORY con riformulazione obbligatoria** (§ sotto) |
| 98 | «B3: LEVER-2 refutata con misura (delta nullo)» | **SCREEN** — sostituire il verbo |
| 99 | «slope fisico ~18.8 MB/worker resta l'oggetto» | **SCREEN** — l'oggetto regge, la cifra non è un pin |
| 100-101 | composizione «residuo committed per-thread post spike + retained» | **VOID come ipotesi formulata** — «residuo committed» è esattamente la grandezza illeggibile (Leijen A-DL-66); riformulare su canale fisico on-thread (A-DL-55) |

**Nessuna riga del file raggiunge il grado VERDICT** salvo l'identità aritmetica della catena
di raddoppio (riga 20), che è un teorema, non una misura.

---

## 4. La conclusione B2 sopravvive a Leijen? **SÌ, ma solo riformulata**

Il rischio è sottile e va detto in chiaro: B2 e la nota-canale di `huge-worker.out` dicono
**cose opposte**, ed è la nota di `huge-worker.out` a cadere, non B2. Ma la *ragione* scritta
in B2 («il decremento non morde») suggerisce un decremento eseguito e inefficace, mentre il
meccanismo vero è **preprocessore**: il decremento non esiste nel binario.

> **Tesi corretta, in una frase difendibile**: *in ogni build census (MI_STAT=0, imposto da
> libmimalloc-sys 0.1.49 build.rs:108-115 → types.h:81-87), l'incremento di `malloc_huge` alla
> nascita della pagina huge è incondizionato (page.c:935-936, macro internal.h:392) mentre il
> decremento alla free è compilato VIA (`mi_stat_free` sotto `#if (MI_STAT>0)` free.c:612, stub
> release free.c:635-637; il decremento di theap.c:362 vive in un blocco commentato ed è dead
> code anche in debug): `malloc_huge` è quindi un contatore di NASCITE — **cumulativo per
> costruzione in quella build**, non «monco per assenza di free» — e la free dei sei chunk
> esiste davvero, provata dalla coppia alloc/dealloc in-band lato Rust.*

Corollari da portare nel testo:
- «cumulativo» è una proprietà **della build**, non della statistica in assoluto: con MI_STAT=1
  la stessa riga tornerebbe informativa (ed è il motivo di A-DL-65).
- **Asimmetria latente n.2** (per quando MI_STAT>0): il decrease va sul
  `_mi_theap_default()` del thread che libera (free.c:615), non sul theap che incrementò ⇒ una
  Σ per-theap può andare **negativa** su free cross-thread. Va scritta ORA, prima che qualcuno
  sommi per theap in m91.
- Il **controllo positivo E2 chiesto dal Concilio WP-94 è soddisfatto** — ma dal lato Rust, e
  la sua conclusione corretta è «il decremento C non è nel binario», non «non morde».

---

## 5. Atti di sanatoria sui file già committati

Tutti gli atti sono **di carta**, costo ≈ una sera, e vanno in **UN solo commit** (l'atto
A-BG-71 richiede esplicitamente «stesso commit della riga ledger»).

### (A) `wp93-harness/huge-sites.out`
| id | riga | correzione | formula |
|---|---|---|---|
| **A1** | 2 | grado per sezione + risoluzione dichiarata | `grade=B1 ADVISORY · B2 ADVISORY · B3 SCREEN` e nuova riga `risoluzione-probe=38229 B/worker (spread inter-build b048b697a4c10688 vs 8e966efd6b3d3e69) — ogni |effetto| sotto questa soglia e NON RILEVATO, mai refutato` |
| **A2** | 19 | **errore aritmetico** | `somma=39223200` (era 39423200; 622576+1245168+2490352+4980720+9961456+19922928 = **39223200**) |
| **A3** | 23-25 | residuo os-good-size ricalcolato | `residuo os-good-size = 39911424-39223200 = 688224` (era 488224). **Corroborazione indipendente**: con la somma corretta `39534144-39223200 = 310944`, dell'ordine dell'anello immediatamente inferiore della catena `(622576-16)/2 = 311280` — con la somma errata il residuo 110944 non corrisponde ad alcun anello |
| **A4** | 61 | verbo del titolo | `== B3: leva LEVER-2 (mi_collect on-thread post-preludio) — EFFETTO NON RILEVATO sotto la risoluzione del probe ==` |
| **A5** | 71 | delta come bound, non come pin | `delta slope=21837 B/worker — SOTTO la risoluzione 38229: verdetto legale «|effetto| < ~0,2%». PIN VIETATO (A-BG-73)` |
| **A6** | 72-74 | inferenza refutata | sostituire con `coerenza=RITIRATA (REFUTATA, Concilio WP-95/Leijen): in release committed non scende MAI su purge — _mi_prim_decommit pone *needs_recommit=false (prim.c:495-504, gate 503) e mi_os_decommit_ex decrementa solo se needs_recommit (os.c:590-591). Da committed invariata NON segue alcuna conclusione su decommit. Retention affermabile solo dal probe fisico on-thread (A-DL-55/66)` |
| **A7** | 55-57 | meccanismo per NOME | sostituire `il decremento NON morde sul canale stat` con `il decremento C NON ESISTE nel binario census: mi_stat_free e sotto #if (MI_STAT>0) (free.c:612, stub release 635-637), MI_STAT=0 via build.rs:108-115 di libmimalloc-sys 0.1.49 => types.h:81-87; theap.c:362 e dead code (blocco commentato). L'increase e incondizionato (page.c:935-936) => malloc_huge conta le NASCITE` |
| **A8** | 51-54 | tesi riformulata | applicare la frase del §4 («cumulativo per costruzione **in quella build**») |
| **A9** | 58-59 | caveat provisioning | aggiungere `caveat: transiente al livello malloc; al livello provisioning lo spike e SINCRONO su W thread al restart (~W*39,5MB) e va coperto dal capacity planning — freed != decommitted != non-provisioned (KS-BB-95-2)`. **Non** citare arena committed a sostegno (illeggibile per A6) |
| **A10** | 8-9 | ricevuta di ripristino | `RIPRISTINO=DICHIARATO — nessuna ricevuta in-band (ne shasum post-ripristino, ne ri-run del gate lever-pins DOPO il probe); ogni claim di parita che vi poggia resta ADVISORY (KS-PP-95-3)`; + riga `smaltimento=porta 8199 chiusa, log grezzi del trace <path> — per NOME` |
| **A11** | 4-5 (banner) | livello stat + auto-contaminazione | `MI_STAT=0 (dichiarato)` nel banner; + `contatori galloc/gfree del run con PHPR_HUGE_TRACE=1 = VOID (auto-contaminazione: backtrace e format passano dal GlobalAlloc)` (A-DL-65, A-PP-78) |
| **A12** | 80-83, 84-88 | etichette di grado | `chunk_capacity … [SCREEN: capacita residua != coda mai toccata — serve il campo used in banda]`; `4.42 = MAGNITUDINE R=1, PIN VIETATO fino a R>=3 (A-BG-74)` |
| **A13** | 100-101 | composizione dello slope | rimuovere «residuo committed per-thread» come termine; riformulare su canale fisico on-thread |

### (B) `wp92-harness/huge-worker.out` — supersessione formale (A-BG-71)
UNA riga in testa, **corpo intatto** (è evidenza storica), **due clausole per NOME**:

```
SUPERSEDED-IN-PART (Concilio WP-95, atto A-BG-71 — vedi wp93-harness/huge-sites.out B1/B2):
  clausola-1 REFUTATA: «riserva FISSA mai liberata» — i sei chunk sono TUTTI deallocati
    dentro la prima richiesta (coppia in-band Rust, huge-sites.out:43-48).
  clausola-2 REFUTATA: nota-canale righe 6-8 «il decremento ESISTE ... quindi current==total
    NON e un contatore monco: e assenza di free» — a MI_STAT=0 (ogni build census) il
    decremento e compilato VIA (free.c:612, stub 635-637) e theap.c:362 e dead code; il
    contatore E monco per costruzione in release.
  RESTANO VALIDI: 39911424 B e 6 blocchi per worker, come CUMULATIVO di nascite di pagine
    huge (mai come retained).
```

### (C) `sessions/WP_SESSION_93.md`
| id | riga | correzione |
|---|---|---|
| **C1** | 30 (B2) | «è la statistica malloc_huge … a non decrementare MAI (cumulativa)» → aggiungere `perche a MI_STAT=0 il decremento non e compilato (free.c:612/635-637); increase incondizionato (page.c:935-936) — cumulativo PER BUILD`; e «il decremento NON morde» → «il decremento NON e nel binario» |
| **C2** | 31 (B3) | `LEVER-2 … REFUTATA CON MISURA` → `LEVER-2 … EFFETTO NON RILEVATO (screen, R=1): delta 21837 SOTTO la risoluzione inter-build 38229 B/worker`; **cancellare** «coerente con arena committed invariata a exit_collect_mi» (inferenza refutata); `4.42` → `4.42 (MAGNITUDINE R=1, pin vietato)`; `39534144` → aggiungere `(R=1)` |
| **C3** | 74 | l'attribuzione dello slope va etichettata «solo da probe fisico on-thread (A-DL-55/66)» |

### (D) `gaps/GAP_TREND.md`, riga **WP-93 (S-93.0)**
| id | correzione |
|---|---|
| **D1** | `LEVER-2 mi_collect refutata con misura (delta slope 0.12 per cento)` → `LEVER-2 mi_collect: EFFETTO NON RILEVATO — delta 21837 B/worker sotto la risoluzione inter-build 38229 (screen R=1, non refutazione)` |
| **D2** | `slope fisico probe 18814309 B/worker` → `slope fisico probe 18814309 B/worker [SCREEN, non pin — mai nel ledger come slope]` |
| **D3** | `somma 39423200` → `somma 39223200` (stesso errore propagato) |
| **D4** | `malloc_huge di m90 è un CUMULATIVO` → aggiungere `per costruzione a MI_STAT=0` |
| **D5** | `4.42` → `4.42 (magnitudine R=1)`; la riga resta «non rimisurato — riferimento resta WP-85», che è il punto (§0) |

### (E) Atti che eccedono i quattro file ma sono conseguenza diretta
- **A-BG-72**: audit per NOME delle derivate m90 che hanno consumato `malloc_huge` come
  retained (repair90-estimators, VCOV 0,778, decomposizione b, «invisibile» 4,48 MB/worker);
  esito a ledger. **Il team lo considera parte della sanatoria, non lavoro nuovo.**
- **A-PP-76**: `huge_note` simmetrico su realloc (main.rs:120-124) — il canale va sanato
  prima di essere riusato in m91, anche se B1/B2 non ne soffrono.

---

## 6. Ordine S-94.0 proposto dal team-misura (FONDAMENTALI-first)

**Precondizione (non è una misura, non consuma il budget di misura)**
- **P0. Gli atti di sanatoria §5 (A+B+C+D+E) in UN commit.** Nessuna cifra nuova può essere
  pubblicata mentre agli atti resta una somma sbagliata e un verbo «refutata» non sostenuto.

**Che cosa si MISURA, in ordine**
1. **COPPIA FULL stessa-sera (media + peak footprint + CPU)** — *questa è la prima misura*.
   Motivo: il contatore è fermo a **WP-85, 8 sessioni**; la leva per-file è governata dalla
   regola WP-48 (predizione-misurata) e **senza un «prima» fresco non ha giudice**; e la
   coppia è l'unica cifra del filone che nasce già verdict-grade. (Gregg A-BG-75; nessuna sedia
   la contesta — Bak e Pedersen semplicemente mettono battery61 davanti.)
2. **battery61 riproducibile in modo nativo (criterio 5)** — debito di 31 sessioni, mezza
   sessione (Pedersen Q5, Bak Q4). **Registro il dissenso ordinale**: Bak e Pedersen la
   vogliono al posto 1. Il team propone 1↔2 in quest'ordine perché la coppia full è
   *precondizione della sessione successiva*, ma se il tempo basta per una sola, la scelta è
   del Concilio plenario, non di questo team.
3. **Probe slope v2 = il canale unico di m91**, nella forma **fusa** dei quattro emendamenti —
   non tre probe diversi:
   - `MI_STAT=1` esplicito e **livello dichiarato nel banner**; `TRACE=1` mai nello stesso run
     (A-DL-65 + A-PP-78, conflitto M-4);
   - **coppia alloc/free IN-BAND nel GlobalAlloc Rust** (soglia ≥524288, chiave thread/theap)
     come canale di fede (KS-DL-95-2);
   - **eco d'arm obbligatoria**: `arm=<nome> fired=<n> thr=<id>`; raw senza `fired==W` ⇒ run
     VOID (A-PP-74) — senza questo, ogni delta nullo è indistinguibile da un arm mai scattato;
   - **R≥5 interleaved (A,B,A,B…) sulla stessa binaria, W∈{1,2,4}**, mediana ± banda 2se
     pubblicate NEL probe; NULLO **solo** come test di equivalenza con soglia ex-ante
     (A-BB-74);
   - **doppia metrica peak + residency post-warmup** — una leva che agisce dopo il picco non
     può muovere il peak (A-BB-75, KS-BB-95-1);
   - **finestra del testimone**: ordine `send → lavoro → dec OUTSTANDING`, o dichiarazione
     in-band di lavoro fuori-testimone (A-PP-75);
   - `huge_note` simmetrico su realloc prima dell'uso (A-PP-76).
4. **Attribuzione dello slope ~18,8 MB/worker per NOME**, ammessa **solo** dal probe fisico
   on-thread (A-DL-55) — mai da `committed` (A-DL-66).

**Che cosa NON entra in S-94.0** (esplicito, per non riaprirlo in sessione)
- ❌ **La leva arene per-file.** Approvata nel merito da tutte e quattro le sedie, ma **nessuna
  leva prima che esista il giudice** (coppia full + probe v2). Entra in S-95.0 con predizione
  scritta ex-ante (hello 44,6 → ~11 MB) e gate parità COMPLETI (corpus 1418 + refl 290 + ORM
  3E/13F + hk 1665) + ricertificazione baseline phpr.
- ❌ **Lo snapshot/precompilazione del preludio** (Bak Q3 n.1): endgame, costo e gate massimi.
- ❌ **Il rifacimento di LEVER-2**: è post-picco per costruzione; non merita un probe dedicato
  finché non esiste la metrica residency che potrebbe vederla.
- ❌ **Qualsiasi pin** di 21837, 18814309, 39534144, 4,42×: pin solo dopo R≥3/R≥5 (A-BG-73/74).
- ❌ **Nuove derivate da m90** finché l'audit A-BG-72 non chiude.
- ⏱ **Apparato**: solo se blocca l'oggetto, timebox permanente (direttiva FONDAMENTALI).
- 🔎 **Da tenere in coda, non in S-94.0**: latenza prima-richiesta per worker (A-BB-76) e spike
  di fattibilità `LoweredPrelude: Send+Sync` (A-BB-77) — cifre di canale, non fondamentali.

---

## 7. Riepilogo delle retrocessioni (la lista corta che il plenario deve votare)

1. `huge-sites.out:19` somma **39423200 → 39223200** (errore aritmetico, +200000 B agli atti).
2. `huge-sites.out:23-25` residuo os-good-size **488224 → 688224**.
3. B3 «REFUTATA con misura» → **effetto non rilevato sotto risoluzione 38229 B/worker**.
4. `huge-sites.out:72-74` (coerenza m90) → **VOID/REFUTATA** per prim.c:504 + os.c:590-591.
5. `huge-sites.out:55-57` «il decremento non morde» → **«il decremento non è compilato»**.
6. `huge-sites.out:9` ripristino pin → **DICHIARATO, non ricevuta**.
7. `huge-worker.out` → **SUPERSEDED-IN-PART**, due clausole per NOME.
8. Delta 21837, slope 18814309, 39534144, 4,42× → **pin vietati**.
