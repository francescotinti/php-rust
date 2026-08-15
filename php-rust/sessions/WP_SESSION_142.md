# WP_SESSION_142 — L-RD1 SPEDITA (catena 9 gate + conferma), invarianza VERIFICATA, quota census 0,24–0,53%, coppia FERMA, peak BIMODALE

**In una frase**: la leva sullo smontaggio degli array è stata spedita con
tutti i controlli e confermata dopo il pin; i dubbi del revisore sono chiusi
dai fatti (il residuo era codice di emergenza mai eseguito; la semantica è
identica al byte); sulla suite Doctrine la leva vale però solo ~0,3–0,5% —
quarta prova che serve la scommessa strutturale, e il dossier per il concilio
è pronto.

**SCOREBOARD** (pin s142 bba8a734+eeb284b6): **arith 5,5 ↓ · prop 5,6 = ·
calls 4,7 ↓ · str 4,2 ↓ · arr 3,2 = · re 2,6 =** · hintcall 7,3 (non rimis.) ·
WP rif **1,765–1,788 @ s142 (N=5, COMPATIBILE; banda_ON unione 0,036)** ·
**leve spedite: 1 (L-RD1)** · incidenti 14 (=; near-miss RA dichiarato sotto).

## Esiti secchi
1·**Catena L-RD1 rc=0** (s142-promozione, copia-gate): batteria 1746/0/2
  inv==s125 · re-hash A′ al byte · pin s142 · corpus 1414×2+golden · fixture
  9/9 · micro senza regressioni · **ORM 3E/13F per NOME** · hk 0E/0F · server
  eeb284b6. Candidato = gemello A′ di f751ef5b (91 byte attribuiti: UUID,
  __DATE__ dep C — S-141 senza SOURCE_DATE_EPOCH, dichiarato —, 2 hash
  derivati; 0 codice). **Conferma post-pin D=+5,0 al CENTRO banda, segni 5/5.**
2·**Az.rev. S-141 #1–#5 CHIUSE**: disasm agli atti — il bl residuo verso glue
  Zval è NEL LANDING PAD di unwind (dopo la ret): firma PIENA, non-Str ·
  parità Hashed/tombstoni/annidati/Rc: **A==B BYTE-ID nei 2 modi** (invarianza
  VERIFICATA) · nesting: soglia ~74–76k INVARIATA (packed e hashed, rc=134
  stack-guard) · strettezza soglia pre-registrata (AL BORDO ⇒ replica).
3·**§3.22 catalogata** (scoperta della micro-parità, PRE-esistente ≠ leva):
  `unset($a[k])` su elemento differisce il `__destruct` al drop dell'array.
4·**Census quota L-RD1** (contatori cfg-gated post-pin; 2 STOP hash ⇒ EMENDA
  dichiarata: byte-identità post-edit IRRAGGIUNGIBILE, panic-location):
  arrays 21,7M · elems 118,8M · r1==r2 esatti · **quota 0,24–0,53% suite ORM**
  = canale residuo sotto risoluzione coppia.
5·**Coppia @ s142 rc=0**: 1,765–1,788 N=5 FERMO come atteso; **peak BIMODALE
  1744–1850: bisezione rewarmup ⇒ ENTRAMBE le ipotesi S-140 refutate**.
6·**Dossier budget di parità v1+census** (`s142-dossier-parita.md`): divario
  37,6 s; canali ciclo-di-vita ~26–28 s; opzioni A/B nominate; residui nominati.

## ⭐ Lezioni (max 3)
- ⭐⭐ La byte-identità di una rebuild A′ vive solo a sorgenti INTATTI: ogni
  edit .rs (pur tutto cfg-gated) sposta i panic-location e cambia l'hash —
  l'instrumentazione post-pin si àncora allo STASH immutabile, non all'hash.
- ⭐⭐ Un bl «residuo» si giudica rispetto alla ret: nel landing pad di unwind
  non è sul cammino eseguito — il disasm ha chiuso il dubbio Str in 10 minuti.
- ⭐ Una micro con `__destruct` rende OSSERVABILE l'ordine di drop: una sola
  fixture ha verificato l'invarianza A==B e scovato §3.22.
- (near-miss dichiarato: Serena ha acceso rust-analyzer in catena — ucciso in
  ~1', fase corpus non-timing; pre-misura pgrep ha preso il respawn.)
