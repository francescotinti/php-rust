# Team SEMANTICA-FINESTRE — Concilio WP-108, fase 2
**Membri**: Stogov (relatore), Hoare, Matsakis · **Data**: 2026-08-07
**Fonti vincolanti**: verbali/verbale-8-stogov.md, verbale-1-hoare.md, verbale-2-matsakis.md

## 1. Convergenze (ID canonico)

**C-SF-1 — Finestra ST (H-A1) CONFERMATA 3/3.** I perché, uno per sedia:
- Stogov: la forma fusa È la forma Zend — ZEND_ASSIGN_OP è UN'op RMW (op2 prima,
  op1 dopo, deref, binary in place); il tris era l'anomalia phpr.
- Hoare: profondità di pila identica in ogni punto fallibile (tris +1+1−2, fusa
  +1−1) + read_slot/reg_store_slot condivisi AL SIMBOLO ⇒ unwind e slot dst
  identici su errore.
- Matsakis: ordine di osservazione invariato per NOME — Pop/Swap senza effetti,
  snapshot OWNED da read_slot, nessun borrow attraversa binary_value_ab ⇒
  nessuna finestra di rientranza NUOVA.

**C-SF-2 — R-ST-108-1 ≡ R-HE-108-1** (doc «Fold rules» contraddice la finestra):
riallineo DOVUTO, testo proposto in T-SF-108-1.

**C-SF-3 — §3.11 divergenza PRE-esistente**, non della leva (Stogov checklist,
Hoare R-HO-108-3 limite LoadVar, Matsakis KS-MA-108-1): cura futura col template
warning-risintetizzato delle SS, MAI de-fusione. D-21 resta: attese di corpus
cercate per NOME nel fail-set congelato.

## 2. Conflitti risolti

**CF-1 — A-MA-108-2 vs KS-LE-108-1** («contatore senza lettore non è un dente»).
Composto, non scelto: la cura completa è contatore incondizionato in release
nell'arm ArgPlace **+ LETTORE obbligato** — gate di fine batteria/corpus che
legge il contatore e pretende ==0 in ogni run release. Il contatore da solo
sarebbe documentazione (KS-MA-108-2); il lettore lo rende dente. → T-SF-108-3.

**CF-2 — R-HO-108-2 (throwing-store)**: difetto EREDITATO dalla famiglia Dst,
non introdotto da H-A1 — nessun addebito alla leva, ma dente obbligato al punto
1; se il dente trova divergenza, arbitro = oracle, e la cura NON passa per la
de-fusione (coerenza C-SF-3). → T-SF-108-2.

**CF-3 — ordine (Matsakis: D-12 prima o insieme a §3.15)**: risolto per
COLLOCAZIONE — la cura D-12 completa entra nel punto 1 (denti), che precede il
punto 2; §3.15 tocca il perimetro ArgPlace solo a gate D-12 verde. → §4.

## 3. Direttive T-SF-108-n

1. **T-SF-108-1 (VINCOLANTE)** — Riallineo doc reg_lower.rs (r.17-21). Testo:
   «LoadVar (warning) is never folded. LoadSlot (silent) È foldabile SOLO nella
   finestra ST (BinarySTDst, r.475-482), che eredita la disciplina silente di
   read_slot; il silenzio su undef qui è la divergenza catalogata §3.11
   (PHPR_DIVERGENCES), non un contratto. Equivalenza al tris ⇐ il pop non muta
   gli slot. Copertura: solo spelling LoadSlot. Ogni nuova finestra nomina
   QUALE disciplina dei due read eredita.» Stesso commit: emendare il commento
   finestra che cita il "contratto". [assorbe R-ST-108-1, R-HE-108-1,
   R-ST-108-2, R-HO-108-3, A-HO-108-3, KS-ST-108-1]
2. **T-SF-108-2 (VINCOLANTE)** — Dente throwing-store famiglia Dst: compound
   assign via typed-ref che lancia nello store, in try/catch, operandi con
   __destruct, flag-on/off/oracle. S-107 punto 1. [assorbe A-HO-108-2,
   R-HO-108-2]
