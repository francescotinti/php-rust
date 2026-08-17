# WP_SESSION_150 — BT1 PROMOSSA (pin s150); scommessa ORM VINTA OLTRE-ATTESA (8,4→7,1×); FR1 chiusa: prezzo strutturale

**In una frase**: la cura di debug_backtrace è entrata nel motore ufficiale
con tutti i controlli verdi, e la suite Doctrine — il banco di prova più
duro — è passata da 8,4 a 7,1 volte il PHP originale (6 secondi guadagnati);
il piccolo rallentamento scoperto in S-146 è stato spiegato (costo di
struttura, non di esecuzione) e chiuso senza tornare indietro.

**SCOREBOARD** (pin **s150 phpr cbbe71735effb165 + server 18c2740774336c82**):
arith 5,5 → · prop 5,5 → · calls 4,8 → · str 4,3 → · arr 3,3 (+1 tick) ·
re 2,5 → · **WP t4 mediana 1,781 COMPATIBILE** (coppie 1,745–1,800, 6/6
pulite; PRIMO giudizio a MEDIANA) · media 2,480–2,555 · **ORM 8,370–8,427 →
7,104–7,149 ↓↓ · dbal 8,20–8,37 → 7,283–7,491 ↓** · corpus **1412×2** (flip
BT1 per NOME) · **leve spedite: 1 (BT1 PROMOSSA)** · incidenti 17→**18**
(flip-handler ai suoi primi due run di record: famiglia per-NOME e commit su
path ignorato — bracci mai collaudati prima; fail-closed ha retto, emende
dichiarate).

## Esiti secchi
1·**p.1 PROMOZIONE rc=0**: batteria 1747/0/2 inventario conforme; corpus
  1414→1412 (PASS: `debug_backtrace_limit`+`bug64239_2`; mutato:
  `debug_backtrace_options`; off↔on ZERO); fixture **10/10** (fx-backtrace
  nel set, dente provato ANCHE in negativo su s145); micro R=5 invariate;
  **guardie 8/8 a R=5 + disasm bl 6014 invariato = incidente 17 RIPARATO**;
  conferma m-backtrace D=+19000 5/5; **bilaterale NETTO B/or=5,50×**
  (pavimento oracle MISURATO, mai più RAW). Identità candidato: il braccio B
  giudicato era fuori-ricetta (senza SOURCE_DATE_EPOCH, target separato) —
  provata AL BYTE (Δ = solo timestamp/UUID/firma; `s150-identita-candidato.md`).
2·**p.2 census controllo**: spiegazione path **CADE** (a 118 char hostcall.n
  335,8M IDENTICO al run corto); +3,2% vs s148 da RE-ISTRUIRE (candidato:
  auto-conteggio della strumentazione per-NOME del probe s149).
3·**p.3 coppia t4**: 6/6 pulite, **mediana 1,781 ∈ [1,738; 1,799]** ⇒
  COMPATIBILE; peak MISTO dichiarato; nessuna deriva (ρ_A=0,03).
4·**p.4 SCOMMESSA VINTA OLTRE-ATTESA**: ORM Δ = **+6,07/+6,70 s** (attesa
  0,8–3,1 era il PAVIMENTO solo-alloc, dichiarato: la costruzione dei ~50
  frame non era prezzata); dbal companion ↓ coerente (deprecations).
5·**p.5 FR1 CHIUSA, esito (b)**: dente PROVATO su dimread (OFF perde
  D=+16,7, 5/5 = numeri S-145); dimrmw10 D=+0,3 sotto soglia; fuso FUORI dal
  loop RMW (dump) ⇒ **+3,00 LOCALIZZATO PER ELIMINAZIONE (non vive
  nell'emissione); indiziato il delta strutturale +3180 B/+26 bl** (rett.
  rev.: eliminazione, non prova diretta) ⇒ NESSUN revert; interruttore
  rimosso al byte (928d87d).

## ⭐ Lezioni (max 3)
- ⭐⭐ Un braccio A/B costruito FUORI ricetta dà un hash irripetibile:
  l'identità si salva solo se il gemello è STASHATO e si prova al byte;
  l'atto A/B deve registrare la ricetta ESATTA, env compreso.
- ⭐⭐ Un'attesa costruita su un PAVIMENTO dichiarato rende l'oltre-attesa
  un esito LEGGIBILE (lavoro non prezzato), non un errore di modello.
- ⭐ Il kill-switch monobinario vale solo col DENTE sull'altro giudice:
  il suo +16,7 esatto trasforma lo «0» sul bersaglio in evidenza.
