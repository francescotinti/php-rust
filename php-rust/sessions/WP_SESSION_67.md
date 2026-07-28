# WP_SESSION_67 — P-2 DE-LEAK SPEDITA (Box::leak morto sull'include path; gate67 PASS a verdetto macchina; coppia+A-A′ CPU≈0; 88 BYTE-ID ×3) + quota B-67.1 + scoperta ordering S-67.2

> ⚡ **WP-67 (2026-07-28 notte, `4fe85a1`→`2b92c78` release +
> `046c961`/`f0f0e72` census/doc)** — sintesi a 9 recepita INTEGRALE in
> `wp67-harness/design67.md` PRIMA di ogni codice; predizioni P67-* in
> `predictions67.md` (+locked + copia repo `sessions/WP67_PREDICTIONS.md`,
> commit 34aed31) PRIMA dei letti. Verdetti integrali: design67 §8.

## Debiti d'apertura consegnati (commit e3d8bdc + 1330fd9)

- H-67.1 breach leggibile (uc_log porta seed_slots/seed_len; contatore
  dichiarato unreachable-by-design); **H-67.3/M-67.6 tripwire
  tail∩seed VIVO in `unit_slot_pos`, promosso a FATAL in ogni build**
  (il gate gira `--release`: un debug_assert non sarebbe mai
  esercitato) + test negativo should_panic VERDE (cargo 1646→**1647**);
  M-67.4 commento infallibilità; E-67.1 finestra `l` depurata dal
  nested autoload (pop PRIMA del `?`) + split dup_fp/dup_cold in lcsum
  (E6 sbloccabile, KS67-2); E-67.3 bordo validate_goto dichiarato;
  **G-67.1/K-67.5 evento `reqmark`** (vocabolario 10→11) — delimitatore
  di richiesta nel log, offset+sleep DEPRECATO; B-67.2 contatore
  diretto parked_modules/parked_bytes ai due siti Box::leak; K-67.1/2/3
  gate67.sh (trap-FAIL, tot≥1646, purge log, **reverse PER NOME** con
  baseline+PROVENANCE); L-67.3/G-67.3 `analyze-l67.pl` (regex [KMG]:
  l661 footprint 1.740,8 MiB letto vs 0.0 del parser rotto; per-causa
  (a)=commit−Σbin, (b)=phys−commit, somma ESATTA; dirty/swap separati)
  — sotto swap (a)/(b) si gonfiano in direzioni opposte: la colonna
  pulita vuole la run senza pressione (G-67.4), phys standing RESTANO
  SOSPESI.

## B-67.1 — quota server (probe67-census PASS, verdict-file; worker unico)

Scoperta di metodo: il census dumpa a OGNI run top-level ⇒ census
server PER-RICHIESTA gratis (delta dump_i−dump_{i−1}). Steady R9→R10:
**Δparked = 22 moduli = 1,62 MiB/richiesta** (15 impure + 7 eval — gli
eval sono invisibili a uc.log, il contatore diretto li vede);
**Δ(lower+compile) ≈ 21 ms/richiesta**; per-file script-loader 8,4 +
comment 6,1 + ProviderRegistry 1,5 ms = 74-75% (emend. Hejlsberg). Q1-a/c/d DENTRO le bande.

## P-2 DE-LEAK (release = 2b92c78, binario phpr-wp67 c0f5cfff…)

- **Forma**: `RetainSet(elsa::FrozenVec<Rc<Module>>)` per-richiesta
  creato PRIMA del Vm (drop-order dal borrow-checker, P-67.5);
  `Vm::park_module` = costruttore UNICO di `&'m Module` (M-67.1:
  hit/compile/eval/deferred); cache Rc-owned (superseded/evicted
  DROPPANO); hit = clone+park (K-M67.2/H-67.4); eval muoiono a fine
  richiesta; main = borrow del chiamante (M-67.2, eccezione
  documentata). **elsa =1.11.2 con sign-off utente esplicito**
  (H-67.2); audit unsafe crate a verbale. **Zero unsafe aggiunti, UNO
  RIMOSSO** (census walk ptr→Weak; dead_units contati) — verificato a
  macchina sul diff.
- **GATE67 PASS fails=0 a VERDETTO MACCHINA**: cargo 1647/0 ·
  sentinelle 5 assi BYTE-ID · sentinels65+KS-S6+seed_prefix_short=0 ·
  KE-e · P1 · S-65.3+diff-oracle · corpus 1421 · refl 290 · ORM 3E/13F
  · hk 1665 OK · reverse **2F PER NOME**. (Primo run FAIL onesto:
  /private/tmp/wp11-gates sparito col reboot — ripristino dai tarball
  wp9-harness; il verdict FAIL fu scritto per via ORDINARIA, non dalla trap — emend. Klabnik, K-68.4 chiede il self-test.)
