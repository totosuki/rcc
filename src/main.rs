use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprint!("引数の個数が正しくありません\n");
        process::exit(1);
    }

    print!(".intel_syntax noprefix\n");
    print!(".globl main\n");
    print!("main:\n");
    print!("  mov rax, {}\n", args[1]);
    print!("  ret\n");
}
