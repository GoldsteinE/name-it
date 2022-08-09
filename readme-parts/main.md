Function attributes (including doc comments) are preserved. Created type will have the same visibility as the function itself and the same size, alignment, `Send` and `Sync` as the original future.

**MSRV** is 1.61. As far as I know, it’s impossible to make it work on older Rust versions.

## Safety

I don't see why this would be unsound. Miri likes it, I discussed it with other people, and all unsafe involved isn't particulary criminal.

To address some particular concerns:

1. Transmuting any type to an array of `MaybeUninit<u8>` and back is currently considered sound.

2. Alignment of the generated type is preserved.

3. Generated type is never used unpinned, except in destructor, and is never moved in destructor.

4. Lifetime of the generated type is tied to the lifetimes of every input, so use-after-free is not possible.

Nonetheless, I can't be completely sure that it is sound yet. If you find any soundness problems, please file an issue.

## Limitations

### Absolute

1. It emulates TAITs, but it can’t emulate GATs, so async trait methods capturing `self` by ref are no-go.

2. It can’t be directly applied to a method or associated function. You can move the body of the method into a free function and make your method a thin wrapper.

### (Probably) solvable

1. It doesn't currently support generics. There's no fundamental problem with it, it just needs to be implemented in the macros (help wanted!).

2. All arguments must be simple identifiers (so things like `(x, y): (i32, i32)` are not allowed). Again, this should be possible to implement, it's just not implemented yet.

3. While the underlying trick could work on most `impl Trait` types, this crate only implements it for `async fn`. It's not clear how to make the macro work for any trait.

