# WP_SESSION_49 — leva collector (purge tombstone al trigger) + verdetto Fase 1.2

> ⚡ **WP-49 (2026-07-24, `08fe322`+`951e2fd`+`e2d7805`)** — **la leva collector
> ricava dall'attribuzione WP-48 il più grande recupero CPU full-suite della
> roadmap: master-CPU 19:10 → ≈13:55 (−27%, 3,39× → ≈2,46×), fail-set
> BYTE-IDENTICO a run33, RSS transiente max 5431MB INVARIATO. Mechanism-check
> a conservazione: round classify 1005→297 (−70%), classify_ms 375k→142k
> (−62%), freed 14,16M→14,23M (+0,5% = stessa garbage in ⅓ dei round),
> soglia adattiva ferma alla base 50k. In più l'A/B media migliora ENTRAMBE
> le metriche: CPU −1,19% (5/6 round), peak fisico −2,22%.**

## Obiettivo 1 — analisi PRIMA della leva (mandato del prompt)

Strumento nuovo (`08fe322`, solo gc-census): log PER-COLLECT
(`PHPR_GC_COLLECT_LOG`) — una riga per round classify con call/round/src
(auto|explicit), root oggetto+container sopravvissuti al drain, quota LIVE
(root non-white del round), reroot (ricorrenza con ritiro degli id liberati),
soglia in vigore, ms di classify del round. Census full baseline (binario
`phpr-memgc49`, riproduce ESATTAMENTE i numeri WP-48: 1005 round / 9,93M
root / 14,16M freed — collect pattern DETERMINISTICO):

- **0 chiamate esplicite** (né PHPUnit né WP chiamano `gc_collect_cycles()`);
  **528 call ≈ 1,9 round/call, max 3** → il loop-until-dry di
  `collect_cycles_inner` (divergenza dal cap `did_rerun_gc` di Zend,
  verificato su zend_gc.c:2178) NON è il moltiplicatore: i round 2 costano
  0-11ms. Ipotesi rerun e ipotesi esplicite SCAGIONATE dai dati.
- **La causa: trigger a maggioranza tombstone in 530/530 call.** Il trigger
  confrontava la lunghezza RAW dei buffer con la soglia, ma `gc_ctr_roots`
  tiene i `Weak` morti fino al drain del collect: i collect scattavano a
  soglia 350k-1M con soli ~7-30k root sopravvissuti (~97% tombstone), un
  classify dell'intero grafo vivo ogni pochi secondi. Zend non li conta mai:
  `gc_remove_from_buffer` toglie il root alla morte per refcount (O(1)).
  L'escalation della soglia (fino a 1,05M) e l'isteresi WP-47 combattevano
  un ARTEFATTO, non la pressione reale.
- Contorno: live_roots 11,1% dei sopravvissuti (l'88,9% classifica white —
  i collect TROVANO garbage vera: la leva deve ridurre la cadenza, non il
  lavoro utile); reroot 8,7% → **leva (b) "rooting selettivo" non
  giustificata dai numeri**; collect concentrati nei minuti 10-20 della run
  (461/530 call).

## La leva (`951e2fd`)

Al trigger (raw ≥ max(soglia, floor)): purge dei morti — `retain` dei
`CtrWeak` con `strong_count>0` e degli id ancora in `created` (gli id
oggetto lasciano già i buffer a free/recycle; i container non avevano
NESSUNA rimozione) — poi collect SOLO se i VIVI raggiungono la soglia.
La garbage ciclica resta strong-held dai propri archi ⇒ sopravvive al purge
e continua a fare pressione: si perdono solo i tombstone. `gc_purge_floor`
(vivi+50k, reset al drain) impedisce il ri-purge degenere quando i vivi
restano appena sotto soglia. Bonus footprint: un `Weak` morto pinna
l'allocazione RcBox — il purge la rilascia.

## Verdetti (giudice = full run, guardia = A/B media; direttiva del prompt)

- **Full run36 (parità)**: 30.472 test 0E/2F/86W/73S, **fail-set
  BYTE-IDENTICO a run33** (88 nomi). **Master-CPU ≈13:55 vs 19:10 run35 =
  −27% (3,39× → ≈2,46×** su oracle 5:39; riferimento WP-40 2,06×).
  Wall ~18 min. **RSS transiente max 5431MB IDENTICO** a run34/35
  (guardia footprint: il batching dei collect non alza il picco).
