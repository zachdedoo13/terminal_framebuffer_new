use std::hint::black_box;
use std::ops::{Deref, DerefMut, Range};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use threadpool::ThreadPool;

static CORE_COUNT: LazyLock<usize> = LazyLock::new(|| num_cpus::get());
static THREAD_POOL: LazyLock<ThreadPool> = LazyLock::new(|| ThreadPool::new(*CORE_COUNT));

pub fn init_threadpool() {
   let _ = &*THREAD_POOL;
}

pub fn par_iter_mut<'v, T: 'static, F>(data: &mut Vec<T>, func: F)
where
    F: Fn(&mut T, usize) + Send + Sync + 'static,
{
   let func = Arc::new(func);

   let ptr = Arc::new(C(data.as_mut_ptr()));
   for job in get_work(*CORE_COUNT, data.len()) {
      let func = func.clone();
      let pc = Arc::clone(&ptr);

      THREAD_POOL.execute(move || {
         // let slice_ptr = unsafe { pc.add(job.start) };
         // let slice = unsafe { &mut (*std::ptr::slice_from_raw_parts_mut(slice_ptr, job.len())) };
         // for (i, dta) in slice.iter_mut().enumerate() {
         //     func(dta, job.start + i);
         // }
         let start = unsafe { pc.add(job.start) };
         for i in 0..job.len() {
            let dta = unsafe { &mut *start.add(i) };
            func(dta, job.start + i);
         }
      });
   }

   THREAD_POOL.join();
}

pub fn iter_mut<'v, T: 'static, F>(data: &mut Vec<T>, func: F)
where
    F: Fn(&mut T, usize) + Send + Sync + Clone + 'static,
{
   for (i, v) in data.iter_mut().enumerate() {
      func(v, i);
   }
}


const ITERATIONS: usize = 3;
pub fn time<F: FnMut()>(mut f: F) -> Vec<Duration> {
   let mut c = Vec::with_capacity(ITERATIONS);
   for _ in 0..ITERATIONS {
      let st = Instant::now();
      f();
      c.push(st.elapsed());
   }
   c
}

#[allow(dead_code)]
fn bench() {
   const AS: usize = 150_000_000;

   let data = (0..AS).into_iter().collect::<Vec<usize>>();

   init_threadpool();
   let value = 5;

   let mut d = data.clone();
   let no_par = time(|| {
      iter_mut::<usize, _>(&mut d, move |v, _| {
         *v = program(*v) + value;
      });
   });
   black_box(d);

   let mut d = data.clone();
   let my_par = time(|| {
      par_iter_mut::<usize, _>(&mut d, move |v, _| {
         *v = program(*v) + value;
      });
   });
   black_box(d);

   let mut d = data.clone();
   let rayon = time(|| {
      use rayon::prelude::*;
      d.par_iter_mut()
          .enumerate()
          .for_each(|(_, v)| *v = program(*v) + value);
   });
   black_box(d);

   println!("NO {no_par:?}, MINE {my_par:?}, RAYON {rayon:?}");
}

fn program(input: usize) -> usize {
   ((input as f64)
       .sqrt()
       .powi(32)
       .to_radians()
       .to_degrees()
       .to_radians()
       .abs()
       + 3.0) as usize
}

fn get_work(cores: usize, len: usize) -> Vec<Range<usize>> {
   let jobs = if len < cores { len } else { cores };

   let main = len / cores;
   let mut rem = len % cores;
   let mut out = Vec::with_capacity(cores);
   for _ in 0..jobs {
      if rem > 0 {
         out.push(main + 1);
         rem -= 1;
      } else {
         out.push(main);
      }
   }
   let mut lts = 0;
   out.into_iter()
       .map(|v| {
          let o = lts..(lts + v);
          lts = o.end;
          o
       })
       .collect::<Vec<Range<usize>>>()
}


pub struct C<T>(T);
unsafe impl<T> Send for C<T> {}
unsafe impl<T> Sync for C<T> {}
impl<T> Deref for C<T> {
   type Target = T;

   fn deref(&self) -> &Self::Target {
      &self.0
   }
}
impl<T> DerefMut for C<T> {
   fn deref_mut(&mut self) -> &mut Self::Target {
      &mut self.0
   }
}