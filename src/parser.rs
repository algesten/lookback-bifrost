use std::marker::PhantomData;
use crate::token::{tokenize, Chunk, Token, TokenIter, SYMBOL};

//include!("display.rs");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ast<'a, T> {
    pub tree: Vec<Tree<'a, T>>,
    _ph: PhantomData<T>,
}

impl<'a, T> Ast<'a, T> {
    pub fn new(tree: Vec<Tree<'a, T>>) -> Ast<'a, T> {
        Ast {
            tree,
            _ph: PhantomData,
        }
    }

    pub fn has_type<P>(&self, mut pred: P) -> bool
    where
        P: FnMut(&TypeExpr<'a, T>) -> bool,
    {
        for tr in &self.tree {
            match tr {
                Tree::Ty(t) => {
                    for f in &t.fields {
                        if pred(&f.expr) {
                            return true;
                        }
                        for a in &f.args {
                            if pred(&a.expr) {
                                return true;
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        false
    }

    pub fn get_tree(&'a self, name: &str) -> Option<&'a Tree<'a, T>> {
        self.tree.iter().find(|f| f.name() == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tree<'a, T> {
    Dr(Directive<'a, T>),
    Sc(Scalar<'a, T>),
    Ty(Type<'a, T>),
    En(Enum<'a, T>),
    Un(Union<'a, T>),
}

impl<'a, T> Tree<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Tree::Dr(t) => t.name,
            Tree::Sc(t) => t.name,
            Tree::Ty(t) => t.name,
            Tree::En(e) => e.name,
            Tree::Un(u) => u.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Object,
    FieldDefinition,
    InputFieldDefinition,
    Unknown,
}

impl<'a> From<&'a str> for Target {
    fn from(s: &'a str) -> Self {
        match s {
            "OBJECT" => Target::Object,
            "FIELD_DEFINITION" => Target::FieldDefinition,
            "INPUT_FIELD_DEFINITION" => Target::InputFieldDefinition,
            _ => Target::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedTarget<T>(pub Target, PhantomData<T>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Directive<'a, T> {
    pub doc: Option<&'a str>,
    pub name: &'a str,
    pub fields: Vec<Field<'a, T>>,
    pub targets: Vec<TypedTarget<T>>,
    _ph: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Scalar<'a, T> {
    pub doc: Option<&'a str>,
    pub name: &'a str,
    _ph: PhantomData<T>,
}

// type Starship {
//   id: ID!
//   name: String!
//   length(unit: LengthUnit = METER): Float
// }
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Type<'a, T> {
    pub is_input: bool,
    pub doc: Option<&'a str>,
    pub name: &'a str,
    pub fields: Vec<Field<'a, T>>,
    pub dir_args: Vec<DirArg<'a, T>>,
    _ph: PhantomData<T>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Field<'a, T> {
    pub doc: Option<&'a str>,
    pub name: &'a str,
    pub expr: TypeExpr<'a, T>,
    pub args: Vec<FieldArg<'a, T>>,
    pub dir_args: Vec<DirArg<'a, T>>,
    _ph: PhantomData<T>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldArg<'a, T> {
    pub name: &'a str,
    pub expr: TypeExpr<'a, T>,
    pub def: Option<&'a str>,
    _ph: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeExpr<'a, T> {
    pub typ: &'a str,
    pub null: bool,
    pub arr: Arr,
    _ph: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirArg<'a, T> {
    pub name: &'a str,
    // TODO parse the args
    _ph: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Arr {
    No,
    Yes { null: bool },
}

impl Arr {
    pub fn is_arr(&self) -> bool {
        match self {
            Arr::Yes { .. } => true,
            _ => false,
        }
    }
    pub fn is_null(&self) -> bool {
        match self {
            Arr::Yes { null } => *null,
            _ => false,
        }
    }
}

// enum Episode {
//   NEWHOPE
//   EMPIRE
//   JEDI
// }
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Enum<'a, T> {
    pub doc: Option<&'a str>,
    pub name: &'a str,
    pub values: Vec<EnumValue<'a, T>>,
    _ph: PhantomData<T>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumValue<'a, T> {
    pub doc: Option<&'a str>,
    pub value: &'a str,
    _ph: PhantomData<T>,
}

// union SearchResult = Human | Droid | Starship
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Union<'a, T> {
    pub doc: Option<&'a str>,
    pub name: &'a str,
    pub names: Vec<&'a str>,
    _ph: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyntaxError {
    pub cause: String,
    pub context: String,
}

pub type ParseResult<T> = Result<T, SyntaxError>;

fn syntax_error(cause: &str, context: &Chunk, source: &str) -> SyntaxError {
    let cc = Chunk {
        index: context.index,
        len: source.len() - context.index,
        token: Token::Unknown,
    };
    let x = cc.apply(source);
    SyntaxError {
        cause: cause.to_string(),
        context: (&x[0..10.min(x.len())]).to_string(),
    }
}

fn err_syntax_error<T>(cause: &str, context: &Chunk, source: &str) -> ParseResult<T> {
    Err(syntax_error(cause, context, source))
}

fn unexpected_end(cause: &str) -> SyntaxError {
    SyntaxError {
        cause: cause.to_string(),
        context: "".to_string(),
    }
}

fn expect_name<'a>(source: &'a str, tok: &mut TokenIter) -> ParseResult<&'a str> {
    tok.consume()
        .ok_or_else(|| unexpected_end("Expected name, but found end of input"))
        .and_then(|chunk| {
            if chunk.token == Token::Name {
                Ok(chunk.apply(source))
            } else {
                err_syntax_error("Expected name", &chunk, source)
            }
        })
}

fn expect_word<'a>(source: &'a str, tok: &mut TokenIter, word: &str) -> ParseResult<()> {
    tok.consume()
        .ok_or_else(|| unexpected_end("Expected word, but found end of input"))
        .and_then(|chunk| {
            if chunk.token == Token::Name {
                let found = chunk.apply(source);
                if found == word {
                    Ok(())
                } else {
                    err_syntax_error(&format!("Expected word: {}", word), &chunk, source)
                }
            } else {
                err_syntax_error("Expected name token", &chunk, source)
            }
        })
}

fn expect_symbol(source: &str, tok: &mut TokenIter, symbol: SYMBOL) -> ParseResult<Chunk> {
    tok.consume()
        .ok_or_else(|| unexpected_end("Expected symbol, but found end of input"))
        .and_then(|chunk| {
            let is_expected_symbol = if let Token::Symbol(s) = &chunk.token {
                Ok(*s == symbol)
            } else {
                err_syntax_error(&format!("Expected symbol: {:?}", symbol), &chunk, source)
            }?;
            if is_expected_symbol {
                Ok(chunk)
            } else {
                err_syntax_error(&format!("Not expected symbol: {:?}", symbol), &chunk, source)
            }
        })
}

pub fn parse<T>(source: &str) -> ParseResult<Ast<T>> {
    let mut tok = tokenize(source);
    let mut tree = vec![];
    loop {
        let doc = parse_doc(source, &mut tok)?;
        tok.skip_white();
        match tok.consume() {
            None => break,
            Some(chunk) => {
                let tr = match chunk.token {
                    Token::Name => match chunk.apply(source) {
                        "directive" => parse_directive(source, &mut tok, doc),
                        "scalar" => parse_scalar(source, &mut tok, doc),
                        "input" => parse_type(source, &mut tok, doc, true),
                        "type" => parse_type(source, &mut tok, doc, false),
                        "enum" => parse_enum(source, &mut tok, doc),
                        "union" => parse_union(source, &mut tok, doc),
                        _ => err_syntax_error("Unknown keyword", &chunk, source),
                    },
                    _ => err_syntax_error("Unexpected input", &chunk, source),
                }?;
                tree.push(tr);
            }
        }
    }
    Ok(Ast {
        tree,
        _ph: PhantomData,
    })
}

fn parse_doc<'a>(source: &'a str, tok: &mut TokenIter) -> ParseResult<Option<&'a str>> {
    tok.skip_white();
    if tok.peek_is_symbol(SYMBOL::DQuote) {
        let start = tok.consume().unwrap();
        tok.find(|t| t.is_symbol(SYMBOL::DQuote))
            .map(|c| start.extend(&c))
            .map(|c| Chunk::new(c.index + 1, c.len - 2, Token::Name))
            .map(|c| Some(c.apply(source)))
            .ok_or_else(|| syntax_error("Unbalanced doc quotes", &start, source))
    } else if tok.peek_is_symbol(SYMBOL::TDQuote) {
        let start = tok.consume().unwrap();
        tok.find(|t| t.is_symbol(SYMBOL::TDQuote))
            .map(|c| start.extend(&c))
            .map(|c| Chunk::new(c.index + 3, c.len - 6, Token::Name))
            .map(|c| Some(c.apply(source)))
            .ok_or_else(|| syntax_error("Unbalanced doc triple-quotes", &start, source))
    } else {
        Ok(None)
    }
}

fn parse_directive<'a, T>(
    source: &'a str,
    tok: &mut TokenIter,
    doc: Option<&'a str>,
) -> ParseResult<Tree<'a, T>> {
    // keyword is "directive" and tok is positioned after that
    tok.skip_white();
    expect_symbol(source, tok, SYMBOL::Aruba)?;
    let name = expect_name(source, tok)?;
    expect_symbol(source, tok, SYMBOL::OpParen)?;
    let mut fields: Vec<Field<T>> = vec![];
    loop {
        tok.skip_white();
        if tok.peek_is_symbol(SYMBOL::ClParen) {
            tok.consume();
            break;
        }
        let doc = parse_doc(source, tok)?;
        tok.skip_white();
        fields.push(parse_field(source, tok, doc)?);
    }
    tok.skip_white();
    expect_word(source, tok, "on")?;
    tok.skip_white();
    let mut targets: Vec<TypedTarget<T>> = vec![];
    loop {
        let t = expect_name(source, tok)?;
        targets.push(TypedTarget(t.into(), PhantomData));
        if tok.peek_is_white_with_lf(source) {
            break;
        }
        tok.skip_white();
        expect_symbol(source, tok, SYMBOL::Pipe)?;
        tok.skip_white();
    }
    Ok(Tree::Dr(Directive {
        doc,
        name,
        fields,
        targets,
        _ph: PhantomData,
    }))
}

fn parse_scalar<'a, T>(
    source: &'a str,
    tok: &mut TokenIter,
    doc: Option<&'a str>,
) -> ParseResult<Tree<'a, T>> {
    // keyword is "directive" and tok is positioned after that
    tok.skip_white();
    let name = expect_name(source, tok)?;
    Ok(Tree::Sc(Scalar {
        doc,
        name,
        _ph: PhantomData,
    }))
}

fn parse_type<'a, T>(
    source: &'a str,
    tok: &mut TokenIter,
    doc: Option<&'a str>,
    is_input: bool,
) -> ParseResult<Tree<'a, T>> {
    // keyword is "type" and tok is positioned after that
    tok.skip_white();
    let name = expect_name(source, tok)?;
    tok.skip_white();
    let dir_args = parse_dir_args(source, tok)?;
    expect_symbol(source, tok, SYMBOL::OpCurl)?;
    let mut fields: Vec<Field<T>> = vec![];
    loop {
        tok.skip_white();
        if tok.peek_is_symbol(SYMBOL::ClCurl) {
            tok.consume();
            break;
        }
        let doc = parse_doc(source, tok)?;
        tok.skip_white();
        fields.push(parse_field(source, tok, doc)?);
    }
    Ok(Tree::Ty(Type {
        is_input,
        doc,
        name,
        fields,
        dir_args,
        _ph: PhantomData,
    }))
}

fn parse_field<'a, T>(
    source: &'a str,
    tok: &mut TokenIter,
    doc: Option<&'a str>,
) -> ParseResult<Field<'a, T>> {
    let name = expect_name(source, tok)?;
    tok.skip_white();
    let mut args = vec![];
    if tok.peek_is_symbol(SYMBOL::OpParen) {
        tok.consume();
        loop {
            tok.skip_white();
            if tok.peek_is_symbol(SYMBOL::ClParen) {
                tok.consume();
                break;
            }
            tok.skip_white();
            if tok.peek_is_symbol(SYMBOL::Comma) {
                tok.consume();
                tok.skip_white();
            }
            args.push(parse_field_arg(source, tok)?);
        }
    }
    tok.skip_white();
    expect_symbol(source, tok, SYMBOL::Colon)?;
    let expr = parse_type_expr(source, tok)?;
    let dir_args = parse_dir_args(source, tok)?;
    Ok(Field {
        doc,
        name,
        expr,
        args,
        dir_args,
        _ph: PhantomData,
    })
}

fn parse_type_expr<'a, T>(source: &'a str, tok: &mut TokenIter) -> ParseResult<TypeExpr<'a, T>> {
    tok.skip_white();
    let is_arr = tok.peek_is_symbol(SYMBOL::OpSquar);
    if is_arr {
        tok.consume();
        tok.skip_white();
    }
    let typ = expect_name(source, tok)?;
    let null = !tok.peek_is_symbol(SYMBOL::Exclam);
    if !null {
        tok.consume();
    }
    let arr_null = if is_arr {
        tok.skip_white();
        expect_symbol(source, tok, SYMBOL::ClSquar)?;
        let arr_null = !tok.peek_is_symbol(SYMBOL::Exclam);
        if !arr_null {
            tok.consume();
        }
        arr_null
    } else {
        true
    };
    let arr = if is_arr {
        Arr::Yes { null: arr_null }
    } else {
        Arr::No
    };
    Ok(TypeExpr {
        typ,
        null,
        arr,
        _ph: PhantomData,
    })
}

fn parse_field_arg<'a, T>(source: &'a str, tok: &mut TokenIter) -> ParseResult<FieldArg<'a, T>> {
    let name = expect_name(source, tok)?;
    tok.skip_white();
    expect_symbol(source, tok, SYMBOL::Colon)?;
    let expr = parse_type_expr(source, tok)?;
    tok.skip_white();
    let def = if tok.peek_is_symbol(SYMBOL::Equals) {
        tok.consume();
        tok.skip_white();
        Some(expect_name(source, tok)?)
    } else {
        None
    };
    Ok(FieldArg {
        name,
        expr,
        def,
        _ph: PhantomData,
    })
}

