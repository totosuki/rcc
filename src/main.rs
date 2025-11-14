use std::env;
use std::process;

#[derive(PartialEq, Eq)]
enum TokenKind {
    TK_RESERVED,
    TK_NUM,
    TK_EOF,
}

struct Token {
    kind: TokenKind,
    val: usize,
    str: Vec<char>,
}

impl Token {
    pub fn new(kind: TokenKind, val: usize, str: Vec<char>) -> Self {
        Token {kind, val, str}
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize, // 現在位置
}

impl Parser  {
    pub fn new(tokens: Vec<Token>, pos: usize) -> Self {
        Parser {tokens, pos}
    }

    fn consume(&mut self, op: char) -> bool {
        let token = &self.tokens[self.pos];
        if token.kind != TokenKind::TK_RESERVED || token.str[0] != op {
            return false;
        }
        self.pos += 1;
        true
    }

    fn expect(&mut self, op: char) {
        let token = &self.tokens[self.pos];
        if token.kind != TokenKind::TK_RESERVED || token.str[0] != op {
            error(&format!("{}ではありません", op));
        }
        self.pos += 1;
    }

    fn expect_number(&mut self) -> usize {
        let token = &self.tokens[self.pos];
        if token.kind != TokenKind::TK_NUM {
            error("数ではありません");
        }
        let val = token.val;
        self.pos += 1;
        val
    }

    fn at_eof(&self) -> bool {
        let token = &self.tokens[self.pos];
        token.kind == TokenKind::TK_EOF
    }

    fn new_token(&mut self, kind: TokenKind, val: Option<usize>, str: Vec<char>) {
        match val {
            Some(v) => self.tokens.push(Token::new(kind, v, str)),
            None => self.tokens.push(Token::new(kind, 0, str)),
        };
    }

    fn tokenize(&mut self, text: String) {
        let text: Vec<char> = text.chars().collect();
        let mut p: usize = 0;
        let mut t: char;

        while p < text.len() {
            t = text[p];
            if t.is_whitespace() {
                p += 1;
                continue;
            }

            if t == '+' || t == '-' {
                self.new_token(TokenKind::TK_RESERVED, None, vec![t]);
                p += 1;
                continue;
            }

            let mut chars: Vec<char> = vec![];
            while t.is_digit(10) {
                chars.push(t);
                p += 1;
                if p >= text.len() {
                    break;
                }
                t = text[p];
            }
            if chars.len() > 0 {
                let numstr: String = chars.iter().collect();
                let num = Some(numstr.parse::<usize>().unwrap());
                self.new_token(TokenKind::TK_NUM, num, chars);
                continue;
            }

            error("トークナイズできません。")
        }

        self.new_token(TokenKind::TK_EOF, None, vec![]);
    }
}

fn error(fmt: &str) -> ! {
    eprintln!("{}", fmt);
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprint!("引数の個数が正しくありません\n");
        process::exit(1);
    }

    let mut parser: Parser = Parser::new(vec![], 0);
    parser.tokenize(args[1].clone());

    print!(".intel_syntax noprefix\n");
    print!(".globl main\n");
    print!("main:\n");

    print!("  mov rax, {}\n", parser.expect_number());

    while !parser.at_eof() {
        if parser.consume('+') {
            print!("  add rax, {}\n", parser.expect_number());
            continue;
        }

        parser.expect('-');
        print!("  sub rax, {}\n", parser.expect_number());
    }

    print!("  ret\n");
}
