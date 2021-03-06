use super::*;

#[test]
fn field_map_reorder_test() {
    let int = |value: &str| UntypedExpr::Int {
        value: value.to_string(),
        location: SrcSpan { start: 0, end: 0 },
    };

    struct Case {
        arity: usize,
        fields: HashMap<String, usize>,
        args: Vec<CallArg<UntypedExpr>>,
        expected_result: Result<(), Error>,
        expected_args: Vec<CallArg<UntypedExpr>>,
    }

    impl Case {
        fn test(self) {
            let mut args = self.args;
            let fm = FieldMap {
                arity: self.arity,
                fields: self.fields,
            };
            let location = &SrcSpan { start: 0, end: 0 };
            assert_eq!(self.expected_result, fm.reorder(&mut args, location));
            assert_eq!(self.expected_args, args);
        }
    }

    Case {
        arity: 0,
        fields: HashMap::new(),
        args: vec![],
        expected_result: Ok(()),
        expected_args: vec![],
    }
    .test();

    Case {
        arity: 3,
        fields: HashMap::new(),
        args: vec![
            CallArg {
                location: Default::default(),
                label: None,
                value: int("1"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("2"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("3"),
            },
        ],
        expected_result: Ok(()),
        expected_args: vec![
            CallArg {
                location: Default::default(),
                label: None,
                value: int("1"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("2"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("3"),
            },
        ],
    }
    .test();

    Case {
        arity: 3,
        fields: [("last".to_string(), 2)].iter().cloned().collect(),
        args: vec![
            CallArg {
                location: Default::default(),
                label: None,
                value: int("1"),
            },
            CallArg {
                location: Default::default(),
                label: Some("last".to_string()),
                value: int("2"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("3"),
            },
        ],
        expected_result: Ok(()),
        expected_args: vec![
            CallArg {
                location: Default::default(),
                label: None,
                value: int("1"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("3"),
            },
            CallArg {
                location: Default::default(),
                label: Some("last".to_string()),
                value: int("2"),
            },
        ],
    }
    .test();

    Case {
        arity: 3,
        fields: [("last".to_string(), 2)].iter().cloned().collect(),
        args: vec![
            CallArg {
                location: Default::default(),
                label: None,
                value: int("1"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("2"),
            },
            CallArg {
                location: Default::default(),
                label: Some("last".to_string()),
                value: int("3"),
            },
        ],
        expected_result: Ok(()),
        expected_args: vec![
            CallArg {
                location: Default::default(),
                label: None,
                value: int("1"),
            },
            CallArg {
                location: Default::default(),
                label: None,
                value: int("2"),
            },
            CallArg {
                location: Default::default(),
                label: Some("last".to_string()),
                value: int("3"),
            },
        ],
    }
    .test();
}

#[test]
fn infer_module_type_retention_test() {
    let module: UntypedModule = crate::ast::Module {
        documentation: vec![],
        name: vec!["ok".to_string()],
        statements: vec![],
        type_info: (),
    };

    let (result, _) = infer_module(module, &HashMap::new());
    let module = result.expect("Should infer OK");

    assert_eq!(
        module.type_info,
        Module {
            name: vec!["ok".to_string()],
            types: HashMap::new(), // Core type constructors like String and Int are not included
            values: HashMap::new(),
            accessors: HashMap::new(),
        }
    );
}

#[test]
fn infer_test() {
    macro_rules! assert_infer {
        ($src:expr, $typ:expr $(,)?) => {
            println!("\n{}\n", $src);
            let mut printer = pretty::Printer::new();
            let ast = crate::grammar::ExprParser::new()
                .parse($src)
                .expect("syntax error");
            let result = infer(ast, 1, &mut Env::new(&[], &HashMap::new()))
                .expect("should successfully infer");
            assert_eq!(
                ($src, printer.pretty_print(result.typ().as_ref(), 0),),
                ($src, $typ.to_string()),
            );
        };
    }

    assert_infer!("True", "Bool");
    assert_infer!("False", "Bool");
    assert_infer!("1", "Int");
    assert_infer!("-2", "Int");
    assert_infer!("1.0", "Float");
    assert_infer!("-8.0", "Float");
    assert_infer!("\"ok\"", "String");
    assert_infer!("\"ok\"", "String");
    assert_infer!("[]", "List(a)");
    assert_infer!("4 % 1", "Int");
    assert_infer!("4 > 1", "Bool");
    assert_infer!("4 >= 1", "Bool");
    assert_infer!("4 <= 1", "Bool");
    assert_infer!("4 < 1", "Bool");

    // let
    assert_infer!("let x = 1 2", "Int");
    assert_infer!("let x = 1 x", "Int");
    assert_infer!("let x = 2.0 x", "Float");
    assert_infer!("let x = 2 let y = x y", "Int");
    assert_infer!(
        "let tuple(tuple(_, _) as x, _) = tuple(tuple(0, 1.0), []) x",
        "tuple(Int, Float)"
    );

    // list
    assert_infer!("[]", "List(a)");
    assert_infer!("[1]", "List(Int)");
    assert_infer!("[1, 2, 3]", "List(Int)");
    assert_infer!("[[]]", "List(List(a))");
    assert_infer!("[[1.0, 2.0]]", "List(List(Float))");
    assert_infer!("[fn(x) { x }]", "List(fn(a) -> a)");
    assert_infer!("[fn(x) { x + 1 }]", "List(fn(Int) -> Int)");
    assert_infer!("[fn(x) { x }, fn(x) { x + 1 }]", "List(fn(Int) -> Int)");
    assert_infer!("[fn(x) { x + 1 }, fn(x) { x }]", "List(fn(Int) -> Int)");
    assert_infer!("[[], []]", "List(List(a))");
    assert_infer!("[[], [1]]", "List(List(Int))");

    assert_infer!("[1, ..[2, ..[]]]", "List(Int)");
    assert_infer!("[1 | [2 | []]]", "List(Int)"); // Deprecated syntax
    assert_infer!("[fn(x) { x }, ..[]]", "List(fn(a) -> a)");
    assert_infer!("[fn(x) { x } | []]", "List(fn(a) -> a)"); // Deprecated syntax
    assert_infer!("let x = [1, ..[]] [2, ..x]", "List(Int)");
    assert_infer!("let x = [1 | []] [2 | x]", "List(Int)"); // Deprecated syntax

    // Trailing commas
    assert_infer!("[1, ..[2, ..[],]]", "List(Int)");
    assert_infer!("[fn(x) { x },..[]]", "List(fn(a) -> a)");

    assert_infer!("let f = fn(x) { x } [f, f]", "List(fn(a) -> a)");
    assert_infer!("[tuple([], [])]", "List(tuple(List(a), List(b)))");

    // anon structs
    assert_infer!("tuple(1)", "tuple(Int)");
    assert_infer!("tuple(1, 2.0)", "tuple(Int, Float)");
    assert_infer!("tuple(1, 2.0, 3)", "tuple(Int, Float, Int)");
    assert_infer!(
        "tuple(1, 2.0, tuple(1, 1))",
        "tuple(Int, Float, tuple(Int, Int))",
    );

    // fn
    assert_infer!("fn(x) { x }", "fn(a) -> a");
    assert_infer!("fn(x) { x }", "fn(a) -> a");
    assert_infer!("fn(x, y) { x }", "fn(a, b) -> a");
    assert_infer!("fn(x, y) { [] }", "fn(a, b) -> List(c)");
    assert_infer!("let x = 1.0 1", "Int");
    assert_infer!("let id = fn(x) { x } id(1)", "Int");
    assert_infer!("let x = fn() { 1.0 } x()", "Float");
    assert_infer!("fn(x) { x }(1)", "Int");
    assert_infer!("fn() { 1 }", "fn() -> Int");
    assert_infer!("fn() { 1.1 }", "fn() -> Float");
    assert_infer!("fn(x) { 1.1 }", "fn(a) -> Float");
    assert_infer!("fn(x) { x }", "fn(a) -> a");
    assert_infer!("let x = fn(x) { 1.1 } x", "fn(a) -> Float");
    assert_infer!("fn(x, y, z) { 1 }", "fn(a, b, c) -> Int");
    assert_infer!("fn(x) { let y = x y }", "fn(a) -> a");
    assert_infer!("let id = fn(x) { x } id(1)", "Int");
    assert_infer!(
        "let constant = fn(x) { fn(y) { x } } let one = constant(1) one(2.0)",
        "Int",
    );
    assert_infer!("fn(f) { f(1) }", "fn(fn(Int) -> a) -> a");
    assert_infer!("fn(f, x) { f(x) }", "fn(fn(a) -> b, a) -> b");
    assert_infer!("fn(f) { fn(x) { f(x) } }", "fn(fn(a) -> b) -> fn(a) -> b");
    assert_infer!(
        "fn(f) { fn(x) { fn(y) { f(x, y) } } }",
        "fn(fn(a, b) -> c) -> fn(a) -> fn(b) -> c",
    );
    assert_infer!(
        "fn(f) { fn(x, y) { f(x)(y) } }",
        "fn(fn(a) -> fn(b) -> c) -> fn(a, b) -> c",
    );
    assert_infer!(
        "fn(f) { fn(x) { let ff = f ff(x) } }",
        "fn(fn(a) -> b) -> fn(a) -> b",
    );
    assert_infer!(
        "fn(f) { fn(x, y) { let ff = f(x) ff(y) } }",
        "fn(fn(a) -> fn(b) -> c) -> fn(a, b) -> c",
    );
    assert_infer!("fn(x) { fn(y) { x } }", "fn(a) -> fn(b) -> a");
    assert_infer!("fn(f) { f() }", "fn(fn() -> a) -> a");
    assert_infer!("fn(f, x) { f(f(x)) }", "fn(fn(a) -> a, a) -> a");
    assert_infer!(
        "let id = fn(a) { a } fn(x) { x(id) }",
        "fn(fn(fn(a) -> a) -> b) -> b",
    );
    assert_infer!("let add = fn(x, y) { x + y } add(_, 2)", "fn(Int) -> Int");
    assert_infer!("fn(x) { tuple(1, x) }", "fn(a) -> tuple(Int, a)");
    assert_infer!("fn(x, y) { tuple(x, y) }", "fn(a, b) -> tuple(a, b)");
    assert_infer!("fn(x) { tuple(x, x) }", "fn(a) -> tuple(a, a)");
    assert_infer!("fn(x) -> Int { x }", "fn(Int) -> Int");
    assert_infer!("fn(x) -> a { x }", "fn(a) -> a");
    assert_infer!("fn() -> Int { 2 }", "fn() -> Int");

    // case
    assert_infer!("case 1 { a -> 1 }", "Int");
    assert_infer!("case 1 { a -> 1.0 b -> 2.0 c -> 3.0 }", "Float");
    assert_infer!("case 1 { a -> a }", "Int");
    assert_infer!("case 1 { 1 -> 10 2 -> 20 x -> x * 10 }", "Int");
    assert_infer!("case 2.0 { 2.0 -> 1 x -> 0 }", "Int");
    assert_infer!(r#"case "ok" { "ko" -> 1 x -> 0 }"#, "Int");

    // Multiple subject case
    assert_infer!("case 1, 2.0 { a, b -> a }", "Int");
    assert_infer!("case 1, 2.0 { a, b -> b }", "Float");
    assert_infer!("case 1, 2.0, 3 { a, b, c -> a + c }", "Int");

    // let
    assert_infer!("let [] = [] 1", "Int");
    assert_infer!("let [a] = [1] a", "Int");
    assert_infer!("let [a, 2] = [1] a", "Int");
    assert_infer!("let [a | b] = [1] a", "Int");
    assert_infer!("let [a | _] = [1] a", "Int");
    assert_infer!("fn(x) { let [a] = x a }", "fn(List(a)) -> a");
    assert_infer!("fn(x) { let [a] = x a + 1 }", "fn(List(Int)) -> Int");
    assert_infer!("let _x = 1 2.0", "Float");
    assert_infer!("let _ = 1 2.0", "Float");
    assert_infer!("let tuple(tag, x) = tuple(1.0, 1) x", "Int");
    assert_infer!("fn(x) { let tuple(a, b) = x a }", "fn(tuple(a, b)) -> a");

    // assert
    assert_infer!("assert [] = [] 1", "Int");
    assert_infer!("assert [a] = [1] a", "Int");
    assert_infer!("assert [a, 2] = [1] a", "Int");
    assert_infer!("assert [a | b] = [1] a", "Int");
    assert_infer!("assert [a | _] = [1] a", "Int");
    assert_infer!("assert [a, .._] = [1] a", "Int");
    assert_infer!("assert [a, .._,] = [1] a", "Int");
    assert_infer!("fn(x) { assert [a] = x a }", "fn(List(a)) -> a");
    assert_infer!("fn(x) { assert [a] = x a + 1 }", "fn(List(Int)) -> Int");
    assert_infer!("assert _x = 1 2.0", "Float");
    assert_infer!("assert _ = 1 2.0", "Float");
    assert_infer!("assert tuple(tag, x) = tuple(1.0, 1) x", "Int");
    assert_infer!("fn(x) { assert tuple(a, b) = x a }", "fn(tuple(a, b)) -> a");

    // Nil
    assert_infer!("Nil", "Nil");

    // todo
    assert_infer!("todo", "a");
    assert_infer!("1 == todo", "Bool");
    assert_infer!("todo != 1", "Bool");
    assert_infer!("todo + 1", "Int");

    // tuple index
    assert_infer!("tuple(1, 2.0).0", "Int");
    assert_infer!("tuple(1, 2.0).1", "Float");

    // pipe |>
    assert_infer!("1 |> fn(x) { x }", "Int");
    assert_infer!("1.0 |> fn(x) { x }", "Float");
    assert_infer!("let id = fn(x) { x } 1 |> id", "Int");
    assert_infer!("let id = fn(x) { x } 1.0 |> id", "Float");
    assert_infer!("let add = fn(x, y) { x + y } 1 |> add(_, 2)", "Int");
    assert_infer!("let add = fn(x, y) { x + y } 1 |> add(2, _)", "Int");
    assert_infer!("let add = fn(x, y) { x + y } 1 |> add(2)", "Int");
    assert_infer!("let id = fn(x) { x } 1 |> id()", "Int");
    assert_infer!("let add = fn(x) { fn(y) { y + x } } 1 |> add(1)", "Int");
    assert_infer!(
        "let add = fn(x, _, _) { fn(y) { y + x } } 1 |> add(1, 2, 3)",
        "Int"
    );
}

#[test]
fn infer_error_test() {
    macro_rules! assert_error {
        ($src:expr, $error:expr $(,)?) => {
            let ast = crate::grammar::ExprParser::new()
                .parse($src)
                .expect("syntax error");
            let result = infer(ast, 1, &mut Env::new(&[], &HashMap::new()))
                .expect_err("should infer an error");
            assert_eq!(($src, sort_options($error)), ($src, sort_options(result)));
        };
    }

    assert_error!(
        "1 + 1.0",
        Error::CouldNotUnify {
            location: SrcSpan { start: 4, end: 7 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "1 +. 1.0",
        Error::CouldNotUnify {
            location: SrcSpan { start: 0, end: 1 },
            expected: float(),
            given: int(),
        },
    );

    assert_error!(
        "1 == 1.0",
        Error::CouldNotUnify {
            location: SrcSpan { start: 5, end: 8 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "1 > 1.0",
        Error::CouldNotUnify {
            location: SrcSpan { start: 4, end: 7 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "1.0 >. 1",
        Error::CouldNotUnify {
            location: SrcSpan { start: 7, end: 8 },
            expected: float(),
            given: int(),
        },
    );

    assert_error!(
        "x",
        Error::UnknownVariable {
            location: SrcSpan { start: 0, end: 1 },
            name: "x".to_string(),
            variables: env_vars(),
        },
    );

    assert_error!(
        "x",
        Error::UnknownVariable {
            location: SrcSpan { start: 0, end: 1 },
            name: "x".to_string(),
            variables: env_vars(),
        },
    );

    assert_error!(
        "let id = fn(x) { x } id()",
        Error::IncorrectArity {
            location: SrcSpan { start: 21, end: 25 },
            expected: 1,
            given: 0,
        },
    );

    assert_error!(
        "let id = fn(x) { x } id(1, 2)",
        Error::IncorrectArity {
            location: SrcSpan { start: 21, end: 29 },
            expected: 1,
            given: 2,
        },
    );

    assert_error!(
        "case 1 { a -> 1 b -> 2.0 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 21, end: 24 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "case 1.0 { 1 -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 11, end: 12 },
            expected: float(),
            given: int(),
        },
    );

    assert_error!(
        "case 1 { 1.0 -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 9, end: 12 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "case 1, 2.0 { a, b -> a + b }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 26, end: 27 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "case 1, 2.0 { a, b -> a 1, 2 -> 0 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 27, end: 28 },
            expected: float(),
            given: int(),
        },
    );

    assert_error!(
        "fn() { 1 } == fn(x) { x + 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 14, end: 29 },
            expected: Arc::new(Type::Fn {
                args: vec![],
                retrn: int(),
            }),
            given: Arc::new(Type::Fn {
                args: vec![Arc::new(Type::Var {
                    typ: Arc::new(RefCell::new(TypeVar::Link { typ: int() })),
                })],
                retrn: int(),
            }),
        },
    );

    assert_error!(
        "let f = fn(x: Int) { x } f(1.0)",
        Error::CouldNotUnify {
            location: SrcSpan { start: 27, end: 30 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "fn() -> Int { 2.0 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 14, end: 17 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "fn(x: Int) -> Float { x }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 22, end: 23 },
            expected: float(),
            given: int(),
        },
    );

    assert_error!(
        "case 1 { x -> 1 1 -> x }",
        Error::UnknownVariable {
            location: SrcSpan { start: 21, end: 22 },
            name: "x".to_string(),
            variables: env_vars(),
        },
    );

    assert_error!(
        "case 1 { _, _ -> 1 }",
        Error::IncorrectNumClausePatterns {
            location: SrcSpan { start: 9, end: 18 },
            expected: 1,
            given: 2,
        },
    );

    assert_error!(
        "let id = fn(x) { x(x) } 1",
        Error::RecursiveType {
            location: SrcSpan { start: 19, end: 20 },
        },
    );

    assert_error!(
        "let True(x) = 1 x",
        Error::IncorrectArity {
            location: SrcSpan { start: 4, end: 11 },
            expected: 0,
            given: 1,
        },
    );

    assert_error!(
        "let Ok(1, x) = 1 x",
        Error::IncorrectArity {
            location: SrcSpan { start: 4, end: 12 },
            expected: 1,
            given: 2,
        },
    );

    assert_error!(
        "let x = 1 x.whatever",
        Error::UnknownField {
            location: SrcSpan { start: 11, end: 20 },
            typ: int(),
            label: "whatever".to_string(),
            fields: vec![],
        },
    );

    assert_error!(
        "tuple(1, 2) == tuple(1, 2, 3)",
        Error::CouldNotUnify {
            location: SrcSpan { start: 15, end: 29 },
            expected: tuple(vec![int(), int()]),
            given: tuple(vec![int(), int(), int()])
        },
    );

    assert_error!(
        "tuple(1.0, 2, 3) == tuple(1, 2, 3)",
        Error::CouldNotUnify {
            location: SrcSpan { start: 20, end: 34 },
            expected: tuple(vec![float(), int(), int()]),
            given: tuple(vec![int(), int(), int()]),
        },
    );

    assert_error!(
        "[1.0] == [1]",
        Error::CouldNotUnify {
            location: SrcSpan { start: 9, end: 12 },
            expected: list(Arc::new(Type::Var {
                typ: Arc::new(RefCell::new(TypeVar::Link { typ: float() }))
            })),
            given: list(Arc::new(Type::Var {
                typ: Arc::new(RefCell::new(TypeVar::Link { typ: int() }))
            }))
        },
    );

    assert_error!(
        "let x = 1 let y = 1.0 case x { _ if x == y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 36, end: 42 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "let x = 1.0 let y = 1 case x { _ if x == y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 36, end: 42 },
            expected: float(),
            given: int(),
        },
    );

    assert_error!(
        "let x = 1.0 case x { _ if x -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 26, end: 27 },
            expected: bool(),
            given: float(),
        },
    );

    assert_error!(
        "case tuple(1, 1.0) { tuple(x, _) | tuple(_, x) -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 44, end: 45 },
            expected: int(),
            given: float(),
        },
    );

    assert_error!(
        "case [3.33], 1 { x, y if x > y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 25, end: 26 },
            expected: int(),
            given: list(float())
        }
    );

    assert_error!(
        "case 1, 2.22, \"three\" { x, _, y if x > y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 39, end: 40 },
            expected: int(),
            given: string()
        }
    );

    assert_error!(
        "case [3.33], 1 { x, y if x >= y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 25, end: 26 },
            expected: int(),
            given: list(float())
        }
    );

    assert_error!(
        "case 1, 2.22, \"three\" { x, _, y if x >= y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 40, end: 41 },
            expected: int(),
            given: string()
        }
    );

    assert_error!(
        "case [3.33], 1 { x, y if x < y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 25, end: 26 },
            expected: int(),
            given: list(float())
        }
    );

    assert_error!(
        "case 1, 2.22, \"three\" { x, _, y if x < y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 39, end: 40 },
            expected: int(),
            given: string()
        }
    );

    assert_error!(
        "case [3.33], 1 { x, y if x <= y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 25, end: 26 },
            expected: int(),
            given: list(float())
        }
    );

    assert_error!(
        "case 1, 2.22, \"three\" { x, _, y if x <= y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 40, end: 41 },
            expected: int(),
            given: string()
        }
    );

    assert_error!(
        "case [3], 1.1 { x, y if x >. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 24, end: 25 },
            expected: float(),
            given: list(int())
        }
    );

    assert_error!(
        "case 2.22, 1, \"three\" { x, _, y if x >. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 40, end: 41 },
            expected: float(),
            given: string()
        }
    );

    assert_error!(
        "case [3], 1.1 { x, y if x >=. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 24, end: 25 },
            expected: float(),
            given: list(int())
        }
    );

    assert_error!(
        "case 2.22, 1, \"three\" { x, _, y if x >=. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 41, end: 42 },
            expected: float(),
            given: string()
        }
    );

    assert_error!(
        "case [3], 1.1 { x, y if x <. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 24, end: 25 },
            expected: float(),
            given: list(int())
        }
    );

    assert_error!(
        "case 2.22, 1, \"three\" { x, _, y if x <. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 40, end: 41 },
            expected: float(),
            given: string()
        }
    );

    assert_error!(
        "case [3], 1.1 { x, y if x <=. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 24, end: 25 },
            expected: float(),
            given: list(int())
        }
    );

    assert_error!(
        "case 2.22, 1, \"three\" { x, _, y if x <=. y -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 41, end: 42 },
            expected: float(),
            given: string()
        }
    );

    assert_error!(
        "case [1] { [x] | x -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 17, end: 18 },
            expected: int(),
            given: list(int()),
        },
    );

    assert_error!(
        "case [1] { [x] | [] as x -> 1 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 17, end: 19 },
            expected: int(),
            given: list(int()),
        },
    );

    assert_error!(
        "case [1] { [x] | [x, y] -> 1 }",
        Error::ExtraVarInAlternativePattern {
            location: SrcSpan { start: 21, end: 22 },
            name: "y".to_string()
        },
    );

    assert_error!(
        "case tuple(1, 2) { tuple(1, y) | tuple(x, y) -> 1 }",
        Error::ExtraVarInAlternativePattern {
            location: SrcSpan { start: 39, end: 40 },
            name: "x".to_string()
        },
    );

    assert_error!(
        "case tuple(1, 2) { tuple(1, y) | tuple(x, y) -> 1 }",
        Error::ExtraVarInAlternativePattern {
            location: SrcSpan { start: 39, end: 40 },
            name: "x".to_string()
        },
    );

    assert_error!(
        "let x = 1 case tuple(1, 2) { tuple(1, y) | tuple(x, y) -> 1 }",
        Error::ExtraVarInAlternativePattern {
            location: SrcSpan { start: 49, end: 50 },
            name: "x".to_string()
        },
    );

    // Duplicate vars

    assert_error!(
        "case tuple(1, 2) { tuple(x, x) -> 1 }",
        Error::DuplicateVarInPattern {
            location: SrcSpan { start: 28, end: 29 },
            name: "x".to_string()
        },
    );

    assert_error!(
        "case [3.33], 1 { x, x if x > x -> 1 }",
        Error::DuplicateVarInPattern {
            location: SrcSpan { start: 20, end: 21 },
            name: "x".to_string()
        },
    );

    assert_error!(
        "case [1, 2, 3] { [x, x, y] -> 1 }",
        Error::DuplicateVarInPattern {
            location: SrcSpan { start: 21, end: 22 },
            name: "x".to_string()
        },
    );

    // Tuple indexing

    assert_error!(
        "tuple(0, 1).2",
        Error::OutOfBoundsTupleIndex {
            location: SrcSpan { start: 11, end: 13 },
            index: 2,
            size: 2
        },
    );

    assert_error!(
        "Nil.2",
        Error::NotATuple {
            location: SrcSpan { start: 0, end: 3 },
            given: nil(),
        },
    );

    assert_error!(
        "fn(a) { a.2 }",
        Error::NotATupleUnbound {
            location: SrcSpan { start: 8, end: 9 },
        },
    );

    // Record field access

    assert_error!(
        "fn(a) { a.field }",
        Error::RecordAccessUnknownType {
            location: SrcSpan { start: 8, end: 9 },
        },
    );

    assert_error!(
        "fn(a: a) { a.field }",
        Error::UnknownField {
            location: SrcSpan { start: 12, end: 18 },
            label: "field".to_string(),
            fields: vec![],
            typ: Arc::new(Type::Var {
                typ: Arc::new(RefCell::new(TypeVar::Generic { id: 7 })),
            }),
        },
    );

    assert_error!(
        "let add = fn(x, y) { x + y } 1 |> add(unknown)",
        Error::UnknownVariable {
            location: SrcSpan { start: 38, end: 45 },
            name: "unknown".to_string(),
            variables: env_vars_with(&["add"]),
        },
    );
}