fn parse_dir_args<'a, T>(source: &'a str, tok: &mut TokenIter) -> ParseResult<Vec<DirArg<'a, T>>> {
    let mut args = vec![];
    loop {
        tok.skip_white();
        if tok.peek_is_symbol(SYMBOL::Aruba) {
            args.push(parse_dir_arg(source, tok)?);
        } else {
            break;
        }
    }
    Ok(args)
}

// expect to be positioned on '@' in @can(action: "participate.view")
fn parse_dir_arg<'a, T>(source: &'a str, tok: &mut TokenIter) -> ParseResult<DirArg<'a, T>> {
    tok.consume();
    let name = expect_name(source, tok)?;
    expect_symbol(source, tok, SYMBOL::OpParen)?;
    loop {
        // TODO, parse args
        if tok.peek_is_symbol(SYMBOL::ClParen) {
            tok.consume();
            break;
        }
        tok.consume();
    }
    Ok(DirArg {
        name,
        _ph: PhantomData
    })
}

fn parse_enum<'a, T>(
    source: &'a str,
    tok: &mut TokenIter,
    doc: Option<&'a str>,
) -> ParseResult<Tree<'a, T>> {
    // keyword is "enum" and tok is positioned after that
    tok.skip_white();
    let name = expect_name(source, tok)?;
    tok.skip_white();
    expect_symbol(source, tok, SYMBOL::OpCurl)?;
    let mut values = vec![];
    loop {
        tok.skip_white();
        if tok.peek_is_symbol(SYMBOL::ClCurl) {
            tok.consume();
            break;
        }
        let doc = parse_doc(source, tok)?;
        tok.skip_white();
        let value = expect_name(source, tok)?;
        tok.skip_white();
        if tok.peek_is_symbol(SYMBOL::Comma) {
            tok.consume();
        }
        values.push(EnumValue {
            doc,
            value,
            _ph: PhantomData,
        });
    }
    Ok(Tree::En(Enum {
        doc,
        name,
        values,
        _ph: PhantomData,
    }))
}

