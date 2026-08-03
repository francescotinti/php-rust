# Team CIFRE — Concilio WP-93 (fase 2)

Relatore: team-cifre. Sedie: 3 Klabnik, 4 Hejlsberg, 9 Gregg. I verbali individuali
(`verbale-3-klabnik.md`, `verbale-4-hejlsberg.md`, `verbale-9-gregg.md`) restano la
fonte VINCOLANTE; questo file li colloca.

## CONVERGENZE

1. **La delibera applicata a metà è il pattern comune** (3, 4). Klabnik: il PASS
   verdict-grade vive in un giudice spoofabile (F6) e ADVISORY-PASS esce 0; Hejlsberg:
   grammar v2 morde solo nel ramo `--same-rev`, head-check assente su REFUSE/trap,
   campaign v2 senza dente. Stessa malattia: enforcement su UN percorso, convenzione
   sugli altri. Cura comune: il dente morde su TUTTI i terminali/percorsi (A-SK-78/79,
   A-AH61/62/63).
2. **Le cifre non-macchina sono AUTORITÀ, non misure, e vanno marchiate** (3, 4, 9).
   Klabnik A-SK-75 (%ALLOW 2.8/46.25 revocate o provenienzate) e A-SK-77 (budget fuori
   corpus); Hejlsberg A-AH64 (max-FAIL = storia, mai verità); Gregg A-BG64/KS-BG-93-2
   (il grado si legge sulla riga, mai ereditato dal silenzio — b_peak mediana muta).
3. **L'identità di ciò che firma va dimostrata, non dichiarata** (3, 9). A-SK-78
   (self-tether `hash-object($0)` == blob HEAD) e A-BG65 (il log bite-test dichiara
   «commit figlio di head», non il proprio sha impossibile). Corollario condiviso di
   A-BG53/WP-90.
4. **Perimetro per COMPLEMENTO, non per allowlist** (3, 4). A-SK-80 (ogni .md
   committato esige riga manifest, verbali e doc GitHub inclusi: 153 cifre oggi fuori)
   e il buco `battery-9[1-9]*` che non copre le 3 cifre (4, Q2b).
5. **Fail-closed sulla tokenizzazione/parsing** (3, 4): A-SK-76 (run+lettera si
   rifiuta, non si tronca) converge con i residui minori di Hejlsberg (case
   substring-match su writer=, sha256 non ancorato).
6. **I bite-test con controllo negativo sono il criterio** (3, 9): i 6 forge di
   Klabnik hanno tutti il negativo che FALLISCE; Gregg conferma la lezione ⭐⭐
   «il bite-test decide» con evento-prova. KS-SK-93-2: i sei forge diventano T17-T22.

## CONFLITTI

- **PASS di Gregg vs «gate NON verdict-grade» di Klabnik: COMPATIBILI, perimetri
  disgiunti.** Gregg giudica il CONTENUTO: le cifre S-91.0 pubblicate coincidono con
  la macchina al byte, censimento manuale per NOME riga-per-riga. Klabnik giudica il
  CHECKER: la sua capacità di discriminare un forger futuro. Un doc onesto passa un
  gate bucato — le due cose coesistono. Il punto di frizione reale: il sigillo
  automatico che dovrebbe garantire *in futuro* ciò che Gregg ha verificato *a mano*
  è spoofabile (F6, judge_sha di un omonimo). Quindi il PASS di Gregg vale come fatto
  storico verificato dalla sedia 9, NON come garanzia riproducibile dal gate. Nessuna
  sedia dissente da questa collocazione.
- **Nessun conflitto 4↔9**: il max-FAIL di Hejlsberg è mechanism-level e verificato
  NON-live (m89 g3 PASS, m90 g2 PASS dai ledger) — coerente col censimento di Gregg.
- Dissensi da registrare: NESSUNO sostanziale; solo differenza di grado (3 e 4:
  refutazioni capitali; 9: nit di trascrizione).

## PRIORITÀ PROPOSTE per S-92.0

1. **A-SK-78 + bite-test F6 (giudice-copia)** — SÌ, è la falla di autorità massima:
   forgia lo strato di IDENTITÀ su cui poggiano tutte le altre firme (judge_sha nei
   ledger di Hejlsberg, i sigilli che renderebbero macchinale il censimento di Gregg).
   Blocca ogni PASS verdict-grade futuro (KS-SK-93-1). Primo atto.
2. **A-SK-79 + A-AH61 (grado nel codice d'uscita; grammar v2 su ENTRAMBI i percorsi
   di consumazione)** — blocca ogni consumazione di battery 9x (KS-AH-93-1).
3. **A-AH63 (dente campaign v2 pre-nascita) + A-AH64/A-SK-77 (igiene corpus:
   max-PASS-only, budget fuori)** — da committare PRIMA di qualunque m91; blocca la
   campagna.
4. **A-AH62 (head-check su REFUSE/trap) + A-SK-74/75/76/80/81 + T17-T22 permanenti**
   — hardening gate; 2.8/46.25 fuori entro S-92.0 (KS-SK-93-3).
5. **A-BG62/63/64/65 (arrotondamento dichiarato, segno, token di grado, log figlio)**
   — piccoli, non bloccanti, chiudibili in coda.

## S-91.0: CONSUMABILE?

**SÌ, con condizioni — nessuna ri-giudicatura.** Motivazione a tre gambe: (a) i buchi
sono *would-have-allowed* (mechanism-level), nessuna evidenza di forge avvenuto;
(b) il censimento indipendente della sedia 9 copre al byte il perimetro cifre
(seconda gamba oltre il judge_sha spoofabile); (c) il max-FAIL è verificato non-live.
Condizioni: (1) 2.8 e 46.25 non citabili finché non ri-provenienzate (A-SK-75);
(2) b_peak(mediana) riceve token di grado esplicito ovunque (A-BG64), testata resta
DA RIFARE; (3) ogni consumazione FUTURA di battery/campagne 9x avviene sotto i denti
nuovi — un PASS S-91.0 già consumato resta valido, nessun nuovo PASS verdict-grade
prima degli emendamenti di priorità 1-2.