#[test]
fn infer_module_test() {
    macro_rules! assert_infer {
        ($src:expr, $module:expr $(,)?) => {
            let (src, _) = crate::parser::strip_extra($src);
            let ast = crate::grammar::ModuleParser::new()
                .parse(&src)
                .expect("syntax error");
            let (result, _) = infer_module(ast, &HashMap::new());
            let ast = result.expect("should successfully infer");
            let mut constructors: Vec<(_, _)> = ast
                .type_info
                .values
                .iter()
                .map(|(k, v)| {
                    let mut printer = pretty::Printer::new();
                    (k.clone(), printer.pretty_print(&v.typ, 0))
                })
                .collect();
            constructors.sort();
            let expected: Vec<_> = $module
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            assert_eq!(($src, constructors), ($src, expected));
        };
    }

    assert_infer!(
        "pub fn repeat(i, x) {
           case i {
             0 -> []
             i -> [x | repeat(i - 1, x)]
           }
         }",
        vec![("repeat", "fn(Int, a) -> List(a)")],
    );

    assert_infer!(
        "fn private() { 1 }
         pub fn public() { 1 }",
        vec![("public", "fn() -> Int")],
    );

    assert_infer!(
        "pub type Is { Yes No }
         pub fn yes() { Yes }
         pub fn no() { No }",
        vec![
            ("No", "Is"),
            ("Yes", "Is"),
            ("no", "fn() -> Is"),
            ("yes", "fn() -> Is"),
        ],
    );

    assert_infer!(
        "pub type Num { I(Int) }
         pub fn one() { I(1) }",
        vec![("I", "fn(Int) -> Num"), ("one", "fn() -> Num")],
    );

    assert_infer!(
        "pub fn id(x) { x }
         pub fn float() { id(1.0) }
         pub fn int() { id(1) }",
        vec![
            ("float", "fn() -> Float"),
            ("id", "fn(a) -> a"),
            ("int", "fn() -> Int"),
        ],
    );

    assert_infer!(
        "pub type Box(a) { Box(a) }
        pub fn int() { Box(1) }
        pub fn float() { Box(1.0) }",
        vec![
            ("Box", "fn(a) -> Box(a)"),
            ("float", "fn() -> Box(Float)"),
            ("int", "fn() -> Box(Int)"),
        ],
    );

    assert_infer!(
        "pub type Singleton { Singleton }
        pub fn go(x) { let Singleton = x 1 }",
        vec![("Singleton", "Singleton"), ("go", "fn(Singleton) -> Int")],
    );

    assert_infer!(
        "pub type Box(a) { Box(a) }
        pub fn unbox(x) { let Box(a) = x a }",
        vec![("Box", "fn(a) -> Box(a)"), ("unbox", "fn(Box(a)) -> a")],
    );

    assert_infer!(
        "pub type I { I(Int) }
        pub fn open(x) { case x { I(i) -> i  } }",
        vec![("I", "fn(Int) -> I"), ("open", "fn(I) -> Int")],
    );

    assert_infer!(
        "pub fn status() { 1 } pub fn list_of(x) { [x] }",
        vec![("list_of", "fn(a) -> List(a)"), ("status", "fn() -> Int")],
    );

    assert_infer!(
        "pub external fn go(String) -> String = \"\" \"\"",
        vec![("go", "fn(String) -> String")],
    );

    assert_infer!(
        "pub external fn go(Int) -> Float = \"\" \"\"",
        vec![("go", "fn(Int) -> Float")],
    );

    assert_infer!(
        "pub external fn go(Int) -> Int = \"\" \"\"",
        vec![("go", "fn(Int) -> Int")],
    );

    assert_infer!(
        "pub external fn ok() -> fn(Int) -> Int = \"\" \"\"",
        vec![("ok", "fn() -> fn(Int) -> Int")],
    );

    assert_infer!(
        "pub external fn go(Int) -> b = \"\" \"\"",
        vec![("go", "fn(Int) -> a")],
    );

    assert_infer!(
        "pub external fn go(Bool) -> b = \"\" \"\"",
        vec![("go", "fn(Bool) -> a")],
    );

    assert_infer!(
        "pub external fn go(List(a)) -> a = \"\" \"\"",
        vec![("go", "fn(List(a)) -> a")],
    );

    assert_infer!(
        "external fn go(Int) -> b = \"\" \"\"
        pub fn x() { go(1) }",
        vec![("x", "fn() -> a")],
    );

    assert_infer!(
        "external fn id(a) -> a = \"\" \"\"
        pub fn i(x) { id(x) }
        pub fn a() { id(1) }
        pub fn b() { id(1.0) }",
        vec![
            ("a", "fn() -> Int"),
            ("b", "fn() -> Float"),
            ("i", "fn(a) -> a"),
        ],
    );

    assert_infer!(
        "pub external fn len(List(a)) -> Int = \"\" \"\"",
        vec![("len", "fn(List(a)) -> Int")],
    );

    assert_infer!(
        "pub external type Connection\n
         pub external fn is_open(Connection) -> Bool = \"\" \"\"",
        vec![("is_open", "fn(Connection) -> Bool")],
    );

    assert_infer!(
        "pub external type Pair(thing, thing)\n
         pub external fn pair(a) -> Pair(a, a) = \"\" \"\"",
        vec![("pair", "fn(a) -> Pair(a, a)")],
    );

    assert_infer!(
        "pub fn one() { 1 }
         pub fn zero() { one() - 1 }
         pub fn two() { one() + zero() }",
        vec![
            ("one", "fn() -> Int"),
            ("two", "fn() -> Int"),
            ("zero", "fn() -> Int"),
        ],
    );

    assert_infer!(
        "pub fn one() { 1 }
         pub fn zero() { one() - 1 }
         pub fn two() { one() + zero() }",
        vec![
            ("one", "fn() -> Int"),
            ("two", "fn() -> Int"),
            ("zero", "fn() -> Int"),
        ],
    );

    // Type annotations
    assert_infer!("pub fn go(x: Int) { x }", vec![("go", "fn(Int) -> Int")],);
    assert_infer!("pub fn go(x: b) -> b { x }", vec![("go", "fn(a) -> a")],);
    assert_infer!("pub fn go(x) -> b { x }", vec![("go", "fn(a) -> a")],);
    assert_infer!("pub fn go(x: b) { x }", vec![("go", "fn(a) -> a")],);
    assert_infer!(
        "pub fn go(x: List(b)) -> List(b) { x }",
        vec![("go", "fn(List(a)) -> List(a)")],
    );
    assert_infer!(
        "pub fn go(x: List(b)) { x }",
        vec![("go", "fn(List(a)) -> List(a)")],
    );
    assert_infer!(
        "pub fn go(x: List(String)) { x }",
        vec![("go", "fn(List(String)) -> List(String)")],
    );
    assert_infer!("pub fn go(x: b, y: c) { x }", vec![("go", "fn(a, b) -> a")],);
    assert_infer!("pub fn go(x) -> Int { x }", vec![("go", "fn(Int) -> Int")],);

    assert_infer!(
        "type Html = String
         pub fn go() { 1 }",
        vec![("go", "fn() -> Int")],
    );
    assert_infer!(
        "pub fn length(list) {
           case list {
           [] -> 0
           [x | xs] -> length(xs) + 1
           }
        }",
        vec![("length", "fn(List(a)) -> Int")],
    );

    // Structs
    assert_infer!(
        "pub type Box { Box(boxed: Int) }",
        vec![("Box", "fn(Int) -> Box")]
    );
    assert_infer!(
        "pub type Tup(a, b) { Tup(first: a, second: b) }",
        vec![("Tup", "fn(a, b) -> Tup(a, b)")]
    );
    assert_infer!(
        "pub type Tup(a, b, c) { Tup(first: a, second: b, third: c) }
         pub fn third(t) { let Tup(_, third: a, _) = t a }",
        vec![
            ("Tup", "fn(a, b, c) -> Tup(a, b, c)"),
            ("third", "fn(Tup(a, b, c)) -> c"),
        ],
    );
    assert_infer!(
        "pub type Box(x) { Box(label: String, contents: x) }
         pub fn id(x: Box(y)) { x }",
        vec![
            ("Box", "fn(String, a) -> Box(a)"),
            ("id", "fn(Box(a)) -> Box(a)"),
        ],
    );

    // Anon structs
    assert_infer!(
        "pub fn ok(x) { tuple(1, x) }",
        vec![("ok", "fn(a) -> tuple(Int, a)")],
    );

    assert_infer!(
        "pub external fn ok(Int) -> tuple(Int, Int) = \"\" \"\"",
        vec![("ok", "fn(Int) -> tuple(Int, Int)")],
    );

    assert_infer!(
        "pub external fn go(tuple(a, c)) -> c = \"\" \"\"",
        vec![("go", "fn(tuple(a, b)) -> b")],
    );

    assert_infer!(
        "pub fn always(ignore _a, return b) { b }",
        vec![("always", "fn(a, b) -> b")],
    );

    // Using types before they are defined

    assert_infer!(
        "pub type I { I(Num) } pub type Num { Num }",
        vec![("I", "fn(Num) -> I"), ("Num", "Num")]
    );

    assert_infer!(
        "pub type I { I(Num) } pub external type Num",
        vec![("I", "fn(Num) -> I")]
    );

    // We can create an aliases
    assert_infer!(
        "type IntString = Result(Int, String)
         pub fn ok_one() -> IntString { Ok(1) }",
        vec![("ok_one", "fn() -> Result(Int, String)")]
    );

    // We can create an alias with the same name as a built in type
    assert_infer!(
        "type Int = Float
         pub fn ok_one() -> Int { 1.0 }",
        vec![("ok_one", "fn() -> Float")]
    );

    // We can access fields on custom types with only one record
    assert_infer!(
        "
pub type Person { Person(name: String, age: Int) }
pub fn get_age(person: Person) { person.age }
pub fn get_name(person: Person) { person.name }",
        vec![
            ("Person", "fn(String, Int) -> Person"),
            ("get_age", "fn(Person) -> Int"),
            ("get_name", "fn(Person) -> String"),
        ]
    );

    // We can access fields on custom types with only one record
    assert_infer!(
        "
pub type One { One(name: String) }
pub type Two { Two(one: One) }
pub fn get(x: Two) { x.one.name }",
        vec![
            ("One", "fn(String) -> One"),
            ("Two", "fn(One) -> Two"),
            ("get", "fn(Two) -> String"),
        ]
    );

    // Field access correctly handles type parameters
    assert_infer!(
        "
pub type Box(a) { Box(inner: a) }
pub fn get_box(x: Box(Box(a))) { x.inner }
pub fn get_generic(x: Box(a)) { x.inner }
pub fn get_get_box(x: Box(Box(a))) { x.inner.inner }
pub fn get_int(x: Box(Int)) { x.inner }
pub fn get_string(x: Box(String)) { x.inner }
",
        vec![
            ("Box", "fn(a) -> Box(a)"),
            ("get_box", "fn(Box(Box(a))) -> Box(a)"),
            ("get_generic", "fn(Box(a)) -> a"),
            ("get_get_box", "fn(Box(Box(a))) -> a"),
            ("get_int", "fn(Box(Int)) -> Int"),
            ("get_string", "fn(Box(String)) -> String"),
        ]
    );
}

