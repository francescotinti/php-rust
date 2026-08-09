# Revisione S-120 — lente SEMANTICA (revisore singolo)

## Verdetti
1. Invarianza semantica L-RE1: **REGGE**. 2. Promozione pin s120: **REGGE** (con riserve di metodo). 3. Letture coppia WP: **RIDIMENSIONATO**.

## Istruttoria sulla patch (riga per riga, sorgente HEAD)
- `caps_from_locations`: gli offset del crate regex cadono sempre su confini di char del subject decodificato ⇒ `text[start..end]` non panica; byte identici a `m.as_str()`; `latin1_fix` opera dopo, invariato.
- `has_named_groups` coerente con `capture_names` in entrambe le direzioni: fast-path preso solo quando ogni variante darebbe tutti-None (stesso filtro del prefisso sintetico; per costruzione di demix i nomi sintetici coesistono sempre con nomi veri ⇒ slow-path). Nome utente che inizia col prefisso sintetico: già nascosto PRIMA della patch, divergenza pre-esistente. Remap/Anchored: delega conservativa corretta (falso ⇒ inner senza nomi ⇒ proiezione tutta-None).
- `captures_array` owned: `groups.next().flatten()` allineato indice-per-indice col vecchio `caps.get(i)`; `limit` mai oltre len in entrambi i rami; `capture_value_owned` replica `capture_value` ramo per ramo (NULL/offset inclusi); il clone per la chiave nominata c'era anche prima.
- RefCell: il borrow vive solo dentro captures/captures_read, senza codice utente in mezzo; replace_callback materializza i match PRIMA del callback e `captures_iter` non tocca lo scratch ⇒ nessun doppio borrow trovato; !Sync sancito dal compilatore.
- Borrow pattern/subject in ho_preg_match: ZStr owned, nessuna mutazione a valle prima dell'uso.

## Ciò che il claim tace
- La copertura dei gate sulle piste calde (nomi, NULL, offset, latin1, `(?|)`) è INDIRETTA: nessuna fixture nominata le esercita; l'invarianza regge sulla lettura qui sopra + la parità WP ×4 (che preg lo esercita davvero).
- Due emende in corsa, dichiarate ma entrambe favorevoli: (a) census v1→v2 — la baseline arr 4,02 contro 245,01 rivela denominatori mescolati nella tabella S-119 ⇒ la classifica della categoria arr è sospetta; (b) smoke: prop concorde −5,00/−1,67 era ≤ −1,00 ⇒ il punto 7 pre-registrato imponeva early-stop; si è proceduto citando la banda. Il full ha dato ragione, ma un criterio derogato in corsa non è più pre-registrato.
- Coppia WP: «dimostra»/«sciolti» con N=2 per modo sovra-afferma: direzione plausibile, magnitudine L-RE1 su WP non ripartita (tra-sere e tra-pin, ammesso nel GAP). Deviazione dal piano sostanziale ma giustificata.

## Azioni S-121
1. Fixture preg bilaterale congelata per NOME: nomi (duplicati e prefisso sintetico inclusi), NULL, offset, subject latin1, `(?|)`.
2. Rifare la colonna arr della classifica a denominatore dichiarato prima di scegliere la leva arr.
3. Riconciliare per iscritto smoke vs banda: precedenza decisa PRIMA.
4. Gamba ABAB s119↔s120 stessa sera per ripartire L-RE1 su WP.
5. Grado pieno del server s120 (debito dichiarato).
