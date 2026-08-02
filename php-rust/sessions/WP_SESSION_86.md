# WP_SESSION_86.md — S-86.0 "LA CONTRO-PROVA E LE TRE REFUTAZIONI" — ✅ 8 punti Concilio WP-87 + VERDICT86 PASS + scomposizione CONFERMATA + purge refutato

**In una frase**: abbiamo rifatto l'esperimento di memoria al contrario e
i numeri sono tornati esatti al byte (la nostra "contabilità" della
memoria per worker è quindi affidabile), abbiamo dimostrato che il
sistema di misura rifiuta da solo una misura inquinata, e abbiamo
scoperto che il sospetto principale per le oscillazioni del picco di
memoria era innocente — il tutto rendendo i controlli automatici ancora
più severi.

**Data**: 2026-08-02
**Scope**: ordine vincolante Concilio WP-87 §Sintesi (8 punti) eseguito.
Modello verificato all'apertura: Fable 5.
**Commit**: 6be233c → 7266da9 → 8704e93 → df1f61c → ffc8164 → 4ba31e5 →
5b9a831 → 3818edd → f463813 → b46fc07 → 9d50b47 → 59eaa2e → 70824ea →
8429763 → c259bc6 → 45d3714 → **4fe33dc** (tutti su main, pushati).
**Binari**: phpr **ea56b874c76d3558** (nuova baseline parità, stash
additivo `phpr-wp86`; bit mossi da A-TH36 guardia-in-testa + A-DS36
putord + A-MS32 &mut() + A-MS33 !Send/!Sync + A-MS35 + A-DS34; corpus
1418 + refl 290 per NOME IDENTICI in parity-full, 3× battery);
campagna a c259bc6 (battery 15/15 STESSA rev — nessuna equivalenza);
union b2074e451cbc7fc3 · mem-census 874e744ede57b4ca ·
driver_sha 699db00a9808489e (measure78 + parametro purge).

## Ordine WP-87 eseguito (p1-p8)

| # | Esito | Commit |
|---|---|---|
| 1 | **Sanatorie**: A-SK44 collect-then-emit (bite-tested, RE-RUN byte-identico) · A-SK45 dente ord≥2 · A-BG40 head_unmoved + A-BG39 attempt-ledger nel supplement · MEASURE85: overclaim A-BG37 corretto, scomposizione MODEL-GRADE (A-BG42), ×W su SOLO m85.dl28s, envelope emendato (A-BG41) · retro-dich. A-AH42/A-BB48 · A-DS34 nota methods_ci · A-DS36 doc+putord+pin adiacenza | 6be233c, 7266da9 |
| 2 | **Sigilli v4**: A-MS32 `&mut ()` (E0716 vs constant promotion) · A-MS33 !Send/!Sync + doctests compile_fail,E0277 · A-TH36 guardia in TESTA al put (finestre unwind chiuse) · A-MS35 · A-TH37 marker #[used] — **KH87-2 positivo VERIFICATO** (tainted=FAIL nm+strings, clean=PASS) · A-TH38/39 pin+decoy | 8704e93 |
| 3 | **Battery v4**: A-SK42 porcelain fail-closed (bite-tested) + k/k CONTATO · A-SK41 stamp ledgerato committed · A-AH40 matrix nel .done (set-difference) · A-AH41 rustc -V in archivio · A-SK43 allowlist bande + MiB-only (bite-tested) | df1f61c |
| 4 | **Probe/eredità**: A-PP35 reqns-guard.pl condiviso (morde w=2 E w=10) · A-PP33 arm no-warm (vera req1) · A-PP34 displacement zero-delta · A-PP36 dispatch-count in-band · A-MS34 PROBE_ACTIVE nel hook · A-DS37 template VA · A-DL33 knob burst · A-DL34 semantica finish() dichiarata + controlli · A-DL15 retry quiesce | ffc8164 |
| 5 | **A-DL32** fix phys_peak (1 riga) · strumenti: purge param (effetto in-band verbose), lower_span µs, fns/stmts, mi_bin thr= + **census_line ATOMICA** (garbling preesistente) | 4ba31e5, 5b9a831 |
| 6-7 | **CAMPAGNA measure86 + VERDICT86 PASS** (battery 15/15 ×3 — due REFUSE del checker su se stesso sanati per NOME): vedi `wp86-harness/MEASURE86_RESULTS.md` | 3818edd…45d3714 |
| 8 | **ROADMAP**: catalogo §3.3-ter (hoisting=opcache) + §3.3-quater (covariance LSP, decisione CORRECT) · A-DS35 PRIMO item in [[php-rust-todo-master]] | b46fc07 |

## Misure (verdict86.out, fail-closed)

- **VCAL**: NET_H = 7.349.977 B · NET_P = 7.803.281 B — TERZA campagna
  consecutiva byte-identica.
