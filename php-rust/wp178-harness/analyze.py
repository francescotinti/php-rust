"""Aggregate S-178 rows; count proportions are never time proportions."""
from pathlib import Path
from collections import Counter,defaultdict
import sys,json
p=Path(sys.argv[1]); groups=defaultdict(Counter);classes=defaultdict(Counter);files=defaultdict(Counter);cross=Counter();total=Counter()
def owner(c):
    if c.startswith('PHPUnit\\'):return 'PHPUnit'
    if c.startswith('Doctrine\\Tests\\'):return 'Doctrine tests'
    if c.startswith('Doctrine\\ORM\\'):return 'Doctrine ORM'
    if c.startswith('Doctrine\\'):return 'Doctrine dependencies'
    return 'Other'
def origin(f):
    if f=='prelude':return 'Runtime prelude'
    if '/vendor/phpunit/' in f:return 'PHPUnit'
    if '/vendor/sebastian/' in f:return 'Sebastian dependencies'
    if '/orm-work/tests/' in f:return 'Doctrine tests'
    if '/orm-work/src/' in f:return 'Doctrine ORM'
    if '/vendor/doctrine/' in f:return 'Doctrine dependencies'
    return 'Other'
rows=[]
for line in p.read_text().splitlines():
    if line.startswith('#'):continue
    c,f,path,priv,ro,n,sr,orr,ns=line.split('\t');n,sr,orr,ns=map(int,(n,sr,orr,ns));priv,ro=int(priv),int(ro)
    d=Counter(calls=n,site_repeat=sr,object_repeat=orr,ns=ns)
    d[path]+=n
    if path=='miss':
        d[f'private={priv},readonly={ro}']+=n
        if priv==1:d['private']+=n;d['private_object_repeat']+=orr;d['private_site_repeat']+=sr
        if ro==1:d['readonly']+=n
    total.update(d);groups[owner(c)].update(d);classes[c].update(d);files[origin(f)].update(d)
    cross[(owner(c),origin(f))]+=n
    rows.append(dict(cls=c,file=f,path=path,private=priv,readonly=ro,calls=n,site_repeat=sr,object_repeat=orr,ns=ns))
print(json.dumps(dict(total=total,receiver=groups,caller=files,top_classes=sorted(classes.items(),key=lambda x:-x[1]['miss'])[:25],cross={str(k):v for k,v in cross.items()},rows=rows),indent=2))
