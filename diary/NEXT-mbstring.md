# Design pass — `mb_*` (mbstring), batch 1 (UTF-8 core)

> Generato con assistenza AI (Opus 4.8). Sbloccato dalla ricompilazione oracle
> con `--enable-mbstring` (vedi tooling-hardening). Oracle PHP 8.5.7, default
> `mb_internal_encoding() == "UTF-8"`.

## Filosofia / scoperta abilitante

mbstring ha ~80 funzioni; la maggioranza del valore reale è nelle funzioni
**stringa code-point su UTF-8**. Scoperta chiave dalla recon: il **case mapping
Unicode di Rust std** (`char::to_uppercase`/`to_lowercase`) **combacia con PHP**
sui casi non banali — `ß→SS`, `ı→I`, `İ→i̇` (i + U+0307, 2 cp), final-sigma
`ς→Σ`. E `str::chars()` conta i code point come `mb_strlen`. Quindi gran parte
del batch 1 è **std-backed**, zero tabelle esterne. Pattern builtin PURO
(ABI `fn(&[Zval], &mut Ctx)`, zero modifiche all'evaluator), come step 17/29.

## Decisioni (D-MB)

- **D-MB1 — encoding**: batch 1 supporta SOLO UTF-8 (+ alias `UTF-8`/`utf-8`/
  `UTF8`; `ASCII`/`US-ASCII` come sottoinsieme UTF-8). Un encoding diverso →
  `ValueError` "mb_X(): Argument #N ($encoding) must be a valid encoding, "Y"
  given" (messaggio oracle-esatto). **Scope-out** dichiarato: encoding non-UTF-8
  *validi* (Shift-JIS, EUC, …) li riportiamo come invalidi (divergenza
  documentata) finché non arriva `encoding_rs` in un batch successivo.
- **D-MB2 — code point unit**: si interpreta il `PhpStr` (byte) come UTF-8.
  Helper `cp_indices` che cammina i byte e per ogni scalare UTF-8 valido emette
  1 unità; ogni byte invalido = 1 unità (così `mb_strlen("a\xFF\xFEb")==4` come
  oracle). Le funzioni operano su indici di unità → byte. **Scope-out**: il
  *rendering* esatto dei byte invalidi (sostituzione) può divergere; il
  conteggio/offset è corretto. Input UTF-8 valido (il 99% del corpus) è esatto.
- **D-MB3 — case mapping**: deleghiamo a `char::to_uppercase/to_lowercase`
  (full Unicode). `mb_convert_case`: UPPER/LOWER/TITLE (TITLE = prima lettera di
  ogni "parola" maiuscola, resto minuscolo; boundary = transizione
  non-cased→cased, come PHP). `FOLD` → `char::to_lowercase` (approssimazione di
  case-folding; rivedere se il corpus lo richiede). `*_SIMPLE` (4-7) →
  mapping 1:1 senze espansione (scope-out iniziale, raro). Mode fuori range →
  `ValueError` "must be one of the MB_CASE_* constants".
- **D-MB4 — costanti**: aggiungere `MB_CASE_UPPER=0 / LOWER=1 / TITLE=2 /
  FOLD=3 / UPPER_SIMPLE=4 / LOWER_SIMPLE=5 / TITLE_SIMPLE=6 / FOLD_SIMPLE=7` a
  `resolve_constant` (lower.rs).
- **D-MB5 — modulo**: nuovo `php-builtins/src/mbstring.rs`, registrato in
  `lib.rs::registry()` via `add(b"mb_...", mbstring::...)`. Testato in
  `php-builtins/tests/builtins.rs` (registry COMPLETA).

## Scope batch 1 (funzioni) + sotto-step TDD

- **mb-1** — core: `mb_strlen`, `mb_substr` (start/len negativi, len omessa),
  `mb_str_split` (len default 1, vuoto→[]) + helper `cp_indices`.
- **mb-2** — case: `mb_strtoupper`, `mb_strtolower`, `mb_convert_case`
  (UPPER/LOWER/TITLE), `mb_ucfirst`, `mb_lcfirst` (8.4).
- **mb-3** — search: `mb_strpos`/`mb_stripos`/`mb_strrpos`/`mb_strripos`
  (offset code-point, not-found→false), `mb_strstr`/`mb_stristr`/`mb_strrchr`/
  `mb_strrichr`, `mb_substr_count`.
- **mb-4** — misc: `mb_ord` (vuoto→ValueError), `mb_chr` (fuori range→false),
  `mb_str_pad` (8.3), `mb_trim`/`mb_ltrim`/`mb_rtrim` (8.4), `mb_check_encoding`.

## Scope-out dichiarato (batch ≥2 o mai)

- `mb_convert_encoding` verso/da non-UTF-8 (serve `encoding_rs`).
- `mb_detect_encoding`/`mb_list_encodings`/`mb_encoding_aliases` (euristiche/metadati).
- `mb_strwidth`/`mb_strimwidth`/`mb_strcut` (tabelle East Asian width).
- `mb_ereg*`/`mb_split`/`mb_regex_*` (oniguruma — famiglia regex separata).
- `mb_convert_kana`, `mb_encode/decode_mimeheader`, `*_numericentity`,
  `mb_send_mail`, `mb_parse_str`, `mb_http_*`, `mb_language`,
  `mb_internal_encoding`/`mb_detect_order`/`mb_substitute_character` (stato/config;
  eventuale stub che ritorna "UTF-8").

## Note tooling

Oracle a mano via file `.php` temp (non `-r`, mangia escaping su multibyte).
Validare in `php-builtins/tests/builtins.rs::out()` (registry completa).
