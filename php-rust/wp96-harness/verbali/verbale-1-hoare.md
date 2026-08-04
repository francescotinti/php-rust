# Verbale 1 — Tony Hoare (Concilio WP-96, giudizio su S-94.0)

**VERDETTO: CON EMENDAMENTI**, con **opposizione nominata a due voci**: la
riga WP-94 di `GAP_TREND.md` e il «criterio 5 SODDISFATTO».

## Refutazioni (sostanziali)

**R1 — I tre giudizi della tabella sono artefatti del DENOMINATORE.** Il pin
phpr è INVARIATO, quindi nessuna delle tre letture può parlare di phpr.
Le cifre committate lo dicono da sole:
- *media footprint «REGRESSO»*: il numeratore phpr 1170785648 B (1170,8 MB)
  è **piatto** sullo storico (WP-63 1170,6 · WP-65 1150,6 · WP-64 1186,9);
  è l'**oracle** a essere sceso a 346,3 MB da 382,2-393,7 (−9,6…−12%).
  Col denominatore storico il rapporto è 2,97-3,06 = **dentro la banda**.
  GAP_TREND contiene già due precedenti identici (⚠️ WP-30; G3 su WP-62).
- *full CPU «il più basso mai registrato»*: la banda 2,06-2,11× divide per
  un oracle **CITATO** (5:39 = 339 s); S-94.0 divide per un oracle
  **misurato** (447,84 s, +32%). Rapporti con denominatori diversi non si
  ordinano. Col metodo storico stanotte darebbe **838,59/339 = 2,47×**, cioè
  un REGRESSO. Il numeratore è pure cambiato di definizione (master-CPU
  user dal tail `.rss` → tree user+sys dell'albero).
- *full peak «MEGLIO»*: 1993459800 B = **1,993 GB decimali**, cioè **dentro**
  la banda 1,98-2,03 GB; sembra migliore solo perché convertito in MiB
  (1901,11) mentre la banda non dichiara la sua unità.

**R2 — La coppia di S-94.0 NON è il giudice della leva.** NEXT_SESSION §WP-95
la eleva a «prima» della leva per-file. La regola vincolante del progetto è
l'opposta (WP-65: *la coppia build-ADIACENTE è l'unico giudice del costo*;
deriva inter-giornata osservata fino a 2,6%, WP-55). Una baseline di
un'altra sera è un riferimento di trend, mai il «prima» di una leva.

**R3 — battery61 non falsifica.** La lezione 4 della sessione («il probe deve
provare che l'operazione RIESCE») è rimasta in prosa: nel giudice non c'è
**nessun predicato positivo**. Login fallito su entrambi i lati ⇒ due pagine
di login identiche ⇒ `rc=0`. Inoltre `norm()` è un normalizzatore
**generico** (`[0-9a-f]{10}\b`) malgrado il commento: cancella qualunque
token da 10 hex e, per via del `\b`, **mutila la coda degli md5** (due md5
che differiscono solo negli ultimi 10 caratteri diventano identici). Infine
le due gambe girano **senza reset DB fra loro** (a differenza di pair94):
il protocollo è asimmetrico per costruzione.

**R4 — Un hash di binario non è un'identità.** Il «pin che non torna» è posto
come dilemma a due; manca la terza ipotesi (stesso albero, toolchain /
lockfile / feature / RUSTFLAGS / target-dir diversi), e il rebuild ripetuto
proposto non la separa.

## Emendamenti

- **A-TH-76** «Nessun rapporto senza denominatore omogeneo»: la riga WP-94 di
  GAP_TREND si riscrive senza MEGLIO/REGRESSO/«mai registrato»; le tre
  letture si declassano a NON-COMPARABILI e si nominano numeratore e
  denominatore di ogni banda storica prima di qualsiasi ordinamento.
- **A-TH-77** «Un identificatore di metrica denota UNA definizione»:
  `full_master_cpu` è tree user+sys, non master-user; media è user-only.
  Rinominare entrambe e dichiarare l'unità (GB vs GiB) in ogni banda.
- **A-TH-78** «Il "prima" della leva è la sua coppia adiacente»: S-95.0
  misura old **e** new la stessa sera, stesso protocollo; la coppia S-94.0
  resta baseline di trend.
- **A-TH-79** «Un gate d'accettazione porta un predicato positivo»:
  battery61 pretende `Location: …/wp-admin/`, cookie `wordpress_logged_in_`
  e un marcatore admin-only nel body, su **entrambi** i lati.
- **A-TH-80** «Normalizzare per NOME»: nonce ancorato all'attributo, conteggio
  delle sostituzioni uguale sui due lati; niente classi hex libere.
- **A-TH-81** «PATH è ambiente scelto dal chiamante»: la cura A1 chiude
  BASH_SOURCE/symlink/BASH_ENV ma lascia `PATH=…:/opt/homebrew/bin:$PATH`
  (dir scrivibile dallo stesso utente) **fuori** da `GATE_SANE`. PATH fisso
  senza coda del chiamante e dentro il predicato. (IFS: congetturato canale,
  **refutato a macchina** — bash lo reimposta all'avvio.)
- **A-TH-82** «Un canale di misura muto si dichiara»: in `huge_note` il
  `try_with` in Err salta in silenzio; contatore atomico degli scarti nel
  banner. Per il probe v2 il ledger indicizza per **puntatore** (nota
  spostata dopo l'alloc interna), mai per taglia.

## Kill-switch

- **KS-TH-96-1**: se una lettura di trend confronta rapporti con denominatori
  di regime diverso ⇒ la riga è VOID.
- **KS-TH-96-2**: se la leva è giudicata contro una coppia di un'altra sera
  ⇒ verdetto VOID, ripetere in coppia adiacente.
- **KS-TH-96-3**: se battery61 in ordine invertito (phpr-first, DB resettato)
  non replica il verdetto ⇒ criterio 5 torna APERTO.
- **KS-TH-96-4**: se un forge che sostituisce un tool risolto fuori da
  `/usr/bin:/bin:/usr/sbin` ottiene `PASS --all` rc=0 ⇒ A-SK-91 riaperta.
