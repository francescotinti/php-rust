# Revisione adversariale S-184 — lente MISURA (L-RT1+L-RT2 → pin s182)

Generato con assistenza AI (Claude Fable 5).

**Verdetto: REGGE CON RILIEVI**.

## Ciò che regge
- Cifra solida: ABAB R=5, segni 5/5, rumore 0,67/0,33, |A−Z| 1,00 ≪ D (`s184-rt2-verdetto.out`).
- Nessun confondente: `git diff --stat f5f746cf 454a77c3` tocca solo `vm/run.rs` (commit L-RT1 e L-RT2) e il profilo `dev-release` (inerte).
- Confermata sul pin canonico: +12,33 5/5 (`s184-promo-verdetto.out`, «conferma post-pin»).
- Guardie piatte, batteria 1748/0, corpus zero flip, pin 159eee40 ricostruito al byte due volte (`azrev4-out/verdetto.out` (a)).

## Rilievi
1. **Surplus non spiegato, meccanismo non firmato.** D 12,50/13,17 contro attesa [3,5;6,5] (`s184-criterio-rt2.md` p.4; verdetto «ATTESA … FUORI»). Il p.9 è un'ipotesi. Il braccio tree-RT1 9ef70777 era disponibile (usato per il disasm) e NON è stato misurato. Provato: «commit 454a77c3 vale +12,5», non «Ret fuso + ret_slow vale +12,5» (REGOLE 4).
2. **Gate di meccanismo neutralizzato.** `ab-out/corsa2-rc5/…verdetto.out`: «Δsp_refs=76 FUORI attesa (≤40) … rc=5»; poi p.8 «≤ +100 (classe osservata, dichiarata)», rieseguito con `SKIP_BUILD=1` sugli stessi hash. Soglia riscritta fino a far passare il binario già letto.
3. **Costo dichiarato non misurato prima del pin.** p.7: «UNA chiamata in più per i Ret non ammessi (metodi, main, hint): sotto-risoluzione sui giudici». Nessun giudice a tempo con Ret di metodo; t25 assente. Segno ignoto su ORM/WordPress.
4. **Disasm del pin assente.** Gate su B ce6ae73f; pin 159eee40 «candidato A CONTENUTO» (promo-verdetto): tempo confermato, codice non letto.
5. **Finestra sporca alla fine.** `s184-rt2-verdetto.out`, «sentinelle FINE 19:40:07»: mobileassetd 73,3 %, AuthenticationServicesAgent 46,6 % — oltre il gate p.6 «> 50 % ⇒ attesa», applicato solo all'avvio; il watchdog non guarda la CPU. Coppia 5 in linea.
6. **PREV calls-dq perso.** Verdetto «|A−PREV| = n/d» mentre `s183-rt1-verdetto.out` ha A=89,67 sullo stesso 19a2faa8: banda 1,00 computabile, tenuta solo per prop-dq.

## Azioni S-185
1. A/B a quattro bracci su calls-dq: pin s181 / tree-RT1 9ef70777 / forma 1 (fc5d922a) / pin s182, attesa pre-registrata per ciascuno.
2. Guardia a tempo con Ret di metodo nel loop (prop-dq con `$o->get()`), pin s182 vs stash s181, R=5; regressione ⇒ leva in istruttoria.
3. Disasm `run_loop` del pin 159eee40 agli atti; soglia sp_refs derivata (Z vs pin) e scritta una volta: se morde si rifà la build, non la soglia.
4. Sentinella CPU anche alla fine e nel watchdog; PREV same-binary per tutte le categorie.
