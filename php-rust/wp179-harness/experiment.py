"""Sequential driver. Every timing run has fresh E2; no builds overlap this driver."""
from pathlib import Path
import json,subprocess,os,tarfile,re,shutil,hashlib,sys,time
from guarded import run,OUT,LOCK,TOKEN
h=Path(__file__).resolve().parent
base=(h/'orm-baseline-failnames.txt').read_text().splitlines()
probe=str(OUT/'phpr-probe');control=str(OUT/'phpr-control')
work=OUT/'work/orm-work'
def suite_run(name,cmd,cwd,env=None,timing=False):
 shutil.copy2(OUT/'phpunit-cache-seed.json',work/'.phpunit.cache/test-results')
 return run(name,cmd,cwd,env,timing=timing)

def check(name):
 d=json.loads((OUT/(name+'.done.json')).read_text());assert d['guard_rc']==0,(name,d)
 raw=(OUT/(name+'.log')).read_text(errors='replace').replace('\0','')
 names=sorted(set(re.findall(r'^\d+\) (.*)$',raw,re.M)))
 summary=re.findall(r'^Tests: .*$',raw,re.M)[-1]
 assert d['process_rc']==2,(name,d)
 assert names==sorted(base),(name,names)
 assert summary=='Tests: 3484, Assertions: 11989, Errors: 3, Failures: 13, Skipped: 59, Incomplete: 2.',summary
 (OUT/(name+'.parity.json')).write_text(json.dumps(dict(summary=summary,failnames=names,baseline16=True),indent=2))
 return d
try:
 d=json.loads((OUT/'build-probe.done.json').read_text());assert d['guard_rc']==0 and d['process_rc']==0,d
 shutil.copy2('/Users/francescotinti/Claude/phpr-s179-cost-target/release/phpr',probe)
 subprocess.run(['python3',str(h/'validate.py'),probe],check=True)
 tar=Path('/Volumes/Extreme Pro/Claude/wp9-harness/gates/orm-work.tgz');sha=hashlib.sha256(tar.read_bytes()).hexdigest();assert sha=='206cf384f4102b9522c73e378310fad1685586dd1dc124e55fdfd2597220d1bb'
 if not work.exists():
  (OUT/'work').mkdir()
  with tarfile.open(tar) as t:
   members=[m for m in t.getmembers() if not any(part.startswith('._') for part in Path(m.name).parts)]
   t.extractall(OUT/'work',members=members)
 env={'PHPR_S179_MODE':'count','PHPR_S179_OUT':str(OUT/'count.tsv')}
 if not (OUT/'count-frozen.parity.json').exists():
  suite_run('count-frozen',[probe,'vendor/bin/phpunit','--no-coverage'],work,env);check('count-frozen')
  with (OUT/'count-analysis.json').open('w') as f:subprocess.run(['python3',str(h/'analyze.py'),str(OUT/'count.tsv')],stdout=f,check=True)
 data=json.loads((OUT/'count-analysis.json').read_text());assert int(data['meta']['calls'])==4979840,data['meta'];assert data['total']['private_nonro']==1795090,data['total']
 if not (OUT/'warmup.parity.json').exists():
  suite_run('warmup',[control,'vendor/bin/phpunit','--no-coverage'],work);check('warmup')
 records=json.loads((OUT/'timings.json').read_text()) if (OUT/'timings.json').exists() else []
 for rep in range(3):
  for label,exe,mode,rate in [('pure',control,'',1024),('offa',probe,'',1024),('time256',probe,'time',256),('offb',probe,'',1024),('time1024',probe,'time',1024)]:
   if any(r['rep']==rep and r['label']==label for r in records):continue
   name=f'r{rep}-{label}';env={'PHPR_S179_MODE':mode,'PHPR_S179_RATE':str(rate),'PHPR_S179_SEED':str(1979+rep*7919),'PHPR_S179_OUT':str(OUT/(name+'.tsv'))}
   for attempt in range(10):
    actual=name+f'-a{attempt}'
    if (OUT/(actual+'.done.json')).exists():continue
    env['PHPR_S179_OUT']=str(OUT/(actual+'.tsv'))
    print('START',actual,flush=True);d=suite_run(actual,[exe,'vendor/bin/phpunit','--no-coverage'],work,env,timing=True)
    if d['guard_rc']!=8 and d.get('guard_reason')!='background_cpu':break
    print('E2 WAIT',actual,flush=True);time.sleep(60)
   d=check(actual);records.append(dict(name=actual,label=label,rep=rep,**d));(OUT/'timings.json').write_text(json.dumps(records,indent=2));print('DONE',actual,d.get('workload_time'),flush=True)
 (OUT/'experiment.done.json').write_text(json.dumps(dict(status='complete',runs=len(records))))
except Exception as e:
 (OUT/'experiment.done.json').write_text(json.dumps(dict(status='stopped',reason=repr(e))));raise
finally:
 if LOCK.exists() and LOCK.read_text().strip()==TOKEN:LOCK.unlink()