- **A/B media 6 round interleaved** (old = `phpr-wp48` d13dd71 stashato in
  pre-flight): **CPU −1,19% (5/6 new<old), peak fisico −2,22%
  (1,754→1,715G)** — prima leva della roadmap che migliora ENTRAMBE le
  metriche; recupera parte del +7% media di WP-46. Giornata rumorosa nei
  primi round (69s entrambi i lati): fede all'interleaved, non ai rapporti
  assoluti di giornata.
- **Mechanism-check (census full pre/post, stesso giorno)**: round 1005→297
  (−70%), call 528→149, classify_ms 375.061→141.828 (−62%; il 494,9s WP-48
  era la stessa metrica su giornata scarica), **freed 14.155.545→14.227.019
  (+0,5%): la STESSA garbage raccolta in un terzo dei round** — identità di
  conservazione, non una speranza. Soglia finale 1.050.000→**50.000 (base)**.
  RSS max census 1975→1717MB (−13%: i tombstone purgati liberano RcBox).
- Parità: cargo **1639/0**; corpus **1421 BYTE-IDENTICO** per nome; gate22
  completo verde col conteggio (vedi sotto).

## Obiettivo 2 — Fase 1.2 `created`→Weak: FALSIFICATA dalla predizione misurata

Metodo WP-48 (contatore PRIMA della leva): split census del canale created
(`created-dead-rc1` = entry con strong−buffered == 1, camminate PRIMA del
resto del canale). Esito sul media group: **`created-dead-rc1[n=14] =
113KB` su 114,73MB del canale (0,1%)**. Il 99,9% del pinnato ha strong>1:
è garbage CICLICA non ancora collettata (+ catene che pendono da cicli) — e
un registry Weak NON la libererebbe: i membri di un ciclo si tengono in
vita con gli archi del ciclo, non con l'edge di `created`. **La leva
eviction/Weak è falsificata; l'OBIETTIVO (de-pinnare il canale: 114,7MB
media / 385,2MB full a fine run) resta aperto** e la leva coerente è il
collect al confine (Fase 1.4 disciplina per-test) o a fine-run — bilancio
che riconcilia: 114,73MB = 0,11MB rc1 + ~114,6MB cicli/catene.
Sentinelle dtor-order: mai toccate (nessun cambio al registry).

## ⭐ Lezioni

- ⭐⭐ **Il pattern dei collect sulla full è DETERMINISTICO** (1005 round
  identici tra WP-48 e la baseline WP-49 sotto contesa CPU): i CONTEGGI
  sono il metro giusto del mechanism-check, i ms no (375s vs 495s stessa
  distribuzione su giornate diverse).
- ⭐⭐ **Un buffer che non rimuove i morti trasforma la soglia in un timer
  sul churn**: il trigger contava tombstone, e soglia adattiva + isteresi
  ottimizzavano l'artefatto. Prima di tarare una soglia, verificare COSA
  conta davvero.
- ⭐ Il log per-collect ha scagionato in un colpo le due ipotesi in scaletta
  (rerun-cap Zend, chiamate esplicite) e indicato la terza: strumentare la
  distribuzione PRIMA di scegliere la leva paga anche quando l'attribuzione
  aggregata c'è già.
- ⭐ La falsificazione rc==1 (13 oggetti!) ha ucciso in 2 minuti di census
  una leva da giorni di lavoro (created→Weak con audit di tutti gli
  strong_count): predizione-misurata come kill-switch economico.
- ⚠️ Residuo nuovo osservato: gli ingressi light-sweep salgono 209,8M→1.014,7M
  post-leva (i buffer restano pieni più a lungo ⇒ meno fast-path
  sweep-empty): corpi economici, il netto è già nei verdetti, ma è un
  candidato micro-leva.

## Prossimo (WP-50)

1. Residuo full post-leva: ≈13:55 vs 11:39 WP-40 (≈1,20×): classify
   residuo ~142s census (su binario parità meno), reflect-cache thrash
   (890k miss/16% hit, cap 16384 ≈ +45MB da MISURARE), light-sweep 1,01G
   ingressi.
2. Fase 1.4 (disciplina di confine per-test) ora ANCHE come leva footprint
   per il canale created (385MB full a fine run = cicli non collettati);
   attenzione al costo CPU (collect per test = il male appena curato:
   servono i numeri per-test prima).
3. Fase 1.3 cold-box Object (~96B×istanze) e attribuzione transiente 5,4G
   (invariato: è pacing-dipendente, serve snapshot al picco sulla full).
