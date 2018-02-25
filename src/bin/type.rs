//! type.rs is a sample typechecker
//! written in rust.

/// The tag enum holds the
/// information which type or
/// type combination is under
/// consideration.
#[derive(Debug)]
enum Tag {
    Foo,
    Bar,
    Baz,
    Sum,
    Fun
}

/// This generic struct represents
/// plain types or combination of types
/// for example the sum [A+B] or fun (A->B) type
#[derive(Debug)]
struct Type {
    tag: Tag,
    left: Option<Box<Type>>,
    right: Option<Box<Type>>
}

impl Type {
    /// Create a basic type
    fn basic(tag: Tag) -> Type {
        Type {
            tag: tag,
            left: None,
            right: None
        }
    }

    /// Create a sum type with left and right side 
    fn sum(left: Type, right: Type) -> Type {
        Type {
            tag: Tag::Sum,
            left: Some(Box::new(left)),
            right: Some(Box::new(right))
        }
    }

    /// create a function type where the input is transformed
    fn fun(input: Type, ret: Type) -> Type {
        Type {
            tag: Tag::Fun,
            left: Some(Box::new(input)),
            right: Some(Box::new(ret))
        }
    }
}

/// Implement equality for types
fn judgement(t: &Type) -> bool {
    match t {
        &Type { ref tag, left: None, right: None } => {
            match tag {
                &Tag::Foo | &Tag::Bar | &Tag::Baz => true,
                _ => false
            }
        },
        &Type { tag: Tag::Sum, left: Some(ref l), right: Some(ref r)} => {
            judgement(&l) && judgement(&r)
        },
        &Type { tag: Tag::Fun, left: Some(ref l), right: Some(ref r)} => {
            println!("in" );
            judgement(&l) && judgement(&r)
        },
        _ => false,
    }
}

fn main() {
    println!("Hello");
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn valid_judgement() {
        let foo = Type::basic(Tag::Foo);
        assert!(judgement(&foo), true);

        let bar = Type::basic(Tag::Bar);
        assert!(judgement(&bar), true);

        let baz  = Type::basic(Tag::Baz);
        assert!(judgement(&baz), true);

        let sum = Type::sum(foo, bar);
        assert!(judgement(&sum), true);

        let fun = Type::fun(baz, Type::basic(Tag::Foo));
        assert!(judgement(&fun), true);
    }

    #[test]
    fn invalid_judgment() {
        let t = Type {tag: Tag::Fun, left: None, right: None};
        assert!(judgement(&t), false);
        // fails and panics at pattern matching, I'm in doubt if
        // it is a bug in rust
        let t = Type {tag: Tag::Fun,
                      left: Some(Type::basic(Tag::Fun)),
                      right: Some(Type::basic(Tag::Sum))};
        // panics again
        assert!(judgement(&t), false);
    }
}
