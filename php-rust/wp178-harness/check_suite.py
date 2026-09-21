from pathlib import Path
import re,sys,json
out=Path('/private/tmp/phpr-s178-investigation');h=Path(__file__).resolve().parent
base=(h.parent/'wp125-harness/orm-baseline-failnames.txt').read_text().splitlines()
result={}
for run in sys.argv[1:]:
 done=json.loads((out/(run+'.done')).read_text());raw=(out/(run+'.log')).read_text(errors='replace').replace('\0','')
 names=sorted(set(re.findall(r'^\d+\) (.*)$',raw,re.M)))
 (out/(run+'.failnames')).write_text('\n'.join(names)+'\n')
 summaries=re.findall(r'^Tests: .*$',raw,re.M)
 assert done['rc']==2,(run,done)
 assert names==sorted(base),(run,names,base)
 assert summaries and 'Tests: 3484,' in summaries[-1] and 'Errors: 3, Failures: 13,' in summaries[-1],(run,summaries)
 result[run]={'rc':done['rc'],'summary':summaries[-1],'fail_set':'baseline16 identical'}
(out/'parity.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
