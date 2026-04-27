use std::env;
use std::process;

#[derive(PartialEq, Eq)]
enum TokenKind {
    TkReserved,
    TkNum,
    TkEof,
}

#[derive(PartialEq, Eq)]
enum NodeKind {
    NdAdd,
    NdSub,
    NdMul,
    NdDiv,
    NdNum,
}

struct Token {
    kind: TokenKind,
    val: usize, // kindがTkNumの場合
    str: Vec<char>,
    pos: usize, // tokenizer.user_inputにおけるこのトークンの開始位置
    len: usize,
}

impl Token {
    pub fn new(kind: TokenKind, val: usize, str: Vec<char>, pos: usize, len: usize) -> Self {
        Token {
            kind,
            val,
            str,
            pos,
            len,
        }
    }
}

struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: Option<usize>,
}

impl Node {
    pub fn new_node(kind: NodeKind, lhs: Node, rhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
            val: None,
        }
    }

    pub fn new_node_num(val: usize) -> Self {
        Node {
            kind: NodeKind::NdNum,
            lhs: None,
            rhs: None,
            val: Some(val),
        }
    }
}

struct Tokenizer {
    tokens: Vec<Token>,
    pos: usize, // 現在何番目のトークンを見ているか
    user_input: String,
}

impl Tokenizer {
    pub fn new(tokens: Vec<Token>, pos: usize, user_input: String) -> Self {
        Tokenizer {
            tokens,
            pos,
            user_input,
        }
    }

    fn consume(&mut self, op: &[char]) -> bool {
        let token = &self.tokens[self.pos];
        if token.kind != TokenKind::TkReserved || token.str != op {
            return false;
        }
        self.pos += 1;
        true
    }

    fn expect(&mut self, op: &[char]) {
        let token = &self.tokens[self.pos];
        if token.kind != TokenKind::TkReserved || token.str != op {
            error(
                &self.user_input,
                &token.pos,
                &format!("{}ではありません", String::from_iter(op)),
            );
        }
        self.pos += 1;
    }

    fn expect_number(&mut self) -> usize {
        let token = &self.tokens[self.pos];
        if token.kind != TokenKind::TkNum {
            error(&self.user_input, &token.pos, "数ではありません");
        }
        let val = token.val;
        self.pos += 1;
        val
    }

    fn at_eof(&self) -> bool {
        let token = &self.tokens[self.pos];
        token.kind == TokenKind::TkEof
    }

    fn new_token(
        &mut self,
        kind: TokenKind,
        val: Option<usize>, // EOFの場合Noneになる
        str: Vec<char>,
        pos: usize,
        len: usize,
    ) {
        match val {
            Some(v) => self.tokens.push(Token::new(kind, v, str, pos, len)),
            None => self.tokens.push(Token::new(kind, 0, str, pos, len)),
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

            if text[p..].starts_with(&['=', '='])
                || text[p..].starts_with(&['!', '='])
                || text[p..].starts_with(&['>', '='])
                || text[p..].starts_with(&['<', '='])
            {
                self.new_token(
                    TokenKind::TkReserved,
                    None,
                    vec![text[p], text[p + 1]],
                    p,
                    2,
                );
                p += 2;
                continue;
            }

            if "+-*/()><".contains(t) {
                self.new_token(TokenKind::TkReserved, None, vec![t], p, 1);
                p += 1;
                continue;
            }

            let mut chars: Vec<char> = vec![];
            let mut len: usize = 0;
            while t.is_digit(10) {
                chars.push(t);
                p += 1;
                len += 1;
                if p >= text.len() {
                    break;
                }
                t = text[p];
            }
            if chars.len() > 0 {
                let numstr: String = chars.iter().collect();
                let num = Some(numstr.parse::<usize>().unwrap());
                self.new_token(TokenKind::TkNum, num, chars, p, len);
                continue;
            }

            error(&self.user_input, &p, "トークナイズできません。")
        }

        self.new_token(TokenKind::TkEof, None, vec![], p, 1);
    }
}

struct Parser {
    tokenizer: Tokenizer,
}

impl Parser {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Parser { tokenizer }
    }

    pub fn expr(&mut self) -> Node {
        let mut node: Node = self.mul();

        loop {
            if self.tokenizer.consume(&['+']) {
                node = Node::new_node(NodeKind::NdAdd, node, self.mul());
            } else if self.tokenizer.consume(&['-']) {
                node = Node::new_node(NodeKind::NdSub, node, self.mul());
            } else {
                return node;
            }
        }
    }

    pub fn mul(&mut self) -> Node {
        let mut node: Node = self.unary();

        loop {
            if self.tokenizer.consume(&['*']) {
                node = Node::new_node(NodeKind::NdMul, node, self.unary());
            } else if self.tokenizer.consume(&['/']) {
                node = Node::new_node(NodeKind::NdDiv, node, self.unary());
            } else {
                return node;
            }
        }
    }

    pub fn unary(&mut self) -> Node {
        if self.tokenizer.consume(&['+']) {
            self.primary()
        } else if self.tokenizer.consume(&['-']) {
            Node::new_node(NodeKind::NdSub, Node::new_node_num(0), self.primary())
        } else {
            self.primary()
        }
    }

    pub fn primary(&mut self) -> Node {
        if self.tokenizer.consume(&['(']) {
            let node: Node = self.expr();
            self.tokenizer.expect(&[')']);
            return node;
        }

        Node::new_node_num(self.tokenizer.expect_number())
    }
}

fn generate(node: Node) {
    if node.kind == NodeKind::NdNum {
        print!("  push {}\n", node.val.unwrap());
        return;
    }

    generate(*node.lhs.unwrap());
    generate(*node.rhs.unwrap());

    print!("  pop rdi\n");
    print!("  pop rax\n");

    match node.kind {
        NodeKind::NdAdd => {
            print!("  add rax, rdi\n");
        }
        NodeKind::NdSub => {
            print!("  sub rax, rdi\n");
        }
        NodeKind::NdMul => {
            print!("  imul rax, rdi\n");
        }
        NodeKind::NdDiv => {
            print!("  cqo\n");
            print!("  idiv rdi\n");
        }
        _ => (),
    }

    print!("  push rax\n");
}

fn error(user_input: &String, loc: &usize, fmt: &str) -> ! {
    eprintln!("{}", user_input);
    eprint!("{}", " ".repeat(*loc));
    eprintln!("^ {}", fmt);
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprint!("引数の個数が正しくありません\n");
        process::exit(1);
    }

    let mut tokenizer: Tokenizer = Tokenizer::new(vec![], 0, args[1].clone());
    tokenizer.tokenize(args[1].clone());
    let mut parser: Parser = Parser::new(tokenizer);
    let node: Node = parser.expr();

    print!(".intel_syntax noprefix\n");
    print!(".globl main\n");
    print!("main:\n");

    generate(node);

    print!("  pop rax\n");
    print!("  ret\n");
}
