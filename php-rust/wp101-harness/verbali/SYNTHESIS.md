# Concilio WP-101 — SINTESI DI CONVERGENZA (su S-99.0 e programma S-100)

## §FONDAMENTALI (prima di tutto, regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: il più alto da molte
sessioni — Gregg (mandato inverso) dà PASS. Nuovo per NOME: collaudo
WordPress full+media CHIUSO PER NOME sull'emissione H-A2+H-B2 (debito
regola-n.2 SALDATO); sei rapporti freschi sui DUE motori nella stessa
finestra (gamba oracle risanata; H-C 13,8 e H-D 8,6 rianimate); D
decomposto con una build di misura (call/marshalling vs traffico Vec);
predizione pre-registrata del percorso registro (rollout Add lì = a
tavolino); pin phpr E php-server COLLAUDATI (era: due rotazioni server
senza un solo collaudo, e il pin dichiarato era INCOLLAUDABILE — mancava
la feature axum-server); sigillo eager + dente anti-putenv.

**(b) Contatore sessioni-senza-misura**: full/media = **WP-99 = QUESTA
sessione (0)** — azzerato dopo 4 sessioni. Sei categorie: fresche di oggi
sui due motori.

**(c) Rischio d'oggetto più trascurato**: la gamba ORACLE del peak media
è salita del **+28,7% senza un nome** (R=1 per lato) — finché non è
spiegata con spread R≥2, ogni rapporto peak è una banda, non un gate. E
il CONTRATTO del flag è scoperto solo ora: `enabled()` è presence-based,
**`PHPR_REG_LOWER=0` ACCENDE il pass** — la bozza del flip poggiava su un
contratto che non esiste.

## Verdetti di fase 1 (9/9: nessun MI OPPONGO al lavoro fatto; la BOZZA
## §S-100 è refutata nei punti 1-2 così com'era scritta)

Verbali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali per convergenza:

1. **Il contratto di modo non esiste** (5 sedie: Hoare, Matsakis,
   Hejlsberg, Pedersen, Klabnik): `enabled()` fa `is_some()` ⇒ `=0`
   accende; il flip a default esige un opt-out VALUE-PARSED nominato
   PRIMA di ogni riga, con ri-derivazione dei bracci anti-putenv, M5,
   launcher e funnel per il mondo post-flip (KS-MA-101-1, A-HO-101-2,
   A-HE-101-1, A-PE-101-4, A-KL-101-5).
2. **La bit-identità non si prova col cronometro** (Hoare, Klabnik,
   Hejlsberg): «5,43→5,44 ⇒ emissione bit-identica» è inferenza invalida;
   ogni claim d'identità d'emissione = SOLO diff di dump/output
   (KS-HO-101-2, KS-KL-101-2, KS-HE-101-4).
3. **Il server non è MAI girato flag-on e «collaudato: sì» era più largo
   dell'evidenza** (Klabnik, Pedersen): la gamba server = sentinella da 2
   richieste; il registro pin passa a `collaudato:` GRADUATO PER GAMBA
   (A-PE-101-1) + sentinella estesa (N≥16, interleaved, workers>1,
   A-PE-101-3); VIETATO il flip senza parità server flag-on nei DUE modi
   (KS-KL-101-1, KS-PE-101-1).
4. **Le cifre della pre-misura sono BANDE, non tariffe** (Matsakis, Bak,
   Stogov, Hoare): 57/43 → banda 52–62% (C0 è cross-tree: serve C0'
   same-tree per ereditarla, A-MA-101-1/KS-MA-101-2); la banda [0, 0,5]
   del registro è hit-only e per-forma (BinarySC paga `to_zval` sul hit,
   A-ST-101-1); il criterio 0,7 ns delle occorrenze future siede SOTTO il
   pavimento della sonda (1,0 ns): inaggiudicabile — soglia ≥ pavimento
   (KS-BA-101-1).
