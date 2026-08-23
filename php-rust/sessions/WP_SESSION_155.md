# WP_SESSION_155 — coppia t6 + ORM SOTTO 7×; istruttoria CE1 chiusa (0-alloc CONFERMATO); gdc NON pagante; pin INVARIATO s154
**In una frase**: WordPress fermo (atteso), Doctrine per la prima volta
sotto 7 volte l'oracle, spiegato fino al byte il residuo di allocazione del
controllo-esistenza-classi (costo fisso di chiamata, non la cura), e
scartata coi numeri la leva successiva in coda prima che costasse una sessione.
**SCOREBOARD** (pin INVARIATO s154 bddc050320a6af4c + server b3cf348f69739edc):
arith 5,5 → · prop 5,6 → · calls 4,8 → · str 4,3 → · arr 3,3 → · re 2,5 →
(micro/corpus/batteria non rieseguiti: pin invariato, valori di promo s154) ·
WP t6 mediana 1,771 COMPATIBILE · media 2,456–2,510 · **ORM 6,972–7,053 ↓** ·
dbal 7,385–7,422 ↓ · **leve spedite: 0 (tentate 0) — ANOMALIA dichiarata**
(difesa: misure DOVUTE + istruttorie; gdc refutata prima del criterio) · incidenti 19 (=).

## Esiti secchi
1·Coppia t6 @ s154 rc=0: mediana 1,771 ∈ [1,738;1,799], 6/6 pulite, banda_ON
  0,022 (record), parità 6/6, peak 6/6 BASSE (doppio livello non riprodotto),
  deriva assente; attesa CE1 su WP «piccola/nulla» RISPETTATA.
2·ORM @ s154 rc=0: net 6,972–7,053 (Δ +0,41/+0,50 GIÙ fuori rumore) · dbal
  7,385–7,422; parità ORM 16 nomi== e dbal 10 stabile.
3·Sonda ce-count rc=0: k_post=1 (attesa 0 FUORI) → istruttoria CHIUSA al
  sorgente: 1×32 B = args-Vec di pop_keys attribuito al nome (scope s149
  PRIMA di pop_keys); controllo fe-count indipendente k=1 b=16,0 ESATTI;
  **hit-path CE1 = 0 alloc CONFERMATO**; canale == H-D S-103.
4·OLTRE-attesa ORM: attesa micro-fondata 0,05–0,11 s ≪ Δ; meccanismo NOMINATO
  resolve_class_autoload = funnel di 11 siti — magnitudine NON ripartita
  senza census ORM al probe s155 (stash ×2, 3e6b5008482c32d0).
5·gdc-count rc=0: per_classe=3,0 fisso=1 ESATTI ⇒ k_ORM=7180, ~636 chiamate,
  4,56M alloc ≈ 0,031 s ≪ 0,293 ⇒ **fetta NON pagante, declassata**.
6·Az.rev. S-154 recepite (§3 emendata; pin-phpr --braccio; gate a rischio
  pre-dichiarati) · T2/A2: raccomandazione SOSPENDERE (fetta borrow ≤0,17 s;
  rientro = prezzo in-contesto nuovo sopra soglia), ratifica utente · CI:
  corpus-FAIL d'ambiente sui 3 test backtrace (nei gate di record passano).

## ⭐ Lezioni (max 3)
- ⭐⭐ Le attese-conteggio su census per-NOME devono includere il PLUMBING
  dell'attribuzione (scope aperto prima di pop_keys): k=0 era mal posto, la
  cura era sana — controllo su builtin indipendente in un minuto.
- ⭐⭐ Un fit da 4 run (per_classe/fisso) UCCIDE una leva in coda a costo
  ~zero: gdc 3 alloc/classe ma ~636 chiamate ⇒ 0,03 s.
- ⭐ Una cura su un FUNNEL può battere l'attesa calcolata sul solo nome che
  l'ha motivata: attese per-nome ≠ attese per-funnel.
