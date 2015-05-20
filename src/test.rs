mod silly_grammar {
    use Symbol;

    pub struct Foo;

    rusty_peg! {
        parser Parser<'input>: Foo {
            Hi: u32 = ("Hi") => 1;
            Ho: u32 = "Ho" => 2;

            HiOrHo: u32 = (Hi / Ho);

            Sum: u32 = (Sum1 / HiOrHo);
            Sum1: u32 = (<x:HiOrHo>, "+", <y:Sum>) => {x + y*10};

            HiHo: () = (Hi, Ho) => ();

            Rep: Vec<u32> = {HiOrHo};
        }
    }

    fn should_parse_prefix<'input,P:?Sized>(
        symbol: &P,
        text: &'input str)
        -> P::Output
        where P: Symbol<'input,Parser<'input>>
    {
        let mut parser = Parser::new(Foo);
        symbol.parse_prefix(&mut parser, text).unwrap().1
    }

    #[test]
    fn parse_hi_from_hi() {
        assert_eq!(1, should_parse_prefix(&Hi, "Hi"));
    }

    #[test]
    #[should_panic]
    fn parse_hi_from_ho() {
        assert_eq!(2, should_parse_prefix(&Hi, "Ho"));
    }

    #[test]
    fn parse_hiorho_from_hi() {
        assert_eq!(1, should_parse_prefix(&HiOrHo, "Hi"));
    }

    #[test]
    fn parse_hiorho_from_ho() {
        assert_eq!(2, should_parse_prefix(&HiOrHo, "Ho"));
    }

    #[test]
    fn parse_hiho_from_ho() {
        assert_eq!((), should_parse_prefix(&HiHo, "Hi Ho"));
    }

    #[test]
    fn parse_sum_from_ho() {
        assert_eq!(1221, should_parse_prefix(&Sum, "Hi + Ho + Ho + Hi"));
    }

    #[test]
    fn parse_repeat() {
        assert_eq!(vec![1, 2, 2, 1, 2], should_parse_prefix(&Rep, "Hi Ho Ho Hi Ho"));
    }
}

mod classy {
    use regex::Regex;
    use std::collections::HashSet;
    use std::rc::Rc;
    use Symbol;

    #[derive(Debug)]
    pub struct ClassDefn<'input> {
        name: &'input str,
        members: Vec<Rc<MemberDefn<'input>>>
    }

    #[derive(Debug)]
    pub enum MemberDefn<'input> {
        Field(Box<FieldDefn<'input>>),
        Method(Box<MethodDefn<'input>>),
    }

    #[derive(Debug)]
    pub struct FieldDefn<'input> {
        name: &'input str,
        ty: TypeRef<'input>,
    }

    #[derive(Debug)]
    pub struct MethodDefn<'input> {
        name: &'input str,
        arg_tys: Vec<TypeRef<'input>>,
        ret_ty: TypeRef<'input>,
    }

    #[derive(Clone,Debug)]
    pub struct TypeRef<'input> {
        id: &'input str
    }

    #[derive(Debug)]
    pub struct ClassyBase {
        identifier: Regex,
        keywords: HashSet<String>,
    }

    impl ClassyBase {
        fn new() -> ClassyBase {
            ClassyBase {
                identifier: Regex::new("^[a-zA-Z_][a-zA-Z_0-9]*").unwrap(),
                keywords: vec!["class"].into_iter().map(|x| x.to_string()).collect(),
            }
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    struct ID;

    impl<'input> Symbol<'input,Classy<'input>> for ID {
        type Output = &'input str;

        fn pretty_print(&self) -> String {
            format!("{:?}", self)
        }

        fn parse(&self,
                 parser: &mut Classy,
                 start: ::Input<'input>)
                 -> ::ParseResult<'input,&'input str>
        {
            match parser.base.identifier.find(&start.text[start.offset..]) {
                Some((_, offset)) => {
                    let end = start.offset_by(offset);
                    let matched = &start.text[start.offset..end.offset];
                    if !parser.base.keywords.contains(matched) {
                        return Ok((end, matched));
                    }
                }
                None => { }
            }

            Err(::Error { expected: "identifier", offset: start.offset })
        }
    }

    rusty_peg! {
        parser Classy<'input>: ClassyBase {
            CLASS: Rc<ClassDefn<'input>> =
                ("class", <name:ID>, "{", <members:{MEMBER}>, "}") => {
                    Rc::new(ClassDefn { name: name, members: members })
                };

            MEMBER: Rc<MemberDefn<'input>> =
                (FIELD_DEFN / METHOD_DEFN);

            FIELD_DEFN: Rc<MemberDefn<'input>> =
                (<name:ID>, ":", <ty:TYPE_REF>, ";") => {
                    Rc::new(MemberDefn::Field(Box::new(
                        FieldDefn { name: name, ty: ty })))
                };

            TYPE_REF: TypeRef<'input> =
                (<id:ID>) => {
                    TypeRef { id: id }
                };

            METHOD_DEFN: Rc<MemberDefn<'input>> =
                (<name:ID>, "(", <args:{TYPE_REF}>, ")", "->", <ret:TYPE_REF>, ";") => {
                    Rc::new(MemberDefn::Method(Box::new(
                        MethodDefn { name: name, arg_tys: args, ret_ty: ret })))
                };
        }
    }

    #[test]
    fn parse_a_class() {
        let mut classy = Classy::new(ClassyBase::new());
        let (_, result) =
            CLASS.parse_prefix(
                &mut classy,
                "class x { f: u32; g: i32; h(i32) -> u32; }").unwrap();

        assert_eq!(
            normalize_space(&format!("{:#?}", result)),
            normalize_space("ClassDefn {
                name: \"x\",
                members: [
                    Field(
                        FieldDefn {
                            name: \"f\",
                            ty: TypeRef {
                                id: \"u32\"
                            }
                        }
                        ),
                    Field(
                        FieldDefn {
                            name: \"g\",
                            ty: TypeRef {
                                id: \"i32\"
                            }
                        }
                        ),
                    Method(
                        MethodDefn {
                            name: \"h\",
                            arg_tys: [
                                TypeRef {
                                    id: \"i32\"
                                }
                                ],
                            ret_ty: TypeRef {
                                id: \"u32\"
                            }
                        }
                        )
                    ]
            }"));
    }

    fn normalize_space(text: &str) {
        Regex::new(r"\s+").unwrap().replace_all(text, " ");
    }
}

