#!/usr/bin/env python3
"""s192-anatomia-kernel.py <tag> — S-192: kernel samples of a profile recorded by s192-anatomia.py, by system-call
family and by the VM/builtin source line that asks for it. Profile = direction. Generato con assistenza AI (Claude Fable 5.2)."""
import json,gzip,collections,subprocess,re,pathlib,sys
O=pathlib.Path("wp192-harness/out"); B=str(pathlib.Path.home()/"Claude/phpr-target/s192-armD/release/phpr")
tag=sys.argv[1]
_keep=O/f"anatomia-{tag}.bin"/"phpr"
if _keep.exists(): B=str(_keep)
raw=(O/f"anatomia-{tag}.prof.json").read_bytes(); d=json.loads(gzip.decompress(raw) if raw[:2]==b"\x1f\x8b" else raw)
libs=[l.get("name") for l in d["libs"]]
chains=collections.Counter()
for th in d["threads"]:
    stt,ft,fu,rt=th["stackTable"],th["frameTable"],th["funcTable"],th["resourceTable"]
    for s in th["samples"]["stack"]:
        if s is None: continue
        ch=[]; x=s
        while x is not None and len(ch)<14:
            fr=stt["frame"][x]; res=fu["resource"][ft["func"][fr]]
            lib=libs[rt["lib"][res]] if res is not None and res>=0 and rt["lib"][res] is not None else "?"
            ch.append((lib,ft["address"][fr])); x=stt["prefix"][x]
        if ch[0][0]=="libsystem_kernel.dylib": chains[tuple(ch)]+=1
total=sum(len([1 for s in th["samples"]["stack"] if s is not None]) for th in d["threads"])
addrs=sorted({a for ch in chains for (l,a) in ch if l=="phpr"})
f=O/"tmp-k.addrs"; f.write_text("\n".join(hex(0x100000000+a) for a in addrs)+"\n")
nm=dict(zip(addrs,subprocess.run(["atos","-o",B,"-f",str(f)],capture_output=True,text=True).stdout.splitlines()))
site=collections.Counter(); cache={}
STD=("3std","4core","5alloc")
for ch,n in chains.items():
    sysc=None; who="?"
    for lib,a in ch[1:]:
        if lib!="phpr": continue
        s=nm[a]
        m=re.search(r"\(([\w.]+:\d+)\)",s)
        isstd=bool(re.search(r"/rustc/|sys::|3std|_3std",s)) or any(k in s.split(" (in")[0] for k in ("3std2fs","3std3sys","3std3env","3std2io"))
        if isstd:
            if sysc is None:
                t=s.split(" (in")[0]; mm=re.findall(r"\d+([a-z_]+)",t); sysc=mm[-1] if mm else "?"
            continue
        if a not in cache:
            inl=subprocess.run(["atos","-i","-o",B,hex(0x100000000+a)],capture_output=True,text=True).stdout
            ls=[x for x in re.findall(r"\(([\w.]+:\d+)\)",inl) if not x.startswith(("fs.rs","path.rs","env.rs","unix.rs","io.rs"))]
            t=s.split(" (in")[0]; mm=re.findall(r"\d+([A-Za-z_][A-Za-z0-9_]{3,})",t)
            cache[a]=(mm[-1] if mm else "?")+" @ "+(ls[-1] if ls else "?")
        who=cache[a]; break
    site[(sysc or "?",who)]+=n
print(tag,"kernel",sum(chains.values()),f"({100*sum(chains.values())/total:.1f} %)")
for (s,w),n in site.most_common(22): print(f"{n:6d} {100*n/total:4.1f}%  {s:18s} <- {w}")
