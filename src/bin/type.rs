//! type.rs is a sample typechecker
//! written in rust.

/// This enum is either a basic type
/// or a sum [a+b] and [i->o]
#[derive(Debug, Clone)]
enum Type {
    Foo,
    Bar,
    Baz,
    Sum(Box<Type>, Box<Type>), // left + right
    Fn(Box<Type>, Box<Type>),  // input -> output
}

/// Implement equality for types
fn judgment(t: &Type) -> bool {
    match t {
        &Type::Foo | &Type::Bar | &Type::Baz => true,
        &Type::Sum(ref l, ref r) => judgment(&l) && judgment(&r),
        &Type::Fn(ref i, ref o) => judgment(&i) && judgment(&o),
    }
}

/// Check if two types are equal
fn type_equality(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (&Type::Foo, &Type::Foo) | (&Type::Bar, &Type::Bar) | (&Type::Baz, &Type::Baz) => true,
        (&Type::Sum(ref al, ref ar), &Type::Sum(ref bl, ref br)) => {
            type_equality(&al, &bl) && type_equality(&ar, &br)
        }
        (&Type::Fn(ref ai, ref ao), &Type::Fn(ref bi, ref bo)) => {
            type_equality(&ai, &bi) && type_equality(&ao, &bo)
        }
        _ => false,
    }
}

/// Context struct holds process information
/// about the type checking process.
#[derive(Debug, Clone)]
enum Context {
    Empty(),
    Snoc(Box<Context>, String, Type),
}

/// Test if a name is present within the context.
fn not_in(name: &str, g: &Context) -> bool {
    match g {
        &Context::Empty() => true,
        &Context::Snoc(_, ref n, _) if n == name => false,
        &Context::Snoc(ref rest, _, _) => not_in(name, &rest),
    }
}

/// Test if the context is valid.
fn judgment_ctx(g: &Context) -> bool {
    match g {
        &Context::Empty() => true,
        &Context::Snoc(ref rest, ref name, ref t) => {
            judgment_ctx(&rest) && judgment(&t) && not_in(&name, &rest)
        }
    }
}

/// Represent different terms which can be recursive.
enum Term {
    Pair(Box<Term>, Box<Term>),
    Split(Box<Term>, String, Type, String, Type, Box<Term>),
    Lam(String, Box<Term>),
    App(Box<Term>, Box<Term>, Type),
    Var(String),
}

/// Check if a name has a type within the context.
fn var_has_type(v: &str, a: &Type, g: &Context) -> bool {
    match g {
        &Context::Empty() => false,
        &Context::Snoc(ref rest, ref name, ref t) => match name == v {
            true => type_equality(a, &t),
            false => var_has_type(v, a, &rest),
        },
    }
}

/// The real type checking process.
fn judgment_check(g: &Context, term: &Term, t: &Type) -> bool {
    match (term, t) {
        (&Term::Pair(ref fst, ref snd), &Type::Sum(ref l, ref r)) => {
            judgment_check(g, fst, &l) && judgment_check(g, snd, &r)
        }
        (&Term::Split(ref pair, ref name_a, ref type_a, ref name_b, ref type_b, ref body), _) => {
            let ctx = Context::Snoc(
                Box::new(Context::Snoc(
                    Box::new((*g).clone()),
                    name_a.clone(),
                    type_a.clone(),
                )),
                name_b.clone(),
                type_b.clone(),
            );
            judgment_check(
                g,
                pair,
                &Type::Sum(Box::new(type_a.clone()), Box::new(type_b.clone())),
            ) && judgment_check(&ctx, body, t)
        }
        (&Term::Lam(ref name, ref body), &Type::Fn(ref i, ref o)) => judgment_check(
            &Context::Snoc(Box::new((*g).clone()), name.clone(), *i.clone()),
            body,
            &o,
        ),
        (&Term::App(ref fun, ref arg, ref type_arg), _) => {
            judgment_check(
                g,
                fun,
                &Type::Fn(Box::new(type_arg.clone()), Box::new(t.clone())),
            ) && judgment_check(g, arg, type_arg)
        }
        (&Term::Var(ref name), _) => var_has_type(name, t, g),
        _ => false,
    }
}

