# Verbale 5 — Lars Bak (alloc-rate, path caldi, disciplina statistica)
## Concilio WP-96 · oggetto: la coppia full di S-94.0

## VERDETTO — **RIGETTO le quattro letture, non la misura**

I raw di `pair94.out` sono buoni: coppia stessa-sera, conteggi identici,
guardia uploads, identità in banda. È tutto ciò che c'è di verdict-grade.
Le **quattro letture** — tre «MEGLIO» e un «REGRESSO» — sono **tutte
comparazioni con una citazione**, esattamente ciò che `pair94.out` §grade
dichiara di NON fare («le due gambe si confrontano fra loro, non con una
citazione»), e **tre di esse si ribaltano o si annullano con l'aritmetica
dei raw stessi**. Grado corretto della coppia: **VERDICT sui due rapporti
di stasera; SCREEN su ogni delta cross-sessione**.

## Refutazioni capitali

**R1 — il «REGRESSO» del media footprint è il DENOMINATORE, refutato al
byte.** phpr stasera = 1170785648 B = **1170,8 MB**. Le tre coppie che
FONDANO la banda: WP-63 1170,0 · WP-64 1186,9 · WP-65 1150,6 MB. phpr è
**dentro la banda, al centro**. È l'oracle a essersi mosso: 346,3 MB
contro 393,0 / 393,7 / 382,2 = **−11,9%**. E 3,381/2,979 = **1,135** =
esattamente 1/0,881: **l'intero «regresso» è il movimento dell'oracle**.
Con i denominatori storici: 1170,8/393,0 = **2,979**, in banda 2,9-3,2.
Peggio: GAP_TREND porta già il riquadro d'avvertimento di **riga WP-30**
(«il rapporto sale per rumore dell'oracle, non per una regressione phpr»)
e la **regola G3-Gregg** citata alla riga WP-62 per un oracle **−6,9%**:
«coppia singola, NON entra nel trend». S-94.0 ha commesso l'errore che la
sua stessa tabella documenta. Il backlog «Regresso del media footprint»
di §WP-95 **brucerebbe una sessione a inseguire un denominatore**.

**R2 — il «full CPU il più basso mai registrato» confronta due rapporti di
costruzione diversa.** GAP_TREND §Metodo 2: denominatore **congelato a
5:39 = 339 s**. Verifica: 971/339 = 2,86 ✓, 699/339 = 2,06 ✓. Stasera il
denominatore è **vivo, 447,84 s**. Applicando la ricetta della tabella al
numeratore di stasera: 838,59/339 = **2,474×**, cioè PEGGIO di 2,06-2,11.
Non affermo il segno: affermo che **1,873 è il primo punto di una serie
nuova, N=1, e non ha prior**. «Migliorato in modo netto» non è sostenuto.

**R3 — il peak full non è «SCESO»: è FLAT.** 1993459800 B = **1,993 GB**,
**dentro** la banda 1,98-2,03 GB. E quella banda ha provenienza ambigua
(le righe WP-60/62 danno peak 3,900 GB su runNN tree-user, WP-64 dà 2,035
GB): un riferimento senza workload, R e raw nominati non è un riferimento.

**R4 — VERDICT su R=1 di una MAX-statistic viola KS-BB-92-1.** Il peak è
un estremo, non una media: R=1 è un tiro da una coda, la statistica meno
mediabile che abbiamo. Fu la mia sedia a far declassare la min-statistic e
ripubblicare b_peak a mediana. WP-84 misurò 217,7/228,8/229,2 MiB (5%) sul
peak; WP-62 documenta spread serale phpr ±1,7-2,3% sulla CPU — e il media
CPU 2,639 vs 2,58 è **+2,3%**, cioè dentro quello spread: **flat**.

**R5 — il grado è una costante, non un calcolo.** `pair94.sh` fa
`echo "grade=VERDICT"`: stringa cablata, poi citata come output macchina.
Nessun R, nessuno spread, nessun drift entra in quella parola.

**Ordine oracle-prima**: due bias di segno opposto, **entrambi non
misurati** — phpr sempre secondo (page cache calda) e ultimo su 28 minuti
continui (01:20:29→01:48:45, deriva termica). L'ABBA esiste già in casa
(WP-86, purge). Non invalida stasera; invalida il *claim di precisione*.

**Concedo**: conteggi identici (762/1912/52; 30472/4558029) e i due
failure per NOME sono fedeltà genuina; rc non-giudice è corretto.

## Emendamenti

- **A-BB-67 DENOMINATORE VIVO**: la colonna full-suite di GAP_TREND è
  rifondata; il 5:39 congelato è **ritirato**, le righe storiche marcate
  «costruzione diversa». 1,873 apre serie nuova, N=1, nessun delta.
- **A-BB-68 SANDWICH-DRIFT**: ripetere la gamba più economica (media
  oracle, 36 s = 2% della campagna) **in coda**; pubblicare
  drift=|Δ|/media. Delta cross-sessione < drift ⇒ SCREEN.
- **A-BB-69 GRADO CALCOLATO**: lo script deriva il grado da R, spread e
  drift; VERDICT **vietato** a R=1 su peak. Mai un `echo` costante.
- **A-BB-70 NUMERATORE PUBBLICATO**: ogni riga di trend porta i **due
  assoluti**, il workload, R e il raw. Un rapporto senza denominatore non
  è falsificabile fra sessioni (R1 è la prova).
- **A-BB-71 G3 APPLICATO**: pair94 è coppia singola ⇒ **non aggiorna il
  trend**. Correggere MEASURE94, REPORT_GAP_94, §Le cifre e NEXT_SESSION:
  «riferimenti INVARIATI». Cancellare il backlog «Regresso media».
- **A-BB-72 ABBA sulla gamba media** (o,p,p,o): costo 108 s.

## Kill-switch

- **KS-BB-96-1**: claim cross-sessione il cui denominatore non è misurato
  nella stessa campagna **e con la stessa costruzione** del riferimento
  citato ⇒ **VOID**, rimosso dai documenti finché non ri-derivato.
- **KS-BB-96-2**: grado VERDICT su max-statistic con R=1 ⇒ **declassato a
  SCREEN d'ufficio**, dal gate cifre, senza discussione.
- **KS-BB-96-3**: se il drift di A-BB-68 supera il delta rivendicato, la
  campagna è **invalida per quella metrica** (non «indicativa»).