3. **T-SF-108-3 (VINCOLANTE)** — D-10 con gamba a debug_assertions attive O
   census con assert contatore==0; PIÙ cura D-12 completa: contatore release
   incondizionato nell'arm ArgPlace + lettore/gate ==0 a fine batteria/corpus.
   «Backstop rumoroso: saldato» dichiarabile SOLO a gate verde. [assorbe
   A-MA-108-1, A-MA-108-2, R-MA-108-4, KS-MA-108-2; compone KS-LE-108-1]
4. **T-SF-108-4 (VINCOLANTE)** — Dente drop-order = PREREQUISITO di admission
   della leva D-20 (metà-Zend): leva su decay/call-path senza dente = VOID;
   nessun allargamento del direct-bind col commento D-9 declassato. [assorbe
   A-MA-108-3, KS-HO-108-2; estende KS-MA-107-1]
5. **T-SF-108-5 (VINCOLANTE)** — §3.11: rimando incrociato §3.11 ↔ BinarySTDst
   a catalogo; cura futura = warning risintetizzato DENTRO l'op fusa col
   template SS (nome byte-identico), MAI de-fusione (micro come gate della
   cura); la candidata IncDecSlot+Pop DICHIARA l'accoppiamento §3.11 nel
   criterio. [assorbe A-ST-108-1, A-ST-108-2]
6. **T-SF-108-6 (VINCOLANTE, kill-switch)** — Estensioni finestra: qualsiasi
   estensione di BinarySTDst (forma value, rhs const, spelling LoadVar) senza
   criterio proprio pre-registrato E T-SF-108-2 verde = VOID; una finestra che
   assorbe op CON effetti (LoadVar-warning, FetchDim) NON eredita il verdetto
   H-A1. [assorbe KS-HO-108-1, KS-MA-108-1]
7. **T-SF-108-7 (RACC.)** — Commento sigillo trivial-arms: dichiarare SOLO il
   provato («no-Drop/bit-copy», non «scalari»); lista costruttori co-locata o
   cross-linkata col fast path consumatore. [assorbe A-HO-108-1, R-HO-108-1]
8. **T-SF-108-8 (RACC.)** — Opportunità nominate, solo con criterio proprio:
   guardia binary_fast senza-clone per BinarySTDst (residuo 11,6); censimento
   una-tantum dell'elisione gc_note nella famiglia *Dst. [assorbe A-ST-108-3,
   nota R-MA-108-3]

Vincolanti: 6 (T-SF-108-1..6) · Raccomandazioni: 2 (T-SF-108-7/8).

## 4. Modifiche all'ordine §S-107

Sequenza 1-5 CONFERMATA 3/3 (§3.15 testa di fedeltà, denti prima della leva).
Emendamenti:
- **Punto 1 (denti) — ENTRANO**: dente throwing-store famiglia Dst
  (T-SF-108-2); gamba debug_assertions/census ESPLICITA nel dente D-10 + cura
  D-12 contatore-più-lettore (T-SF-108-3), da chiudere PRIMA di aprire il
  punto 2 (soddisfa l'emendamento Matsakis su §3.15/ArgPlace); dente
  drop-order resta, ri-etichettato «prerequisito D-20» (T-SF-108-4).
- **Punto 1 bis (stesso giro di commit, costo doc)**: riallineo T-SF-108-1.
- **Punto 3 (leva)**: se la candidata è IncDecSlot+Pop, il criterio dichiara
  l'accoppiamento §3.11 (T-SF-108-5).
- **BACKLOG per NOME (NON entra al punto 1)**: guardia binary_fast BinarySTDst;
  censimento gc_note famiglia *Dst; commento sigillo trivial-arms (T-SF-108-7);
  ogni estensione della finestra ST (vietata senza criterio, T-SF-108-6);
  cura §3.11 vera e propria (template SS) — resta a catalogo finché non ha
  sessione dedicata.

— Stogov, relatore, per il team SEMANTICA-FINESTRE.
