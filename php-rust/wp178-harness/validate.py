"""Positive/negative probe controls and bilateral byte parity before ORM."""
from pathlib import Path
import os,sys,subprocess,json
h=Path(__file__).resolve().parent;out=Path('/private/tmp/phpr-s178-investigation');binary=sys.argv[1]
probe=out/'fixture.tsv';assert not probe.exists()
reference=None
for name,exe in [('oracle','/opt/homebrew/opt/php/bin/php'),('pin','/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s175'),('probe',binary)]:
 env=os.environ.copy()
 if name=='probe':env.update(PHPR_S178_MODE='count',PHPR_S178_OUT=str(probe))
 r=subprocess.run([exe,'-d','opcache.enable_cli=0','-d','log_errors=0',str(h/'fixture.php')],env=env,capture_output=True)
 (out/(name+'-fixture.stdout')).write_bytes(r.stdout);(out/(name+'-fixture.stderr')).write_bytes(r.stderr)
 assert r.returncode==0,(name,r.returncode,r.stderr)
 if reference is None:reference=(r.stdout,r.stderr)
 assert (r.stdout,r.stderr)==reference,name
assert reference==(b'9:45:9:3:7\n',b'')
rows=[]
for l in probe.read_text().splitlines():
 if l.startswith('#'):continue
 c,f,path,priv,ro,n,sr,orr,ns=l.split('\t');rows.append((c,path,int(priv),int(ro),int(n),int(sr),int(orr),int(ns)))
checks=[]
for cls,n,repeats,priv,ro in [('MutablePrivate',10,9,1,0),('ReadOnlyPrivate',10,0,1,1),('PublicMutable',10,9,0,0),('ChildPrivate',2,0,1,0)]:
 r=[x for x in rows if x[0]=='S178\\'+cls and x[1]!='init']
 assert sum(x[4] for x in r)==n,(cls,r)
 assert sum(x[6] for x in r)==repeats,(cls,r)
 assert all(x[2]==priv and x[3]==ro for x in r),(cls,r)
 assert all(x[7]==0 for x in r),'counts cannot contain times'
 checks.append(dict(cls=cls,calls=n,object_repeat=repeats,private=priv,readonly=ro))
# Constructor site repeats across distinct readonly instances, unlike object writes.
r=[x for x in rows if x[0]=='S178\\ReadOnlyPrivate' and x[1]!='init']
assert sum(x[5] for x in r)==9,r
(out/'validation.json').write_text(json.dumps({'byte_parity':'oracle=pin=probe','readonly_site_repeat':9,'checks':checks},indent=2))
print('PASS: bilateral bytes, exclusive categories, object generations, inherited private slots, repeated constructor site')
