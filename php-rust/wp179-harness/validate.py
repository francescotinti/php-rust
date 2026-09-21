from pathlib import Path
import subprocess,os,json,sys
h=Path(__file__).resolve().parent;out=Path('/Users/francescotinti/Claude/phpr-s179-cost-resume-results');binary=sys.argv[1]
results={}
for name,exe,mode in [('oracle','/opt/homebrew/opt/php/bin/php',''),('pin','/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-s175',''),('control',str(out/'phpr-control'),''),('off',binary,''),('count',binary,'count'),('time',binary,'time')]:
 e=os.environ.copy();e.update(PHPR_S179_MODE=mode,PHPR_S179_RATE='1',PHPR_S179_OUT=str(out/(name+'-fixture.tsv')))
 r=subprocess.run([exe,'-d','opcache.enable_cli=0','-d','log_errors=0',str(h/'fixture.php')],env=e,capture_output=True)
 (out/(name+'-fixture.stdout')).write_bytes(r.stdout);(out/(name+'-fixture.stderr')).write_bytes(r.stderr)
 assert (r.returncode,r.stdout,r.stderr)==(0,b'12:4:9:11\n',b''),(name,r.returncode,r.stdout,r.stderr)
 results[name]='byte-identical'
def rows(mode):
 ans=[]
 for l in (out/(mode+'-fixture.tsv')).read_text().splitlines():
  if l.startswith('#'):continue
  c,f,private,ro,guards,success,n,ns,corrected,mx=l.split('\t')
  ans.append(dict(cls=c,private=int(private),ro=int(ro),guards=int(guards),success=int(success),n=int(n),ns=int(ns)))
 return ans
rs=rows('count');checks={}
expected={'MutablePrivate':3,'ChildPrivate':2,'ReadonlyPrivate':1,'MagicPrivate':2,'TypedPrivate':2,'UnsetPrivate':2,'RefPrivate':1}
for c,n in expected.items():
 rr=[r for r in rs if r['cls']=='S179\\'+c and r['private']==1]
 assert sum(r['n'] for r in rr)==n,(c,rr)
 checks[c]=rr
for c in ['MutablePrivate','TypedPrivate']:
 assert all(r['guards']&127==127 and r['success'] for r in checks[c]),checks[c]
assert all(not(r['guards']&1) for r in checks['ChildPrivate'])
assert all(r['ro']==1 for r in checks['ReadonlyPrivate'])
assert all(not(r['guards']&8) for r in checks['MagicPrivate'])
assert any(not(r['guards']&32) for r in checks['UnsetPrivate'])
assert all(not(r['guards']&64) for r in checks['RefPrivate'])
assert any(r['cls']=='S179\\HookPrivate' and r['private']==1 and not(r['guards']&4) for r in rs)
assert any(r['cls']=='S179\\PublicMutable' and r['private']==0 for r in rs)
tr=rows('time');assert sum(r['n'] for r in tr)==sum(r['n'] for r in rs)
assert all(r['ns']==0 for r in rs)
assert all(r['ns']>0 for r in tr)
(out/'validation.json').write_text(json.dumps(dict(parity=results,checks=checks,all_count_rows=rs,timer='all events selected rate=1, positive elapsed'),indent=2))
print('PASS bilateral bytes and guard positives/negatives, count/time coverage')
