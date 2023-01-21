mod ast;
mod codegen;
mod lex;
mod parse;

fn main() {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        panic!("Invalid number of arguments");
    }
    let input = argv[1].clone();

    let ast = match parse::parse(input) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("");

    // println!("main:");
    // println!("  # prologue");
    // println!("  push rbp");
    // println!("  mov rbp, rsp");
    // println!("");
    
    codegen::gen(ast);
}