5. **Il flip RITIRA H-B2 sui siti stack non fusi** (Matsakis R2):
   l'emissione on non ha BinaryAdd; dove le finestre non coprono, il flip
   perde la specializzazione — la sorte si decide CON MISURA pre-flip
   (A-MA-101-3, arbitrato team flip-contratto: precondizione d'ordine).
6. **Morsi confermati a macchina e SALDATI in chiusura di S-99**: il
   dente anti-putenv giudicava il chunk del `{main}` (path incluso come
   COSTANTE nel dump — Klabnik R1; riparato: match sull'HEADER, bracci
   ri-verdi); la riga media-peak diceva «si muove l'oracle» ma la gamba
   phpr media è **+2,73% (+31,9 MB), NON piatta** (Leijen; REPORT_GAP_99
   e GAP_TREND sanati con la tabella delle gambe raw, A-LE-101-4 ✓,
   A-GR-101-1 ✓).

## Ordine DEFINITIVO S-100 (regola di ammissione applicata; le
## precondizioni del flip BLOCCANO l'oggetto promozione, quindi entrano)

1. **Contratto di modo** (primo atto, prima di ogni riga del flip):
   grafia dell'opt-out value-parsed nominata e documentata; bracci
   anti-putenv/M5/funnel ri-derivati per il mondo post-flip; launcher
   parità BIMODALE (A-KL-101-6).
2. **Strumenti sanati, in quest'ordine** (team flip-contratto):
   sanatoria dump/`lowered()` (A-HE-100-4) in TESTA; match ESAUSTIVI su
   `visit_addrs` e `bin_op_of` (variante nuova ⇒ NON COMPILA,
   KS-HE-101-2/A-MA-101-2); funnel-probe oltre `{main}` (A-HE-101-4);
   POI la definizione operativa del diff riga-per-riga (A-KL-101-4) +
   assert conteggi↔nomi nei gate (A-KL-101-3).
3. **Sette trappole AssignOp PER NOME** (A-ST-101-3; KS-ST-101-1 blocca
   il flip senza di esse) + **decisione H-B2-sotto-flip con misura**
   (A-MA-101-3).
4. **Collaudo flag-on completo PRE-flip**: diff per-test off↔on dei
   corpus (zero differenze attese, A-KL-101-4); parità server flag-on
   (launcher bimodale + sentinella estesa); spread oracle peak R≥2 +
   nome all'anomalia +28,7% (A-GR-101-2); coppia WP full+media + peak IN
   MODO on con BANDE PRE-REGISTRATE (A-GR-101-3; KS-GR-101-1: senza
   banda = fotografia, non gate; gate peak = phpr-off vs phpr-on
   stessa-sera, KS-LE-101-1).
5. **Flip del default** + rotazione pin con braccio OFF collaudato a
   ogni rotazione (KS-HE-101-3; campo `modo:`/gambe nel registro,
   A-PE-101-1).
6. Se il timebox regge: **H-C prima misura** (conteggio×costo + profilo
   a campioni CO-EQUALE, A-BA-101-2; strumento lato oracle NOMINATO:
   census opcache, A-GR-101-4).

**BACKLOG per NOME** (non slot di sessione): A-HO-101-1 (sigillo di
tipo), A-HO-101-4 + A-PE-101-5 (`--build-info`: identità pin oltre
l'hash churnante), A-PE-101-2 (census env PHPR_* lazy), A-MA-101-1 (C0'
same-tree), A-BA-101-3 (census post-flip, CmpJmp* a parte), A-LE-101-2
(sonda taglia-unità off/on), A-LE-101-3 (N_OPS const-assert +
`size_of::<Op>()` invariato, KS-LE-101-2), A-ST-101-2 (perimetro compare
PRIMA del controfattuale cmp, KS-ST-101-2), A-ST-101-4 (mappa divergenze
solo-flag-on), A-HE-101-3 (controllo positivo chiave unit-cache),
A-GR-101-4 già in ordine al punto 6. KS-ST-101-3: il registro resta
chiuso salvo misura ≥ pavimento.

L'ordine NON è solo apparato: i punti 3-6 sono misure e collaudi
dell'oggetto; i punti 1-2 sono le precondizioni che il flip esige per
non essere VOID (5 sedie convergenti).

## Conflitti registrati

- team flip-contratto: unit-cache key (Hoare: fidarsi del fp; Hejlsberg:
  mai collaudata) → RISOLTO con controllo positivo (A-HE-101-3, backlog).
- team flip-contratto: H-B2-sotto-flip (Matsakis: estendere; altri:
  misurare la perdita) → precondizione d'ordine CON misura, non veto.
- team coda-stack: ordine AssignOp-vs-profilo → compatibili (trappole
  pre-flip; profilo dentro H-C).
- team evidenza-server e team misura: nessun conflitto, solo enfasi.
