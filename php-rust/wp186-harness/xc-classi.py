#!/usr/bin/env python3
# xc-classi.py — S-186 fetta 0: classifica un export `time-profile` di xctrace (Time Profiler, 1 ms)
# per immagine+simbolo SELF (primo frame del backtrace, solo campioni "Running") in classi pre-registrate
# (s186-criterio-fetta0.md p.3). stdlib soltanto. Generato con assistenza AI (Claude Fable 5).
#
# Uso: xc-classi.py in.xml out.classi.tsv out.top.tsv
#   in.xml = `xctrace export --input T --xpath '//trace-toc/run[@number="1"]/data/table[@schema="time-profile"]'`
#   out.classi.tsv = `classe<TAB>secondi` per TUTTE le classi + TOTAL (ns/1e9 dei pesi Running) + #rows/#running
#   out.top.tsv    = top 80 (binario, simbolo demanglato, classe, secondi)
# Demangle Rust v0 EURISTICO (dichiarato): tokenizzazione dei path v0 (identificatori a lunghezza prefissata,
# crate-hash `Cs…_`, backref `B…_`, disambiguatori `s…_` saltati), uniti con `::`; i generici restano in ordine
# di apparizione. Il tempo self di `run_loop` = dispatch + fast path inlined (nessun DWARF inline nel binario
# release): dichiarato nel criterio. `<deduplicated_symbol>` e frame senza nome finiscono in `other`.
import re
import sys
import xml.etree.ElementTree as ET

CLASSES = ["dispatch", "call_ret", "prop_ic", "str", "arr_hash", "obj_life", "alloc", "builtin",
           "compile", "libc", "kernel", "pavimento", "other"]

