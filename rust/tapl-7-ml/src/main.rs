#[derive(Debug, Clone, PartialEq)]
enum Term {
    Var(usize, usize),       // deBruijn_index * term_length
    Abs(String, Box<Term>),  // bound_var_name * partial_term
    App(Box<Term>, Box<Term>), // arg_term * applying_term
    Wrong,                   // Error handling
}

#[derive(Debug, Clone)]
enum Binding {
    NameBind,
}

type Context = Vec<(String, Binding)>;

fn ctxlength(ctx: &Context) -> usize {
    ctx.len()
}

fn index2name(ctx: &Context, n: usize) -> Option<String> {
    if n < ctxlength(ctx) {
        Some(ctx[n].0.clone())
    } else {
        None
    }
}

fn get_or_else<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(v) => v,
        None => default,
    }
}

fn listfind<'a, T, F>(lst: &'a [T], predicate: F) -> Option<&'a T>
where
    F: Fn(&'a T) -> bool,
{
    lst.iter().find(predicate)
}

fn pickfreshname(ctx: &Context, name: String) -> (Context, String) {
    let oldname = listfind(ctx, |(n, _)| n == &name);
    match oldname {
        Some((name, _)) => pickfreshname(ctx, format!("{}'", name)),
        None => {
            let mut new_ctx = ctx.clone();
            new_ctx.push((name.clone(), Binding::NameBind));
            (new_ctx, name)
        }
    }
}

fn string_of_term(ctx: &Context, t: &Term) -> String {
    match t {
        Term::Var(x, n) => {
            if ctxlength(ctx) == *n {
                get_or_else(index2name(ctx, *x), "[bad index]".to_string())
            } else {
                "[bad index]".to_string()
            }
        }
        Term::Abs(name, tm) => {
            let (ctx', name') = pickfreshname(ctx, name.clone());
            format!("(lambda {}. {})", name', string_of_term(&ctx', tm))
        }
        Term::App(t1, t2) => {
            format!("({} {})", string_of_term(ctx, t1), string_of_term(ctx, t2))
        }
        Term::Wrong => "[Wrong Evaluation]".to_string(),
    }
}

fn term_shift(d: isize, t: &Term) -> Term {
    fn walk(c: isize, t: &Term) -> Term {
        match t {
            Term::Var(x, ctxlen) if (*x as isize) < c => Term::Var(*x, ((*ctxlen as isize) + d) as usize),
            Term::Var(x, ctxlen) => Term::Var(((*x as isize) + d) as usize, ((*ctxlen as isize) + d) as usize),
            Term::Abs(name, t) => Term::Abs(name.clone(), Box::new(walk(c + 1, t))),
            Term::App(t1, t2) => Term::App(Box::new(walk(c, t1)), Box::new(walk(c, t2))),
            Term::Wrong => Term::Wrong,
        }
    }
    walk(0, t)
}

fn term_subst(j: usize, s: &Term, t: &Term) -> Term {
    match t {
        Term::Var(k, _) if *k == j => s.clone(),
        Term::Var(_, _) => t.clone(),
        Term::Abs(name, t) => Term::Abs(name.clone(), Box::new(term_subst(j + 1, &term_shift(1, s), t))),
        Term::App(t1, t2) => Term::App(Box::new(term_subst(j, s, t1)), Box::new(term_subst(j, s, t2))),
        Term::Wrong => Term::Wrong,
    }
}

fn term_subst_top(s: &Term, t: &Term) -> Term {
    let s_shifted = term_shift(1, s);
    let t_subst = term_subst(0, &s_shifted, t);
    term_shift(-1, &t_subst)
}

fn is_val(_ctx: &Context, t: &Term) -> bool {
    matches!(t, Term::Abs(_, _))
}

fn eval1(ctx: &Context, t: &Term) -> Term {
    match t {
        Term::App(box Term::Abs(_, t12), v2) if is_val(ctx, v2) => term_subst_top(v2, t12),
        Term::App(v1, t2) if is_val(ctx, v1) => {
            let t2 = eval1(ctx, t2);
            Term::App(Box::new(v1.clone()), Box::new(t2))
        }
        Term::App(t1, t2) => {
            let t1 = eval1(ctx, t1);
            Term::App(Box::new(t1), Box::new(t2.clone()))
        }
        _ => Term::Wrong,
    }
}

fn eval(ctx: &Context, t: &Term) -> Term {
    let t = eval1(ctx, t);
    if t == Term::Wrong {
        t.clone()
    } else {
        eval(ctx, &t)
    }
}


fn main() {
    let testcases = vec![
        // (λ x. x) λ y. y => λ y. y
        (
            Term::App(
                Box::new(Term::Abs("x".to_string(), Box::new(Term::Var(0, 1)))),
                Box::new(Term::Abs("y".to_string(), Box::new(Term::Var(0, 1)))),
            ),
            Term::Abs("y".to_string(), Box::new(Term::Var(0, 1))),
        ),
        // 他のテストケースを追加する...
    ];

    for (i, (tsrc, texpect)) in testcases.iter().enumerate() {
        let tactual = eval1(&vec![], tsrc);
        let result = if *tactual == *texpect {
            "OK"
        } else {
            "**FAILURE**"
        };
        println!(
            "{}. {} => {}\n{}\n",
            i + 1,
            string_of_term(&vec![], tsrc),
            string_of_term(&vec![], &tactual),
            result
        );
    }
}
