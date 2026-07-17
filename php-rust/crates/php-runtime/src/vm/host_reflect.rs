//! Reflection host builtins (ho_reflect_*). Split from vm/host.rs.

use super::*;

impl<'m> super::Vm<'m> {
    /// `__reflect_class_constants($class, $filter = null)`: every class constant
    /// visible on `$class` as a `name => value` array — backs
    /// `ReflectionClass::getConstants`. `$filter` is a visibility bitmask
    /// (IS_PUBLIC=1 / IS_PROTECTED=2 / IS_PRIVATE=4); `null` returns all. Values
    /// come from running each declaring class's value thunk. (Enum cases are not
    /// yet modelled.)
    pub(super) fn ho_reflect_class_constants(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let mut arr = php_types::PhpArray::new();
        let Some(&start) = self.class_index.get(&key) else {
            return Ok(Zval::Array(Rc::new(arr)));
        };
        let filter: Option<i64> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => Some(n),
            _ => None,
        };
        for (name, decl, idx) in self.collect_class_consts(start) {
            if let Some(bits) = filter {
                let vbit = match self.classes[decl].consts[idx].visibility {
                    crate::hir::Visibility::Public => 1,
                    crate::hir::Visibility::Protected => 2,
                    crate::hir::Visibility::Private => 4,
                };
                if bits & vbit == 0 {
                    continue;
                }
            }
            let thunk: &'m Func = &self.classes[decl].consts[idx].func;
            let v = self.run_value_thunk(thunk, Some(decl))?;
            arr.insert(Key::Str(PhpStr::new(name)), v);
        }
        // Enum cases are reported as (public) constants, value = the case singleton.
        if filter.map_or(true, |bits| bits & 1 != 0) {
            let n = self.classes[start].enum_cases.len();
            for i in 0..n {
                let name = self.classes[start].enum_cases[i].name.to_vec();
                let inst = Zval::Object(self.enum_case(start, i as u32));
                arr.insert(Key::Str(PhpStr::new(name)), inst);
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_class_const_names($class)`: the names of every class constant
    /// visible on `$class`, most-derived first — backs
    /// `ReflectionClass::getReflectionConstants` (which wraps each in a
    /// `ReflectionClassConstant`).
    pub(super) fn ho_reflect_class_const_names(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let mut arr = php_types::PhpArray::new();
        if let Some(&start) = self.class_index.get(&key) {
            for (name, _, _) in self.collect_class_consts(start) {
                let _ = arr.append(Zval::Str(PhpStr::new(name)));
            }
            // Enum cases are reported as constants too, after the real ones.
            for c in &self.classes[start].enum_cases {
                let _ = arr.append(Zval::Str(PhpStr::new(c.name.to_vec())));
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_class_const_info($class, $name)`: descriptor array for one class
    /// constant (`value`, `declaringClass`, `visibility`, `final`, `enumCase`), or
    /// `false` if undeclared — backs the `ReflectionClassConstant` accessors. The
    /// value is produced by the declaring class's thunk.
    pub(super) fn ho_reflect_class_const_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let name = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let Some((decl, idx)) = find_const_runtime(&self.classes, cid, &name) else {
            // An enum case is reachable as a class constant: its value is the case
            // singleton, it is implicitly public and is flagged `enumCase`.
            if let Some(ci) = self.enum_case_idx(cid, &name) {
                let value = Zval::Object(self.enum_case(cid, ci as u32));
                let decl_name = self.classes[cid].name.to_vec();
                let mut a = php_types::PhpArray::new();
                a.insert(Key::Str(PhpStr::new(b"value".to_vec())), value);
                a.insert(Key::Str(PhpStr::new(b"declaringClass".to_vec())), Zval::Str(PhpStr::new(decl_name)));
                a.insert(Key::Str(PhpStr::new(b"visibility".to_vec())), Zval::Str(PhpStr::new(b"public".to_vec())));
                a.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(false));
                a.insert(Key::Str(PhpStr::new(b"enumCase".to_vec())), Zval::Bool(true));
                return Ok(Zval::Array(Rc::new(a)));
            }
            return Ok(Zval::Bool(false));
        };
        let vis: &[u8] = match self.classes[decl].consts[idx].visibility {
            crate::hir::Visibility::Public => b"public",
            crate::hir::Visibility::Protected => b"protected",
            crate::hir::Visibility::Private => b"private",
        };
        let is_final = self.classes[decl].consts[idx].is_final;
        let decl_name = self.classes[decl].name.to_vec();
        let thunk: &'m Func = &self.classes[decl].consts[idx].func;
        let value = self.run_value_thunk(thunk, Some(decl))?;
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"value".to_vec())), value);
        a.insert(Key::Str(PhpStr::new(b"declaringClass".to_vec())), Zval::Str(PhpStr::new(decl_name)));
        a.insert(Key::Str(PhpStr::new(b"visibility".to_vec())), Zval::Str(PhpStr::new(vis.to_vec())));
        a.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(is_final));
        a.insert(Key::Str(PhpStr::new(b"enumCase".to_vec())), Zval::Bool(false));
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_enum_backing($class)`: the backing scalar type of a backed enum
    /// as a `ReflectionNamedType` descriptor (`int`/`string`), or `false` for a
    /// pure enum. Derived from the (folded) case values.
    pub(super) fn ho_reflect_enum_backing(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let name: Option<&[u8]> = self.classes[cid].enum_cases.iter().find_map(|c| match &c.value {
            Some(crate::bytecode::Const::Int(_)) => Some(&b"int"[..]),
            Some(crate::bytecode::Const::Str(_)) => Some(&b"string"[..]),
            _ => None,
        });
        let Some(name) = name else { return Ok(Zval::Bool(false)) };
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"name".to_vec())), Zval::Str(PhpStr::new(name.to_vec())));
        a.insert(Key::Str(PhpStr::new(b"builtin".to_vec())), Zval::Bool(true));
        a.insert(Key::Str(PhpStr::new(b"nullable".to_vec())), Zval::Bool(false));
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_classconst_attributes($declClass, $name, $filter = null)`: the
    /// `ReflectionAttribute`s declared on `$declClass::$name`, each carrying the
    /// lazy handle (`__class`, `__classconst`, `__index`).
    pub(super) fn ho_reflect_classconst_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let name = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let Some(ci) = self.classes[cid].consts.iter().position(|k| k.name.as_ref() == name.as_slice()) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => {
                let raw = s.as_bytes();
                Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec())
            }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = self.classes[cid].consts[ci].attributes
            .iter()
            .enumerate()
            .filter(|(_, a)| match &filter {
                None => true,
                Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f),
            })
            .map(|(i, a)| (i, a.name.to_vec()))
            .collect();
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, aname) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(aname)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__classconst", Zval::Str(PhpStr::new(name.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_classconst_attr_new($declClass, $name, $index)` — run the class
    /// constant attribute's `new Attr(args)` thunk (validates TARGET_CLASS_CONSTANT
    /// / repeatability first).
    pub(super) fn ho_reflect_classconst_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.classconst_attr_thunk(&args, false) else { return Ok(Zval::Null) };
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let cid = self.class_index.get(&cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase()).copied().unwrap_or(0);
        let name = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let idx = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        if let Some(ci) = self.classes[cid].consts.iter().position(|k| k.name.as_ref() == name.as_slice()) {
            let list = &self.classes[cid].consts[ci].attributes;
            if let Some(attr) = list.get(idx) {
                let attr_name = attr.name.to_vec();
                let siblings: Vec<Vec<u8>> = list.iter().map(|a| a.name.to_vec()).collect();
                self.validate_attr(&attr_name, &siblings, 16, "class constant")?;
            }
        }
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_classconst_attr_args($declClass, $name, $index)` — run the class
    /// constant attribute's argument-array thunk.
    pub(super) fn ho_reflect_classconst_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.classconst_attr_thunk(&args, true) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let cid = self.class_index.get(&cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase()).copied().unwrap_or(0);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_param_attributes($class, $func, $pos, $filter = null)`: the
    /// `ReflectionAttribute`s on parameter `$pos` of the callable, each carrying
    /// the lazy handle (`__paramclass`, `__paramfunc`, `__parampos`, `__index`).
    pub(super) fn ho_reflect_param_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let func = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let pos = match args.get(2).map(|v| v.deref_clone()) { Some(Zval::Long(n)) => n as usize, _ => return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(3).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let Some((f, _)) = self.resolve_param_owner(&class, &func) else { return empty() };
        let Some(list) = f.param_attributes.get(pos) else { return empty() };
        let matches: Vec<(usize, Vec<u8>)> = list
            .iter()
            .enumerate()
            .filter(|(_, a)| match &filter {
                None => true,
                Some(fl) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(fl),
            })
            .map(|(i, a)| (i, a.name.to_vec()))
            .collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, aname) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(aname)));
                b.props.set(b"__paramclass", Zval::Str(PhpStr::new(class.clone())));
                b.props.set(b"__paramfunc", Zval::Str(PhpStr::new(func.clone())));
                b.props.set(b"__parampos", Zval::Long(pos as i64));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_param_attr_new($class, $func, $pos, $index)` — run the parameter
    /// attribute's `new Attr(args)` thunk (validates TARGET_PARAMETER /
    /// repeatability first).
    pub(super) fn ho_reflect_param_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some((thunk, ctx)) = self.param_attr_thunk(&args, false) else { return Ok(Zval::Null) };
        let class = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let func = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let pos = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        let idx = match args.get(3) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        if let Some((f, _)) = self.resolve_param_owner(&class, &func) {
            if let Some(list) = f.param_attributes.get(pos) {
                if let Some(attr) = list.get(idx) {
                    let attr_name = attr.name.to_vec();
                    let siblings: Vec<Vec<u8>> = list.iter().map(|a| a.name.to_vec()).collect();
                    self.validate_attr(&attr_name, &siblings, 32, "parameter")?;
                }
            }
        }
        let module = ctx.map(|c| self.class_mod(c)).unwrap_or(self.module);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, module);
        frame.class = ctx;
        frame.static_class = ctx;
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_param_attr_args($class, $func, $pos, $index)` — run the parameter
    /// attribute's argument-array thunk.
    pub(super) fn ho_reflect_param_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some((thunk, ctx)) = self.param_attr_thunk(&args, true) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let module = ctx.map(|c| self.class_mod(c)).unwrap_or(self.module);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, module);
        frame.class = ctx;
        frame.static_class = ctx;
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_closure_info($closure)`: the signature descriptor of a closure
    /// value, or `false`. A first-class callable (`strlen(...)`) reflects the
    /// named function it wraps; an ordinary closure reflects its own body via the
    /// module that compiled it.
    pub(super) fn ho_reflect_closure_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Closure(cl)) = args.first().map(|v| v.deref_clone()) else {
            return Ok(Zval::Bool(false));
        };
        if let Some(name) = &cl.named {
            let nm = name.as_bytes().to_vec();
            // A method callable (`$obj->m(...)`, `Closure::fromCallable([$o,'m'])`)
            // carries `Class::method`: reflect the method it wraps, like Zend
            // (getName() then reports the bare method name).
            if let Some(pos) = nm.windows(2).position(|w| w == b"::") {
                let key = nm[..pos]
                    .strip_prefix(b"\\")
                    .unwrap_or(&nm[..pos])
                    .to_ascii_lowercase();
                let mname = nm[pos + 2..].to_vec();
                let Some(&cid) = self.class_index.get(&key) else {
                    return Ok(Zval::Bool(false));
                };
                let Some((m, decl, _)) = self.find_method_reflect(cid, &mname) else {
                    // A magic trampoline (`Closure::fromCallable([$o,'magic'])`):
                    // Zend reflects the bare requested name with no parameters
                    // and no source location (internal-like).
                    if self.find_method_reflect(cid, b"__call").is_some()
                        || self.find_method_reflect(cid, b"__callStatic").is_some()
                    {
                        return Ok(Zval::Array(Rc::new(magic_trampoline_descriptor(&mname))));
                    }
                    return Ok(Zval::Bool(false));
                };
                return Ok(Zval::Array(Rc::new(self.build_func_descriptor(&m.func, Some(decl))?)));
            }
            return match self.find_user_function(&nm) {
                Some(func) => Ok(Zval::Array(Rc::new(self.build_func_descriptor(func, None)?))),
                None => Ok(Zval::Bool(false)),
            };
        }
        let m = self.modules[cl.module_id];
        let Some(func) = m.closures.get(cl.fn_idx) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Array(Rc::new(self.build_func_descriptor(func, None)?)))
    }
    /// `__reflect_closure_bind($closure)`: the closure's binding info as
    /// `[bound_this, scopeClassName|null, is_static]` — backs
    /// `ReflectionFunction::getClosureThis`/`getClosureScopeClass`/`isStatic`.
    pub(super) fn ho_reflect_closure_bind(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Closure(cl)) = args.first().map(|v| v.deref_clone()) else {
            return Ok(Zval::Bool(false));
        };
        let mut out = php_types::PhpArray::new();
        let _ = out.append(cl.bound_this.clone().unwrap_or(Zval::Null));
        let scope = match cl.scope {
            Some(cid) => Zval::Str(PhpStr::new(self.classes[cid].name.to_vec())),
            None => Zval::Null,
        };
        let _ = out.append(scope);
        let _ = out.append(Zval::Bool(cl.is_static));
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_closure_uses($closure)`: the closure's captured variables
    /// (`use (...)`, plus an arrow function's auto-captures) as a `name => value`
    /// map — backs `ReflectionFunction::getClosureUsedVariables()`. A by-reference
    /// capture keeps its `Zval::Ref` (so var_dump shows `&`); names come from the
    /// closure body's slot table.
    pub(super) fn ho_reflect_closure_uses(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let Some(Zval::Closure(cl)) = args.into_iter().next().map(|v| v.deref_clone()) else {
            return empty();
        };
        let Some((func, _)) = self.closure_func_mod(&cl) else { return empty() };
        let mut arr = php_types::PhpArray::new();
        for (slot, val) in &cl.captures {
            if let Some(name) = func.slot_names.get(*slot as usize) {
                arr.insert(Key::from_bytes(name), val.clone());
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_closure_attributes($closure, $filter = null)` — backs
    /// `ReflectionFunction::getAttributes()` for a closure. The handle carries the
    /// closure value itself (`__closure_val`).
    pub(super) fn ho_reflect_closure_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let Some(clos) = args.first().map(|v| v.deref_clone()) else { return empty() };
        let Zval::Closure(cl) = &clos else { return empty() };
        let Some((func, _)) = self.closure_func_mod(cl) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = func.attributes.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__closure_val", clos.clone());
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_closure_attr_new($closure, $index)`.
    pub(super) fn ho_reflect_closure_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_closure_attr(&args, false)
    }
    /// `__reflect_closure_attr_args($closure, $index)`.
    pub(super) fn ho_reflect_closure_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_closure_attr(&args, true)
    }
    /// `__reflect_func_info($name)`: the signature descriptor of a user function, or
    /// `false` if it is unknown (or a builtin, whose signature is not retained).
    pub(super) fn ho_reflect_func_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else { return Ok(Zval::Bool(false)) };
        let name = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let Some(func) = self.find_user_function(&name) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Array(Rc::new(self.build_func_descriptor(func, None)?)))
    }
    /// `__reflect_method_info($class, $method)`: the signature descriptor of a method
    /// plus `static`/`visibility`/`abstract`/`declaringClass`, or `false` if unknown.
    pub(super) fn ho_reflect_method_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cname = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let mname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let Some((m, decl, is_abstract)) = self.find_method_reflect(cid, &mname) else {
            return Ok(Zval::Bool(false));
        };
        let is_static = m.is_static;
        let is_final = m.is_final;
        let vis: &[u8] = match m.visibility {
            Visibility::Public => b"public",
            Visibility::Protected => b"protected",
            Visibility::Private => b"private",
        };
        let decl_name = self.classes[decl].name.to_vec();
        let mut d = self.build_func_descriptor(&m.func, Some(decl))?;
        d.insert(Key::Str(PhpStr::new(b"static".to_vec())), Zval::Bool(is_static));
        d.insert(Key::Str(PhpStr::new(b"byRef".to_vec())), Zval::Bool(m.func.by_ref));
        d.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(is_final));
        d.insert(Key::Str(PhpStr::new(b"visibility".to_vec())), Zval::Str(PhpStr::new(vis.to_vec())));
        d.insert(Key::Str(PhpStr::new(b"abstract".to_vec())), Zval::Bool(is_abstract));
        d.insert(Key::Str(PhpStr::new(b"declaringClass".to_vec())), Zval::Str(PhpStr::new(decl_name)));
        // file / startLine / endLine are added by build_func_descriptor (shared with
        // ReflectionFunction), with a declaration-line fallback for body-less methods.
        Ok(Zval::Array(Rc::new(d)))
    }
    /// `__reflect_invoke($object, $class, $method, $args)` — the engine behind
    /// `ReflectionMethod::invoke`/`invokeArgs`. Resolves `$method` on the *reflected*
    /// class `$class` (non-virtual, as PHP's reflection does — a subclass override is
    /// not selected) and runs it **without** a visibility check: since PHP 8.1
    /// `ReflectionMethod::invoke` calls private/protected methods without
    /// `setAccessible(true)`. `$object` binds `$this` for an instance method (ignored
    /// for a static one). Returns the method's value (or its `Generator` handle).
    pub(super) fn ho_reflect_invoke(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let obj = args
            .first()
            .map(|v| v.deref_clone())
            .filter(|v| matches!(v, Zval::Object(_)));
        let cname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Err(PhpError::Error("ReflectionMethod::invoke(): invalid class".into())),
        };
        let mname = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => {
                return Err(PhpError::Error(
                    "ReflectionMethod::invoke(): invalid method".into(),
                ))
            }
        };
        // Array elements pass AS-IS (no deref): a `[&$x]` element keeps its Ref
        // so a by-ref parameter aliases it (UtilsTest drives the private
        // detectAndCleanUtf8(&$data) through invokeArgs); the binder derefs
        // for by-value parameters as in any call.
        let argv: Vec<Zval> = match args.get(3).map(|v| v.deref_clone()) {
            Some(Zval::Array(a)) => a.iter().map(|(_, v)| v.clone()).collect(),
            _ => Vec::new(),
        };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let cid = *self.class_index.get(&key[..]).ok_or_else(|| {
            PhpError::Error(format!(
                "Class \"{}\" does not exist",
                String::from_utf8_lossy(&cname)
            ))
        })?;
        let (defc, midx) = resolve_method_runtime(&self.classes, cid, &mname)
            .ok_or_else(|| undefined_method(&self.classes, cid, &mname))?;
        // A static method ignores the supplied object; an instance method binds it.
        let this = if self.classes[defc].methods[midx].is_static {
            None
        } else {
            obj
        };
        let baseline = self.frames.len();
        self.enter_authorized_method(cid, this, &mname, argv)?;
        // A generator-body method pushes no frame — its `Generator` handle is left on
        // the caller's stack (mirrors `call_callable`).
        if self.frames.len() == baseline {
            return Ok(self.frames[baseline - 1]
                .stack
                .pop()
                .expect("reflect-invoke result on caller stack"));
        }
        self.drive_to_return(baseline)
    }
    /// `__reflect_class_modifiers($class)`: `['final' => bool, 'abstract' => bool]`
    /// for `ReflectionClass::isFinal()`/`isAbstract()`. Empty (both false) if the
    /// class is unknown.
    pub(super) fn ho_reflect_class_modifiers(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (mut is_final, mut is_abstract) = (false, false);
        if let Some(first) = args.first() {
            let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
            let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
            if let Some(&cid) = self.class_index.get(&key) {
                let cc = self.classes[cid];
                is_final = cc.is_final;
                // An interface is not reported as abstract by Reflection (only an
                // abstract *class* is), though it carries `is_abstract` internally.
                is_abstract = cc.is_abstract && !matches!(cc.instantiable, Instantiable::Interface);
            }
        }
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(is_final));
        a.insert(Key::Str(PhpStr::new(b"abstract".to_vec())), Zval::Bool(is_abstract));
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_new_no_ctor($class)`: allocate an instance of `$class` with its
    /// declared property defaults (typed properties left uninitialized) but
    /// *without* invoking the constructor — `ReflectionClass::newInstanceWithoutConstructor`.
    pub(super) fn ho_reflect_new_no_ctor(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = match args.first() {
            Some(v) => convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec(),
            None => return Err(PhpError::Error(
                "newInstanceWithoutConstructor() expects a class name".to_string(),
            )),
        };
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let v = self.alloc_object(cid)?;
        // Non-constant declared defaults (`= []`, …) live in the `prop_init`
        // thunk, run by `Op::InitProps` at a `new` site — mirror it here.
        if let Zval::Object(rc) = &v {
            self.run_prop_init_thunk(cid, rc);
        }
        Ok(v)
    }
    /// `__reflect_new_lazy_ghost($class, $init)`: allocate an uninitialized lazy
    /// ghost of `$class` whose `$init` closure runs on first access — PHP 8.4
    /// `ReflectionClass::newLazyGhost`.
    pub(super) fn ho_reflect_new_lazy_ghost(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = match args.first() {
            Some(v) => convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec(),
            None => return Err(PhpError::Error("newLazyGhost() expects a class name".to_string())),
        };
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let init = args.get(1).cloned().unwrap_or(Zval::Null);
        let options = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => n as u32,
            _ => 0,
        };
        self.alloc_lazy(cid, init, LazyKind::Ghost, options)
    }
    /// `__reflect_new_lazy_proxy($class, $factory)`: allocate an uninitialized
    /// lazy proxy of `$class` whose `$factory` runs on first access and returns
    /// the real instance the proxy forwards to — PHP 8.4
    /// `ReflectionClass::newLazyProxy`.
    pub(super) fn ho_reflect_new_lazy_proxy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = match args.first() {
            Some(v) => convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec(),
            None => return Err(PhpError::Error("newLazyProxy() expects a class name".to_string())),
        };
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let factory = args.get(1).cloned().unwrap_or(Zval::Null);
        let options = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => n as u32,
            _ => 0,
        };
        self.alloc_lazy(cid, factory, LazyKind::Proxy, options)
    }
    /// `__reflect_reset_lazy($class, $obj, $is_proxy, $init)`: reset an existing
    /// instance back to an uninitialized lazy object (PHP 8.4
    /// `ReflectionClass::resetAsLazyGhost` / `resetAsLazyProxy`). `$obj` must be
    /// an instance of the reflected `$class` (or a subclass) — else a `TypeError`
    /// naming both classes. Returns the (now lazy) object.
    pub(super) fn ho_reflect_reset_lazy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let obj = args.get(1).cloned().unwrap_or(Zval::Null);
        let is_proxy = convert::to_bool(args.get(2).unwrap_or(&Zval::Null), &mut self.diags);
        let op = if is_proxy { "resetAsLazyProxy" } else { "resetAsLazyGhost" };
        let Some(rc) = deref_object(&obj) else {
            return Err(PhpError::TypeError(format!(
                "ReflectionClass::{op}(): Argument #1 ($object) must be of type {}, {} given",
                String::from_utf8_lossy(&self.classes[cid].name),
                args.get(1).unwrap_or(&Zval::Null).type_name_for_error(),
            )));
        };
        let ocid = rc.borrow().class_id as usize;
        if !is_instance_of(&self.classes, self.stringable_id, ocid, cid) {
            return Err(PhpError::TypeError(format!(
                "ReflectionClass::{op}(): Argument #1 ($object) must be of type {}, {} given",
                String::from_utf8_lossy(&self.classes[cid].name),
                String::from_utf8_lossy(&self.classes[ocid].name),
            )));
        }
        // Resetting destroys the object's current incarnation: a fully
        // constructed (non-lazy) object runs its `__destruct` before being reborn
        // lazy (PHP 8.4). An uninitialized lazy wrapper was never constructed, so
        // it does not. Mirrors zend_lazy_objects.c (zend_object_make_lazy): the
        // DESTRUCTOR_CALLED flag is set *before* the call and stays set if the
        // destructor throws (the reset aborts, and the destructor must not run a
        // second time); a completed reset clears it unconditionally — the reborn
        // incarnation destructs again when later realized and dropped.
        let options = match args.get(4).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => n as u32,
            _ => 0,
        };
        let (oid, is_real) = { let b = rc.borrow(); (b.id, b.lazy.is_none()) };
        // SKIP_DESTRUCTOR (16): the displaced incarnation's own destructor is
        // suppressed (reset_as_lazy_may_skip_destructor).
        if is_real
            && options & 16 == 0
            && !self.destructed.contains(&oid)
            && resolve_method_runtime(&self.classes, ocid, b"__destruct").is_some()
        {
            self.destructed.insert(oid);
            self.call_method_sync(obj.clone(), b"__destruct", Vec::new())?;
        }
        self.destructed.remove(&oid);
        let kind = if is_proxy { LazyKind::Proxy } else { LazyKind::Ghost };
        let init = args.get(3).cloned().unwrap_or(Zval::Null);
        self.reject_internal_lazy(ocid)?;
        // Reset through the *reflected* class's layout: a subclass's additional
        // properties are preserved (install_lazy's reflected-scope rules).
        self.install_lazy(&rc, cid, kind, init, options)?;
        Ok(obj)
    }
    /// `__reflect_prop_names($class)`: the declared instance properties of
    /// `$class` (flattened, declaration order) as a list — backs
    /// `ReflectionClass::getProperties`. Virtual/static properties are omitted
    /// (only the `prop_defaults` slots).
    pub(super) fn ho_reflect_prop_names(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut a = php_types::PhpArray::new();
        if let Some(first) = args.first() {
            let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
            let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
            // An opaque handle class (GdImage) reflects with no properties.
            if php_types::is_opaque_handle_class(&key) {
                return Ok(Zval::Array(std::rc::Rc::new(a)));
            }
            if let Some(&cid) = self.class_index.get(&key) {
                for (n, _) in &self.classes[cid].prop_defaults {
                    // Slots are storage-keyed (mangled for privates); reflection
                    // speaks source-level names.
                    let _ = a.append(Zval::Str(PhpStr::new(php_types::prop_display_name(n).to_vec())));
                }
            }
        }
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_method_names($class)`: every method name visible on `$class`
    /// regardless of visibility — declaration order, child-most override first,
    /// walking the parent chain (a parent's `private` methods excluded, like
    /// `ReflectionClass::getMethods`). `get_class_methods` can't back this: it
    /// filters to public when called from outside the class (PHPUnit's hook
    /// discovery needs the protected `#[Before]` methods).
    pub(super) fn ho_reflect_method_names(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut a = php_types::PhpArray::new();
        if let Some(first) = args.first() {
            let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
            let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
            // An opaque handle class (GdImage) reflects with no methods.
            if php_types::is_opaque_handle_class(&key) {
                return Ok(Zval::Array(std::rc::Rc::new(a)));
            }
            if let Some(&cid) = self.class_index.get(&key) {
                let mut seen: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
                let mut out: Vec<Vec<u8>> = Vec::new();
                // Parent chain first (child-most override wins), then the
                // interface closure: an interface's signatures live in
                // `abstract_sigs`, which mock generation must see too.
                let mut queue: Vec<(usize, usize)> = vec![(cid, 0)];
                while let Some((c, depth)) = queue.pop() {
                    let cls = &self.classes[c];
                    for m in cls.methods.iter().chain(cls.abstract_sigs.iter()) {
                        if depth > 0 && matches!(m.visibility, crate::hir::Visibility::Private) {
                            continue;
                        }
                        if seen.insert(m.name.to_ascii_lowercase()) {
                            out.push(m.name.to_vec());
                        }
                    }
                    if let Some(p) = cls.parent {
                        queue.push((p as usize, depth + 1));
                    }
                    for &i in &cls.interfaces {
                        queue.push((i as usize, depth + 1));
                    }
                }
                for n in out {
                    let _ = a.append(Zval::Str(PhpStr::new(n)));
                }
            }
        }
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_prop_defaults($class)`: `ReflectionClass::getDefaultProperties`
    /// — statics first (their *current* value, like Zend), then the instance
    /// defaults in declaration order; a typed property without a default
    /// (`Undef`) is omitted. Names are source-level (unmangled).
    pub(super) fn ho_reflect_prop_defaults(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut out = php_types::PhpArray::new();
        let Some(first) = args.first() else { return Ok(Zval::Array(Rc::new(out))) };
        let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Array(Rc::new(out))) };
        // Statics along the parent chain (child-most first, like the function
        // table walks elsewhere).
        let mut cur = Some(cid);
        while let Some(c) = cur {
            for (i, sp) in self.classes[c].static_props.iter().enumerate() {
                let name = sp.name.to_vec();
                if out.get(&Key::from_bytes(&name)).is_some() {
                    continue;
                }
                let cell_key = (c, name.clone());
                let v = if let Some(cell) = self.static_props.get(&cell_key) {
                    cell.borrow().deref_clone()
                } else {
                    match &self.classes[c].static_props[i].init {
                        StaticInit::Const(k) => k.to_zval(),
                        StaticInit::Thunk(_) => Zval::Null,
                    }
                };
                out.insert(Key::from_bytes(&name), v);
            }
            cur = self.classes[c].parent;
        }
        let cc = self.classes[cid];
        for (n, d) in &cc.prop_defaults {
            if cc.uninit_props.iter().any(|u| u == n) {
                continue; // typed, no default: absent from the map
            }
            let disp = php_types::prop_display_name(n).to_vec();
            if out.get(&Key::from_bytes(&disp)).is_none() {
                out.insert(Key::from_bytes(&disp), d.to_zval());
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_prop_is_static($class, $prop)`: whether `$prop` is a static
    /// property of `$class` — backs `ReflectionProperty::isStatic`.
    pub(super) fn ho_reflect_prop_is_static(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let is = self
            .class_index
            .get(&key)
            .is_some_and(|&cid| self.classes[cid].static_props.iter().any(|sp| sp.name.as_ref() == prop.as_slice()));
        Ok(Zval::Bool(is))
    }
    /// `__reflect_prop_type($class, $prop)`: the declared type of `$prop` as the
    /// descriptor `ReflectionNamedType` is built from (`false` for an untyped
    /// property) — backs `ReflectionProperty::getType` / `hasType`. `$class` is the
    /// property's declaring class, so its (flattened) `prop_info` holds the
    /// most-derived declaration's type.
    pub(super) fn ho_reflect_prop_type(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let pi = self
            .class_index
            .get(&key)
            .and_then(|&cid| self.classes[cid].prop_info.get(prop.as_slice()));
        // A composite (union/intersection) type reflects through the dedicated
        // descriptor; a single type falls back to the enforced `type_hint`.
        if let Some(z) = pi.and_then(|pi| reflect_type_descriptor(&pi.reflect_type)) {
            return Ok(z);
        }
        Ok(typehint_descriptor(&pi.and_then(|pi| pi.type_hint.clone())))
    }
    /// `__reflect_prop_details($class, $prop)`: a descriptor array backing the
    /// non-type `ReflectionProperty` accessors — `visibility`
    /// (`public`/`protected`/`private`), `readonly`, `static`, `declaringClass`,
    /// `hasDefault` and the constant `default` value. `$class` is the declaring
    /// class, whose flattened `prop_info` / `prop_defaults` carry the resolved
    /// shape; static properties fall back to their `static_props` entry.
    pub(super) fn ho_reflect_prop_details(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let vis_str = |v: crate::hir::Visibility| -> &'static [u8] {
            match v {
                crate::hir::Visibility::Public => b"public",
                crate::hir::Visibility::Protected => b"protected",
                crate::hir::Visibility::Private => b"private",
            }
        };
        let mut a = php_types::PhpArray::new();
        let put = |a: &mut php_types::PhpArray, k: &[u8], v: Zval| {
            a.insert(Key::Str(PhpStr::new(k.to_vec())), v);
        };
        if let Some(&cid) = self.class_index.get(&key) {
            let c = &self.classes[cid];
            if let Some(pi) = c.prop_info.get(prop.as_slice()) {
                let vis = vis_str(pi.visibility).to_vec();
                let readonly = pi.readonly;
                let decl = self.classes[pi.declaring_class].name.to_vec();
                let uninit = c.uninit_props.iter().any(|n| n.as_ref() == prop.as_slice());
                let default = c.prop_defaults.iter().find(|(n, _)| n.as_ref() == prop.as_slice()).map(|(_, k)| k.to_zval());
                put(&mut a, b"visibility", Zval::Str(PhpStr::new(vis)));
                put(&mut a, b"readonly", Zval::Bool(readonly));
                put(&mut a, b"static", Zval::Bool(false));
                put(&mut a, b"declaringClass", Zval::Str(PhpStr::new(decl)));
                put(&mut a, b"hasDefault", Zval::Bool(!uninit));
                put(&mut a, b"default", default.unwrap_or(Zval::Null));
                let doc = pi.doc.as_ref().map_or(Zval::Bool(false), |d| Zval::Str(PhpStr::new(d.to_vec())));
                put(&mut a, b"doc", doc);
                return Ok(Zval::Array(Rc::new(a)));
            }
            if let Some(sp) = c.static_props.iter().find(|sp| sp.name.as_ref() == prop.as_slice()) {
                let vis = vis_str(sp.visibility).to_vec();
                let decl = c.name.to_vec();
                put(&mut a, b"visibility", Zval::Str(PhpStr::new(vis)));
                put(&mut a, b"readonly", Zval::Bool(false));
                put(&mut a, b"static", Zval::Bool(true));
                put(&mut a, b"declaringClass", Zval::Str(PhpStr::new(decl)));
                put(&mut a, b"hasDefault", Zval::Bool(true));
                put(&mut a, b"default", Zval::Null);
                return Ok(Zval::Array(Rc::new(a)));
            }
        }
        put(&mut a, b"visibility", Zval::Str(PhpStr::new(b"public".to_vec())));
        put(&mut a, b"readonly", Zval::Bool(false));
        put(&mut a, b"static", Zval::Bool(false));
        put(&mut a, b"declaringClass", Zval::Str(PhpStr::new(cls.clone())));
        put(&mut a, b"hasDefault", Zval::Bool(true));
        put(&mut a, b"default", Zval::Null);
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_prop_initialized($class, $prop, $obj)`: whether `$prop` holds a
    /// value on `$obj` (vs an uninitialized typed property) — backs
    /// `ReflectionProperty::isInitialized`. Reads the raw slot without triggering
    /// lazy initialization; a non-object (or absent slot) reads as initialized.
    pub(super) fn ho_reflect_prop_initialized(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let obj = args.get(2).cloned().unwrap_or(Zval::Null);
        if !matches!(obj, Zval::Object(_)) {
            return Ok(Zval::Bool(true));
        }
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let key = match self.class_index.get(&lc).copied() {
            Some(c) => self.prop_decl_storage_key(c, &prop),
            None => prop.clone(),
        };
        // Inspect the raw slot: `Undef` (never initialized) and a removed
        // entry (explicitly unset) both read as NOT initialized — silently
        // (read_property would raise the Undefined-property warning).
        let init = match &obj {
            Zval::Object(o) => !matches!(o.borrow().props.get(&key), None | Some(Zval::Undef)),
            _ => true,
        };
        Ok(Zval::Bool(init))
    }
    /// `__reflect_prop_get($class, $prop, $obj)`: read property `$prop` (declared
    /// in `$class`) of `$obj` ignoring visibility — backs
    /// `ReflectionProperty::getValue`. A lazy object initializes first; a proxy
    /// forwards to its real instance.
    pub(super) fn ho_reflect_prop_get(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let obj = self.realize_full(&args.get(2).cloned().unwrap_or(Zval::Null))?;
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let cid = self.class_index.get(&lc).copied();
        let key = match cid {
            Some(c) => self.prop_decl_storage_key(c, &prop),
            None => prop.clone(),
        };
        Ok(read_property(&obj, &key, &mut self.diags))
    }
    /// `__reflect_prop_set($class, $prop, $obj, $value)`: write `$value` into
    /// property `$prop` (declared in `$class`) of `$obj` ignoring visibility —
    /// backs `ReflectionProperty::setValue`. A lazy object initializes first; a
    /// proxy forwards to its real instance. EXCEPT: a property already made
    /// non-lazy by `skipLazyInitialization`/`setRawValueWithoutLazyInitialization`
    /// is written straight into the still-lazy wrapper without running the
    /// initializer (rfc_example_004/005).
    pub(super) fn ho_reflect_prop_set(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let obj_arg = args.get(2).cloned().unwrap_or(Zval::Null);
        let value = args.get(3).cloned().unwrap_or(Zval::Null);
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let cid = self.class_index.get(&lc).copied();
        let key = match cid {
            Some(c) => self.prop_decl_storage_key(c, &prop),
            None => prop.clone(),
        };
        // A setValue on a still-uninitialized lazy wrapper whose target property
        // has already been dropped from the lazy set (skipLazyInitialization) must
        // NOT trigger the initializer — materialize the single slot in place.
        let skip_materialized = {
            let mut target = obj_arg.clone();
            for _ in 0..64 {
                let next = self.proxy_redirect(target.clone());
                let same = match (deref_object(&target), deref_object(&next)) {
                    (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                    _ => true,
                };
                target = next;
                if same {
                    break;
                }
            }
            deref_object(&target).is_some_and(|o| {
                let b = o.borrow();
                b.lazy.is_some()
                    && b.proxy_instance.is_none()
                    && !self
                        .lazy_props
                        .get(&b.id)
                        .is_some_and(|set| set.iter().any(|n| n.as_ref() == key.as_slice()))
            })
        };
        if skip_materialized {
            self.lazy_materialize(&obj_arg, &key, value)?;
            return Ok(Zval::Null);
        }
        let obj = self.realize_full(&obj_arg)?;
        if let Some(old) = write_property(&obj, &key, value)? {
            self.gc_note(&old);
        }
        Ok(Zval::Null)
    }
    /// `__reflect_static_prop_get($class, $prop)`: read a static property
    /// ignoring visibility — backs `ReflectionProperty::getValue()` on statics.
    /// A constant default initializes lazily; a not-yet-run thunk default reads
    /// NULL (declared residue).
    pub(super) fn ho_reflect_static_prop_get(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&lc) else { return Ok(Zval::Null) };
        let Some((decl, idx)) = find_static_prop(&self.classes, cid, &prop) else {
            return Ok(Zval::Null);
        };
        let key = (decl, prop);
        if let Some(cell) = self.static_props.get(&key) {
            return Ok(cell.borrow().deref_clone());
        }
        match &self.classes[decl].static_props[idx].init {
            StaticInit::Const(c) => {
                let v = c.to_zval();
                self.static_props.insert(key, Rc::new(RefCell::new(v.clone())));
                Ok(v)
            }
            StaticInit::Thunk(_) => Ok(Zval::Null),
        }
    }
    /// `__reflect_static_prop_set($class, $prop, $value)`: write a static
    /// property ignoring visibility — backs `ReflectionProperty::setValue` with
    /// a NULL object (Composer pokes `InstalledVersions::$selfDir`). Returns
    /// whether the property exists.
    pub(super) fn ho_reflect_static_prop_set(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let value = args.get(2).cloned().unwrap_or(Zval::Null).deref_clone();
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&lc) else { return Ok(Zval::Bool(false)) };
        let Some((decl, _)) = find_static_prop(&self.classes, cid, &prop) else {
            return Ok(Zval::Bool(false));
        };
        let key = (decl, prop);
        match self.static_props.get(&key) {
            Some(cell) => *cell.borrow_mut() = value,
            None => {
                self.static_props.insert(key, Rc::new(RefCell::new(value)));
            }
        }
        Ok(Zval::Bool(true))
    }
    /// `__reflect_static_vars($class|null, $function)`: a function's local
    /// `static $v` variables as a `name => value` map — backs
    /// `ReflectionFunctionAbstract::getStaticVariables()`. The current persistent
    /// cell value wins; before the function has run the declared initial value is
    /// used. `$class` selects a method, else a free function.
    pub(super) fn ho_reflect_static_vars(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let fname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return empty(),
        };
        // (name, cell id, folded initial value). The Func ref lives for `'m`, so it
        // coexists with the `self.statics` reads below.
        let entries: Vec<(Vec<u8>, u32, Zval)> = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Str(c)) => {
                let key = c.as_bytes().strip_prefix(b"\\").unwrap_or(c.as_bytes()).to_ascii_lowercase();
                let Some(&cid) = self.class_index.get(&key) else { return empty() };
                let Some((m, _, _)) = self.find_method_reflect(cid, &fname) else { return empty() };
                m.func.static_vars.iter().map(|s| (s.name.to_vec(), s.id, static_var_init(&s.init))).collect()
            }
            _ => {
                let Some(func) = self.find_user_function(&fname) else { return empty() };
                func.static_vars.iter().map(|s| (s.name.to_vec(), s.id, static_var_init(&s.init))).collect()
            }
        };
        let mut arr = php_types::PhpArray::new();
        for (name, id, init) in entries {
            let val = self
                .statics
                .get(id as usize)
                .and_then(|c| c.as_ref())
                .map(|c| c.borrow().deref_clone())
                .unwrap_or(init);
            arr.insert(Key::from_bytes(&name), val);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_static_props($class)`: all static properties of `$class` (its own
    /// and inherited) as a `name => value` map — backs
    /// `ReflectionClass::getStaticProperties()`. Derived class first; a name already
    /// seen (child redeclaration) keeps the derived value. A const default is
    /// realized lazily, a not-yet-run thunk reads NULL (as the single-prop getter).
    pub(super) fn ho_reflect_static_props(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&lc) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let mut chain: Vec<usize> = Vec::new();
        let mut c = Some(cid);
        while let Some(ci) = c {
            chain.push(ci);
            c = self.classes[ci].parent;
        }
        let mut seen: HashSet<Vec<u8>> = HashSet::default();
        let mut out = php_types::PhpArray::new();
        for ci in chain {
            let props: Vec<(usize, Vec<u8>)> = self.classes[ci]
                .static_props
                .iter()
                .enumerate()
                .map(|(idx, sp)| (idx, sp.name.as_ref().to_vec()))
                .collect();
            for (idx, name) in props {
                if !seen.insert(name.clone()) {
                    continue;
                }
                let key = (ci, name.clone());
                let val = if let Some(cell) = self.static_props.get(&key) {
                    cell.borrow().deref_clone()
                } else {
                    match &self.classes[ci].static_props[idx].init {
                        StaticInit::Const(cst) => cst.to_zval(),
                        StaticInit::Thunk(_) => Zval::Null,
                    }
                };
                out.insert(Key::from_bytes(&name), val);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_prop_attributes($class, $prop, $filter = null)`: the host backing
    /// of `ReflectionProperty::getAttributes()`. Returns `ReflectionAttribute`s for
    /// the `#[…]` declared on `$class::$prop`, each carrying the lazy handle
    /// (`__class`, `__prop`, `__index`) the materializers below use.
    pub(super) fn ho_reflect_prop_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => {
                let raw = s.as_bytes();
                Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec())
            }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = match self.classes[cid].prop_attributes.get(prop.as_slice()) {
            Some(list) => list
                .iter()
                .enumerate()
                .filter(|(_, a)| match &filter {
                    None => true,
                    Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f),
                })
                .map(|(i, a)| (i, a.name.to_vec()))
                .collect(),
            None => return empty(),
        };
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__prop", Zval::Str(PhpStr::new(prop.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_prop_attr_new($class, $prop, $index)` — run the property
    /// attribute's `new Attr(args)` thunk (mirrors `__reflect_attr_newinstance`).
    pub(super) fn ho_reflect_prop_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.prop_attr_thunk(&args, false) else { return Ok(Zval::Null) };
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let cid = self.class_index.get(&cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase()).copied().unwrap_or(0);
        // Validate the property attribute's target/repeatability first.
        let prop = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let idx = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        if let Some(list) = self.classes[cid].prop_attributes.get(prop.as_slice()) {
            if let Some(attr) = list.get(idx) {
                let attr_name = attr.name.to_vec();
                let siblings: Vec<Vec<u8>> = list.iter().map(|a| a.name.to_vec()).collect();
                self.validate_attr(&attr_name, &siblings, 8, "property")?;
            }
        }
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_prop_attr_args($class, $prop, $index)` — run the property
    /// attribute's argument-array thunk (mirrors `__reflect_attr_arguments`).
    pub(super) fn ho_reflect_prop_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.prop_attr_thunk(&args, true) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let cid = self.class_index.get(&{
            let c = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
            c.strip_prefix(b"\\").unwrap_or(&c).to_ascii_lowercase()
        }).copied().unwrap_or(0);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_func_attributes($func, $filter = null)` — backs
    /// `ReflectionFunction::getAttributes()`. Each `ReflectionAttribute` carries
    /// the `__func` handle the materializers below resolve.
    pub(super) fn ho_reflect_func_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let fname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let Some(func) = self.find_user_function(&fname) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = func.attributes.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__func", Zval::Str(PhpStr::new(fname.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_func_attr_new($func, $index)`.
    pub(super) fn ho_reflect_func_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_func_attr(&args, false)
    }
    /// `__reflect_func_attr_args($func, $index)`.
    pub(super) fn ho_reflect_func_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_func_attr(&args, true)
    }
    /// `__reflect_method_attributes($class, $method, $filter = null)` — backs
    /// `ReflectionMethod::getAttributes()`. Handle: `__class` + `__method`.
    pub(super) fn ho_reflect_method_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let method = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        // `find_method_reflect` also searches abstract signatures and the interface
        // graph, so an interface/abstract method's `#[…]` attributes are visible
        // (resolve_method_runtime only sees concrete `.methods`).
        let Some((m, _defc, _)) = self.find_method_reflect(cid, &method) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = m.func.attributes.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__method", Zval::Str(PhpStr::new(method.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_method_attr_new($class, $method, $index)`.
    pub(super) fn ho_reflect_method_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_method_attr(&args, false)
    }
    /// `__reflect_method_attr_args($class, $method, $index)`.
    pub(super) fn ho_reflect_method_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_method_attr(&args, true)
    }
    /// `__reflect_const_attributes($const, $filter = null)` — backs
    /// `ReflectionConstant::getAttributes()`. Top-level constants are
    /// case-sensitive; the handle is `__const`.
    pub(super) fn ho_reflect_const_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_vec();
        let Some(attrs) = self.module.const_attributes.get(key.as_slice()) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = attrs.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__const", Zval::Str(PhpStr::new(key.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_const_attr_new($const, $index)`.
    pub(super) fn ho_reflect_const_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_const_attr(&args, false)
    }
    /// `__reflect_const_attr_args($const, $index)`.
    pub(super) fn ho_reflect_const_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_const_attr(&args, true)
    }
    /// `__reflect_class_attributes($class, $filter = null)`: the host backing of
    /// `ReflectionClass::getAttributes()`. Returns an array of `ReflectionAttribute`
    /// objects, one per `#[…]` declared on `$class` (optionally filtered by
    /// attribute name). Each carries `name` plus the private handle (`__class`,
    /// `__index`) the other reflection builtins use to materialise it lazily.
    pub(super) fn ho_reflect_class_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let Some(first) = args.first() else { return empty() };
        let cname = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else {
            return empty();
        };
        // A non-empty string second argument restricts the result to that attribute
        // class (case-insensitively, leading `\` stripped) — `getAttributes($name)`.
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => {
                let raw = s.as_bytes();
                Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec())
            }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = self.classes[cid]
            .attributes
            .iter()
            .enumerate()
            .filter(|(_, a)| match &filter {
                None => true,
                Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f),
            })
            .map(|(i, a)| (i, a.name.to_vec()))
            .collect();
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_attr_newinstance($class, $index)`: build the attribute object by
    /// running its retained `new Attr(args)` thunk in the attributed class's context
    /// (so `self::`/constants in the argument list resolve as written).
    pub(super) fn ho_reflect_attr_newinstance(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (cid, idx) = self.reflect_attr_handle(&args)?;
        let cc = self.classes[cid];
        // Validate the class attribute's target/repeatability first.
        let attr_name = cc.attributes[idx].name.to_vec();
        let siblings: Vec<Vec<u8>> = cc.attributes.iter().map(|a| a.name.to_vec()).collect();
        self.validate_attr(&attr_name, &siblings, 1, "class")?;
        let thunk = &cc.attributes[idx].new_thunk;
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_attr_arguments($class, $index)`: run the attribute's argument-array
    /// thunk (positional args int-keyed, named args string-keyed) — `getArguments()`.
    pub(super) fn ho_reflect_attr_arguments(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (cid, idx) = self.reflect_attr_handle(&args)?;
        let cc = self.classes[cid];
        let thunk = &cc.attributes[idx].args_thunk;
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_prop_declaring_class($class, $prop)`: the class that *declares*
    /// `$prop` — the most-derived class in `$class`'s ancestry whose own (instance
    /// or static) property list contains it. A child that redeclares an inherited
    /// property shadows the parent, so this returns the child, matching
    /// `ReflectionProperty::$class`. `false` if no class declares it.
    pub(super) fn ho_reflect_prop_declaring_class(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cname = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let pname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let mut cur = Some(cid);
        while let Some(c) = cur {
            let cc = self.classes[c];
            let declares = cc.own_prop_vis.iter().any(|(n, _)| n.as_ref() == pname.as_slice())
                || cc.static_props.iter().any(|sp| sp.name.as_ref() == pname.as_slice());
            if declares {
                return Ok(Zval::Str(PhpStr::new(cc.name.to_vec())));
            }
            cur = cc.parent;
        }
        Ok(Zval::Bool(false))
    }
    /// `__reflect_object_bind($reflectionObject, $instance)`: records the instance a
    /// `ReflectionObject` was built for (keyed by the ReflectionObject's id), so the
    /// prelude need not hold it as a var_dump-visible property.
    pub(super) fn ho_reflect_object_bind(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let ro = args.first().map(|v| v.deref_clone());
        let inst = args.get(1).map(|v| v.deref_clone());
        if let (Some(Zval::Object(r)), Some(inst)) = (ro, inst) {
            let id = r.borrow().id;
            self.reflect_object_bound.insert(id, inst);
        }
        Ok(Zval::Null)
    }
    /// `__reflect_object_instance($reflectionObject)`: the bound instance
    /// itself (null when unbound) — lets the prelude route
    /// `ReflectionObject::getProperty` on a *dynamic* property through the
    /// instance-taking `ReflectionProperty` constructor.
    pub(super) fn ho_reflect_object_instance(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Object(ro)) = args.into_iter().next().map(|v| v.deref_clone()) else {
            return Ok(Zval::Null);
        };
        let ro_id = ro.borrow().id;
        Ok(self.reflect_object_bound.get(&ro_id).cloned().unwrap_or(Zval::Null))
    }
    /// `__reflect_object_dynprops($reflectionObject)`: the names of the bound
    /// instance's *dynamic* (undeclared, unmangled) properties, in instance order.
    /// Reads the property table directly — it does **not** realise a lazy object
    /// (PHP's `ReflectionObject::__toString` enumerates dynamic props without
    /// triggering init: Zend/tests/lazy_objects/init_trigger_reflection_object_toString).
    pub(super) fn ho_reflect_object_dynprops(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Object(ro)) = args.into_iter().next().map(|v| v.deref_clone()) else {
            return Ok(Zval::Array(Rc::new(PhpArray::new())));
        };
        let ro_id = ro.borrow().id;
        let Some(Zval::Object(o)) = self.reflect_object_bound.get(&ro_id).cloned() else {
            return Ok(Zval::Array(Rc::new(PhpArray::new())));
        };
        let cid = o.borrow().class_id as usize;
        // Declared property names across the whole parent chain.
        let mut declared: HashSet<Box<[u8]>> = HashSet::default();
        let mut c = Some(cid);
        while let Some(ci) = c {
            for (name, _) in &self.classes[ci].own_prop_vis {
                declared.insert(name.clone());
            }
            c = self.classes[ci].parent;
        }
        let mut arr = PhpArray::new();
        let b = o.borrow();
        for (name, _) in b.props.iter() {
            if !declared.contains(name) && !name.starts_with(b"\0") {
                let _ = arr.append(Zval::Str(PhpStr::new(name.to_vec())));
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_class_loc(name) -> [file|false, startLine, endLine]`: the file
    /// that declared the class (from its first method's compiled unit; the
    /// property-init thunk as fallback) and the line span covered by its method
    /// bodies — `false`/0 for a prelude ("internal") class or one with no
    /// compiled body. Serves ReflectionClass::getFileName/getStartLine/getEndLine
    /// (the span is an approximation from the op line tables, not the `class`
    /// keyword's line).
    pub(super) fn ho_reflect_class_loc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut out = PhpArray::new();
        let Some(cid) = self.resolve_named_class_with_autoload(&args)? else {
            // Zend's class table also holds traits: getFileName/getStartLine/
            // getEndLine on a trait name report its declaring unit, located
            // via its methods' op line tables (a trait is flattened into its
            // consumers and records no file — same approximation as the
            // first-method fallback for classes below).
            if let Some((file, start, end)) = self.trait_loc(&args) {
                let _ = out.append(Zval::Str(PhpStr::new(file)));
                let _ = out.append(Zval::Long(i64::from(start)));
                let _ = out.append(Zval::Long(i64::from(end)));
            } else {
                let _ = out.append(Zval::Bool(false));
                let _ = out.append(Zval::Long(0));
                let _ = out.append(Zval::Long(0));
            }
            return Ok(Zval::Array(Rc::new(out)));
        };
        let c = &self.classes[cid];
        // Prefer the declaring unit recorded on the class itself; older paths
        // derived it from the first method, kept as fallback for classes whose
        // `file` is empty (e.g. synthesized ones).
        let file: Option<&[u8]> = Some(&c.file[..])
            .filter(|f| !f.is_empty())
            .or_else(|| c.methods.first().map(|m| &m.func.file[..]))
            .or_else(|| c.prop_init.as_ref().map(|f| &f.file[..]));
        match file {
            Some(f) if f != b"prelude" => {
                let _ = out.append(Zval::Str(PhpStr::new(f.to_vec())));
                // getStartLine is the `class` keyword's line; getEndLine the closing
                // `}` line, both recorded from the source span. Fall back to the
                // method op-line span for a class compiled before this was tracked.
                let start = c.line;
                let end = if c.end_line > 0 {
                    c.end_line
                } else {
                    let mut e = 0u32;
                    for m in c.methods.iter().filter(|m| m.func.file[..] == f[..]) {
                        for &l in m.func.lines.iter() {
                            e = e.max(l);
                        }
                    }
                    e.max(start)
                };
                let _ = out.append(Zval::Long(i64::from(start)));
                let _ = out.append(Zval::Long(i64::from(end)));
            }
            _ => {
                let _ = out.append(Zval::Bool(false));
                let _ = out.append(Zval::Long(0));
                let _ = out.append(Zval::Long(0));
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_class_real_name(name) -> string|false`: the CANONICAL declared
    /// name of a class (class names resolve case-insensitively, but the reflected
    /// `ReflectionClass::$name` must carry the real casing, not the argument's).
    /// `false` when the class does not exist. Lets the prelude normalize a
    /// `new ReflectionClass('MY\CLASS')` name back to `My\Class`
    /// (Doctrine ClassMetadata::initializeReflection: ClassMetadataTest::testClassCaseSensitivity).
    pub(super) fn ho_reflect_class_real_name(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        match self.resolve_named_class_with_autoload(&args)? {
            Some(cid) => Ok(Zval::Str(PhpStr::new(self.classes[cid].name.to_vec()))),
            // A trait name reflects too (Zend single class table): report its
            // declared casing.
            None => Ok(match self.named_trait(&args) {
                Some(name) => Zval::Str(PhpStr::new(name)),
                None => Zval::Bool(false),
            }),
        }
    }

    /// The real (as-declared) name of the trait named by `args[0]`, matched
    /// case-insensitively on the fully-qualified name; `None` for non-traits.
    fn named_trait(&mut self, args: &[Zval]) -> Option<Vec<u8>> {
        let a = args.first()?;
        let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
        let b = raw.as_bytes();
        let want = b.strip_prefix(b"\\").unwrap_or(b).to_vec();
        self.seed_traits
            .iter()
            .map(|(_, t)| t)
            .find(|t| t.name.eq_ignore_ascii_case(&want))
            .map(|t| t.name.to_vec())
    }

    /// The declaring file and approximate line span of the trait named by
    /// `args[0]`, from its methods' op line tables (`None` for an unknown name,
    /// a method-less trait, or a prelude one).
    fn trait_loc(&mut self, args: &[Zval]) -> Option<(Vec<u8>, u32, u32)> {
        let a = args.first()?;
        let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
        let b = raw.as_bytes();
        let want = b.strip_prefix(b"\\").unwrap_or(b).to_vec();
        let t = self
            .seed_traits
            .iter()
            .map(|(_, t)| t)
            .find(|t| t.name.eq_ignore_ascii_case(&want))?;
        let file = t.methods.first().map(|m| m.decl.file.to_vec())?;
        if file == b"prelude" {
            return None;
        }
        let (mut start, mut end) = (u32::MAX, 0u32);
        for m in t.methods.iter().filter(|m| m.decl.file[..] == file[..]) {
            start = start.min(m.decl.line);
            end = end.max(m.decl.end_line.max(m.decl.line));
        }
        if start == u32::MAX {
            (start, end) = (0, 0);
        }
        Some((file, start, end))
    }
    /// `__reflect_ref_id(array, key) -> string|false`: the identity of the
    /// reference an array element holds, or `false` when the element is not a
    /// reference (or the key/argument is invalid). Two elements that alias the same
    /// reference report the same id — the contract `ReflectionReference::getId()`
    /// relies on (Symfony var-exporter / deepclone reference tracking). The id is
    /// the reference cell's address rendered as text; only equality is meaningful.
    pub(super) fn ho_reflect_ref_id(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Array(a)) = args.first() else {
            return Ok(Zval::Bool(false));
        };
        let Some(key) = args.get(1).and_then(arrays::coerce_key_silent) else {
            return Ok(Zval::Bool(false));
        };
        match a.get(&key) {
            Some(Zval::Ref(cell)) => {
                let id = format!("{:p}", Rc::as_ptr(cell));
                Ok(Zval::Str(PhpStr::new(id.into_bytes())))
            }
            _ => Ok(Zval::Bool(false)),
        }
    }
    /// `__reflect_gen_info(gen) -> [line, file|false, this, funcName]`: the state of
    /// a `Generator`'s suspended frame, backing `ReflectionGenerator`. A running or
    /// finished generator (no parked frame) reports line 0 / file false / this null.
    pub(super) fn ho_reflect_gen_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let g = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Generator(g)) => g,
            _ => return Ok(Zval::Bool(false)),
        };
        let (id, func_name, is_done) = {
            let b = g.borrow();
            (b.id, b.func_name.clone(), matches!(b.status, GenStatus::Done))
        };
        let mut out = PhpArray::new();
        match self.generators.get(&id) {
            Some(frame) => {
                let line = frame.func.lines.get(frame.ip).copied().unwrap_or(0);
                out.insert(Key::Int(0), Zval::Long(i64::from(line)));
                out.insert(Key::Int(1), Zval::Str(PhpStr::new(frame.func.file.to_vec())));
                out.insert(Key::Int(2), frame.this.clone().unwrap_or(Zval::Null));
            }
            None => {
                out.insert(Key::Int(0), Zval::Long(0));
                out.insert(Key::Int(1), Zval::Bool(false));
                out.insert(Key::Int(2), Zval::Null);
            }
        }
        out.insert(Key::Int(3), Zval::Str(PhpStr::new(func_name.to_vec())));
        out.insert(Key::Int(4), Zval::Bool(is_done));
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_fiber_info(fiber) -> [line, file|false, callable]`: the suspended
    /// frame of a `Fiber` and the callable it was constructed with, backing
    /// `ReflectionFiber`. A not-started / finished fiber reports line 0 / file false.
    pub(super) fn ho_reflect_fiber_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let obj = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Object(o)) => o,
            _ => return Ok(Zval::Bool(false)),
        };
        let (id, cid) = {
            let b = obj.borrow();
            (b.id, b.class_id as usize)
        };
        let key = self.host_prop_key(cid, b"callable");
        let callable = obj.borrow().props.get(key.as_slice()).cloned().unwrap_or(Zval::Null);
        let mut out = PhpArray::new();
        match self.fibers.get(&id).and_then(|s| s.parked.last()) {
            Some(frame) => {
                let line = frame.func.lines.get(frame.ip).copied().unwrap_or(0);
                out.insert(Key::Int(0), Zval::Long(i64::from(line)));
                out.insert(Key::Int(1), Zval::Str(PhpStr::new(frame.func.file.to_vec())));
            }
            None => {
                out.insert(Key::Int(0), Zval::Long(0));
                out.insert(Key::Int(1), Zval::Bool(false));
            }
        }
        out.insert(Key::Int(2), callable);
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_class_doc(name) -> string|false`: the class declaration's
    /// retained `/** ... */` doc comment (ReflectionClass::getDocComment), false
    /// for none / an unknown class / a prelude ("internal") class.
    pub(super) fn ho_reflect_class_doc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cid) = self.resolve_named_class_with_autoload(&args)? else {
            return Ok(Zval::Bool(false));
        };
        let c = &self.classes[cid];
        Ok(match (&c.doc, &c.file[..]) {
            (Some(d), f) if f != b"prelude" => Zval::Str(PhpStr::new(d.to_vec())),
            _ => Zval::Bool(false),
        })
    }
}

/// The descriptor of a magic-trampoline closure (`Closure::fromCallable` on a
/// `__call`/`__callStatic`-served name): Zend reports the bare requested name,
/// zero parameters and no source location (internal-like function).
fn magic_trampoline_descriptor(name: &[u8]) -> php_types::PhpArray {
    let mut d = php_types::PhpArray::new();
    let mut put = |k: &[u8], v: Zval| {
        d.insert(Key::Str(PhpStr::new(k.to_vec())), v);
    };
    put(b"name", Zval::Str(PhpStr::new(name.to_vec())));
    put(b"returnType", Zval::Bool(false));
    put(b"params", Zval::Array(Rc::new(php_types::PhpArray::new())));
    put(b"doc", Zval::Bool(false));
    put(b"isGenerator", Zval::Bool(false));
    put(b"byRef", Zval::Bool(false));
    put(b"file", Zval::Bool(false));
    put(b"startLine", Zval::Bool(false));
    put(b"endLine", Zval::Bool(false));
    d
}