# Tabella ORDINATA (prima regola che morde vince), applicata a "<binario> <simbolo demanglato>".
# Lato oracle (php) e lato phpr convivono: le regole dell'oracle citano simboli C (prefisso `_`), quelle di phpr
# i path Rust demanglati. Le immagini di sistema vengono PRIMA (valgono per entrambi i lati).
RULES = [
    # --- outlining del compilatore (OUTLINED_FUNCTION_N, entrambi i lati): NON attribuibile -> other, quota dichiarata ---
    ("other",     re.compile(r" OUTLINED_FUNCTION_\d+")),
    # --- immagini di sistema ---
    ("pavimento", re.compile(r"^(dyld|libdyld\.dylib) ")),
    ("kernel",    re.compile(r"^libsystem_kernel\.dylib ")),
    ("alloc",     re.compile(r"^libsystem_malloc\.dylib ")),
    ("libc",      re.compile(r"^(libsystem_c\.dylib|libsystem_platform\.dylib|libc\+\+|libsystem_pthread\.dylib|libobjc\.A\.dylib|libdispatch\.dylib|libsystem_m\.dylib|libsystem_info\.dylib|libsystem_notify\.dylib|libsystem_trace\.dylib|CoreFoundation|Foundation|libswiftCore\.dylib|libxpc\.dylib|libiconv|libz\.|libcrypto|libssl|libicu|ICU) ")),
    ("pavimento", re.compile(r"^libsqlite3")),
    ("libc",      re.compile(r"^libc\+\+[^ ]* ")),
    # --- pavimento per simbolo (PDO/sqlite3/avvio/arresto), entrambi i lati ---
    ("pavimento", re.compile(r"(sqlite3|zm_startup|zm_shutdown|register_class_|zm_activate|zm_deactivate|zm_globals|zm_info|_?pdo_|php_pdo|rusqlite|^phpr main$|^phpr start$|^php _main$|^php start$|php_module_startup|php_module_shutdown|php_request_startup|php_request_shutdown|zend_startup|zend_shutdown|zend_deactivate|zend_activate|php_cli|sapi_|zend_register_|php_register_|module_startup|zend_post_startup|php_init_config|zend_startup_module|zend_destroy_modules|php_free_shutdown|zend_ini_)", re.I)),
    # --- allocatore (phpr: mimalloc `mi_*` + Rust alloc; oracle: Zend MM) ---
    ("alloc",     re.compile(r"(\bmi_[a-z_]+|\b_mi_[a-z_]+|^phpr _?malloc$|^phpr _?free$|realloc|__rust_alloc|__rust_dealloc|__rust_realloc|RawVec|finish_grow|grow_amortized|_emalloc|_efree|_erealloc|_safe_emalloc|zend_mm_|zend_malloc|zend_free|_ecalloc|_estrndup|_estrdup|_safe_erealloc|zend_mm)")),
    # --- compilazione del sorgente PHP (opcache off su entrambi i lati) ---
    ("compile",   re.compile(r"(zend_compile|zend_lex|lex_scan|zend_ast|zendparse|zend_emit|pass_two|zend_resolve|zend_stream|zend_language|opcache|zend_scanner|zend_build|zend_attr|zend_const_expr|zend_do_|zend_begin|zend_end_|zend_file_|op_array|zend_op\b|init_op|zend_add_literal|zend_string_init_interned|mago|php_parser|php_lexer|::parser::|::lexer::|tokeniz|::lower::|::hir::|::bytecode::|::compile|liveness|::syntax::|::ast::|Lowerer|Compiler|Parser|Lexer|^php (parse|get_token|yylex|scan)$)")),
    # --- oracle: handler Zend per opcode (ordine: call/ret, builtin, prop, str, arr, obj, resto=dispatch) ---
    ("call_ret",  re.compile(r"ZEND_(INIT_METHOD_CALL|INIT_FCALL|INIT_DYNAMIC_CALL|INIT_STATIC_METHOD_CALL|INIT_USER_CALL|INIT_PARENT_PROPERTY_HOOK_CALL|DO_FCALL|DO_UCALL|DO_FCALL_BY_NAME|RETURN|RECV|SEND_|CHECK_FUNC_ARG|CHECK_UNDEF_ARGS|FUNC_ARG|FUNC_GET_ARGS|FUNC_NUM_ARGS|VERIFY_RETURN|INIT_NS_FCALL|CALLABLE|YIELD|GENERATOR|NEW_FUNC|DECLARE_LAMBDA|VERIFY_NEVER|FRAMELESS_ICALL)")),
    ("builtin",   re.compile(r"ZEND_DO_ICALL")),
    ("prop_ic",   re.compile(r"ZEND_(FETCH_OBJ|ASSIGN_OBJ|FETCH_CLASS|FETCH_STATIC_PROP|ASSIGN_STATIC_PROP|PRE_INC_OBJ|POST_INC_OBJ|PRE_DEC_OBJ|POST_DEC_OBJ|ISSET_ISEMPTY_PROP|UNSET_OBJ|FETCH_CLASS_CONSTANT|FETCH_THIS|INIT_STATIC|ASSIGN_OBJ_OP|ASSIGN_OBJ_REF|PRE_INC_STATIC|POST_INC_STATIC|INSTANCEOF)")),
    ("str",       re.compile(r"ZEND_(CONCAT|FAST_CONCAT|ROPE|CAST|STRLEN|ECHO|ADD_CHAR|ADD_STRING|ADD_VAR)")),
    ("arr_hash",  re.compile(r"ZEND_(FETCH_DIM|ASSIGN_DIM|INIT_ARRAY|ADD_ARRAY_ELEMENT|ADD_ARRAY_UNPACK|FE_|IN_ARRAY|COUNT|ISSET_ISEMPTY_DIM|UNSET_DIM|FETCH_LIST|FETCH_GLOBALS|BIND_GLOBAL|ARRAY_KEY_EXISTS|ASSIGN_DIM_OP|UNPACK|SEPARATE)")),
    ("obj_life",  re.compile(r"ZEND_(NEW|CLONE|FREE|DECLARE_|DISCARD_EXCEPTION|HANDLE_EXCEPTION)")),
    ("dispatch",  re.compile(r"(ZEND_.*_HANDLER|^php _?execute_ex$|^php _?zend_execute|zend_vm_enter|ZEND_NULL_HANDLER|zend_jit|zend_vm_stack_push_call_frame)")),
    # --- oracle: funzioni C non-handler ---
    ("obj_life",  re.compile(r"(zend_objects|zend_object_std|gc_possible_root|zend_gc|rc_dtor_func|zval_ptr_dtor|zend_std_call_destructor|zend_object_dtor|zend_objects_store|destroy_op_array|zend_std_init|zend_value_dtor|_zval_dtor|zend_refcounted|zend_object_release|zend_std_get_gc|zend_std_free_obj|zend_weakref|zend_release|zend_iterator_dtor|gc_remove|gc_check|zend_object_alloc|zend_std_clone|zend_std_get_properties_for|zend_objects_clone)")),
    ("call_ret",  re.compile(r"(zend_call_function|zend_vm_stack|i_init_func|zend_execute_internal|zend_fetch_function|zend_std_get_method|zend_leave_helper|i_free_compiled|zend_vm_call|zend_call_|execute_data|zend_closure|zend_init_func|zend_get_closure|zend_is_callable|zend_fcall|zend_param_|zend_frameless|zend_std_get_static_method|zend_std_get_constructor|zend_verify_arg|zend_verify_|zend_missing_arg|zend_handle_undef|zend_vm_init|i_zend_|zend_init_dynamic_call|zend_init_method_call|zend_user_func|zend_generator)")),
    ("prop_ic",   re.compile(r"(zend_std_read_property|zend_std_write_property|zend_std_get_property|zend_get_property|property_info|zend_std_has_property|zend_std_unset_property|zend_hash_find_known_hash.*prop|zend_std_read_dimension|zend_std_get_properties|zend_std_compare|zend_std_cast|zend_std_get_class_name|zend_lookup_class|zend_fetch_class|zend_get_class|zend_std_get_debug|zend_update_class|zend_class_init|zend_get_constant|zend_fetch_dimension_address_inner_W.*obj|zend_std_|zend_property_|zend_declare_|zend_do_implement|zend_do_inherit|zend_build_properties|zend_object_properties)")),
    ("str",       re.compile(r"(zend_string|concat_function|smart_str|zend_strpprintf|zend_long_to_str|zend_ulong_to_str|_zend_string|php_sprintf|zend_spprintf|zend_vspprintf|zend_memn|zend_str|zend_binary|zend_interned|zend_dtoa|zend_gcvt|zend_strtod|xbuf_format|php_printf_to_smart|php_conv|php_formatted|zend_strval|zval_get_string|zend_make_printable|zend_concat|zend_ascii|zendi_smart_str|_zval_get_string|php_addslashes|php_str_|zend_is_identical|zend_rope|zend_flf_concat|zend_smart_str|string_cold|zend_accel_str)")),
    ("arr_hash",  re.compile(r"(zend_hash|_zend_hash|zend_array|zend_new_array|zend_inline_hash|zend_symtable|array_init|zend_sort|zend_insert_sort|zend_fetch_dimension_address|zend_find_array|php_array_|zend_array_dup|zend_array_destroy|zend_vm_dim|zend_assign_to_variable|zend_assign_dim|zval_get_long|zendi_|zend_compare|compare_function|zend_is_true|zend_is_numeric|_is_numeric|zend_add_|add_function|sub_function|mul_function|div_function|mod_function|zend_binary_op|zend_operators|zval_update_constant|zend_atol|zend_long|zend_fetch_dim|zend_assign_op|increment_function|decrement_function|is_equal_function|is_smaller|zend_hash_packed|zend_array_count|zend_fetch_list|zend_wrong_string_offset|zend_fe_|zend_iterator|zend_object_iterator|zend_user_it)")),
    ("builtin",   re.compile(r"(^php _?(zif_|zim_|zm_|php_|spl_|date_|pcre|mbfl|json|dom_|lxb|lexbor|ir_|mysqlnd|ini_|session|reflection|timelib|zend_fetch_resource|zend_parse_|zend_parse_arg|zend_wrong_param|zend_list|zend_register_resource|zend_throw|zend_error|zend_exception|zend_class_implements|zend_implement|zend_ce_|php_output|php_var|php_implode|php_explode|php_trim|php_strtr|php_ucfirst|php_basename|php_math|php_round|php_random|php_mt_|_php_math|zend_is_iterable|zend_get_executed|zend_get_callable|zend_make_callable|array_|str_|zend_wrong_|zend_argument|zend_type_error|zend_value_error|zend_strtol|zend_dval|zend_ini_|php_info|zend_get_type|zend_zval_type|zend_get_parameters|_call_user_function|zend_eval|zend_include|zend_exceptions|zend_throw_exception|zend_clear_exception|zend_rethrow|zend_dispatch_try|cleanup_unfinished|zend_object_std_dtor_props)|^php <deduplicated_symbol>)")),
    # --- phpr: crate php-builtins / host builtins (prima dei generici, per NOME di modulo); il MUTANTE S-186
    #     (`builtin_spin_mutant_s186`, spin fuori linea in CallBuiltin) deve cadere QUI (criterio p.6) ---
    ("builtin",   re.compile(r"(builtin_spin_mutant_s186|builtin|php_builtins::|php_cli::builtins|::builtins::|CallBuiltin|hostcall|HostBuiltin|host_builtin|call_host|::ext::|::stdlib::|::spl::|::pcre::|regex::|::json::|serde|var_dump|var_export|sprintf|printf|format_fn|::registry|Registry|undefined_builtin|Builtin)")),
    # --- phpr: ciclo di vita oggetto (alloc, nota, sweep, dtor, drop-glue di Object) ---
    ("obj_life",  re.compile(r"(gc_|sweep|destruct|Destruct|gc_track|gc_note|new_object|ObjectStore|objects::|::object::|Object\b|Object::|\bdtor|cycle|::gc::|release_child|unbuffer|collect|drop_glue.*(Object|RefCell|object)|drop_in_place.*(Object|RefCell|object)|Weak|WeakHandle|__destruct|obj_free|instantiate|new_instance|Op::New|op_new|::new_obj)")),
    # --- phpr: call/ret (fast vs generale), frame ---
    ("call_ret",  re.compile(r"(methodcall|method_call|ret_slow|call_user|::calls::|decay_arg|bind_arg|Frame|with_buffers|push_frame|pop_frame|invoke|closure|Closure|simple_call|CallFast|call_fast|callfast|call_fn|call_function|enter_frame|frames|CallUser|UserFn|user_fn|fncall|funcall|callable|Callable|bound_this|bind_|\bleave\b|leave_|\bret_|_ret\b|Ret\b|return_|args_|arg_pack|FrameExt|dyn_vars|ArgPlace|verify_arg|coerce_arg|type_hint|TypeHint)")),
    # --- phpr: proprietà / inline cache / classi ---
    ("prop_ic",   re.compile(r"(prop|Prop|ic_|\bic\b|IC\b|property|Property|fetch_obj|assign_obj|slot_of|class_of|lookup_class|ClassId|class_id|::class::|classes|Class\b|vtable|method_of|find_method|resolve_method|instanceof|static_prop|const_fetch|ClassConst|deref_object|overload_receiver|oop::|::oop)")),
    # --- phpr: stringhe ---
    ("str",       re.compile(r"(zstr|PhpStr|concat|interp|from_i64|from_f64|to_zstr|to_string|ToString|str_|String|fmt::|Display|write_fmt|format|utf8|bytes::|memchr|\bstr\b|rope|Rope|interned|Intern|echo|Echo|output|Output|out_buf|write_all|stdout|heredoc)")),
    # --- phpr: array / hash ---
    ("arr_hash",  re.compile(r"(zarr|PhpArray|ZArr|::array::|::arrays::|Array\b|hash|Hash|OrderedMap|hashbrown|Hashbrown|IndexMap|\bdim\b|dim_|_dim|table|Table|bucket|Bucket|packed|Packed|key_|KeyKind|ArrKey|entry|Entry|foreach|Foreach|\blist\b|unpack|in_array|coerce_key|displaced|array_)")),
    # --- phpr: dispatch (run_loop + aritmetica/confronti/conversioni/traffico valori inlined o fuori linea) ---
    ("libc",      re.compile(r"^[^ ]+\.dylib ")),  # ogni altra libreria di sistema (dichiarato)
    ("dispatch",  re.compile(r"(run_loop|dispatch|exec_op|reg_load|reg_store|slot|Op::|::run::|vm::run|Vm::|cmp_j|CmpJmp|incdec|incd|binar|Binary|binop|BinOp|apply_binop|::ops::|increment|decrement|convert::|to_bool|to_i64|to_f64|is_true|truthy|coerce|cast|Cast|arith|Arith|numeric|Numeric|compare|Compare|\bcmp\b|make_cell|Zval|zval|Value|value|clone|Clone|drop_glue|drop_in_place|push_mut|Vec|vec::|Option|option::|Box|boxed|core::|std::|alloc::|mem::|ptr::|cmp::|num::|iter::|cell::|rc::|hint|intrinsics|DYLD-STUB|jump|Jump|branch|cond|Cond|pc_|instr|opcode|Opcode|handler|exec_|execute|eval_|interp_|step|assign|Assign|load_|store_|move_|copy_|Const|const_|literal|Literal|temp|Temp|stack|Stack|operand|unwind|Unwind|catch|Catch|try_|exception|Exception|throw|Throw|error|Error|warn|Warn|notice|Notice|diag|Diag|logging|Log|debug|Debug|trace|Trace|assert|panic|Panic|php_runtime::vm|php_types::)")),
]

