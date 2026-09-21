"""Cost diagnostics from sampled regions in the actual workload; never from a microbenchmark."""
from pathlib import Path
from collections import Counter,defaultdict
from statistics import median
import json,re,math
out=Path('/Users/francescotinti/Claude/phpr-s179-cost-resume-results')
counts=json.loads((out/'count-analysis.json').read_text());runs=json.loads((out/'timings.json').read_text());groups=defaultdict(list);samples=[]
for run in runs:
 groups[run['label']].append(run['workload_time']['user'])
 if not run['label'].startswith('time'):continue
 p=out/(run['name']+'.tsv');lines=p.read_text().splitlines();meta=dict(re.findall(r'(\w+)=([^ ]+)',lines[0]));rate=int(meta['rate']);byclass=defaultdict(Counter);total=Counter();byfile=defaultdict(Counter)
 for l in lines[1:]:
  c,f,pr,ro,g,ok,n,ns,adj,mx=l.split('\t');n,adj=int(n),int(adj)
  if (pr,ro)!=('1','0'):continue
  d=Counter(n=n,estimated_ns=adj*rate);total.update(d);byclass[c].update(d)
  family='prelude' if f=='prelude' else ('PHPUnit' if '/vendor/phpunit/' in f else ('ORM' if '/orm-work/src/' in f else ('Doctrine deps' if '/vendor/doctrine/' in f else 'other')))
  byfile[family].update(d)
 samples.append(dict(name=run['name'],rate=rate,user=run['workload_time']['user'],wall=run['workload_time']['real'],private_nonro_seconds=total['estimated_ns']/1e9,wall_share=total['estimated_ns']/1e9/run['workload_time']['real'],sample_n=total['n'],meta=meta,classes=byclass,files=byfile))
offs=groups['offa']+groups['offb'];pure=groups['pure'];medoff=median(offs)
ratios={k:median(v)/medoff-1 for k,v in groups.items()};null=medoff/median(pure)-1
byrate=defaultdict(list)
for r in samples:byrate[r['rate']].append(r['private_nonro_seconds'])
stability=abs(median(byrate[256])/median(byrate[1024])-1) if len(byrate)==2 else None
chronoff=[r['workload_time']['user'] for r in runs if r['label'] in ('offa','offb')];drift=abs(chronoff[-1]/chronoff[0]-1)
valid=(len(runs)==15 and abs(null)<=.05 and drift<=.05 and all(abs(ratios[k])<=.05 for k in ['time256','time1024']) and stability is not None and stability<=.05)
d=dict(valid_cost=valid,completed_runs=len(runs),user_cpu_by_label=dict(groups),median_off=medoff,off_vs_pure=null,ratios_vs_off=ratios,off_drift=drift,frequency_stability=stability,sampled_runs=samples,limits=['Wall-region estimates, not exclusive CPU attribution or speedup.','Counts matching guards are not guaranteed cache hits.','Sampling metadata can warm the receiver; validation cannot eliminate every local instrumentation bias.'])
(out/'timing-analysis.json').write_text(json.dumps(d,indent=2));print(json.dumps({k:v for k,v in d.items() if k!='sampled_runs'},indent=2))