#[test]
fn infer_module_error_test() {
    macro_rules! assert_error {
        ($src:expr, $error:expr $(,)?) => {
            let (src, _) = crate::parser::strip_extra($src);
            let mut ast = crate::grammar::ModuleParser::new()
                .parse(&src)
                .expect("syntax error");
            ast.name = vec!["my_module".to_string()];
            let (result, _) = infer_module(ast, &HashMap::new());
            let ast = result.expect_err("should infer an error");
            assert_eq!(($src, sort_options($error)), ($src, sort_options(ast)));
        };

        ($src:expr) => {
            let ast = crate::grammar::ModuleParser::new()
                .parse($src)
                .expect("syntax error");
            let (result, _) = infer_module(ast, &HashMap::new());
            result.expect_err("should infer an error");
        };
    }

    assert_error!(
        "fn go() { 1 + 2.0 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 14, end: 17 },
            expected: int(),
            given: float(),
        }
    );

    assert_error!(
        "fn go() { 1 + 2.0 }",
        Error::CouldNotUnify {
            location: SrcSpan { start: 14, end: 17 },
            expected: int(),
            given: float(),
        }
    );

    assert_error!(
        "
fn id(x: a, y: a) { x }
pub fn x() { id(1, 1.0) }
                ",
        Error::CouldNotUnify {
            location: SrcSpan { start: 44, end: 47 },
            expected: int(),
            given: float(),
        }
    );

    assert_error!(
        "
fn bar() -> Int {
    5
}

fn run(foo: fn() -> String) {
    foo()
}

fn demo() {
    run(bar)
}",
        Error::CouldNotUnify {
            location: SrcSpan { start: 91, end: 94 },
            expected: Arc::new(Type::Fn {
                args: vec![],
                retrn: string(),
            }),
            given: Arc::new(Type::Fn {
                args: vec![],
                retrn: int(),
            }),
        },
    );

    assert_error!(
        "
fn bar(x: Int) -> Int {
    x * 5
}

fn run(foo: fn(String) -> Int) {
    foo(\"Foo.\")
}

fn demo() {
    run(bar)
}",
        Error::CouldNotUnify {
            location: SrcSpan {
                start: 110,
                end: 113
            },
            expected: Arc::new(Type::Fn {
                args: vec![string()],
                retrn: int(),
            }),
            given: Arc::new(Type::Fn {
                args: vec![int()],
                retrn: int(),
            }),
        },
    );

    assert_error!(
        "external fn go(List(a, b)) -> a = \"\" \"\"",
        Error::IncorrectTypeArity {
            location: SrcSpan { start: 15, end: 25 },
            name: "List".to_string(),
            expected: 1,
            given: 2,
        }
    );

    // We cannot declare two functions with the same name in a module
    assert_error!(
        "fn dupe() { 1 }
         fn dupe() { 2 }",
        Error::DuplicateName {
            location: SrcSpan { start: 25, end: 34 },
            previous_location: SrcSpan { start: 0, end: 9 },
            name: "dupe".to_string(),
        }
    );

    // We cannot declare two functions with the same name in a module
    assert_error!(
        "fn dupe() { 1 }
         fn dupe(x) { x }",
        Error::DuplicateName {
            location: SrcSpan { start: 25, end: 35 },
            previous_location: SrcSpan { start: 0, end: 9 },
            name: "dupe".to_string(),
        }
    );

    // We cannot declare two functions with the same name in a module
    assert_error!(
        "fn dupe() { 1 }
         external fn dupe(x) -> x = \"\" \"\"",
        Error::DuplicateName {
            location: SrcSpan { start: 25, end: 57 },
            previous_location: SrcSpan { start: 0, end: 9 },
            name: "dupe".to_string(),
        }
    );

    // We cannot declare two functions with the same name in a module
    assert_error!(
        "external fn dupe(x) -> x = \"\" \"\"
         fn dupe() { 1 }",
        Error::DuplicateName {
            location: SrcSpan { start: 42, end: 51 },
            previous_location: SrcSpan { start: 0, end: 32 },
            name: "dupe".to_string(),
        }
    );

    // We cannot declare two type constructors with the same name in a module
    assert_error!(
        "type Box { Box(x: Int) }
         type Boxy { Box(Int) }",
        Error::DuplicateName {
            location: SrcSpan { start: 46, end: 54 },
            previous_location: SrcSpan { start: 11, end: 22 },
            name: "Box".to_string(),
        }
    );

    // We cannot declare two type constructors with the same name in a module
    assert_error!(
        "type Boxy { Box(Int) }
         type Box { Box(x: Int) }",
        Error::DuplicateName {
            location: SrcSpan { start: 43, end: 54 },
            previous_location: SrcSpan { start: 12, end: 20 },
            name: "Box".to_string(),
        }
    );

    // We cannot declare two type constructors with the same name in a module
    assert_error!(
        "type Boxy { Box(Int) Box(Float) }",
        Error::DuplicateName {
            location: SrcSpan { start: 21, end: 31 },
            previous_location: SrcSpan { start: 12, end: 20 },
            name: "Box".to_string(),
        }
    );

    // We cannot declare two types with the same name in a module
    assert_error!(
        "type DupType { A }
         type DupType { B }",
        Error::DuplicateTypeName {
            location: SrcSpan { start: 28, end: 41 },
            previous_location: SrcSpan { start: 0, end: 13 },
            name: "DupType".to_string(),
        }
    );

    assert_error!(
        r#"external type PrivateType
           pub external fn leak_type() -> PrivateType = "" """#,
        Error::PrivateTypeLeak {
            location: SrcSpan { start: 37, end: 87 },
            leaked: Type::App {
                args: vec![],
                public: false,
                module: vec!["my_module".to_string()],
                name: "PrivateType".to_string(),
            },
        }
    );

    assert_error!(
        r#"external type PrivateType
           external fn go() -> PrivateType = "" ""
           pub fn leak_type() { go() }"#,
        Error::PrivateTypeLeak {
            location: SrcSpan {
                start: 88,
                end: 106,
            },
            leaked: Type::App {
                args: vec![],
                public: false,
                module: vec!["my_module".to_string()],
                name: "PrivateType".to_string(),
            },
        }
    );

    assert_error!(
        r#"external type PrivateType
           external fn go() -> PrivateType = "" ""
           pub fn leak_type() { [go()] }"#,
        Error::PrivateTypeLeak {
            location: SrcSpan {
                start: 88,
                end: 106,
            },
            leaked: Type::App {
                args: vec![],
                public: false,
                module: vec!["my_module".to_string()],
                name: "PrivateType".to_string(),
            },
        }
    );

    assert_error!(
        r#"external type PrivateType
                    pub external fn go(PrivateType) -> Int = "" """#,
        Error::PrivateTypeLeak {
            location: SrcSpan { start: 46, end: 92 },
            leaked: Type::App {
                args: vec![],
                public: false,
                module: vec!["my_module".to_string()],
                name: "PrivateType".to_string(),
            },
        }
    );

    assert_error!(
        r#"external type PrivateType
           pub type LeakType { Variant(PrivateType) }"#,
        Error::PrivateTypeLeak {
            location: SrcSpan { start: 57, end: 77 },
            leaked: Type::App {
                args: vec![],
                public: false,
                module: vec!["my_module".to_string()],
                name: "PrivateType".to_string(),
            },
        }
    );

    assert_error!(
        r#"fn id(x) { x } fn y() { id(x: 4) }"#,
        Error::UnexpectedLabelledArg {
            label: "x".to_string(),
            location: SrcSpan { start: 27, end: 31 },
        }
    );

    assert_error!(
        r#"type X { X(a: Int, b: Int, c: Int) }
                    fn x() { X(b: 1, a: 1, 1) }"#,
        Error::PositionalArgumentAfterLabelled {
            location: SrcSpan { start: 80, end: 81 },
        }
    );

    assert_error!(
        r#"type Thing { Thing(unknown: x) }"#,
        Error::UnknownType {
            location: SrcSpan { start: 28, end: 29 },
            name: "x".to_string(),
            types: env_types_with(&["Thing"]),
        }
    );

    assert_error!(
        r#"fn one() { 1 }
           fn main() { case 1 { _ if one -> 1 } }"#,
        Error::NonLocalClauseGuardVariable {
            location: SrcSpan { start: 52, end: 55 },
            name: "one".to_string(),
        }
    );

    // We cannot refer to unknown types in an alias
    assert_error!(
        "type IntMap = IllMap(Int, Int)",
        Error::UnknownType {
            location: SrcSpan { start: 14, end: 30 },
            name: "IllMap".to_string(),
            types: env_types(),
        }
    );

    // We cannot refer to unknown types in an alias
    assert_error!(
        "type IntMap = Map(Inf, Int)",
        Error::UnknownType {
            location: SrcSpan { start: 18, end: 21 },
            name: "Inf".to_string(),
            types: env_types(),
        }
    );

    // We cannot reuse an alias name in the same module
    assert_error!(
        "type X = Int type X = Int",
        Error::DuplicateTypeName {
            location: SrcSpan { start: 13, end: 25 },
            previous_location: SrcSpan { start: 0, end: 12 },
            name: "X".to_string(),
        }
    );

    // We cannot use undeclared type vars in a type alias
    assert_error!(
        "type X = List(a)",
        Error::UnknownType {
            location: SrcSpan { start: 14, end: 15 },
            name: "a".to_string(),
            types: env_types(),
        }
    );

    // An unknown field should report the possible fields' labels
    assert_error!(
        "
pub type Box(a) { Box(inner: a) }
pub fn main(box: Box(Int)) { box.unknown }
",
        Error::UnknownField {
            location: SrcSpan { start: 67, end: 75 },
            label: "unknown".to_string(),
            fields: vec!["inner".to_string()],
            typ: Arc::new(Type::App {
                args: vec![int()],
                public: true,
                module: vec!["my_module".to_string()],
                name: "Box".to_string(),
            }),
        },
    );

    // An unknown field should report the possible fields' labels
    assert_error!(
        "
pub type Box(a) { Box(inner: a) }
pub fn main(box: Box(Box(Int))) { box.inner.unknown }
    ",
        Error::UnknownField {
            location: SrcSpan { start: 78, end: 86 },
            label: "unknown".to_string(),
            fields: vec!["inner".to_string()],
            typ: Arc::new(Type::Var {
                typ: Arc::new(RefCell::new(TypeVar::Link {
                    typ: Arc::new(Type::App {
                        args: vec![int()],
                        public: true,
                        module: vec!["my_module".to_string()],
                        name: "Box".to_string(),
                    }),
                })),
            }),
        },
    );

    assert_error!(
        "
type Triple {
    Triple(a: Int, b: Int, c: Int)
}

fn main() {
  let triple = Triple(1,2,3)
  let Triple(a, b, c, ..) = triple
  a
}",
        Error::UnnecessarySpreadOperator {
            location: SrcSpan {
                start: 116,
                end: 118
            },
            arity: 3
        }
    );

    // Duplicate var in record
    assert_error!(
        r#"type X { X(a: Int, b: Int, c: Int) }
                    fn x() {
                        case X(1,2,3) { X(x, y, x) -> 1 }
                    }"#,
        Error::DuplicateVarInPattern {
            location: SrcSpan {
                start: 114,
                end: 115
            },
            name: "x".to_string()
        },
    );

    // Cases were we can't so easily check for equality-
    // i.e. because the contents of the error are non-deterministic.
    assert_error!("fn inc(x: a) { x + 1 }");
}

