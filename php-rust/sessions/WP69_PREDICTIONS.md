# predictions69 — pre-registri WP-69 (K-69.2/G-69.5, PRIMA di profilo e letti L-68.1)

Committate e lockate PRIMA di: (a) qualsiasi lettura del profilo
`sample` dello spike dispatch; (b) qualsiasi letto L-68.1 (HITS-off o
counter). Binario di parità: post-punto-0 (tree 76adc8c). Ogni
predizione riceve disposizione PER BANDA in design69 §9 (K-69.5).

## A. SPIKE DISPATCH (rotta utente; condizione WP-44)

**Meccanismo dichiarato**: il costo del dispatch vive nel NUMERO di
corpi handler caldi del run_loop (verdetto WP-44); il profilo separa
DENTRO i corpi l'operand-fetch/stack-traffic (push/pop dello stack
VM, deref dei payload Rc degli Op) dal lavoro utile. Workload: media
group (CLI, una richiesta — il meccanismo single-request è dichiarato)
+ full-suite finestra campionata.

- **P69-S-a (quota operand-fetch)**: dentro la finestra run_loop del
  profilo outlined, la quota attribuibile a operand-fetch +
  stack-traffic ∈ **[15%, 40%]** del tempo dei corpi caldi
  (sotto il 15% il canale non paga alcun prototipo; sopra il 40%
  sarebbe incoerente con l'attribuzione owner-level WP-54
  corpi+dispatch=41,9% del totale).
- **P69-S-b (concentrazione)**: i top-10 handler per tempo coprono
  ≥ **60%** della finestra run_loop (input WP-42 frequenza×taglia).

**TABELLA-DECISIONE profilo→prototipo (criterio ex-ante, KG69-2)**:
| Esito profilo | Prototipo |
|---|---|
| operand-fetch ≥25% della finestra corpi E concentrato nei top-10 handler | tranche SUPERISTRUZIONI sui bigram più caldi (riduce i corpi caldi — WP-44 ok) |
| dispatch-overhead (match/branch fuori dai corpi) ≥15% E operand-fetch <25% | core-caldo-ridotto (ristrutturazione del loop, MAI nuovi corpi) |
| nessuna delle due soglie raggiunta | **RICHIUSO senza prototipo** (KB69-2: infalsificabile) |
Token-threading: NON candidato (Bak: il match è già jump-table).

**MDE (KB69-2, pre-registrato)**: spread di riferimento = self-spread
IR della tripla serale di WP-68 = **0,26%** (G-69.1: il giudice è ΔIR,
non CPU-user). MDE = 2× = **0,52% IR**. Effetto atteso del prototipo =
(quota profilata eliminabile, cap 50% della quota operand-fetch) ×
(share run_loop sul totale, WP-54 41,9%): se < 0,52% ⇒ arco RICHIUSO
a verbale SENZA prototipo. "Sotto spread" nel giudizio finale =
|Δcoppia IR| < max(self-spread IR della serata, 0,26%).

**Vincoli**: profilo SOLO su binario outlined-per-nome validato in
coppia vs release (B-69.4; outlined MAI giudice); K-69.3 contatore
meccanico dei corpi caldi pre/post nel gate del worktree; KH69-3
zero-unsafe; KS-P69.4 lifecycle frame/RetainSet intoccato.

## B. L-68.1 CENSUS SELF-ACCOUNTING (forma doppia L-69.1)

**Definizione census_own (K-69.2, ELENCO CHIUSO delle tabelle)**:
`census_own_bytes` = Σ CAPACITY-aware di
(1) CENSUS_HITS (Vec backing: capacity × size_of<(Vec<u8>,u64)> +
heap dei path-key: capacity di ciascun Vec<u8>);
(2) CENSUS_UNITS + CENSUS_SPLITS (capacity × size_of riga; le Weak
NON contano il Module puntato);
(3) buffer uc_log non ancora flushati (capacity).
Fuori dal counter (dichiarato): allocazioni transienti dei dump
(vivono dentro la finestra di dump), stringhe di census_line.

- **L-69.P1**: wpdev HITS-OFF slope Σcommitted ∈ **[0, 0,6] KiB/req**
  su finestra di regime R100→R1000 (b683 forma slope-only).
- **L-69.P2**: nk-doc HITS-OFF slope ∈ **[0, 0,1] KiB/req** su
  R500→R1000 (L-69.2: warm-up escluso).
- **L-69.P3**: Δ(slope ON − slope OFF) riconciliato dal counter
  census_own ≥ **95%**; counter bocciato sotto 80% (KL69-2).
- **Kill-switch**: KL69-1/KS69-2 — HITS-off wpdev slope > 2 KiB/req ⇒
  leak ENGINE reale, fronte axum FERMO.

## C. CRON (B-69.5, attese PRIMA della coppia — P-69.6/KS-P68.3)

- **P69-C-a**: con DISABLE_WP_CRON=true e N=1000 (stesso protocollo
  b683), il segmento wp-cron.php NON compare nei reqmark
  (OTHER_SEGS=0 VERO stavolta) e il fp-set steady resta UN solo
  contesto: vivi modrecon ∈ **[500, 540]** (≈518, niente secondo
  set).
- **P69-C-b**: curl esplicito a wp-cron.php a R500 ⇒ fp-set
  R501..R510 ≡ R499 (nessun re-miss dei path front dopo il confine;
  KB69-3 detector: ways_evictions/req a steady = 0).
- **P69-C-c**: delta census del confine attribuito al solo segmento
  cron: Δvivi ∈ **[480, 540]** entry aggiuntive una-tantum
  (≈+43,9 MB/worker della lettura B-69.1), poi piatto.

## D. METRO (KL68-2/L-69.4)

NON si promuove in questa sessione senza reboot (protocollo G-69.6:
reboot → swapusage ≤64 MiB pre/post → Δpageouts≈0 → L-68.1
sottratto → verdict-file). Se la sessione non reboota: NON-ESEGUITA
a verbale, nessun letto phys del metro citabile (KG69-4).
