//! S-179 diagnostic only, injected into an archived source. No production fill.
use std::{cell::RefCell, collections::BTreeMap, time::Instant};
use php_types::Zval;
#[derive(Default)]
struct Row { n: u64, ns: u128, corrected: u128, max: u128 }
struct State { mode: String, rate: u64, rng: u64, calls: u64, floor: u128, p95: u128, rows: BTreeMap<String,Row> }
impl State {
 fn new() -> Self {
  let mut times=Vec::with_capacity(10001);
  for _ in 0..10001 { let t=Instant::now(); std::hint::black_box(()); times.push(t.elapsed().as_nanos()); }
  times.sort_unstable();
  Self { mode:std::env::var("PHPR_S179_MODE").unwrap_or_default(), rate:std::env::var("PHPR_S179_RATE").ok().and_then(|v|v.parse().ok()).unwrap_or(1024).max(1), rng:std::env::var("PHPR_S179_SEED").ok().and_then(|v|v.parse().ok()).unwrap_or(1979).max(1), calls:0, floor:times[5000],p95:times[9500],rows:BTreeMap::new() }
 }
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::new());}
pub struct Probe { start:Option<Instant>, base:Option<String>, pub private:i8,pub readonly:i8,pub guards:u32,pub success:bool,pub count:bool }
impl Probe {
 #[inline]
 pub fn begin(obj:&Zval,file:&[u8])->Self {
  let (record,timing,count)=STATE.with(|s| {let mut s=s.borrow_mut();s.calls+=1;
   let count=s.mode=="count";let timing=s.mode=="time";
   let selected=if timing {let mut x=s.rng;x^=x<<13;x^=x>>7;x^=x<<17;s.rng=x;x % s.rate==0} else {false};
   (count||selected,timing&&selected,count)
  });
  let base=if record {let class=if let Zval::Object(o)=obj {String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned()}else{"<nonobject>".into()};Some(format!("{}\t{}",class,String::from_utf8_lossy(file)))}else{None};
  Self {start:if timing {Some(Instant::now())}else{None},base,private:-1,readonly:-1,guards:0,success:false,count}
 }
}
impl Drop for Probe {
 fn drop(&mut self) {
  let ns=self.start.map(|t|t.elapsed().as_nanos()).unwrap_or(0);
  let Some(base)=self.base.take() else{return};
  let key=format!("{}\t{}\t{}\t{}\t{}",base,self.private,self.readonly,self.guards,self.success as u8);
  STATE.with(|s|{let mut s=s.borrow_mut();let corrected=if self.start.is_some(){ns.saturating_sub(s.floor)}else{0};let r=s.rows.entry(key).or_default();r.n+=1;r.ns+=ns;r.corrected+=corrected;r.max=r.max.max(ns);});
 }
}
pub fn dump(){
 use std::io::Write;
 let Ok(path)=std::env::var("PHPR_S179_OUT") else{return};
 STATE.with(|s|{let s=s.borrow();let mut f=std::fs::File::create(path).expect("diagnostic output");
 writeln!(f,"# mode={} rate={} calls={} floor_ns={} p95_ns={}",s.mode,s.rate,s.calls,s.floor,s.p95).unwrap();
 for (k,r) in &s.rows {writeln!(f,"{}\t{}\t{}\t{}\t{}",k,r.n,r.ns,r.corrected,r.max).unwrap();}
 });
}
