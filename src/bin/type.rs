//! type.rs is a sample typechecker
//! written in rust.
use std::option::Option;

/// The tag enum holds the
/// information which type,
/// type combination or context
/// is under consideration.
#[derive(Debug, PartialEq, Clone)]
enum Tag {
    // base types
    Foo,
    Bar,
    Baz,
    // type combinations
    Sum,
    Fun,
    // context
    Empty,
    Snoc,
    // term reps
    Pair,
    Split,
    Lam,
    App,
    Var,
}

/// This generic struct represents
/// plain types or combination of types
/// for example the sum [A+B] or fun (A->B) type
#[derive(Debug, Clone)]
struct Type {
    tag: Tag,
    left: Option<Box<Type>>,
    right: Option<Box<Type>>,
}

impl Type {
    /// Create a basic type
    fn basic(tag: Tag) -> Type {
        Type {
            tag: tag,
            left: None,
            right: None,
        }
    }

    /// Create a sum type with left and right side
    fn sum(left: Type, right: Type) -> Type {
        Type {
            tag: Tag::Sum,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    /// create a function type where the input is transformed
    fn fun(input: Type, ret: Type) -> Type {
        Type {
            tag: Tag::Fun,
            left: Some(Box::new(input)),
            right: Some(Box::new(ret)),
        }
    }
}

/// Implement equality for types
fn judgment(t: &Type) -> bool {
    match t {
        &Type {
            ref tag,
            left: None,
            right: None,
        } => match tag {
            &Tag::Foo | &Tag::Bar | &Tag::Baz => true,
            _ => false,
        },
        &Type {
            tag: Tag::Sum,
            left: Some(ref l),
            right: Some(ref r),
        } => judgment(&l) && judgment(&r),
        &Type {
            tag: Tag::Fun,
            left: Some(ref l),
            right: Some(ref r),
        } => judgment(&l) && judgment(&r),
        _ => false,
    }
}

/// Check if two types are equal
fn type_equality(a: &Type, b: &Type) -> bool {
    if (Tag::Foo == a.tag && Tag::Foo == b.tag) || (Tag::Bar == a.tag && Tag::Bar == b.tag)
        || (Tag::Baz == a.tag && Tag::Baz == b.tag)
    {
        true
    } else if (Tag::Sum == a.tag && Tag::Sum == b.tag) || (Tag::Fun == a.tag && Tag::Fun == b.tag) {
        type_equality(a.left.as_ref().unwrap(), b.left.as_ref().unwrap())
            && type_equality(a.right.as_ref().unwrap(), b.right.as_ref().unwrap())
    } else {
        false
    }
}

/// Context struct holds process information
/// about the type checking process.
#[derive(Debug, Clone)]
struct Context {
    tag: Tag,
    rest: Option<Box<Context>>,
    name: Option<String>,
    t: Option<Type>,
}

/// Create either an empty context
/// or a nested one.
impl Context {
    fn empty() -> Context {
        Context {
            tag: Tag::Empty,
            rest: None,
            name: None,
            t: None,
        }
    }

    fn snoc(g: Context, name: String, t: Type) -> Context {
        Context {
            tag: Tag::Snoc,
            rest: Some(Box::new(g)),
            name: Some(name),
            t: Some(t),
        }
    }
}

/// Test if a name is present within the context.
fn not_in(name: &str, g: &Context) -> bool {
    match g {
        &Context {
            tag: Tag::Empty, ..
        } => true,
        &Context {
            name: Some(ref name),
            ..
        } => false,
        &Context {
            rest: Some(ref g), ..
        } => not_in(name, &g),
        _ => true, // if it didn't match it's not in context
    }
}

/// Test if the context is valid.
fn judgment_ctx(g: &Context) -> bool {
    match g {
        &Context {
            tag: Tag::Empty, ..
        } => true,
        &Context {
            tag: Tag::Snoc,
            rest: Some(ref r),
            t: Some(ref t),
            name: Some(ref name),
        } => judgment_ctx(&r) && judgment(&t) && not_in(name, &r),
        _ => false,
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
        &Context {
            tag: Tag::Empty, ..
        } => false,
        &Context {
            tag: Tag::Snoc,
            rest: Some(ref r),
            t: Some(ref t),
            name: Some(ref name),
        } => match v == name {
            true => type_equality(a, t),
            false => var_has_type(v, a, r),
        },
        _ => false,
    }
}

/// The real type checking process.
fn judgment_check(g: &Context, term: &Term, t: &Type) -> bool {
    match term {
        &Term::Pair(ref fst, ref snd) if t.tag == Tag::Sum => {
            judgment_check(g, fst, t.left.as_ref().unwrap())
                && judgment_check(g, snd, t.right.as_ref().unwrap())
        }
        &Term::Split(ref pair, ref name_a, ref type_a, ref name_b, ref type_b, ref body) => {
            let ctx = Context::snoc(
                Context::snoc(g.clone(), name_a.clone(), type_a.clone()),
                name_b.clone(),
                type_b.clone(),
            );
            judgment_check(g, pair, &Type::sum(type_a.clone(), type_b.clone()))
                && judgment_check(&ctx, body, t)
        }
        &Term::Lam(ref name, ref body) if t.tag == Tag::Fun => judgment_check(
            &Context::snoc(g.clone(), name.clone(), *t.left.clone().unwrap()),
            body,
            t.right.as_ref().unwrap(),
        ),
        &Term::App(ref fun, ref arg, ref type_arg) => {
            judgment_check(g, fun, &Type::fun(type_arg.clone(), t.clone()))
                && judgment_check(g, arg, type_arg)
        }
        &Term::Var(ref name) => var_has_type(name, t, g),
        _ => false,
    }
}

fn main() {
    // the identity function for foo
    //  !- \x. x : Foo -> Foo
    let ctx = Context::empty();
    let term = Term::Lam("x".into(), Box::new(Term::Var("x".into())));
    let t = Type::fun(Type::basic(Tag::Foo), Type::basic(Tag::Foo));
    assert!(judgment_check(&ctx, &term, &t));

    // the fst function
    // !- \p. split p as (x :: Foo, y :: Bar) in x : Foo*Bar -> Foo
    let ctx = Context::empty();
    let term = Term::Lam(
        "p".into(),
        Box::new(Term::Split(
            Box::new(Term::Var("p".into())),
            "x".into(),
            Type::basic(Tag::Foo),
            "y".into(),
            Type::basic(Tag::Bar),
            Box::new(Term::Var("x".into())),
        )),
    );
    let t = Type::fun(
        Type::sum(Type::basic(Tag::Foo), Type::basic(Tag::Bar)),
        Type::basic(Tag::Foo),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // the const function
    // !- \x. \y. x : Foo -> Bar -> Foo
    let ctx = Context::empty();
    let term = Term::Lam(
        "x".into(),
        Box::new(Term::Lam("y".into(), Box::new(Term::Var("x".into())))),
    );
    let t = Type::fun(
        Type::basic(Tag::Foo),
        Type::fun(Type::basic(Tag::Bar), Type::basic(Tag::Foo)),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // the apply function
    // !- \f. \x. f x : (Foo -> Bar) -> Foo -> Bar
    let ctx = Context::empty();
    let term = Term::Lam(
        "f".into(),
        Box::new(Term::Lam(
            "x".into(),
            Box::new(Term::App(
                Box::new(Term::Var("f".into())),
                Box::new(Term::Var("x".into())),
                Type::basic(Tag::Foo),
            )),
        )),
    );
    let t = Type::fun(
        Type::fun(Type::basic(Tag::Foo), Type::basic(Tag::Bar)),
        Type::fun(Type::basic(Tag::Foo), Type::basic(Tag::Bar)),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // the continuize function or reverse apply function
    // !- \x. \f. f x : Foo -> (Foo -> Bar) -> Bar
    let ctx = Context::empty();
    let term = Term::Lam(
        "x".into(),
        Box::new(Term::Lam(
            "f".into(),
            Box::new(Term::App(
                Box::new(Term::Var("f".into())),
                Box::new(Term::Var("x".into())),
                Type::basic(Tag::Foo),
            )),
        )),
    );
    let t = Type::fun(
        Type::basic(Tag::Foo),
        Type::fun(
            Type::fun(Type::basic(Tag::Foo), Type::basic(Tag::Bar)),
            Type::basic(Tag::Bar),
        ),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // currying
    // !- \f. \x. \y. f (x,y) : (Foo*Bar -> Baz) -> Foo -> Bar -> Baz
    let ctx = Context::empty();
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
                    Type::sum(Type::basic(Tag::Foo), Type::basic(Tag::Bar)),
                )),
            )),
        )),
    );
    let t = Type::fun(
        Type::fun(
            Type::sum(Type::basic(Tag::Foo), Type::basic(Tag::Bar)),
            Type::basic(Tag::Baz),
        ),
        Type::fun(
            Type::basic(Tag::Foo),
            Type::fun(Type::basic(Tag::Bar), Type::basic(Tag::Baz)),
        ),
    );
    assert!(judgment_check(&ctx, &term, &t));

    // uncurrying
    // !- \f. \p. split p as (x :: Foo, y :: Bar) in f x y
    //  : (Foo -> Bar -> Baz) -> Foo*Bar -> Baz
    let ctx = Context::empty();
    let term = Term::Lam(
        "f".into(),
        Box::new(Term::Lam(
            "p".into(),
            Box::new(Term::Split(
                Box::new(Term::Var("p".into())),
                "x".into(),
                Type::basic(Tag::Foo),
                "y".into(),
                Type::basic(Tag::Bar),
                Box::new(Term::App(
                    Box::new(Term::App(
                        Box::new(Term::Var("f".into())),
                        Box::new(Term::Var("x".into())),
                        Type::basic(Tag::Foo),
                    )),
                    Box::new(Term::Var("y".into())),
                    Type::basic(Tag::Bar),
                )),
            )),
        )),
    );
    let t = Type::fun(
        Type::fun(
            Type::basic(Tag::Foo),
            Type::fun(Type::basic(Tag::Bar), Type::basic(Tag::Baz)),
        ),
        Type::fun(
            Type::sum(Type::basic(Tag::Foo), Type::basic(Tag::Bar)),
            Type::basic(Tag::Baz),
        ),
    );
    assert!(judgment_check(&ctx, &term, &t));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_judgement() {
        let foo = Type::basic(Tag::Foo);
        assert!(judgment(&foo));

        let bar = Type::basic(Tag::Bar);
        assert!(judgment(&bar));

        let baz = Type::basic(Tag::Baz);
        assert!(judgment(&baz));

        let sum = Type::sum(foo, bar);
        assert!(judgment(&sum));

        let fun = Type::fun(baz, Type::basic(Tag::Foo));
        assert!(judgment(&fun));
    }

    #[test]
    fn invalid_judgment() {
        let t = Type {
            tag: Tag::Fun,
            left: None,
            right: None,
        };
        assert_eq!(judgment(&t), false);

        let t = Type {
            tag: Tag::Fun,
            left: Some(Box::new(Type::basic(Tag::Fun))),
            right: Some(Box::new(Type::basic(Tag::Sum))),
        };
        assert_eq!(judgment(&t), false);
    }
}
