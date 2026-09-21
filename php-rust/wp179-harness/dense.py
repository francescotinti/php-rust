from pathlib import Path
import json,os,shutil,subprocess,time,re
from guarded import run,OUT,LOCK,TOKEN
work=OUT/'work/orm-work';probe=str(OUT/'phpr-probe');base=(OUT/'orm-baseline-failnames.txt').read_text().splitlines()
def check(name):
 d=json.loads((OUT/(name+'.done.json')).read_text());assert d['guard_rc']==0 and d['process_rc']==2,(name,d)
 raw=(OUT/(name+'.log')).read_text(errors='replace').replace('\0','');names=sorted(set(re.findall(r'^\d+\) (.*)$',raw,re.M)));summary=re.findall(r'^Tests: .*$',raw,re.M)[-1]
 assert names==sorted(base),(name,names)
 assert summary=='Tests: 3484, Assertions: 11989, Errors: 3, Failures: 13, Skipped: 59, Incomplete: 2.',summary
 (OUT/(name+'.parity.json')).write_text(json.dumps(dict(summary=summary,failnames=names,baseline16=True),indent=2));return d
records=[]
try:
 for rep,seed in enumerate([20011,30011,40009]):
  for label,mode,rate in [('offa','',16),('time16','time',16),('offb','',64),('time64','time',64)]:
   for attempt in range(10):
    name=f'd{rep}-{label}-a{attempt}';e=dict(PHPR_S179_MODE=mode,PHPR_S179_RATE=str(rate),PHPR_S179_SEED=str(seed),PHPR_S179_OUT=str(OUT/(name+'.tsv')))
    shutil.copy2(OUT/'phpunit-cache-seed.json',work/'.phpunit.cache/test-results')
    print('START',name,flush=True);d=run(name,[probe,'vendor/bin/phpunit','--no-coverage'],work,e,timing=True)
    if d['guard_rc']!=8 and d.get('guard_reason')!='background_cpu':break
    print('GUARD WAIT',name,flush=True);time.sleep(60)
   d=check(name);records.append(dict(name=name,label=label,rep=rep,**d));(OUT/'dense-timings.json').write_text(json.dumps(records,indent=2));print('DONE',name,d['workload_time'],flush=True)
 (OUT/'dense.done.json').write_text(json.dumps(dict(status='complete',runs=len(records))))
except Exception as e:
 (OUT/'dense.done.json').write_text(json.dumps(dict(status='stopped',reason=repr(e))));raise
finally:
 if LOCK.exists() and LOCK.read_text().strip()==TOKEN:LOCK.unlink()
