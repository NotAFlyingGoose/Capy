use std::io::{self, Write, Stdout};

use ast::AstNode;
use line_index::LineIndex;
use parser::parse_repl_line;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    writeln!(stdout, "Capy Programming Language 0.1.0")?;

    let mut input = String::new();

    let mut continued = false;
    loop {
        write!(stdout, "{} ", if continued { " " } else { ">" })?;
        continued = false;
        stdout.flush()?;

        stdin.read_line(&mut input)?;
        if input.trim_end().ends_with("\\") {
            input = input.trim_end().to_string();
            input.pop();
            input.push('\n');
            continued = true;
            continue;
        }

        eval(&input, &mut stdout)?;

        input.clear();
    }
}

fn eval(input: &str, stdout: &mut Stdout) -> io::Result<()> {
    let mut interner = interner::Interner::default();

    let parse = parse_repl_line(&lexer::lex(&input), &input);
    writeln!(stdout, "{:?}", parse)?;

    let tree = parse.syntax_tree();

    let syntax_errors = parse
        .errors()
        .iter()
        .cloned()
        .map(diagnostics::Diagnostic::from_syntax);

    let root = ast::Root::cast(tree.root(), tree).unwrap();

    let validation_diagnostics = ast::validation::validate(root, tree);
        
    // let ast_vals = root
    //     .stmts(tree)
    //     .filter_map(|stmt| if let ast::Stmt::VarDef(var_def) = stmt {
    //         Some(var_def.value(tree))
    //     } else if let ast::Stmt::Return(ret) = stmt {
    //         Some(ret.value(tree))
    //     } else {
    //         None
    //     })
    //     .collect::<Vec<_>>();
    // dbg!(ast_vals);

    let world_index = hir::WorldIndex::default();

    let (index, indexing_diagnostics) = hir::index(root, tree, &mut interner);

    let (bodies, lowering_diagnostics) = hir::lower(root, tree, &index, &world_index, &mut interner);

    for name in index.definition_names() {
        println!("{} = {:?}", interner.lookup(name.0), index.get_definition(name))
    }

    println!("{}", bodies.debug(&interner));

    // dbg!(hir::lower(root, tree));

    // hir_typed::infer_all

    let line_index = LineIndex::new(&input);

    let diagnostics: Vec<diagnostics::Diagnostic> = syntax_errors
        .chain(validation_diagnostics.iter()
            .cloned()
            .map(diagnostics::Diagnostic::from_validation))
        .chain(indexing_diagnostics.iter()
            .cloned()
            .map(diagnostics::Diagnostic::from_indexing))
        .chain(lowering_diagnostics.iter()
            .cloned()
            .map(diagnostics::Diagnostic::from_lowering))
        .collect();

    for diagnostic in diagnostics {
        for line in diagnostic.display(&input, &interner, &line_index) {
            write!(stdout, "{}\n", line)?;
        }
    }

    Ok(())
}
