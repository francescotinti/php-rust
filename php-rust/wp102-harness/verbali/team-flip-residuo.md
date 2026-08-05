# Team «flip-residuo» — Concilio WP-102, fase 2

**Relatore**: sedia 1 (Hoare). **Team**: 1 Hoare, 4 Hejlsberg, 3 Klabnik.
**Fonti**: verbale-1-hoare.md, verbale-4-hejlsberg.md, verbale-3-klabnik.md (restano VINCOLANTI; questa nota non li sostituisce).

## Nota di sessione (fatti accaduti DOPO la fase 1)

1. Il fix di `emit_binary` (lettura da `ctx.reg_lower` al posto di `reg_lower::enabled()`, **A-HO-102-1**) è **GIÀ APPLICATO**; i denti mirati sono **verdi**. Batteria completa **in corso** al momento della stesura.
2. Il corpus-diff e il corpus-gate verranno **RI-ESEGUITI sul PIN NUOVO post-fix**: la stessa ri-esecuzione soddisfa sia **A-KL-102-1** (evidenza sull'albero giudicato) sia il collaudo del fix stesso.
3. Le refutazioni R1 restano a verbale su S-100 *come spedita* (concordanza esplicita di sedia 4): il fix in-flight non le sana retroattivamente.

## Convergenze (con numeri emendamento)

**C1 — Refutazione CAPITALE condivisa: sito ambientale residuo in `emit_binary`** (Hoare R1 ≡ Hejlsberg R1, trovata indipendentemente). Il claim S-100 «il modo è un INPUT del funnel, niente premesse ambientali» era FALSO alla chiusura: `compile/mod.rs:780` consultava il globale OnceLock a una chiamata di distanza dal campo `ctx.reg_lower`. Conseguenza concorde: il braccio OFF in-process sotto batteria default-ON non era l'emissione OFF di produzione (due fonti di verità). Rimedio: **A-HO-102-1** (applicato, v. nota §1) + doc-comment stale sanato nello stesso commit.

**C2 — Il fix senza dente è regressione garantita** (Hejlsberg R2/**A-HE-102-1** + dente richiesto in A-HO-102-1 + Klabnik KS-KL-102-1 in spirito). La violazione era invisibile alla batteria; commento ≠ dente. **KS-HE-102-2**: A-HO-102-1 non si committa senza A-HE-102-1 (braccio OFF con `BinaryAdd` PRESENTE, braccio ON senza `Binary(Add)`; sanare il commento di `stage2v3_flag_off`, reg_lower.rs:936, che col fix diventa FALSO).

**C3 — La purezza del funnel si prova ESEGUENDO nei due modi espliciti** (Hejlsberg **A-HE-102-2**; Hoare: dente dump-hash braccio OFF ≡ produzione OFF `env =0` processo separato; Klabnik **A-KL-102-3**: dente absent ≡ `=1` a parità di dump-hash). Tre formulazioni dello stesso principio: nessun braccio della matrice modi resta presunto.

**C4 — Commenti-gate morti o stale da sanare per NOME** (Hoare: doc-comment `emit_binary`; Hejlsberg: stage2v3 reg_lower.rs:936 e reg_lower.rs:289 senza perimetro; Klabnik: antiputenv.rs:108 cita dente MORTO). Un commento che indica un tripwire inesistente è un gate di carta.

**C5 — H-C1 non si iscrive senza precondizioni dure** (tutte e tre le sedie, da angoli diversi): Hoare **A-HO-102-3**/**A-HO-102-4**/**KS-HO-102-1**/**KS-HO-102-2** (scelta di design (a) refcount+COW vs (b) fusione in-handler PRIMA dell'iscrizione; fixture aliasing; corpo condiviso dei due handler Add); Klabnik R4/**A-KL-102-4**/**KS-KL-102-2**/**KS-KL-102-3** (matrice fixture chiusa per NOME inclusi alias byref, timing `__destruct`, weakref/GC, typed coercion; soffitto pre-registrato ~27% ⇒ H-C1 NON è la cura del 12,4→3; WP pair non derogabile); Hejlsberg **KS-HE-102-1** (niente H-C1 prima della batteria due-modi nella stessa rotazione). Convergenza piena: parità d'emissione ≠ parità di RUNTIME (Klabnik R4, leak WP-78 come precedente).

**C6 — Evidenza solo dall'albero giudicato** (Klabnik R1 CAPITALE + **A-KL-102-1** + **KS-KL-102-1**; nessuna sedia dissente). Soddisfatta dalla ri-esecuzione sul pin nuovo (nota §2).

## Conflitti (posizione di ciascuna sedia)

**K1 — Forma del fix `emit_binary`.**
- *Hoare*: preferenza dichiarata per la variante FORTE — emettere `BinaryAdd` **incondizionatamente** (le finestre fondono entrambe le grafie per dichiarazione propria), facendo sparire del tutto l'`enabled()` compile-side. Nota del relatore: la variante forte sanerebbe anche il controesempio prop-init di Hejlsberg R3.
- *Hejlsberg*: concordanza registrata sulla variante `ctx.reg_lower` (quella applicata), ma vincolata al dente (KS-HE-102-2); in più pretende perimetro del tripwire dichiarato + controesempio `self::K+1` pinnato come eccezione ATTESA (**A-HE-102-4**), che con la variante applicata resta necessario.
- *Klabnik*: nessuna posizione sulla forma.
- Stato: il fix applicato è la variante debole; la variante forte resta proposta aperta per S-101 (assorbirebbe A-HE-102-4).

**K2 — Rimedio alla molteplicità del modo (OnceLock + ctx + UnitKey, «tre posti tenuti d'accordo per convenzione»).**
- *Hoare* (**A-HO-102-2**): sigillo di TIPO — testimone ZST reso da `seal_reg_lower_mode()` e preteso dal confine di compilazione: l'omissione NON COMPILA. Promozione da backlog a **S-101**.
- *Hejlsberg* (R5 + **A-HE-102-5**): modo stampato NEL `Module`, `UnitKey.reg_mode` derivato da lì, dente a chiave costruita a mano (A-HE-101-3 oggi INESERCITABILE).
- *Klabnik*: non si pronuncia sul meccanismo.
- Conflitto reale: non sul merito (complementari) ma sulla **priorità** — Hoare lo vuole in S-101, Hejlsberg non ne fa un keystone. Il team NON compone d'ufficio: entrambe le voci restano a verbale.

**K3 — Ordine delle precondizioni H-C1.** Hoare antepone la scelta di design (KS-HO-102-2); Klabnik antepone matrice fixture + soffitto pre-registrato (A-KL-102-4, KS-KL-102-2); Hejlsberg antepone la batteria due-modi (KS-HE-102-1). Nessuna contraddizione di merito: sono TRE gate cumulativi; il conflitto è solo su quale citare come primo. Il team li registra come congiunzione (tutti e tre necessari).

## Priorità proposte per l'ordine S-101 (perimetro flip-residuo)

Regola applicata: **apparato in ordine solo se blocca l'oggetto**; il resto = backlog per NOME.

### BLOCCANTE (in ordine)

1. **Chiusura del fix A-HO-102-1 secondo KS-HE-102-2**: commit del fix (già applicato e verde sui denti mirati) SOLO insieme ad **A-HE-102-1** (dente OFF-con-BinaryAdd / ON-senza-Binary(Add) via `compile_mode`) e alla sanatoria dei commenti C4 (stage2v3, doc-comment emit_binary, antiputenv.rs:108). Attesa: esito batteria completa in corso.
2. **A-KL-102-1** — corpus-diff + corpus-gate RI-ESEGUITI sul PIN NUOVO post-fix (già programmato, nota §2). Nessun gate futuro cita evidenza pre-pin (KS-KL-102-1). Chiude anche il buco carry-over dei chunk FAIL (byte-diff mai ri-giudicato sul pin).
3. **A-HE-102-2** — batteria cargo nei DUE modi espliciti (`=0` e `=1`) alla rotazione. È il cancello di KS-HE-102-1: senza di essa H-C1 non si iscrive.
4. **A-KL-102-3** — dente absent ≡ `=1` a parità di dump-hash (post-flip il percorso di produzione è giudicato da UN bit: va pinnato per costruzione, non per letterale).
5. **Gate d'iscrizione H-C1** (blocca H-C1, non il residuo flip; da consegnare all'ordine S-101 come precondizione cumulativa C5): scelta di design (a)/(b) dichiarata PRIMA + matrice fixture chiusa per NOME (aliasing Hoare + byref/`__destruct`/weakref/typed Klabnik) + soffitto pre-registrato KS-KL-102-2 + WP pair KS-KL-102-3.

### BACKLOG per NOME (apparato: non blocca l'oggetto)

- **A-HO-102-2** — testimone ZST del sigillo (Hoare lo vuole BLOCCANTE in S-101: dissenso registrato in K2; il relatore lo colloca qui perché non blocca né il fix né la ri-esecuzione sul pin).
- **A-HE-102-5** — modo nel `Module` + `UnitKey.reg_mode` derivato + dente a chiave manuale (chiude A-HE-101-3).
- **A-HE-102-3** — destrutturare `CompiledClass` in `all_funcs` (esaustività 3/3).
- **A-HE-102-4** — perimetro tripwire per NOME + controesempio prop-init pinnato (decade se S-101 adotta la variante forte di K1).
- **A-HE-102-6** — fixture ereditarietà dump + politica dedup dichiarata.
- **A-KL-102-2** — carve-out nondet per TOKEN + prova entropia intra-modo ANCHE flag-on.
- **A-HO-102-4** — corpo condiviso handler `Binary(Add)`/`BinaryAdd` (il differenziale degrada a cintura di regressione). Diventa bloccante SOLO all'iscrizione di lavoro d'emissione ulteriore (KS-HO-102-1, oggi soddisfatto dal fix).
- Variante forte di `emit_binary` (K1, Hoare) — proposta aperta, da giudicare in S-101.
