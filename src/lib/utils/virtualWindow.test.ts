import { describe, it } from 'vitest';
import assert from 'node:assert/strict';
import { HeightIndex, WindowedList, computeWindow } from './virtualWindow';

describe('HeightIndex / computeWindow', () => {
  it('matches a brute-force reference and keeps window invariants', () => {

    // brute-force reference
    function ref(h:number[]){ const o=[0]; for(const x of h) o.push(o.at(-1)!+x); return o; }
    function refIndexAt(o:number[], y:number){ let r=0; for(let i=0;i<o.length-1;i++){ if(o[i]<=y) r=i; } return r; }

    let seed=12345; const rnd=()=> (seed=(seed*1664525+1013904223)>>>0)/2**32;
    for (const n of [0,1,2,3,7,64,100,1000,4097]) {
      const h = Array.from({length:n},()=>72+Math.floor(rnd()*200));
      const idx = new HeightIndex(); idx.reset(h);
      let o = ref(h);
      for (let t=0;t<200 && n>0;t++){
        if (rnd()<0.5){ const i=Math.floor(rnd()*n); const v=72+Math.floor(rnd()*300); idx.set(i,v); h[i]=v; o=ref(h); }
        const i=Math.floor(rnd()*(n+1)); assert.ok(Math.abs(idx.offset(i)-o[i])<1e-6);
        const y=rnd()*o[n]; assert.equal(idx.indexAt(y), refIndexAt(o,y));
      }
      assert.ok(Math.abs(idx.total()-o[n])<1e-6);
    }
    // window invariants
    const n=50000; const h=Array.from({length:n},()=>72+Math.floor(rnd()*100));
    const idx=new HeightIndex(); idx.reset(h);
    for (const st of [0, 1234, 999999, idx.total()-500, idx.total()+5000]) {
      const w=computeWindow(idx, st, 800, 600);
      assert.ok(w.start>=0 && w.end<=n && w.start<=w.end);
      assert.ok(Math.abs(w.topPad + (idx.offset(w.end)-idx.offset(w.start)) + w.bottomPad - idx.total())<1e-6);
      assert.ok(w.end-w.start < 60, `rendered ${w.end-w.start}`);
      if (st < idx.total()) assert.ok(idx.offset(w.start) <= Math.max(0,st-600)+1e-6 || w.start===0);
    }
  });
});

describe('WindowedList', () => {
  const items = (n: number, prefix = 's') => Array.from({ length: n }, (_, i) => ({ id: `${prefix}${i}` }));

  it('uses the estimate until rows are measured, then the measured heights', () => {
    const w = new WindowedList(100, 0);
    w.sync(items(10));
    assert.equal(w.totalHeight(), 1000);
    assert.equal(w.setRowHeight('s2', 150), true);
    assert.equal(w.offsetOf(3), 350);
    assert.equal(w.setRowHeight('s2', 150.2), false); // sub-pixel noise ignored
    assert.equal(w.setRowHeight('gone', 99), false);  // unknown id ignored
  });

  it('does not rebuild when ids and order are unchanged, rebuilds otherwise', () => {
    const w = new WindowedList(100, 0);
    assert.equal(w.sync(items(5)), true);
    assert.equal(w.sync(items(5)), false);          // new array, same ids (a keystroke)
    assert.equal(w.sync(items(6)), true);           // appended
    const reordered = items(6).reverse();
    assert.equal(w.sync(reordered), true);          // order changed
  });

  it('keeps measured heights when the list is filtered or reordered', () => {
    const w = new WindowedList(100, 0);
    w.sync(items(5));
    w.setRowHeight('s3', 300);
    w.sync([{ id: 's3' }, { id: 's0' }]);           // filtered + reordered
    assert.equal(w.offsetOf(1), 300);
    assert.equal(w.totalHeight(), 400);
  });

  it('switches to the average measured height once enough rows are measured', () => {
    const w = new WindowedList(100, 0);
    w.sync(items(50));
    for (let i = 0; i < 25; i++) w.setRowHeight(`s${i}`, 200);
    w.sync(items(60));                               // forces a rebuild
    assert.equal(w.offsetOf(25), 25 * 200);
    assert.equal(w.offsetOf(60) - w.offsetOf(50), 10 * 200); // new rows use the 200 average
  });

  it('returns a bounded window that covers the viewport', () => {
    const w = new WindowedList(80, 400);
    w.sync(items(100000));
    const r = w.window(40000, 800);
    assert.ok(r.end - r.start < 40);
    assert.ok(w.offsetOf(r.start) <= 40000 - 400 + 80);
    assert.ok(w.offsetOf(r.end) >= 40000 + 800);
    assert.equal(w.window(0, 800).start, 0);
    assert.deepEqual(new WindowedList().window(0, 800), { start: 0, end: 0, topPad: 0, bottomPad: 0 });
  });
});