DEDUP = "<deduplicated_symbol>"


def _skip_base62(s, i):
    """skip a base62 run ending with '_' (crate hash, backref, disambiguator); returns index after '_'"""
    n = len(s)
    while i < n and s[i] != "_":
        i += 1
    return i + 1


def demangle(name):
    """Rust v0 (`_R…`) / legacy (`_ZN…E`) heuristic demangle: identifiers in order of appearance."""
    if name.startswith("_R"):
        s = name[2:]
        parts = []
        inst = []            # indici dei nomi di crate incontrati DOPO il primo path (crate di istanziazione dei generici)
        crate_next = False
        i, n = 0, len(s)
        while i < n:
            ch = s[i]
            if ch.isdigit():
                m = re.match(r"\d+", s[i:])
                ln = int(m.group(0))
                j = i + len(m.group(0))
                if j < n and s[j] == "_":
                    j += 1  # identifier starting with a digit/underscore is prefixed by `_`
                ident = s[j:j + ln]
                if ident and ln > 0:
                    if crate_next and parts:
                        inst.append(len(parts))
                    parts.append(ident)
                crate_next = False
                i = j + ln
            elif ch == "C" and i + 1 < n and s[i + 1] == "s":
                crate_next = True
                i = _skip_base62(s, i + 2)            # crate hash `Cs<base62>_`, then the crate name identifier follows
            elif ch == "B" or ch == "s":
                i = _skip_base62(s, i + 1)            # backref `B<base62>_` / disambiguator `s<base62>_`
            elif ch == "u":
                # punycode identifier `u<len><chars>`: treat like a plain identifier
                m = re.match(r"\d+", s[i + 1:])
                if m:
                    ln = int(m.group(0)); j = i + 1 + len(m.group(0))
                    if j < n and s[j] == "_":
                        j += 1
                    parts.append(s[j:j + ln]); i = j + ln
                else:
                    i += 1
            else:
                i += 1  # tags: N<ns> M X Y I E L R Q K A S P O F T D G etc.
        # il crate di ISTANZIAZIONE in coda (`drop_glue<Zval>` istanziato in php_builtins) non è un path: si toglie
        if inst and inst[-1] == len(parts) - 1 and len(parts) > 1:
            parts.pop()
        parts = [p for p in parts if p]
        if parts:
            return "::".join(parts)
        return name
    if name.startswith("_ZN") or name.startswith("__ZN"):
        s = name[name.index("N") + 1:]
        parts = []
        i, n = 0, len(s)
        while i < n:
            m = re.match(r"\d+", s[i:])
            if not m:
                break
            ln = int(m.group(0)); j = i + len(m.group(0))
            parts.append(s[j:j + ln]); i = j + ln
        parts = [p for p in parts if not re.fullmatch(r"h[0-9a-f]{16}", p)]
        if parts:
            return "::".join(parts)
    return name


