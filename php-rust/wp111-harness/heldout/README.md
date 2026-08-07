# Giudici HELD-OUT — congelati S-111 PRIMA della progettazione della leva

**Perché** (cautela Codex 20260807, recepita in NEXT_SESSION §S-111): i sei
micro sono regolarissimi (tipi stabili, branch prevedibili, cache calde); una
leva di dispatch progettata su di essi rischia l'interprete benchmark-specifico.
Questi tre giudici stanno FUORI dal ciclo di progettazione.

**Protocollo di congelamento** (dichiarato nell'atto, REGOLE §2):
- scritti e committati PRIMA di ogni istruttoria sulla forma della leva;
- collaudo alla nascita = SOLO parità d'output byte-per-byte oracle↔phpr
  (nessun cronometro, nessun rapporto letto o registrato);
- calibrazione di N sul SOLO oracle (bersaglio ~1–2 s netti);
- i RAPPORTI si misurano e si leggono SOLO a leva conclusa, con
  `run-heldout.sh` (stessa ricetta di run-micro.sh: pavimenti per-binario,
  mediana su R, N emesso dal sorgente);
- i file NON si modificano durante l'arco della leva (né mai, senza
  dichiararlo: un giudice emendato è un giudice nuovo).

**I tre giudici** (dalla lista Codex, in ordine):
1. `poly.php` — tipi alternati sugli operandi + dispatch megamorfico (4 classi
   in rotazione): branch di tipo imprevedibili, il caso che i micro non hanno.
2. `err.php` — error-path e coercizioni: miss di chiave frequenti (`??`),
   warning soppressi con `@` (BEGIN/END_SILENCE), eccezioni lanciate e prese
   (intdiv/0 raro + RuntimeException ~1/1000): lo slow-path diagnostico.
3. `wploop.php` — hot-loop estratto per forma da WordPress: tabella filtri
   con closure (`apply_filters`), sanitize_key (strtolower+preg_replace),
   escape (str_replace multiplo), interpolazione, in_array strict.

**Tare dichiarate (revisore S-111, az.4)**: `poly.php` ha pattern di dispatch
di PERIODO 16 — polimorfismo reale, ma un predittore moderno lo impara: non
spacciarlo per «branch imprevedibili». La parità d'output di `err.php` NON
certifica la diagnostica soppressa dal `@` (proprio lì il revisore ha trovato
la divergenza §3.17: riga sbagliata del warning non-numeric). Prima della
prossima lettura comparativa va PRE-registrata una soglia held-out (az.5).

**Verdetto della leva sui giudici**: la leva threaded-dispatch è un
DISCRIMINATORE tra famiglie di dispatch; se migliora i micro ma peggiora o
lascia fermi questi tre, la famiglia NON generalizza e il verdetto lo dice.
