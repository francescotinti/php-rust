"""S-179 standalone sequential supervisor; only terminates its own process group."""
import argparse,json,os,signal,subprocess,time,shutil,resource,re
from pathlib import Path
OUT=Path('/Users/francescotinti/Claude/phpr-s179-cost-resume-results')
LOCK=Path('/private/tmp/phpr-measure.lock');TOKEN='s179-cost-resume-cb7f'
def run(name,cmd,cwd,env=None,timing=False,timeout=1800):
 assert LOCK.read_text().strip()==TOKEN
 assert not (OUT/(name+'.done.json')).exists(),name
 sent=open(OUT/(name+'.sentinels.jsonl'),'w')
 def snapshot(pgid=None):
  raw=subprocess.check_output(['ps','-Ao','pid,pgid,pcpu,comm'],text=True)
  rows=[x.split(None,3) for x in raw.splitlines()[1:]]
  total=sum(float(x[2]) for x in rows); own=sum(float(x[2]) for x in rows if pgid is not None and int(x[1])==pgid)
  d=dict(time=time.time(),cpu=total,background_cpu=total-own,top_processes=sorted([dict(pid=int(x[0]),pgid=int(x[1]),cpu=float(x[2]),command=x[3]) for x in rows],key=lambda x:-x['cpu'])[:8],free=shutil.disk_usage('/System/Volumes/Data').free,swap=subprocess.check_output(['sysctl','vm.swapusage'],text=True).strip())
  sent.write(json.dumps(d)+'\n');sent.flush();return d
 def healthy(d):return d['free']>=10*1024**3 and LOCK.read_text().strip()==TOKEN
 result=dict(command=cmd,cwd=str(cwd),env=env or {},timing=timing)
 if timing:
  for i in range(4):
   d=snapshot()
   if not healthy(d) or d['cpu']>=150:
    result.update(guard_rc=8,reason='E2 or disk/lock');break
   if i<3:time.sleep(30)
 if not result.get('guard_rc') and healthy(snapshot()):
  with open(OUT/(name+'.log'),'w') as log:
   e=os.environ.copy();e.update(env or {})
   before=resource.getrusage(resource.RUSAGE_CHILDREN);start=time.monotonic()
   p=subprocess.Popen(["/usr/bin/time","-lp"]+cmd,cwd=cwd,env=e,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
   guard=0
   while p.poll() is None:
    d=snapshot(p.pid)
    if not healthy(d) or time.monotonic()-start>timeout or (timing and d['background_cpu']>=150):
     guard=9;result['guard_reason']='background_cpu' if healthy(d) and time.monotonic()-start<=timeout else 'disk-lock-timeout';os.killpg(p.pid,signal.SIGTERM)
     try:p.wait(timeout=5)
     except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
     break
    time.sleep(1 if timing else 5)
   after=resource.getrusage(resource.RUSAGE_CHILDREN)
   result.update(guard_rc=guard,process_rc=p.returncode,wall=time.monotonic()-start,user=after.ru_utime-before.ru_utime,system=after.ru_stime-before.ru_stime)
 else:result.setdefault('guard_rc',9)
 logpath=OUT/(name+'.log')
 if logpath.exists():
  raw=logpath.read_text(errors='replace')
  result['workload_time']={k:float(v) for k,v in re.findall(r'^(real|user|sys) ([0-9.]+)$',raw,re.M)}
 snapshot();sent.close();(OUT/(name+'.done.json')).write_text(json.dumps(result,indent=2));return result
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('job');a=p.parse_args();d=json.loads(Path(a.job).read_text());print(json.dumps(run(**d)))
