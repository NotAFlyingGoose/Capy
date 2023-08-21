pub mod validation;

use syntax::{NodeKind, SyntaxNode, SyntaxToken, SyntaxTree, TokenKind};
use text_size::{TextRange, TextSize};

pub trait AstNode: Copy + Sized {
    fn cast(node: SyntaxNode, tree: &SyntaxTree) -> Option<Self>;

    fn syntax(self) -> SyntaxNode;

    fn text(self, tree: &SyntaxTree) -> &str {
        self.syntax().text(tree)
    }

    fn range(self, tree: &SyntaxTree) -> TextRange {
        self.syntax().range(tree)
    }
}

pub trait AstToken: Sized {
    fn cast(token: SyntaxToken, tree: &SyntaxTree) -> Option<Self>;

    fn syntax(self) -> SyntaxToken;

    fn text(self, tree: &SyntaxTree) -> &str {
        self.syntax().text(tree)
    }

    fn range(self, tree: &SyntaxTree) -> TextRange {
        self.syntax().range(tree)
    }

    fn range_after(self, tree: &SyntaxTree) -> TextRange {
        let range = self.syntax().range(tree);
        TextRange::new(range.end(), range.end() + TextSize::from(1))
    }
}

macro_rules! def_ast_node {
    ($kind:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $kind(SyntaxNode);

        impl AstNode for $kind {
            fn cast(node: SyntaxNode, tree: &SyntaxTree) -> Option<Self> {
                if node.kind(tree) == NodeKind::$kind {
                    Some(Self(node))
                } else {
                    None
                }
            }

            fn syntax(self) -> SyntaxNode {
                self.0
            }
        }

        impl std::fmt::Debug for $kind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("$kind").finish()
            }
        }
    };
}

macro_rules! def_ast_token {
    ($kind:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $kind(SyntaxToken);

        impl AstToken for $kind {
            fn cast(token: SyntaxToken, tree: &SyntaxTree) -> Option<Self> {
                if token.kind(tree) == TokenKind::$kind {
                    Some(Self(token))
                } else {
                    None
                }
            }

            fn syntax(self) -> SyntaxToken {
                self.0
            }
        }

        impl std::fmt::Debug for $kind {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("$kind").finish()
            }
        }
    };
}

macro_rules! def_multi_node {
    (
        $node_name:ident:
        $($simple_child_name:ident -> $simple_child_node_kind:ident)*
        ;
        $($multi_child_name:ident -> $multi_child_node_kind:ident)*
        ;
        $(fn $fn_name:ident () -> $fn_return_ty:ty)*
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $node_name {
            $($simple_child_name($simple_child_node_kind),)*
            $($multi_child_name($multi_child_node_kind),)*
        }

        def_multi_node!(@create_impl $node_name $($fn_name -> $fn_return_ty)* ; ($($simple_child_name -> $simple_child_node_kind),*) @ ($($multi_child_name -> $multi_child_node_kind),*));

        impl AstNode for $node_name {
            fn cast(node: SyntaxNode, tree: &SyntaxTree) -> Option<Self> {
                $(
                    if let Some(inner) = $multi_child_node_kind::cast(node, tree) {
                        return Some(Self::$multi_child_name(inner))
                    }
                )*

                match node.kind(tree) {
                    $(NodeKind::$simple_child_node_kind =>
                        Some(Self::$simple_child_name($simple_child_node_kind(node))),)*
                    _ => None,
                }
            }

            fn syntax(self) -> SyntaxNode {
                match self {
                    $(Self::$simple_child_name(node) => node.syntax(),)*
                    $(Self::$multi_child_name(node) => node.syntax(),)*
                }
            }
        }

        impl std::fmt::Debug for $node_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$simple_child_name(node) => f.debug_tuple("$simple_child_name").field(node).finish(),)*
                    $(Self::$multi_child_name(node) => f.debug_tuple("$multi_child_name").field(node).finish(),)*
                }
            }
        }
    };
    (@create_impl $node_name:ident $($fn_name:ident -> $fn_return_ty:ty)* ; $simple:tt @ $multi:tt) => {
        impl $node_name {
            $(
                def_multi_node!(@create_fn $fn_name -> $fn_return_ty ; $simple ; $multi);
            )*
        }
    };
    (
        @create_fn
        $fn_name:ident -> $fn_return_ty:ty
        ;
        ($($simple_child_name:ident -> $simple_child_node_kind:ident),*)
        ;
        ($($multi_child_name:ident -> $multi_child_node_kind:ident),*)
    ) => {
        #[allow(dead_code)]
        pub fn $fn_name(self, tree: &SyntaxTree) -> $fn_return_ty {
            match self {
                $(Self::$simple_child_name(inner) => inner.$fn_name(tree),)*
                $(Self::$multi_child_name(inner) => inner.$fn_name(tree),)*
            }
        }
    };
}

