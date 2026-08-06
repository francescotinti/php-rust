# TEAM-GATE (Klabnik + Matsakis + Hejlsberg) — Concilio WP-105, fase 2
Relatore: team-GATE. Fonti: verbale-3 (KL), verbale-2 (MA), verbale-4 (HE).

## (a) Pacchetto unico di emendamento denti — "NESSUN ARBITRO SENZA ROSSO"
Le tre sedie dicono la stessa cosa da angoli diversi: un dente che non ha mai prodotto un rosso (MA-R1), un braccio senza controllo positivo (HE-R2), un claim su zoo incompleto (HE-R3, MA-R2 sui hook) non arbitrano. Pacchetto unico **DENTI-105**, tre articoli:
1. **Mutation-check obbligatorio**: ogni fixture-arbitro esibisce un rosso archiviato da build sabotata prima di entrare nei gate (A-MA-105-1; KS-MA-105-1: senza rosso, RC-MA-104 si RIAPRE).
2. **Ogni braccio si prova, non si deduce**: `==1` PER CORPO con match ancorato e lista corpi nel messaggio (A-HE-105-1); `=0` con controllo POSITIVO (`Z::{prop-init}` presente, Add>1) (A-HE-105-2) — recidiva A-PE-102-1.
3. **Zoo pinnato all'enumerazione dell'emettitore, non alla fantasia**: hook get/set (convergenza MA-R2/HE-R3 ⇒ 19c e BODY_ZOO condividono le fixture hook), default-param con closure, arrow fn, static-init; tripwire intestazioni-corpo == insieme atteso per NOME (A-HE-105-3 + A-MA-105-2). Più KS-HE-105-3: l'attesa non si aggiorna nello stesso commit della causa.

## (b) Disciplina di pin — REGOLA SCRITTA
**Regola PIN-105** (da A-KL-105-1/2/4 + HE-R4): (i) pin BILATERALE — anche l'oracle porta `ORACLE_PIN_ATTESO`, assente/mismatch ⇒ VOID; (ii) **pin di chiusura = sequenza atomica** build→hash→PIN+STASH→batteria→fixture→corpus×2, stesso hash verificato in testa a ogni launcher, qualunque rebuild azzera; (iii) `collaudo.done` porta hash+stash, cross-mode VOID su PIN_SRV diverso; (iv) un pin non pinna il layout: `size==16` si integra con `align_of` const-asserito + fingerprint della definizione di Zval (KS-HE-105-1). KS-KL-105-1: «GRADATO» senza stash contestuale = retroattivamente NON-GRADATO.

## (c) Parity-null verificabile
A-KL-105-3: parity-null = **funzionale** (corpus+batteria+fixture SUL pin di chiusura) **E strumentale** (micro 6 categorie SUL pin di chiusura, Δ entro banda pre-registrata vs pin precedente). Senza il braccio strumentale la coppia WP NON è differibile; nessun numero in baseline da pin diverso senza dichiararlo NELLA riga (KS-KL-105-2). Nota HE: bande solo da rumore MISURATO quella sera (KS-HE-105-2), banda-layout 0,67 = N=1, non citabile.

## (d) Blocca / non blocca S-104
**BLOCCANO la leva H-C2** (dentro la sua finestra, non prefissi nuovi): mutation-check A-MA-105-1 (pre-atto), dente A-HE-105-1/2/3, dente fast-out A-MA-105-4, KS-HE-105-1 scritto nel criterio PRIMA di aprire, pin launcher A-KL-105-1/4/5. Il verdetto A/B R=7 resta primo atto, invariato per tutte e tre le sedie.
**APPARATO ⇒ backlog per NOME** (regola di ammissione: entra solo se il suo assente falsifica un claim di S-104): A-MA-105-3 ledger fine-vita (KS-MA-105-2: la sua assenza degrada solo l'ATTRIBUZIONE, non la promozione aggregata); A-MA-105-2/19c (timebox igiene); A-HE-105-4 leva-nulla ricampionata (DENTRO la campagna A/B, non prima); allineamento rc 0/1/66 (KL-R6).
