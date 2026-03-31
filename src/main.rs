// Reproducer for a bug in wasmtime 43's StringPool::try_clone().
//
// Root cause
// ----------
// Linker<T> interns import names into a StringPool to use cheap integer Atom
// indices as HashMap keys.  StringPool stores two parallel structures:
//
//   strings: Vec<Box<str>>          — owns the string data
//   map:     HashMap<&'static str, Atom>  — keys are transmuted borrows of
//                                           the boxes above
//
// When StringPool is cloned (via TryClone, called by Linker::clone()),
// `strings.try_clone()` allocates *new* Box<str> objects, but
// `map.try_clone()` copies the old &'static str pointers that still point
// into the *original* pool's memory.  Once the original Linker is dropped,
// those map keys dangle, and any subsequent call that looks up a string
// (e.g. instantiate_pre resolving a module's imports) triggers
// use-after-free and silently returns "not found".
//
// How to observe the bug
// ----------------------
// 1. Create a Linker and register a host function ("env"."answer").
// 2. Clone the Linker with `linker.clone()`.
// 3. Drop the original Linker — its StringPool::strings are freed, leaving
//    dangling &'static str keys in the clone's map.
// 4. Call `instantiate_pre` on the clone: it calls pool.get_atom("env"),
//    accesses freed memory, gets None, and returns the error
//    "unknown import: `env::answer` has not been defined".
//
// Expected: both calls to instantiate_pre succeed and return 42.
// Actual (wasmtime 43): the second call fails with "unknown import".

use wasmtime::{Engine, Linker, Module, Store};

// A tiny WAT module that imports `env.answer() -> i32` and re-exports it
// as `run`.
const WAT: &str = r#"
(module
  (import "env" "answer" (func $answer (result i32)))
  (func (export "run") (result i32)
    call $answer
  )
)
"#;

fn main() -> wasmtime::Result<()> {
  let engine = Engine::default();
  let module = Module::new(&engine, WAT)?;

  // --- Step 1: build the original linker and register the host function ---
  let original_linker = {
    let mut linker: Linker<()> = Linker::new(&engine);
    linker.func_wrap("env", "answer", || -> i32 { 42 })?;
    linker
  };

  // --- Step 2: clone the linker, then drop the original ---
  let cloned_linker = original_linker.clone();
  drop(original_linker); // StringPool::strings freed; clone's map now dangles

  // --- Step 3: instantiate_pre on the clone ---
  // On wasmtime 43 this fails with:
  //   "unknown import: `env::answer` has not been defined"
  // because pool.get_atom("env") dereferences a dangling &'static str.
  let instance_pre = cloned_linker.instantiate_pre(&module)?;

  // --- Step 4: run the wasm function and check the result ---
  let mut store = Store::new(&engine, ());
  let instance = instance_pre.instantiate(&mut store)?;
  let run = instance.get_typed_func::<(), i32>(&mut store, "run")?;
  let result = run.call(&mut store, ())?;

  assert_eq!(result, 42, "expected 42 (the answer), got {result}");
  println!("run() returned {result} — the answer to life, the universe, and everything.");
  Ok(())
}