fn parse_union<'a, T>(
    source: &'a str,
    tok: &mut TokenIter,
    doc: Option<&'a str>,
) -> ParseResult<Tree<'a, T>> {
    // keyword is "union" and tok is positioned after that
    tok.skip_white();
    let name = expect_name(source, tok)?;
    tok.skip_white();
    expect_symbol(source, tok, SYMBOL::Equals)?;
    let mut names = vec![];
    loop {
        tok.skip_white();
        names.push(expect_name(source, tok)?);
        tok.skip_white();
        if !tok.peek_is_symbol(SYMBOL::Pipe) {
            break;
        }
    }
    Ok(Tree::Un(Union {
        doc,
        name,
        names,
        _ph: PhantomData,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conv_pass::Pass;

    #[test]
    fn parse_type_no_doc() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            type Participant {
              _id: ID
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "type Participant {\
             \n  _id: ID\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_some_doc() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            "Some doc"
            type Participant {
              _id: ID!
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "\"Some doc\"\
             \ntype Participant {\
             \n  _id: ID!\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_full_doc() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            "Some doc"
            type Participant {
            """Even "more" doc"""
              _id: ID
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "\"Some doc\"\
             \ntype Participant {\
             \n  \"\"\"Even \"more\" doc\"\"\"\
             \n  _id: ID\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_field_args() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            type Query {
              recording(_id: ID!): Recording
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "type Query {\
             \n  recording(_id: ID!): Recording\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_multi_field_args() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            type Mutation {
              deleteUser(userId: ID!, dryRun: Boolean) : DeleteUserResponse
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "type Mutation {\
             \n  deleteUser(userId: ID!, dryRun: Boolean): DeleteUserResponse\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_array_field_nullable() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            type User {
              projects: [ Project! ]
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "type User {\
             \n  projects: [Project!]\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_array_field() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            type User {
              projects: [ Project! ]!
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "type User {\
             \n  projects: [Project!]!\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_array_field_args() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            type Query {
              usersByEmail(email: [ String! ]): [ User ]
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "type Query {\
             \n  usersByEmail(email: [String!]): [User]\
             \n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_type_with_field_comment() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            type Query {
                # access control kicks in on User level
                user(_id: ID!): User
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "type Query {\n  user(_id: ID!): User\n}\n"
        );
        Ok(())
    }

    #[test]
    fn parse_enum_with_comma() -> ParseResult<()> {
        let r = parse::<Pass>(
            r#"
            enum Foo {
                Value1,
                Value2,
            }"#,
        )?;
        assert_eq!(
            r.to_string(),
            "enum Foo {\n  Value1,\n  Value2,\n}\n"
        );
        Ok(())
    }


}
