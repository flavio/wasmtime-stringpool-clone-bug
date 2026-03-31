# wasmtime-stringpool-clone-bug

Reproducer for a soundness bug introduced in wasmtime 43.0:

> **fix(environ): repair unsound `StringPool::try_clone()`**
>
> The 43.0 release introduced a soundness bug in `StringPool::try_clone()`: the
> cloned map retains `&'static str` keys pointing into the original pool's
> strings storage. Once the original `Linker` is dropped those keys dangle.
>
> Cloning a `Linker`, then dropping the original one, leaves a linker whose
> registered imports could no longer be found, causing instantiation to fail
> with "unknown import".

## What the reproducer does

1. Creates a `Linker` and registers a host function (`env.answer() -> i32`).
2. Clones the `Linker`, then drops the original — freeing the original pool's
   string storage and leaving dangling pointers in the clone's map.
3. Calls `instantiate_pre` on the clone to resolve imports.
4. Runs the wasm function and asserts the result is `42`.

On wasmtime **42.0** all steps succeed. On wasmtime **43.0** step 3 fails with:

```
unknown import: `env::answer` has not been defined
```

## Running

Two Makefile targets are provided, each using an isolated manifest and build
directory:

```sh
# Run with wasmtime 42.0 (succeeds)
make run-v42

# Run with wasmtime 43.0 (reproduces the bug)
make run-v43

# Run both in sequence
make
```

Requirements: a working Rust toolchain with `cargo`.

## License

MIT — see [LICENSE](LICENSE).
