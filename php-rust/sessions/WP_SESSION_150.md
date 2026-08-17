# WP_SESSION_150 — BT1 PROMOSSA (pin s150); scommessa ORM VINTA OLTRE-ATTESA (8,4→7,1×); FR1 chiusa per eliminazione

**In una frase**: la cura di debug_backtrace è entrata nel motore ufficiale
con tutti i controlli verdi e la suite Doctrine è scesa da 8,4 a 7,1 volte il
PHP originale (6 s guadagnati); il piccolo rallentamento di S-146 è stato
circoscritto (non è nell'esecuzione del fuso) e chiuso senza revert.

**SCOREBOARD** (pin **s150 phpr cbbe71735effb165 + server 18c2740774336c82**):
arith 5,5 → · prop 5,5 → · calls 4,8 → · str 4,3 → · arr 3,3 (+1 tick) ·
re 2,5 → · **WP t4 mediana 1,781 COMPATIBILE** (1,745–1,800, 6/6 pulite;
PRIMO giudizio a MEDIANA) · media 2,480–2,555 · **ORM 8,370–8,427 →
7,104–7,149 ↓↓ · dbal 8,20–8,37 → 7,283–7,491 ↓** · corpus **1412×2** ·
**leve spedite: 1 (BT1 PROMOSSA)** · incidenti 17→**18** (flip-handler coi
rami mai collaudati: due stop fail-closed, emende dichiarate).

## Esiti secchi
1·**p.1 PROMOZIONE rc=0**: batteria 1747/0/2; corpus 1414→1412 (PASS:
  `debug_backtrace_limit`+`bug64239_2`; mutato: `debug_backtrace_options`);
  fixture **10/10** (fx-backtrace, dente provato anche in negativo); micro
  invariate; **guardie 8/8 R=5 + disasm bl 6014 invariato = incidente 17
  RIPARATO**; m-backtrace D=+19000 5/5; **bilaterale NETTO 5,50×** (pavimenti
  misurati). Identità candidato↔braccio giudicato provata AL BYTE (il braccio
  era fuori-ricetta: Δ = solo timestamp/UUID/firma, `s150-identita-candidato.md`).
2·**p.2 census controllo**: spiegazione path **CADE** (a 118 char 335,8M
  identico al run corto); +3,2% vs s148 da RE-ISTRUIRE (candidato:
  auto-conteggio del probe s149).
3·**p.3 coppia t4**: 6/6 pulite, **mediana 1,781 ∈ [1,738; 1,799]** ⇒
  COMPATIBILE; peak MISTO dichiarato; nessuna deriva.
4·**p.4 SCOMMESSA VINTA OLTRE-ATTESA**: ORM Δ **+6,07/+6,70 s** (attesa
  0,8–3,1 = PAVIMENTO solo-alloc dichiarato); dbal companion ↓ coerente.
5·**p.5 FR1 CHIUSA, esito (b)**: dente su dimread (D=+16,7, 5/5 = numeri
  S-145); dimrmw10 D=+0,3 sotto soglia; fuso FUORI dal loop RMW ⇒ **+3,00
  LOCALIZZATO PER ELIMINAZIONE, indiziato il delta strutturale +3180 B/+26 bl**
  (rett. rev.) ⇒ NESSUN revert; interruttore rimosso al byte (928d87d).

## ⭐ Lezioni (max 3)
- ⭐⭐ Un braccio A/B fuori ricetta dà un hash irripetibile: gemello SEMPRE
  stashato e ricetta ESATTA (env incluso) nell'atto A/B.
- ⭐⭐ Un'attesa su un PAVIMENTO dichiarato rende l'oltre-attesa un esito
  leggibile (lavoro non prezzato), non un errore di modello.
- ⭐ Il kill-switch monobinario vale solo col DENTE sull'altro giudice: il
  +16,7 esatto trasforma lo «0» sul bersaglio in evidenza.
