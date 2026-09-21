"""Sequential detached job supervisor. Does not stop unrelated processes."""
import argparse,json,os,shutil,signal,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('name');p.add_argument('--cwd',required=True);p.add_argument('--timeout',type=int,default=1800);p.add_argument('--timing',action='store_true');p.add_argument('cmd',nargs=argparse.REMAINDER);a=p.parse_args()
out=Path('/private/tmp/phpr-s178-investigation'); lock=Path('/private/tmp/phpr-measure.lock')
assert lock.read_text().strip()=='s178-orm-a98b'
cmd=a.cmd[1:] if a.cmd and a.cmd[0]=='--' else a.cmd
log=open(out/(a.name+'.log'),'w');sent=open(out/(a.name+'.sentinels.jsonl'),'w')
def snapshot():
    cpu=sum(float(x) for x in subprocess.check_output(['ps','-Ao','%cpu'],text=True).splitlines()[1:])
    d={'time':time.time(),'free':shutil.disk_usage('/System/Volumes/Data').free,'cpu':cpu,'swap':subprocess.check_output(['sysctl','vm.swapusage'],text=True).strip()}
    sent.write(json.dumps(d)+'\n');sent.flush();return d
rc=0
if a.timing:
    for i in range(4):
        d=snapshot()
        if d['cpu']>=150 or d['free']<10*1024**3: rc=8;break
        if i<3:time.sleep(30)
if not rc:
    d=snapshot()
    if d['free']<10*1024**3:rc=9
if not rc:
    proc=subprocess.Popen(cmd,cwd=a.cwd,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
    start=time.time()
    while proc.poll() is None:
        d=snapshot()
        if lock.read_text().strip()!='s178-orm-a98b' or d['free']<10*1024**3 or time.time()-start>a.timeout:
            os.killpg(proc.pid,signal.SIGTERM);time.sleep(2)
            if proc.poll() is None:os.killpg(proc.pid,signal.SIGKILL)
            proc.wait();rc=9;break
        time.sleep(5)
    if not rc:rc=proc.returncode
snapshot();(out/(a.name+'.done')).write_text(json.dumps({'rc':rc,'command':cmd,'cwd':a.cwd})+'\n')
