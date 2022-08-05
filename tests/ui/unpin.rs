use name_it::name_it;

#[name_it(Foo)]
async fn foo() {
}

impl Unpin for _foo_impl::Foo<'_> {}

fn assert_unpin(_: impl Unpin) {}

fn main() {
    assert_unpin(foo());
}