def_ast_node!(Root);

impl Root {
    pub fn defs(self, tree: &SyntaxTree) -> impl Iterator<Item = Define> + '_ {
        self.stmts(tree).filter_map(|stmt| match stmt {
            Stmt::Define(def) => Some(def),
            _ => None,
        })
    }

    pub fn stmts(self, tree: &SyntaxTree) -> impl Iterator<Item = Stmt> + '_ {
        nodes(self, tree)
    }
}

def_ast_node!(Lambda);

impl Lambda {
    pub fn param_list(self, tree: &SyntaxTree) -> Option<ParamList> {
        node(self, tree)
    }

    pub fn return_ty(self, tree: &SyntaxTree) -> Option<Ty> {
        node(self, tree)
    }

    pub fn body(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }

    pub fn r#extern(self, tree: &SyntaxTree) -> Option<Extern> {
        token(self, tree)
    }
}

def_multi_node! {
    Stmt:
    Assign -> Assign
    Expr -> ExprStmt
    ;
    Define -> Define
    ;
}

def_multi_node! {
    Define:
    Binding -> Binding
    Variable -> VarDef
    ;
    ;
    fn name() -> Option<Ident>
    fn colon() -> Option<Colon>
    fn ty() -> Option<Ty>
    fn value() -> Option<Expr>
}

def_ast_node!(Binding);

impl Binding {
    pub fn name(self, tree: &SyntaxTree) -> Option<Ident> {
        token(self, tree)
    }

    pub fn colon(self, tree: &SyntaxTree) -> Option<Colon> {
        token(self, tree)
    }

    pub fn ty(self, tree: &SyntaxTree) -> Option<Ty> {
        node(self, tree)
    }

    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(VarDef);

impl VarDef {
    pub fn name(self, tree: &SyntaxTree) -> Option<Ident> {
        token(self, tree)
    }

    pub fn colon(self, tree: &SyntaxTree) -> Option<Colon> {
        token(self, tree)
    }

    pub fn ty(self, tree: &SyntaxTree) -> Option<Ty> {
        node(self, tree)
    }

    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(Assign);

impl Assign {
    pub fn source(self, tree: &SyntaxTree) -> Option<Source> {
        node(self, tree)
    }

    pub fn equals(self, tree: &SyntaxTree) -> Option<Equals> {
        token(self, tree)
    }

    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(ParamList);

impl ParamList {
    pub fn params(self, tree: &SyntaxTree) -> impl Iterator<Item = Param> + '_ {
        nodes(self, tree)
    }
}

def_ast_node!(Param);

impl Param {
    pub fn name(self, tree: &SyntaxTree) -> Option<Ident> {
        token(self, tree)
    }