fn main() {
    // the identity function for foo
    //  !- \x. x : Foo -> Foo
    let ctx = Context::Empty();
    let term = Term::Lam("x".into(), Box::new(Term::Var("x".into())));
    let t = Type::Fn(Box::new(Type::Foo), Box::new(Type::Foo));
    assert!(judgment_check(&ctx, &term, &t));

    // the fst function
    // !- \p. split p as (x :: Foo, y :: Bar) in x : Foo*Bar -> Foo
    let ctx = Context::Empty();
    let term = Term::Lam(
        "p".into(),
        Box::new(Term::Split(
            Box::new(Term::Var("p".into())),
            "x".into(),
            Type::Foo,
            "y".into(),
            Type::Bar,
            Box::new(Term::Var("x".into())),
        )),
    );
    let t = Type::Fn(
        Box::new(Type::Sum(Box::new(Type::Foo), Box::new(Type::Bar))),
        Box::new(Type::Foo),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // the const function
    // !- \x. \y. x : Foo -> Bar -> Foo
    let ctx = Context::Empty();
    let term = Term::Lam(
        "x".into(),
        Box::new(Term::Lam("y".into(), Box::new(Term::Var("x".into())))),
    );
    let t = Type::Fn(
        Box::new(Type::Foo),
        Box::new(Type::Fn(Box::new(Type::Bar), Box::new(Type::Foo))),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // the apply function
    // !- \f. \x. f x : (Foo -> Bar) -> Foo -> Bar
    let ctx = Context::Empty();
    let term = Term::Lam(
        "f".into(),
        Box::new(Term::Lam(
            "x".into(),
            Box::new(Term::App(
                Box::new(Term::Var("f".into())),
                Box::new(Term::Var("x".into())),
                Type::Foo,
            )),
        )),
    );
    let t = Type::Fn(
        Box::new(Type::Fn(Box::new(Type::Foo), Box::new(Type::Bar))),
        Box::new(Type::Fn(Box::new(Type::Foo), Box::new(Type::Bar))),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // the continuize function or reverse apply function
    // !- \x. \f. f x : Foo -> (Foo -> Bar) -> Bar
    let ctx = Context::Empty();
    let term = Term::Lam(
        "x".into(),
        Box::new(Term::Lam(
            "f".into(),
            Box::new(Term::App(
                Box::new(Term::Var("f".into())),
                Box::new(Term::Var("x".into())),
                Type::Foo,
            )),
        )),
    );
    let t = Type::Fn(
        Box::new(Type::Foo),
        Box::new(Type::Fn(
            Box::new(Type::Fn(Box::new(Type::Foo), Box::new(Type::Bar))),
            Box::new(Type::Bar),
        )),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // currying
    // !- \f. \x. \y. f (x,y) : (Foo*Bar -> Baz) -> Foo -> Bar -> Baz
    let ctx = Context::Empty();
    let term = Term::Lam(
        "f".into(),
        Box::new(Term::Lam(
            "x".into(),
            Box::new(Term::Lam(
                "y".into(),
                Box::new(Term::App(
                    Box::new(Term::Var("f".into())),
                    Box::new(Term::Pair(
                        Box::new(Term::Var("x".into())),
                        Box::new(Term::Var("y".into())),
                    )),
                    Type::Sum(Box::new(Type::Foo), Box::new(Type::Bar)),
                )),
            )),
        )),
    );
    let t = Type::Fn(
        Box::new(Type::Fn(
            Box::new(Type::Sum(Box::new(Type::Foo), Box::new(Type::Bar))),
            Box::new(Type::Baz),
        )),
        Box::new(Type::Fn(
            Box::new(Type::Foo),
            Box::new(Type::Fn(Box::new(Type::Bar), Box::new(Type::Baz))),
        )),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // uncurrying
    // !- \f. \p. split p as (x :: Foo, y :: Bar) in f x y
    //  : (Foo -> Bar -> Baz) -> Foo*Bar -> Baz
    let ctx = Context::Empty();
    let term = Term::Lam(
        "f".into(),
        Box::new(Term::Lam(
            "p".into(),
            Box::new(Term::Split(
                Box::new(Term::Var("p".into())),
                "x".into(),
                Type::Foo,
                "y".into(),
                Type::Bar,
                Box::new(Term::App(
                    Box::new(Term::App(
                        Box::new(Term::Var("f".into())),
                        Box::new(Term::Var("x".into())),
                        Type::Foo,
                    )),
                    Box::new(Term::Var("y".into())),
                    Type::Bar,
                )),
            )),
        )),
    );
    let t = Type::Fn(
        Box::new(Type::Fn(
            Box::new(Type::Foo),
            Box::new(Type::Fn(Box::new(Type::Bar), Box::new(Type::Baz))),
        )),
        Box::new(Type::Fn(
            Box::new(Type::Sum(Box::new(Type::Foo), Box::new(Type::Bar))),
            Box::new(Type::Baz),
        )),
    );
    assert!(judgment_check(&ctx, &term, &t));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_judgement() {
        let foo = Type::Foo;
        assert!(judgment(&foo));

        let bar = Type::Bar;
        assert!(judgment(&bar));

        let baz = Type::Baz;
        assert!(judgment(&baz));

        let sum = Type::Sum(Box::new(Type::Foo), Box::new(Type::Bar));
        assert!(judgment(&sum));

        let fun = Type::Fn(Box::new(baz), Box::new(foo));
        assert!(judgment(&fun));
    }
}