- **VINV — A-BB45 SUPERATA**: ordine invertito (pad ord1, hello ord2
  stesso thread): pad = NET_P ESATTO, hello ord2 = **6.842 B ESATTO
  sulla predizione ex-ante** — la scomposizione additiva è confermata
  dal contro-esperimento (promozione a WP-88; KB-87-1 fino ad allora).
- **VBURST — A-DL33 PROVATA**: +1.048.576 B esatto e riga DECLASSIFICATA
  dal dente di calibrazione — fail-closed della PIPELINE, non del
  contatore.
- **VW500 — A-BB48 CHIUSA**: len=500 ⇒ 4 call / 2.002 B esatti sul
  modello piecewise; controllo hello 2/196.
- **VABBA — purge REFUTATO come driver**: spread(purge=0) = 20,33 MiB >
  spread(default) = 13,48 MiB (r2..r9, interleave ABBA, effetto in-band)
  — INCONCLUSIVE per soglia Bak, il candidato principale del Concilio
  NON spiega lo spread; pin resta RITIRATO (envelope-only).
- **VOVL — A-BB47 OPEN**: 10 tentativi ledgerati, mai overlap (hello si
  abbassa in µs); per-thread sotto concorrenza resta CANDIDATO.
- **VW123 — NAMED-DEVIATION**: Δpeak/worker 22,53 / 14,34 MiB ≫ banda
  3.605.572 B ±5% — protocollo (peak d'avvio) ≠ derivazione KL-85-2
  (steady W=10); a WP-88.

## 🔵 Scoperte

1. **Il default heap di mimalloc v3 è CONDIVISO fra i thread**: i due
   worker riportano lo STESSO puntatore heap (dente `heap=` in-band al
   primo smoke) — lo split per-thread via heap-visit è IMPOSSIBILE su
   questo allocatore; la metà fisica per-thread di A-DL31 è refutata
   DALLO STRUMENTO, non dal protocollo.
2. **`writeln!` scrive per frammento di format**: due teardown W=2
   concorrenti garblavano le righe census MID-LINE (`pid=pid=...`) —
   difetto PREESISTENTE a tutte le campagne multi-thread, smascherato
   dallo smoke della riga dispatch A-PP36; fix: write_all singolo su
   O_APPEND (atomico).
3. **La contro-prova a ordine invertito chiude al byte**: 6.842 B
   predetto ex-ante e misurato esatto — il confound dei nodi condivisi
   (996.838 B) non morde sulla coppia hello-dopo-pad.
4. **Il purge timing NON è il driver dello spread VP**: il braccio col
   purge forzato ha lo spread PIÙ LARGO — la refutazione sperimentale
   del candidato №1 del Concilio.
5. **LLVM elide le coppie alloc/free non osservate** anche nei controlli
   di misura (net 0 sul primo controllo di deflazione): black_box è
   parte del protocollo, non un ornamento.
6. **`git show HEAD:<path>` è relativo al GIT ROOT** (php-rust/ prefix)
   — il checker A-SK41 ha rifiutato il SUO stesso stamp per path; la
   classe era già nota a object_changed.

## ⭐ Lezioni

1. ⭐⭐ **Un checker nuovo va fatto girare contro la propria battery
   prima di fidarsene**: i due REFUSE (riga PASS non terminale; sed
   greedy sui due campi sha) erano difetti del checker, trovati perché
   il checker ha morso il suo stesso output — fail-closed che si
   auto-verifica.
2. ⭐⭐ **Il dente di onestà in-band paga al primo smoke**: heap= ha
   smascherato lo split impossibile PRIMA che una campagna producesse
   cifre per-thread false.
3. ⭐⭐ **Una predizione nominata ex-ante vale più di dieci consistenze
   ex-post**: 6.842 B era scritto nel verdetto PRIMA del run — il match
   esatto è la conferma più forte che questa serie abbia prodotto.
4. ⭐ macOS bash 3.2 + `set -u` abortisce sull'array vuoto
   (`"${a[@]}"`) — l'idioma if/else batte l'eleganza dell'array env.
5. ⭐ Le righe di un file O_TRUNC condiviso non si appendono mid-run: il
   label va scritto DOPO la wait (o il canale è write_all su O_APPEND).

## Residui / NON fatto (dichiarati, per NOME)

- **A-BB47**: overlap mai qualificato — serve coppia di fixture a
  lowering ~ms per lato (design a WP-88).
- **A-DL31 metà fisica piena**: per-thread refutata dallo strumento;
  forma alternativa (Δcommitted processo, win=0) da disegnare a WP-88.
- **Attribuzione spread VP**: purge refutato; candidati residui
  (first-touch, ordine spawn) a WP-88; pin envelope-only.
- **Promozione scomposizione** (A-BB45 superata): delibera WP-88.
- **A-DS35 implementazione** (covariance LSP): primo item engine, WP-88+.
- **A-AH38 + dry-run KS-AH-86-1**: invariati (nessuna fase base-arm).
- **A-MS27 / A-PP18 / A-PP27**: invariati (backlog).