def classify(binary, sym):
    sym = re.sub(r"\.(cold|llvm|constprop|part|isra)\.?\d*$", "", sym)
    if sym in ("?", "", DEDUP) or sym.startswith("<"):
        # frame senza nome / simbolo deduplicato: attribuzione per IMMAGINE soltanto (regole di sistema), altrimenti other
        text = f"{binary} "
        for cls, rx in RULES[1:5]:
            if rx.search(text):
                return cls
        return "other"
    text = f"{binary} {sym}"
    for cls, rx in RULES:
        if rx.search(text):
            return cls
    return "other"


def main():
    if len(sys.argv) != 4:
        sys.exit("uso: xc-classi.py in.xml out.classi.tsv out.top.tsv")
    src, out_cls, out_top = sys.argv[1:4]
    root = ET.parse(src).getroot()
    ids = {}
    for el in root.iter():
        i = el.get("id")
        if i is not None:
            ids[i] = el

    def resolve(el):
        if el is None:
            return None
        r = el.get("ref")
        return ids.get(r) if r is not None else el

    def frame_info(fr):
        fr = resolve(fr)
        if fr is None:
            return ("?", "?")
        name = fr.get("name") or "?"
        b = resolve(fr.find("binary"))
        bname = (b.get("name") if b is not None else None) or "?"
        return (bname, name)

    agg, total, nrows, nrun = {}, 0, 0, 0
    for row in root.iter("row"):
        nrows += 1
        st = resolve(row.find("thread-state"))
        if st is None or (st.text or "").strip() != "Running":
            continue
        w = resolve(row.find("weight"))
        try:
            wt = int((w.text or "0").strip()) if w is not None else 0
        except ValueError:
            wt = 0
        tb = resolve(row.find("tagged-backtrace"))
        bt = resolve(tb.find("backtrace")) if tb is not None else resolve(row.find("backtrace"))
        frames = list(bt.findall("frame")) if bt is not None else []
        key = frame_info(frames[0]) if frames else ("?", "?")
        nrun += 1
        total += wt
        agg[key] = agg.get(key, 0) + wt

    per_class = {c: 0 for c in CLASSES}
    top = []
    for (b, s), wt in agg.items():
        d = demangle(s)
        c = classify(b, d)
        per_class[c] += wt
        top.append((wt, b, d, c))
    top.sort(reverse=True)
    with open(out_cls, "w") as f:
        for c in CLASSES:
            f.write(f"{c}\t{per_class[c] / 1e9:.3f}\n")
        f.write(f"TOTAL\t{total / 1e9:.3f}\n#rows\t{nrows}\n#running\t{nrun}\n")
    with open(out_top, "w") as f:
        f.write("binary\tsymbol\tclass\tseconds\n")
        for wt, b, d, c in top[:80]:
            f.write(f"{b}\t{d}\t{c}\t{wt / 1e9:.3f}\n")
    print(f"rows={nrows} running={nrun} total_s={total / 1e9:.3f} " +
          " ".join(f"{c}={per_class[c] / 1e9:.3f}" for c in CLASSES))


if __name__ == "__main__":
    main()