- **Leak-shape (probe67-nk, N=1000, verdict PASS)**: pendenza
  Σcommitted **+2,55 KiB/req** (soglia 50; residuo = bookkeeping del
  census, dichiarato); **dead_units 1998/2003, vivi 5**, mod_owned
  PIATTO; Δstubs_entries=0 (KS-P67.3). A/B pre-P2 (memgc66): +2 moduli
  VIVI/richiesta, 4,84 KiB/req ritenuti = la stessa churn che post-P2
  muore. KB67-1 soddisfatto sul fixture (wpdev: 1,62 MiB/req ora
  muoiono col RetainSet; la CPU dei 15 resta — leva impure WP-68).
- **Rigiochi**: KS-P1 post-P2 PASS (probe67-ksp1, verdict-file, attese
  esplicite: cold=15 ATTESO, dc=0, fp=0, hit 497/512, fp-seq identiche
  NEL probe); S-66.4 PASS.
- **Catena serale (run55 triple, PRIMO protocollo coppia+A-A′)**:
  new **777,19u** · old (wp66) **787,45u** · new′ **786,95u** — delta
  coppia −1,30% (1,3029%) della STESSA TAGLIA dello spread A-A′ **1,26%** (1,2558% — emend. concilio: non "dentro"; new′≈old a 0,06% ⇒ rumore/posizione-1) ⇒
  **costo CPU P-2 compatibile con zero**, nessuna cifra citabile
  (KG67-1 rispettata COL self-pair, prima volta). **Fail-set 88
  BYTE-ID = run33 in tutte e tre le run.** KH67-3 non scatta.

## 🔴 S-67.2 — SCOPERTA: divergenza ordering PRE-ESISTENTE (cold)

Fixture ordering-echo: oracle `U-top·[autoload]·U-after` (late binding
al DECLARE, e RI-esegue l'autoload a ogni richiesta); phpr
`[autoload]·U-top·U-after` (autoload nel LOWERING). Divergenza
pre-esistente, catalogata in PHPR_DIVERGENCES §3 (f0f0e72) e pinnata
come fixture di regressione nel gate67 (fixtures/gate-ordering.sh).
Conseguenza WP-68: publish+dep-replay dà hit==cold (coerente ma ≠ Zend
nell'ordering); **defer-always chiuderebbe ANCHE la divergenza cold**
(la nota Stogov "in Zend è ESATTAMENTE il caso cachato" ora ha una
prova eseguibile) — decisione al concilio.

## Scope onesto

La cacheabilità unit impure NON è stata eseguita (P-2 era il blocco ed
è spedita con verifica piena; la scoperta S-67.2 cambia il tavolo della
decisione). P-67.4 (figlio editato⇒MISS) oggi vacua per costruzione —
si arma con la leva.

## ⭐ Lezioni

- ⭐⭐ **Il marker di richiesta rende visibile il self-traffic del
  server** (wp-cron POST + loopback GET: 13 richieste servite per 10
  curl): il metodo a offset lo assorbiva IN SILENZIO nei segmenti
  adiacenti — i probe classificano i segmenti per entry-script del
  reqmark, mai per posizione.
- ⭐⭐ **Un tripwire che il gate non esercita non esiste**: il gate gira
  `cargo test --release` ⇒ un debug_assert è invisibile al gate; la
  forma giusta è FATAL in ogni build (K-M5) + test negativo che prova
  che PUÒ scattare.
- ⭐⭐ **Il census del server è per-richiesta gratis** (dump a ogni run
  top-level): i delta tra dump consecutivi quotano bytes/ms per
  richiesta senza strumenti nuovi.
- ⭐⭐ **Formalizzare una fixture scopre le divergenze prima della
  leva**: l'ordering-echo doveva validare il replay e ha invece
  falsificato il COLD (pre-esistente) — pin subito, decidere poi.
- ⭐ /private/tmp non sopravvive al reboot: i gate-work si ripristinano
  dai tarball (wp9-harness), MySQL dal datadir esterno; il pre-flight
  li becca entrambi.
- ⭐ daemonize.pl vuole `<log>` come PRIMO argomento (exec fallisce in
  silenzio nel nipote se lo ometti); e la out-dir del redirect deve
  esistere PRIMA della shell (replica WP-66).

## Parità e stash

Release **phpr-wp67 (c0f5cfff…, tree `2b92c78`)**, stash ADDITIVO
accanto a wp66 (5aa60d56…). Delta engine vs wp66: P-2 (RetainSet +
cache Rc) + tripwire vivo + reqmark. Commit census-only successivi al
release point: `046c961` (PHPR_MI_COLLECT_REQ), `f0f0e72` (catalogo).
Census: phpr-memgc66 (cea61455, tree 1330fd9, PRE-P2) e phpr-memgc67
(95d07638, tree 046c961, POST-P2) in phpr-mem-target/.

## Prossimo (WP-68) — vedi NEXT_SESSION §WP-68