    pub fn ty(self, tree: &SyntaxTree) -> Option<Ty> {
        node(self, tree)
    }
}

def_ast_node!(Ty);

impl Ty {
    /// types are really just expressions
    pub fn expr(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(ExprStmt);

impl ExprStmt {
    pub fn expr(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_multi_node! {
    Expr:
    Cast -> CastExpr
    Ref -> RefExpr
    Deref -> DerefExpr
    Binary -> BinaryExpr
    Unary -> UnaryExpr
    IntLiteral -> IntLiteral
    BoolLiteral -> BoolLiteral
    StringLiteral -> StringLiteral
    Array -> Array
    IndexExpr -> IndexExpr
    VarRef -> VarRef // a reference to a local, global, or parameter. e.g. `foo` in `foo * 2`
    Call -> Call
    Block -> Block
    If -> IfExpr
    While -> WhileExpr
    Distinct -> Distinct
    Lambda -> Lambda
    ;
    ;
}

def_ast_node!(BinaryExpr);

impl BinaryExpr {
    pub fn lhs(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }

    pub fn rhs(self, tree: &SyntaxTree) -> Option<Expr> {
        nodes(self, tree).nth(1)
    }

    pub fn op(self, tree: &SyntaxTree) -> Option<BinaryOp> {
        token(self, tree)
    }
}

def_ast_node!(CastExpr);

impl CastExpr {
    pub fn expr(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }

    pub fn ty(self, tree: &SyntaxTree) -> Option<Ty> {
        node(self, tree)
    }
}

def_ast_node!(RefExpr);

impl RefExpr {
    pub fn mutable(self, tree: &SyntaxTree) -> Option<Mut> {
        token(self, tree)
    }

    pub fn expr(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(DerefExpr);

impl DerefExpr {
    pub fn pointer(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(Distinct);

impl Distinct {
    pub fn ty(self, tree: &SyntaxTree) -> Option<Ty> {
        node(self, tree)
    }
}

def_ast_node!(Block);

impl Block {
    pub fn stmts(self, tree: &SyntaxTree) -> impl Iterator<Item = Stmt> + '_ {
        nodes(self, tree)
    }

    pub fn tail_expr(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(IfExpr);

impl IfExpr {
    pub fn condition(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }

    pub fn body(self, tree: &SyntaxTree) -> Option<Expr> {
        nodes(self, tree).nth(1)
    }

    pub fn else_branch(self, tree: &SyntaxTree) -> Option<ElseBranch> {
        node(self, tree)
    }
}

def_ast_node!(ElseBranch);

impl ElseBranch {
    pub fn body(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(WhileExpr);

impl WhileExpr {
    pub fn condition(self, tree: &SyntaxTree) -> Option<Condition> {
        node(self, tree)
    }

    pub fn body(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(Condition);

impl Condition {
    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(Array);

impl Array {
    pub fn size(self, tree: &SyntaxTree) -> Option<ArraySize> {
        node(self, tree)
    }

    pub fn ty(self, tree: &SyntaxTree) -> Option<Ty> {
        node(self, tree)
    }

    pub fn body(self, tree: &SyntaxTree) -> Option<ArrayBody> {
        node(self, tree)
    }
}

def_ast_node!(ArrayBody);

impl ArrayBody {
    pub fn items(self, tree: &SyntaxTree) -> impl Iterator<Item = ArrayItem> + '_ {
        nodes(self, tree)
    }
}

def_ast_node!(ArraySize);

impl ArraySize {
    pub fn size(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(ArrayItem);

impl ArrayItem {
    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(IndexExpr);

impl IndexExpr {
    pub fn array(self, tree: &SyntaxTree) -> Option<Source> {
        node(self, tree)
    }

    pub fn index(self, tree: &SyntaxTree) -> Option<Index> {
        node(self, tree)
    }
}

def_ast_node!(Index);

impl Index {
    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(Source);

impl Source {
    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(VarRef);

impl VarRef {
    pub fn name(self, tree: &SyntaxTree) -> Option<Path> {
        node(self, tree)
    }
}

def_ast_node!(Call);

impl Call {
    pub fn name(self, tree: &SyntaxTree) -> Option<Path> {
        node(self, tree)
    }

    pub fn arg_list(self, tree: &SyntaxTree) -> Option<ArgList> {
        node(self, tree)
    }
}

def_ast_node!(Path);

impl Path {
    pub fn top_level_name(self, tree: &SyntaxTree) -> Option<Ident> {
        token(self, tree)
    }

    pub fn nested_name(self, tree: &SyntaxTree) -> Option<Ident> {
        tokens(self, tree).nth(1)
    }
}

def_ast_node!(ArgList);

impl ArgList {
    pub fn args(self, tree: &SyntaxTree) -> impl Iterator<Item = Arg> + '_ {
        nodes(self, tree)
    }
}

def_ast_node!(Arg);

impl Arg {
    pub fn value(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }
}

def_ast_node!(IntLiteral);

impl IntLiteral {
    pub fn value(self, tree: &SyntaxTree) -> Option<Int> {
        token(self, tree)
    }
}

def_ast_node!(BoolLiteral);

impl BoolLiteral {
    pub fn value(self, tree: &SyntaxTree) -> Option<Bool> {
        token(self, tree)
    }
}

def_ast_node!(StringLiteral);

impl StringLiteral {
    pub fn components(self, tree: &SyntaxTree) -> impl Iterator<Item = StringComponent> + '_ {
        tokens(self, tree)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // math operations
    Add(Plus),
    Sub(Hyphen),
    Mul(Asterisk),
    Div(Slash),

    // cmp operations
    Lt(Less),
    Gt(Greater),
    Le(LessEquals),
    Ge(GreaterEquals),
    Eq(DoubleEquals),
    Ne(BangEquals),

    // boolean operations
    And(DoubleAnd),
    Or(DoublePipe),
}

impl AstToken for BinaryOp {
    fn cast(token: SyntaxToken, tree: &SyntaxTree) -> Option<Self> {
        match token.kind(tree) {
            TokenKind::Plus => Some(Self::Add(Plus(token))),
            TokenKind::Hyphen => Some(Self::Sub(Hyphen(token))),
            TokenKind::Asterisk => Some(Self::Mul(Asterisk(token))),
            TokenKind::Slash => Some(Self::Div(Slash(token))),
            TokenKind::Less => Some(Self::Lt(Less(token))),
            TokenKind::Greater => Some(Self::Gt(Greater(token))),
            TokenKind::LessEquals => Some(Self::Le(LessEquals(token))),
            TokenKind::GreaterEquals => Some(Self::Ge(GreaterEquals(token))),
            TokenKind::DoubleEquals => Some(Self::Eq(DoubleEquals(token))),
            TokenKind::BangEquals => Some(Self::Ne(BangEquals(token))),
            TokenKind::DoubleAnd => Some(Self::And(DoubleAnd(token))),
            TokenKind::DoublePipe => Some(Self::Or(DoublePipe(token))),
            _ => None,
        }
    }

    fn syntax(self) -> SyntaxToken {
        match self {
            Self::Add(plus) => plus.syntax(),
            Self::Sub(hyphen) => hyphen.syntax(),
            Self::Mul(asterisk) => asterisk.syntax(),
            Self::Div(slash) => slash.syntax(),
            Self::Lt(less) => less.syntax(),
            Self::Gt(greater) => greater.syntax(),
            Self::Le(less_equals) => less_equals.syntax(),
            Self::Ge(greater_equals) => greater_equals.syntax(),
            Self::Eq(double_equals) => double_equals.syntax(),
            Self::Ne(bang_equals) => bang_equals.syntax(),
            Self::And(double_and) => double_and.syntax(),
            Self::Or(double_pipe) => double_pipe.syntax(),
        }
    }
}

def_ast_node!(UnaryExpr);

impl UnaryExpr {
    pub fn expr(self, tree: &SyntaxTree) -> Option<Expr> {
        node(self, tree)
    }

    pub fn op(self, tree: &SyntaxTree) -> Option<UnaryOp> {
        token(self, tree)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    // math operations
    Pos(Plus),
    Neg(Hyphen),

    // boolean operations
    Not(Bang),
}

impl AstToken for UnaryOp {
    fn cast(token: SyntaxToken, tree: &SyntaxTree) -> Option<Self> {
        match token.kind(tree) {
            TokenKind::Plus => Some(Self::Pos(Plus(token))),
            TokenKind::Hyphen => Some(Self::Neg(Hyphen(token))),
            TokenKind::Bang => Some(Self::Not(Bang(token))),
            _ => None,
        }
    }

    fn syntax(self) -> SyntaxToken {
        match self {
            Self::Pos(plus) => plus.syntax(),
            Self::Neg(hyphen) => hyphen.syntax(),
            Self::Not(bang) => bang.syntax(),
        }
    }
}

def_ast_token!(Mut);
def_ast_token!(Extern);
def_ast_token!(Colon);
def_ast_token!(Plus);
def_ast_token!(Hyphen);
def_ast_token!(Asterisk);
def_ast_token!(Slash);
def_ast_token!(Less);
def_ast_token!(Greater);
def_ast_token!(LessEquals);
def_ast_token!(GreaterEquals);
def_ast_token!(DoubleEquals);
def_ast_token!(BangEquals);
def_ast_token!(Equals);
def_ast_token!(Bang);
def_ast_token!(DoubleAnd);
def_ast_token!(DoublePipe);
def_ast_token!(Ident);
def_ast_token!(Int);
def_ast_token!(Bool);

pub enum StringComponent {
    Escape(Escape),
    Contents(StringContents),
}

impl AstToken for StringComponent {
    fn cast(token: SyntaxToken, tree: &SyntaxTree) -> Option<Self> {
        match token.kind(tree) {
            TokenKind::Escape => Some(Self::Escape(Escape(token))),
            TokenKind::StringContents => Some(Self::Contents(StringContents(token))),
            _ => None,
        }
    }

    fn syntax(self) -> SyntaxToken {
        match self {
            Self::Escape(escape) => escape.syntax(),
            Self::Contents(contents) => contents.syntax(),
        }
    }
}

def_ast_token!(Escape);
def_ast_token!(StringContents);

fn nodes<Parent: AstNode, Child: AstNode>(
    node: Parent,
    tree: &SyntaxTree,
) -> impl Iterator<Item = Child> + '_ {
    node.syntax()
        .child_nodes(tree)
        .filter_map(|n| Child::cast(n, tree))
}

fn node<Parent: AstNode, Child: AstNode>(node: Parent, tree: &SyntaxTree) -> Option<Child> {
    node.syntax()
        .child_nodes(tree)
        .find_map(|n| Child::cast(n, tree))
}

fn tokens<Node: AstNode, Token: AstToken>(
    node: Node,
    tree: &SyntaxTree,
) -> impl Iterator<Item = Token> + '_ {
    node.syntax()
        .child_tokens(tree)
        .filter_map(|t| Token::cast(t, tree))
}

fn token<Node: AstNode, Token: AstToken>(node: Node, tree: &SyntaxTree) -> Option<Token> {
    node.syntax()
        .child_tokens(tree)
        .find_map(|t| Token::cast(t, tree))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> (SyntaxTree, Root) {
        let parse = parser::parse_repl_line(&lexer::lex(input), input);
        for error in parse.errors() {
            println!("Syntax Error: {:?}", error);
        }
        let tree = parse.into_syntax_tree();
        let root = Root::cast(tree.root(), &tree).unwrap();

        (tree, root)
    }

    #[test]
    fn cast_root() {
        parse("");
    }

    #[test]
    fn get_statements() {
        let (tree, root) = parse("a := b; a;");
        assert_eq!(root.stmts(&tree).count(), 2);
    }

    #[test]
    fn inspect_statement_kind() {
        let (tree, root) = parse("foo := bar; baz * qux;");
        let mut statements = root.stmts(&tree);

        assert!(matches!(statements.next(), Some(Stmt::Define(_))));
        assert!(matches!(statements.next(), Some(Stmt::Expr(_))));
        assert!(statements.next().is_none());
    }

    #[test]
    fn get_name_of_var_def() {
        let (tree, root) = parse("foo := 10;");
        let statement = root.stmts(&tree).next().unwrap();

        let def = match statement {
            Stmt::Define(var_def) => var_def,
            _ => unreachable!(),
        };

        let var_def = match def {
            Define::Variable(var) => var,
            _ => unreachable!(),
        };

        assert_eq!(var_def.name(&tree).unwrap().text(&tree), "foo");
    }

    #[test]
    fn get_named_ty_of_var_def() {
        let (tree, root) = parse("foo : string = 10;");
        let statement = root.stmts(&tree).next().unwrap();

        let def = match statement {
            Stmt::Define(var_def) => var_def,
            _ => unreachable!(),
        };

        let var_def = match def {
            Define::Variable(var) => var,
            _ => unreachable!(),
        };

        let ty_ref = match var_def.ty(&tree).unwrap().expr(&tree) {
            Some(Expr::VarRef(name)) => name,
            _ => unreachable!(),
        };

        let path = ty_ref.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "string");
        assert!(path.nested_name(&tree).is_none());
    }

    #[test]
    fn get_array_ty_of_var_def() {
        let (tree, root) = parse("foo : [3]i32 = []i32{1, 2, 3};");
        let statement = root.stmts(&tree).next().unwrap();

        let def = match statement {
            Stmt::Define(var_def) => var_def,
            _ => unreachable!(),
        };

        let var_def = match def {
            Define::Variable(var) => var,
            _ => unreachable!(),
        };

        let array_ty = match var_def.ty(&tree).unwrap().expr(&tree) {
            Some(Expr::Array(array)) => array,
            _ => {
                unreachable!()
            }
        };

        let size = array_ty.size(&tree).unwrap().size(&tree);
        assert!(matches!(size, Some(Expr::IntLiteral(_))));
        assert_eq!(size.unwrap().text(&tree), "3");

        assert!(matches!(array_ty.body(&tree), None));

        let sub_ty = match array_ty.ty(&tree).unwrap().expr(&tree) {
            Some(Expr::VarRef(name)) => name,
            _ => unreachable!(),
        };

        let sub_path = sub_ty.name(&tree).unwrap();

        assert_eq!(sub_path.top_level_name(&tree).unwrap().text(&tree), "i32");
        assert!(sub_path.nested_name(&tree).is_none());
    }

    #[test]
    fn get_value_of_var_def() {
        let (tree, root) = parse("bar := 42;");
        let statement = root.stmts(&tree).next().unwrap();

        let def = match statement {
            Stmt::Define(var_def) => var_def,
            _ => unreachable!(),
        };

        let var_def = match def {
            Define::Variable(var) => var,
            _ => unreachable!(),
        };

        assert!(matches!(var_def.value(&tree), Some(Expr::IntLiteral(_))));
    }

    #[test]
    fn get_name_of_binding() {
        let (tree, root) = parse("foo :: 10;");
        let statement = root.stmts(&tree).next().unwrap();

        let def = match statement {
            Stmt::Define(var_def) => var_def,
            _ => unreachable!(),
        };

        let binding = match def {
            Define::Binding(var) => var,
            _ => unreachable!(),
        };

        assert_eq!(binding.name(&tree).unwrap().text(&tree), "foo");
    }

    #[test]
    fn get_value_of_binding() {
        let (tree, root) = parse("bar :: 42;");
        let statement = root.stmts(&tree).next().unwrap();

        let def = match statement {
            Stmt::Define(var_def) => var_def,
            _ => unreachable!(),
        };

        let binding = match def {
            Define::Binding(var) => var,
            _ => unreachable!(),
        };

        assert!(matches!(binding.value(&tree), Some(Expr::IntLiteral(_))));
    }

    #[test]
    fn get_expr_of_assign() {
        let (tree, root) = parse("foo = 10;");
        let statement = root.stmts(&tree).next().unwrap();

        let assign = match statement {
            Stmt::Assign(var_set) => var_set,
            _ => unreachable!(),
        };

        assert!(matches!(
            assign.source(&tree).unwrap().value(&tree),
            Some(Expr::VarRef(_))
        ));
    }

    #[test]
    fn get_value_of_assign() {
        let (tree, root) = parse("bar = 42;");
        let statement = root.stmts(&tree).next().unwrap();

        let assign = match statement {
            Stmt::Assign(assign) => assign,
            _ => unreachable!(),
        };

        assert!(matches!(assign.value(&tree), Some(Expr::IntLiteral(_))));
    }

    #[test]
    fn get_lhs_and_rhs_of_binary_expr() {
        let (tree, root) = parse("foo - 42;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let binary_expr = match expr {
            Some(Expr::Binary(binary_expr)) => binary_expr,
            _ => unreachable!(),
        };

        assert!(matches!(binary_expr.lhs(&tree), Some(Expr::VarRef(_))));
        assert!(matches!(binary_expr.rhs(&tree), Some(Expr::IntLiteral(_))));
    }

    #[test]
    fn get_op_of_binary_expr() {
        let (tree, root) = parse("six * 7;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let binary_expr = match expr {
            Some(Expr::Binary(binary_expr)) => binary_expr,
            _ => unreachable!(),
        };

        assert!(matches!(binary_expr.op(&tree), Some(BinaryOp::Mul(_))));
    }

    #[test]
    fn get_expr_of_unary_expr() {
        let (tree, root) = parse("-foo;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let unary_expr = match expr {
            Some(Expr::Unary(unary_expr)) => unary_expr,
            _ => unreachable!(),
        };

        assert!(matches!(unary_expr.expr(&tree), Some(Expr::VarRef(_))));
    }

    #[test]
    fn get_op_of_unary_expr() {
        let (tree, root) = parse("+7;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let unary_expr = match expr {
            Some(Expr::Unary(unary_expr)) => unary_expr,
            _ => unreachable!(),
        };

        assert!(matches!(unary_expr.op(&tree), Some(UnaryOp::Pos(_))));
    }

    #[test]
    fn get_ty_of_array() {
        let (tree, root) = parse("[]bool{true, false}");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let array_expr = match expr {
            Some(Expr::Array(array)) => array,
            _ => unreachable!(),
        };

        assert_eq!(array_expr.ty(&tree).unwrap().text(&tree), "bool");
    }

    #[test]
    fn get_items_of_array() {
        let (tree, root) = parse("[]i32{4, 8, 15, 16, 23, 42}");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let array_expr = match expr {
            Some(Expr::Array(array)) => array,
            _ => unreachable!(),
        };

        let mut items = array_expr.body(&tree).unwrap().items(&tree);

        assert_eq!(items.next().unwrap().text(&tree), "4");
        assert_eq!(items.next().unwrap().text(&tree), "8");
        assert_eq!(items.next().unwrap().text(&tree), "15");
        assert_eq!(items.next().unwrap().text(&tree), "16");
        assert_eq!(items.next().unwrap().text(&tree), "23");
        assert_eq!(items.next().unwrap().text(&tree), "42");
        assert!(items.next().is_none());
    }

    #[test]
    fn get_array_of_index() {
        let (tree, root) = parse("my_array[0]");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let index_expr = match expr {
            Some(Expr::IndexExpr(index_expr)) => index_expr,
            _ => unreachable!(),
        };

        let array_ref = match index_expr.array(&tree).unwrap().value(&tree) {
            Some(Expr::VarRef(array_ref)) => array_ref,
            _ => unreachable!(),
        };
        let path = array_ref.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "my_array");
        assert!(path.nested_name(&tree).is_none());
    }

    #[test]
    fn get_int_index_of_index() {
        let (tree, root) = parse("list[2]");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let index_expr = match expr {
            Some(Expr::IndexExpr(index_expr)) => index_expr,
            _ => unreachable!(),
        };

        let index_num = match index_expr.index(&tree).unwrap().value(&tree) {
            Some(Expr::IntLiteral(index_index)) => index_index,
            _ => unreachable!(),
        };

        assert_eq!(index_num.value(&tree).unwrap().text(&tree), "2");
    }

    #[test]
    fn get_ref_index_of_index() {
        let (tree, root) = parse("arr[idx]");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let index_expr = match expr {
            Some(Expr::IndexExpr(index_expr)) => index_expr,
            _ => unreachable!(),
        };

        let index_ref = match index_expr.index(&tree).unwrap().value(&tree) {
            Some(Expr::VarRef(index_index)) => index_index,
            _ => unreachable!(),
        };
        let path = index_ref.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "idx");
        assert!(path.nested_name(&tree).is_none());
    }

    #[test]
    fn get_name_of_var_ref() {
        let (tree, root) = parse("idx;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let var_ref = match expr {
            Some(Expr::VarRef(var_ref)) => var_ref,
            _ => unreachable!(),
        };
        let path = var_ref.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "idx");
        assert!(path.nested_name(&tree).is_none());
    }

    #[test]
    fn get_name_of_call() {
        let (tree, root) = parse("print();");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let call = match expr {
            Some(Expr::Call(call)) => call,
            _ => unreachable!(),
        };
        let path = call.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "print");
        assert!(path.nested_name(&tree).is_none());
    }

    #[test]
    fn get_expr_of_ref() {
        let (tree, root) = parse("^foo;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let ref_expr = match expr {
            Some(Expr::Ref(ref_expr)) => ref_expr,
            _ => unreachable!(),
        };

        let var_ref = match ref_expr.expr(&tree) {
            Some(Expr::VarRef(ref_expr)) => ref_expr,
            _ => unreachable!(),
        };
        let path = var_ref.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "foo");
        assert!(path.nested_name(&tree).is_none());
        assert!(ref_expr.mutable(&tree).is_none());
    }

    #[test]
    fn get_mut_of_ref() {
        let (tree, root) = parse("^mut foo;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let ref_expr = match expr {
            Some(Expr::Ref(ref_expr)) => ref_expr,
            _ => unreachable!(),
        };

        let var_ref = match ref_expr.expr(&tree) {
            Some(Expr::VarRef(ref_expr)) => ref_expr,
            _ => unreachable!(),
        };
        let path = var_ref.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "foo");
        assert!(path.nested_name(&tree).is_none());
        assert!(ref_expr.mutable(&tree).is_some());
    }

    #[test]
    fn get_args_of_call() {
        let (tree, root) = parse("mul(10, 20);");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let call = match expr {
            Some(Expr::Call(call)) => call,
            _ => unreachable!(),
        };

        let mut args = call.arg_list(&tree).unwrap().args(&tree);

        assert_eq!(args.next().unwrap().value(&tree).unwrap().text(&tree), "10");
        assert_eq!(args.next().unwrap().value(&tree).unwrap().text(&tree), "20");
        assert!(args.next().is_none());
    }

    #[test]
    fn get_value_of_int_literal() {
        let (tree, root) = parse("92;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let int_literal = match expr {
            Some(Expr::IntLiteral(int_literal)) => int_literal,
            _ => unreachable!(),
        };

        assert_eq!(int_literal.value(&tree).unwrap().text(&tree), "92");
    }

    #[test]
    fn get_components_of_string_literal() {
        let (tree, root) = parse(r#""\"Hello\"";"#);
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let string_literal = match expr {
            Some(Expr::StringLiteral(string_literal)) => string_literal,
            _ => unreachable!(),
        };

        let mut components = string_literal.components(&tree);

        let escaped_quote = match components.next() {
            Some(StringComponent::Escape(escape)) => escape,
            _ => unreachable!(),
        };
        assert_eq!(escaped_quote.text(&tree), "\\\"");

        let text = match components.next() {
            Some(StringComponent::Contents(contents)) => contents,
            _ => unreachable!(),
        };
        assert_eq!(text.text(&tree), "Hello");

        let escaped_quote = match components.next() {
            Some(StringComponent::Escape(escape)) => escape,
            _ => unreachable!(),
        };
        assert_eq!(escaped_quote.text(&tree), "\\\"");

        assert!(components.next().is_none());
    }

    #[test]
    fn get_block_stmts() {
        let (tree, root) = parse("{ a := 10; b = a * {a - 1}; b + 5 };");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let block = match expr {
            Some(Expr::Block(block)) => block,
            _ => unreachable!(),
        };

        let mut statements = block.stmts(&tree);

        assert!(matches!(statements.next(), Some(Stmt::Define(_))));
        assert!(matches!(statements.next(), Some(Stmt::Assign(_))));
        assert!(statements.next().is_none());
    }

    #[test]
    fn get_block_tail() {
        let (tree, root) = parse("{ a = 10; b = a * {a - 1}; b + 5 };");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let block = match expr {
            Some(Expr::Block(block)) => block,
            _ => unreachable!(),
        };

        let tail_expr = block.tail_expr(&tree);

        assert!(matches!(tail_expr, Some(Expr::Binary(_))));
    }

    #[test]
    fn get_if_condition() {
        let (tree, root) = parse("if true {}");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let if_expr = match expr {
            Some(Expr::If(if_expr)) => if_expr,
            _ => unreachable!(),
        };

        let condition = if_expr.condition(&tree);

        assert!(matches!(condition, Some(Expr::BoolLiteral(_))));
    }

    #[test]
    fn get_if_body() {
        let (tree, root) = parse("if false { foo(); }");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let if_expr = match expr {
            Some(Expr::If(if_expr)) => if_expr,
            _ => unreachable!(),
        };

        let block = match if_expr.body(&tree).unwrap() {
            Expr::Block(block) => block,
            _ => unreachable!(),
        };

        let mut statements = block.stmts(&tree);

        assert!(matches!(statements.next(), Some(Stmt::Expr(_))));
    }

    #[test]
    fn get_else_body() {
        let (tree, root) = parse("if false {} else { x := 42; }");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let if_expr = match expr {
            Some(Expr::If(if_expr)) => if_expr,
            _ => unreachable!(),
        };

        let else_branch = if_expr.else_branch(&tree).unwrap();

        let block = match else_branch.body(&tree).unwrap() {
            Expr::Block(block) => block,
            _ => unreachable!(),
        };

        let mut statements = block.stmts(&tree);

        assert!(matches!(statements.next(), Some(Stmt::Define(_))));
    }

    #[test]
    fn get_else_if_condition() {
        let (tree, root) = parse("if false {} else if true {}");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let if_expr = match expr {
            Some(Expr::If(if_expr)) => if_expr,
            _ => unreachable!(),
        };

        let else_branch = if_expr.else_branch(&tree).unwrap();

        let if_expr = match else_branch.body(&tree).unwrap() {
            Expr::If(if_expr) => if_expr,
            _ => unreachable!(),
        };

        let condition = if_expr.condition(&tree);

        assert!(matches!(condition, Some(Expr::BoolLiteral(_))));
    }

    #[test]
    fn get_while_condition() {
        let (tree, root) = parse("while true {}");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let while_expr = match expr {
            Some(Expr::While(while_expr)) => while_expr,
            _ => unreachable!(),
        };

        let condition = while_expr.condition(&tree).unwrap().value(&tree);

        assert!(matches!(condition, Some(Expr::BoolLiteral(_))));
    }

    #[test]
    fn get_while_body() {
        let (tree, root) = parse("while true { bar(); }");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let while_expr = match expr {
            Some(Expr::While(while_expr)) => while_expr,
            _ => unreachable!(),
        };

        let block = match while_expr.body(&tree).unwrap() {
            Expr::Block(block) => block,
            _ => unreachable!(),
        };

        let mut statements = block.stmts(&tree);

        assert!(matches!(statements.next(), Some(Stmt::Expr(_))));
    }

    #[test]
    fn get_loop_condition() {
        let (tree, root) = parse("loop { bar(); }");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let while_expr = match expr {
            Some(Expr::While(while_expr)) => while_expr,
            _ => unreachable!(),
        };

        assert!(while_expr.condition(&tree).is_none());
    }

    #[test]
    fn get_lambda_params() {
        let (tree, root) = parse("(x: i32, y: i32) {};");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let lambda = match expr {
            Some(Expr::Lambda(lambda)) => lambda,
            _ => unreachable!(),
        };

        let mut params = lambda.param_list(&tree).unwrap().params(&tree);

        let param = params.next().unwrap();
        assert_eq!(param.name(&tree).unwrap().text(&tree), "x");
        assert_eq!(param.ty(&tree).unwrap().text(&tree), "i32");

        let param = params.next().unwrap();
        assert_eq!(param.name(&tree).unwrap().text(&tree), "y");
        assert_eq!(param.ty(&tree).unwrap().text(&tree), "i32");

        assert!(params.next().is_none());
    }

    #[test]
    fn get_lambda_return_ty() {
        let (tree, root) = parse("() -> i32 {};");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let lambda = match expr {
            Some(Expr::Lambda(lambda)) => lambda,
            _ => unreachable!(),
        };

        let ty_ref = match lambda.return_ty(&tree).unwrap().expr(&tree) {
            Some(Expr::VarRef(ty_ref)) => ty_ref,
            _ => unreachable!(),
        };

        let path = ty_ref.name(&tree).unwrap();

        assert_eq!(path.top_level_name(&tree).unwrap().text(&tree), "i32");
        assert!(path.nested_name(&tree).is_none());
    }

    #[test]
    fn get_lambda_body() {
        let (tree, root) = parse("() -> {};");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let lambda = match expr {
            Some(Expr::Lambda(lambda)) => lambda,
            _ => unreachable!(),
        };

        let block = match lambda.body(&tree).unwrap() {
            Expr::Block(block) => block,
            _ => unreachable!(),
        };

        assert!(block.stmts(&tree).next().is_none());
    }

    #[test]
    fn get_lambda_extern() {
        let (tree, root) = parse("() -> extern;");
        let statement = root.stmts(&tree).next().unwrap();
        let expr = match statement {
            Stmt::Expr(expr_stmt) => expr_stmt.expr(&tree),
            _ => unreachable!(),
        };

        let lambda = match expr {
            Some(Expr::Lambda(lambda)) => lambda,
            _ => unreachable!(),
        };

        assert!(lambda.body(&tree).is_none());
        assert!(lambda.r#extern(&tree).is_some());
    }
}
