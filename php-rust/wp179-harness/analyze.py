from pathlib import Path
from collections import Counter,defaultdict
import json,sys,re
p=Path(sys.argv[1]);meta=dict(re.findall(r'(\w+)=([^ ]+)',p.read_text().splitlines()[0]));tot=Counter();cls=defaultdict(Counter);files=defaultdict(Counter);rows=[]
for l in p.read_text().splitlines()[1:]:
 c,f,pr,ro,g,ok,n,ns,adj,mx=l.split('\t');pr,ro,g,ok,n,ns,adj,mx=map(int,(pr,ro,g,ok,n,ns,adj,mx));r=dict(cls=c,file=f,private=pr,readonly=ro,guards=g,success=ok,n=n,ns=ns,corrected_ns=adj,max_ns=mx);rows.append(r)
 d=Counter(n=n,ns=ns,corrected_ns=adj)
 if pr==1 and ro==0:
  d.update(private_nonro=n,private_nonro_ns=adj)
  if meta['mode']=='count':
   for bit,name in enumerate(['scope','slot','hook','magic','instance','present','ref']):
    if not g&(1<<bit):d['reject_'+name]+=n
   if g&127==127 and ok:d['eligible']+=n;d['eligible_typed' if g&128 else 'eligible_untyped']+=n
 elif pr==1 and ro==1:d['private_ro']+=n
 elif pr==0:d['nonprivate']+=n
 else:d['unknown']+=n
 tot.update(d);cls[c].update(d);files[f].update(d)
print(json.dumps(dict(meta=meta,total=tot,classes=dict(sorted(cls.items(),key=lambda kv:-kv[1]['private_nonro'])),files=files,rows=rows),indent=2))
