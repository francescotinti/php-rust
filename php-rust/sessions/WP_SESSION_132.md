# WP_SESSION_132 — riferimento WP PER CONFIGURAZIONE + L-LO1 lookup-once SPEDITA (pin s132)

**In una frase**: la velocità di WordPress è stata rimisurata separando per la
prima volta le due configurazioni (riferimento canonico = flag di default:
1,75–1,77), e la seconda cura nominata dal modello — toccare la tabella delle
proprietà UNA volta invece di tre — è stata promossa su tutta la catena.

**SCOREBOARD** (pin NUOVO **s132 6af6e497**5ef8d0bf + server s132 ad17a10d; micro gate promozione):
**arith 5,5 ↘ · prop 5,5 ↘ · calls 5,0 ↗ · str 4,3 = · arr 3,1 ↘ · re 2,5 =**
(mosse ≤0,1 = jitter denominatore) · **WP full ON-ONLY CANONICO = 1,752–1,768**
(@ pin s131; off 1,783 N=1; misto 1,751–1,797; da rifare su s132: L-LO1 morde
FieldAssign non-leaf) · **leve spedite: 1 (L-LO1)** · incidenti: 0 nuovi. 2026-08-12.

## Esiti secchi
1·**Az.rev. S-131 #1–#5 TUTTE cablate PRIMA della prima misura**: #1 soglia =
  max(4, drop-1, spread-batch s131 10,0, SL) · #2 riconciliazione dichiarata ·
  #3 per-config + firma per gamba · #4 trange deterministico · #5 file NUOVO per tentativo.
2·**Coppia WP su s131** (criterio 00f4dd0): 3/4 gambe pulite, quiescenza rc=0 ×5
  · **leg1-off ESCLUSA dal gate CON la firma prevista dalla rev. S-131** (ictx
  +61%, oracle CPU rank 1/4) · **full on-only 1,752–1,768 CANONICO** · media
  user-only 2,453–2,481 · peak 1754–1810 MiB · parità 4/4 → REPORT_GAP_132.
3·**LEVA L-LO1 SPEDITA** (criterio 3fbe3c1 PRE-registrato → codice 92ce9ce): UN
  accesso alla props-map nel ramo non-leaf di field_write_prop_step
  (get_slot_mut(si) via slot WP-29 stampato dalla resolve, fallback get_mut) ·
  smoke +23,3 vs 10,0 · R=5 **+20,0 vs 10,0** · riconciliazione 3,3 IN BANDA,
  dentro UB 30 · guardie 9/9 · promozione rc=0 (batteria 1746/0/2 inventario
  identico, corpus 1414, fixture per NOME, ORM 16 nomi, hk 0E/0F) → **pin s132**.
4·**Submicro s132**: objdatains **7,2** (1200,0, −20 = L-LO1) · churn **8,2** ·
  alloc 7,8 = (ctor non toccato ✓) · dropdef 9,0 (denom. 0,41 sotto-scala, non
  guardiata) · allocni 9,7 · objmap 17,3 =.
5·**Forma ctor RINVIATA dichiarata** (70,8 ns, §S-133). Pre-flight: disco
  sistema <15G sanato (debug/, cache updater, toolchain 1.88 non pinnata).
## ⭐ Lezioni (max 3)
- ⭐⭐ La firma per gamba PRE-cablata cattura l'anomalia sul campo: leg1-off esclusa
  con esattamente la firma che la revisione S-131 aveva descritto a posteriori.
- ⭐⭐ Seconda leva del modello promossa alla prima uscita e stavolta il quadro
  soglia/rumore REGGE (riconciliazione 3,3 in banda; S-131: 21,7 fuori) — la
  soglia fondata sullo spread-batch è quella giusta.
- ⭐ Prima di inventare cache: cercare l'indice che il compilatore STAMPA già
  (slot WP-29 nella resolve) — la lookup-once diventa bypass totale di slot_of.