#[test]
fn infer_module_warning_test() {
    macro_rules! assert_warning {
        ($src:expr, $warning:expr $(,)?) => {
            let (src, _) = crate::parser::strip_extra($src);
            let mut ast = crate::grammar::ModuleParser::new()
                .parse(&src)
                .expect("syntax error");
            ast.name = vec!["my_module".to_string()];
            let (_, warnings) = infer_module(ast, &HashMap::new());

            assert!(!warnings.is_empty());
            assert_eq!($warning, warnings[0]);
        };
    }

    macro_rules! assert_no_warnings {
        ($src:expr $(,)?) => {
            let (src, _) = crate::parser::strip_extra($src);
            let mut ast = crate::grammar::ModuleParser::new()
                .parse(&src)
                .expect("syntax error");
            ast.name = vec!["my_module".to_string()];
            let (_, warnings) = infer_module(ast, &HashMap::new());

            assert!(warnings.is_empty());
        };
    }

    // Old list prepend syntax emits a warning
    assert_warning!(
        "fn main() { [1 | [2, 3]] }",
        Warning::DeprecatedListPrependSyntax {
            location: SrcSpan { start: 15, end: 16 }
        },
    );

    // New list prepend syntax does not emit a warning
    assert_no_warnings!("fn main() { [1 ..[2, 3]] }",);

    // Old list tail pattern matching syntax emits a warning
    assert_warning!(
        "fn main() { let x = [] ; case x { [x | _] -> 1 } }",
        Warning::DeprecatedListPrependSyntax {
            location: SrcSpan { start: 37, end: 38 }
        },
    );

    // New list tail pattern matching syntax does not emit a warning
    assert_no_warnings!("fn main() { let x = [] ; case x { [x, ..] -> 1 } }",);

    // Todos emit warnings
    assert_warning!(
        "fn main() { 1 == todo }",
        Warning::Todo {
            location: SrcSpan { start: 17, end: 21 }
        },
    );

    // Implicitly discarded Results emit warnings
    assert_warning!(
        "
fn foo() { Ok(5) }
fn main() { foo(); 5 }",
        Warning::ImplicitlyDiscardedResult {
            location: SrcSpan { start: 32, end: 37 }
        }
    );

    // Explicitly discarded Results do not emit warnings
    assert_no_warnings!(
        "
fn foo() { Ok(5) }
fn main() { let _ = foo(); 5 }",
    );
}

fn env_types_with(things: &[&str]) -> Vec<String> {
    let mut types: Vec<_> = env_types();
    for thing in things {
        types.push(thing.to_string());
    }
    types
}

fn env_types() -> Vec<String> {
    Env::new(&[], &HashMap::new())
        .module_types
        .keys()
        .map(|s| s.to_string())
        .collect()
}

fn env_vars_with(things: &[&str]) -> Vec<String> {
    let mut types: Vec<_> = env_vars();
    for thing in things {
        types.push(thing.to_string());
    }
    types
}

fn env_vars() -> Vec<String> {
    Env::new(&[], &HashMap::new())
        .local_values
        .keys()
        .map(|s| s.to_string())
        .collect()
}

fn sort_options(e: Error) -> Error {
    match e {
        Error::UnknownType {
            location,
            name,
            mut types,
        } => {
            types.sort();
            Error::UnknownType {
                location,
                name,
                types,
            }
        }

        Error::UnknownVariable {
            location,
            name,
            mut variables,
        } => {
            variables.sort();
            Error::UnknownVariable {
                location,
                name,
                variables,
            }
        }

        _ => e,
    }
}
