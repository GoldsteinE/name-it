So, you have a nice `async fn` and you want to store a future it returns in a struct. There’s
no need for boxing or dynamic dispatch: you statically know the type. You just need to...

# name-it

```rust
#[name_it(Test)]
async fn add(x: i32, y: i32) -> i32 {
    do_something_very_async().await;
    x + y
}

let foo: Test = add(2, 3);
assert_eq!(block_on(foo), 5);
```

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

1. It can’t be directly applied to a method or associated function. You can move the body of the method into a free function and make your method a thin wrapper.

2. It doesn’t support functions with _type_ generics. This is caused by limitations on using generics in const context.

### (Probably) solvable

1. It doesn't currently support explicit _lifetime_ generics either. There's no fundamental problem with it, it just needs to be implemented in the macros (help wanted!).

2. All arguments must be simple identifiers (so things like `(x, y): (i32, i32)` are not allowed). Again, this should be possible to implement, it's just not implemented yet.

3. While the underlying trick could work on most `impl Trait` types, this crate only implements it for `async fn`. It's not clear how to make the macro work for any trait.

## What about `stackfuture`?

Microsoft's [stackfuture] indeed takes a similar approach to solve this problem. Key differences:

1. `name-it` automatically infers type size, alignment and auto traits. `stackfuture` has fixed alignment, manually specified size and is always `Send + !Sync`.

2. `stackfuture` doesn't use macros, so is friendlier to tooling, and supports generics. It can also be directly used in methods, including trait methods.

3. `stackfuture` uses dynamic dispatch. `name-it` is fully static, there're no dynamic function calls.

[stackfuture]: https://github.com/microsoft/stackfuture/
[stackfuture is unsound]: https://github.com/microsoft/stackfuture/issues/9
# License

Blue Oak Model License 1.0.0 is permissive, non-copyleft license. If rare licenses unnerve you, you can use it under Apache 2.0.

Files `tests/futures-*` are from the [futures-rs] project.

[futures-rs]: https://github.com/rust-lang/futures-rs
