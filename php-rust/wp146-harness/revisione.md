# Revisione S-146 — lente MISURA

## Reperto principale
**Il «FERMO» passa per 0,001 — un tick — e la banda che lo ha giudicato viene subito triplicata.** Verdetto r.45/47: coppie proprie 1,733–1,823 vs rif [1,765–1,788]+0,036 ⇒ limite alto 1,824; la coppia massima (leg1, 1,823) dista 0,001 dal FUORI BANDA, pari alla risoluzione dichiarata (criterio p.6, ~0,001 sul rapporto). E leg1 è gamba ELEVATA (ictx 144% med, r.14): il bordo del claim poggia sulla gamba più sospetta. Il canone S-140 p.3 impone replica al fuori-banda: a un tick dal trigger, nessuna replica. Peggio: la finestra ha spread 0,090 (r.48), 2,5× la banda di giudizio, con deriva discendente sistematica correlata al peak (1843→1773 MiB) — non stazionarietà, non rumore — e proprio quello 0,090 è promosso a banda canonica. L'anomalia diventa tolleranza: da S-147 un movimento WP <5% (0,090/1,78) sarà invisibile per costruzione. Il FERMO regge appena; la banda 0,090 è il danno vero.

## Reperti secondari
1. **Dimrmw: magnitudine non ripartita.** Il +3,00 ns/iter è un contrasto s142→s145 di binario intero; «FR1 unico cambio sul cammino» è dichiarato, non misurato, e la sessione stessa indizia il layout (+795 istr). Direzione confermata 5/5; il +3,00 va trattato §4: firmato, non ripartito tra leva e layout.
2. **Soglie del concilio su denominatore vecchio**: KS-146-1 (±0,7% ≈ 0,26–0,30 s) e il tetto 1,52/37,6 s poggiano sulla coppia ORM di S-139, mai rimisurata al pin s145 (REPORT_GAP, ultima riga).
3. **Giudice dimrmw NUOVO, mai usato prima**, con soglia satura dal rumore (1,33 = drop-1): margine 2,3× robusto, ma senza storicità dello strumento.

## Vagliate e respinte
- Gambe ELEVATE contate pulite: lettura pre-registrata (S-136 #2) applicata alla lettera — formalmente regge.
- Igiene dimrmw: sentinelle CLEAN nel `.out`, lock, sequenziale — incidente-15 soddisfatto.
- Rimozione inserzione nel harness copiato: manifest diff + collaudo copia — conforme §2.

## Azioni S-147
1. Replica coppia t2 @ s145 prima di ogni uso della banda nuova; a margine ≤1 tick il FERMO si dichiara «marginale, replicato».
2. Congelare la banda canonica a 0,036; 0,090 resta «spread di finestra da spiegare»; criterio pre-registrato per gli aggiornamenti di banda (mai da finestra singola anomala).
3. Test ordinato pre-registrato sulla deriva peak↔rapporto (soglie prima dei dati).
4. Istruttoria FR1: isolare leva vs layout (mutante a parità di layout, disasm bl-count) prima di ogni revert.
5. Rimisurare la coppia ORM al pin s145 prima del census (denominatori di KS-146-1).
