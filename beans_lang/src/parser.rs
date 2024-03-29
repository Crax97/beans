use super::lexer::*;
use super::node::*;
use super::tokens::Token;
use super::tokens::TokenType;
use super::tokens::TokenType::*;
use std::rc::Rc;

#[cfg(test)]
mod tests {
    use super::*;
    fn run_str(s: &str) {
        let source = String::from(s);
        let mut parser = Parser::new(Lexer::new(source));
        assert!(parser.parse().len() > 0);
    }

    #[test]
    fn test_exprs() {
        run_str("2 + 3 + 3.14;");
        run_str("6 * 3.14 - -++-+(3.14 * 2);");
        run_str("2 << 2 == 8;");
        run_str("8 >> 2 == 2;");
        run_str("3 < 2 and (2 > 3 or true);");
        run_str("pi = 3.14;");
        run_str(
            "lambda (x) 
                    2 + 3;
                    end;",
        );
        run_str(
            "lam = lambda (x) 
                    2 + 3;
                    end;",
        );
        run_str("call(3, 4, 5);")
    }

    #[test]
    fn test_stmts() {
        run_str("var pi = 3.14;");
        run_str(
            "var lam = lambda(x) 
                2 + 3; 
            end;",
        );
        run_str(
            "if 2 < 3 then 
                return 1;
            else
                return 0;
            end",
        );
        run_str(
            "function helloWorld(a, b, c)
                    return \"Hello, World!\";
                end",
        );
        run_str(
            "struct point {
                x,
                y
            }",
        );
        run_str(
            "enum Answer {
                Ok,
                Everything = 42
            }",
        );
        run_str(
            "while true do 
                print(true);
                me.x = 42;
            end",
        );
        run_str(
            "for i in range(1, 10) do 
                print(i);
            end",
        );
    }

    #[test]
    fn test_factorial() {
        let prog = "
        function factorial(n)
            if n == 1 or n == 0 then 
                return 1;
            else
                return n * factorial(n - 1);
            end
        end

            factorial(5);
            print(\"Oh, what an happy world we live in!\");
        ";

        run_str(prog);
    }

    #[test]
    fn complex_program() {
        run_str(
            "struct Point {x, y}
        enum Colors {Red, Blue = 3}
        function dostuff(s, e)
            if e == Colors.Red then
                s.x = 10;
                s.y = 20;
            else 
                s.x = 30;
                s.y = 40;
            end
        end

        var p = Point();
        dostuff(p, Colors.Blue);
        ",
        )
    }
}

pub struct Parser {
    lexer: Lexer,
    had_error: bool,
}

impl Parser {
    pub fn new(lex: Lexer) -> Parser {
        Parser {
            lexer: lex,
            had_error: false,
        }
    }

    pub fn error(&self) -> bool {
        self.had_error
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        self.lexer.do_lex();

        if self.lexer.had_error() {
            self.had_error = true;
            return vec![];
        }

        let mut vec = vec![];
        while !self.lexer.is_at_end() {
            vec.push(self.statement());
            if self.had_error {
                break;
            }
        }
        vec
    }

    fn match_next(&mut self, toks: Vec<TokenType>) -> bool {
        let peek_result = self.lexer.peek();
        if let Some(peek_tok) = peek_result {
            let peek = peek_tok.get_type();
            if toks.contains(&peek) {
                self.lexer.next();
                return true;
            }
        }
        false
    }

    fn syntax_error(&self, t: &Token, msg: String) {
        println!("Syntax error at line {}: {}", t.get_line(), msg);
    }

    fn expect(&mut self, t: TokenType) -> Option<&Token> {
        if !self.match_next(vec![t]) {
            if let Some(token_type) = self.lexer.peek() {
                if self.lexer.is_at_end() {
                    self.syntax_error(token_type, format!("Expected {:?}, got EOF", t));
                } else {
                    self.syntax_error(
                        token_type,
                        format!("Expected {:?}, got {:?}", t, token_type.get_type()),
                    );
                }
            } else {
                println!("Expected {:?}, got EOF", t);
            }
            self.had_error = true;
            return None;
        }
        self.lexer.prev()
    }

    fn statement(&mut self) -> Stmt {
        if self.match_next(vec![Var]) {
            return self.parse_var();
        }
        if self.match_next(vec![Function]) {
            return self.parse_function();
        }
        if self.match_next(vec![Struct]) {
            return self.parse_struct();
        }
        if self.match_next(vec![Enum]) {
            return self.parse_enum();
        }
        if self.match_next(vec![Import]) {
            return self.parse_import();
        }
        if self.match_next(vec![If]) {
            return self.parse_if();
        }
        if self.match_next(vec![While]) {
            return self.parse_while();
        }
        if self.match_next(vec![For]) {
            return self.parse_for();
        }
        if self.match_next(vec![Return]) {
            let expr = self.expr();
            self.expect(Semicolon);
            return Stmt::Return(expr);
        }
        if self.match_next(vec![Break]) {
            self.expect(Semicolon);
            return Stmt::Break;
        }
        if self.match_next(vec![Continue]) {
            self.expect(Semicolon);
            return Stmt::Continue;
        }

        let ex = Stmt::ExprStmt(self.expr());
        self.expect(Semicolon);
        ex
    }

    fn parse_import(&mut self) -> Stmt {
        let module_name = self.expect(Str).unwrap().as_String();
        self.expect(Semicolon);
        Stmt::Import(module_name)
    }

    fn parse_var(&mut self) -> Stmt {
        let id = self.name();
        let def = if self.lexer.peek().unwrap().get_type() == Equals {
            self.lexer.next().unwrap();
            let exp = self.expr();
            Stmt::Var(id, Some(exp))
        } else {
            Stmt::Var(id, None)
        };
        self.expect(Semicolon);
        def
    }

    fn parse_struct(&mut self) -> Stmt {
        let name = self.name();
        self.expect(LeftBrace);
        let mut members: Vec<String> = vec![];
        if self.match_next(vec![RightBrace]) {
            self.syntax_error(
                self.lexer.peek().unwrap(),
                String::from("Structs can't be empty!"),
            );
            self.had_error = true;
        }
        while {
            let next = self.name();
            members.push(next);
            self.match_next(vec![Comma])
        } {}
        self.match_next(vec![RightBrace]);
        Stmt::StructDef(name, members)
    }

    fn parse_enum(&mut self) -> Stmt {
        let name = self.name();
        self.expect(LeftBrace);
        let mut members: Vec<(String, Option<Expr>)> = vec![];
        if self.match_next(vec![RightBrace]) {
            self.syntax_error(
                self.lexer.peek().unwrap(),
                String::from("Enums can't be empty!"),
            );
            self.had_error = true;
        }
        while {
            let next = self.name();
            let expr = if self.lexer.peek().unwrap().get_type() == Equals {
                self.lexer.next().unwrap();
                Some(self.expr())
            } else {
                None
            };

            members.push((next, expr));
            self.match_next(vec![Comma])
        } {}
        self.match_next(vec![RightBrace]);
        Stmt::EnumDef(name, members)
    }

    fn parse_if(&mut self) -> Stmt {
        let mut branches: Vec<(Expr, Vec<Stmt>)> = vec![];
        let if_then = self.if_cond_and_exprs();
        branches.push(if_then);
        while self.lexer.prev().unwrap().get_type() == Elif {
            let elif = self.if_cond_and_exprs();
            branches.push(elif);
        }

        let mut else_block: Vec<Stmt> = vec![];

        if self.lexer.prev().unwrap().get_type() == Else {
            while {
                let stmt = self.statement();
                else_block.push(stmt);
                !self.match_next(vec![End])
            } {}
        }

        Stmt::If(branches, else_block)
    }

    fn if_cond_and_exprs(&mut self) -> (Expr, Vec<Stmt>) {
        let cond = self.expr();
        self.expect(Then);

        let mut body: Vec<Stmt> = vec![];
        while {
            let stmt = self.statement();
            body.push(stmt);
            !self.match_next(vec![End, Elif, Else])
        } {}
        (cond, body)
    }

    fn parse_while(&mut self) -> Stmt {
        let cond = self.expr();
        self.expect(Do);
        let body = self.body();
        Stmt::While(cond, body)
    }

    fn parse_for(&mut self) -> Stmt {
        self.expect(Var);
        let initializer = self.parse_var();

        let condition = self.expr();
        self.expect(Semicolon);

        let updater = self.expr();

        self.expect(Do);
        let mut body = self.body();
        body.push(Stmt::ExprStmt(updater));
        let while_body = Stmt::While(condition, body);

        Stmt::Block(vec![initializer, while_body])
    }

    fn parse_function(&mut self) -> Stmt {
        let id = self.name();
        self.expect(LeftParen);
        let params = self.args();
        let body = self.body();
        Stmt::FunDef(id, params, Rc::new(body))
    }

    fn expr(&mut self) -> Expr {
        if self.match_next(vec![Lambda]) {
            return self.lambda();
        }
        let mut eq = self.or();
        while self.match_next(vec![Equals]) {
            let r = self.expr();
            eq = Expr::Assign(Box::new(eq), Box::new(r));
        }
        eq
    }

    fn lambda(&mut self) -> Expr {
        self.expect(LeftParen);
        let args = self.args();
        let body = self.body();
        Expr::LambdaDef(args, Rc::new(body))
    }

    fn equality(&mut self) -> Expr {
        let mut or = self.comparison();
        while self.match_next(vec![EqualsEquals, BangEquals]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.comparison();
            or = Expr::Binary(Box::new(or), op, Box::new(right))
        }
        or
    }

    fn or(&mut self) -> Expr {
        let mut and = self.and();
        while self.match_next(vec![Or]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.and();
            and = Expr::Binary(Box::new(and), op, Box::new(right))
        }
        and
    }

    fn and(&mut self) -> Expr {
        let mut comparison = self.equality();
        while self.match_next(vec![And]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.equality();
            comparison = Expr::Binary(Box::new(comparison), op, Box::new(right))
        }
        comparison
    }
    fn comparison(&mut self) -> Expr {
        let mut shift = self.shift();
        while self.match_next(vec![Less, LessEquals, More, MoreEquals]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.shift();
            shift = Expr::Binary(Box::new(shift), op, Box::new(right))
        }
        shift
    }
    fn shift(&mut self) -> Expr {
        let mut bit_or = self.bit_or();
        while self.match_next(vec![LessLess, MoreMore]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.bit_or();
            bit_or = Expr::Binary(Box::new(bit_or), op, Box::new(right))
        }
        bit_or
    }
    fn bit_or(&mut self) -> Expr {
        let mut bit_and = self.bit_and();
        while self.match_next(vec![Pipe]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.bit_and();
            bit_and = Expr::Binary(Box::new(bit_and), op, Box::new(right))
        }
        bit_and
    }
    fn bit_and(&mut self) -> Expr {
        let mut sum = self.sum();
        while self.match_next(vec![Ampersand]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.sum();
            sum = Expr::Binary(Box::new(sum), op, Box::new(right))
        }
        sum
    }
    fn sum(&mut self) -> Expr {
        let mut product = self.product();
        while self.match_next(vec![Plus, Minus]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.product();
            product = Expr::Binary(Box::new(product), op, Box::new(right))
        }
        product
    }
    fn product(&mut self) -> Expr {
        let mut unary = self.unary();
        while self.match_next(vec![Star, Slash, Mod]) {
            let op = self.lexer.prev().unwrap().get_type();
            let right = self.unary();
            unary = Expr::Binary(Box::new(unary), op, Box::new(right))
        }
        unary
    }
    fn unary(&mut self) -> Expr {
        while self.match_next(vec![Plus, Minus, Not]) {
            let tok = self.lexer.prev().unwrap();
            return Expr::Unary(tok.get_type(), Box::new(self.unary()));
        }

        self.call()
    }

    fn call(&mut self) -> Expr {
        let mut l = self.index();
        while self.match_next(vec![Dot]) {
            let r = self.name();
            l = Expr::Get(Box::new(l), Box::new(Expr::Id(r)));
        }
        if self.match_next(vec![LeftParen]) {
            let params = self.params();
            l = Expr::Call(Box::new(l), params);
        }
        l
    }

    fn index(&mut self) -> Expr {
        let mut e = self.literal();
        while self.match_next(vec![LeftSquare]) {
            let ind = self.lexer.next().unwrap();
            match ind.get_type() {
                Identifier | Num => e = Expr::Get(Box::new(e), Box::new(Expr::new_from_tok(ind))),
                _ => panic!("Indexes can only be identifiers or numbers!"),
            }
            self.expect(RightSquare);
        }
        e
    }

    fn literal(&mut self) -> Expr {
        if self.match_next(vec![Num, Str, Identifier, True, False]) {
            return Expr::new_from_tok(self.lexer.prev().unwrap());
        }
        if self.match_next(vec![LeftParen]) {
            let e = self.expr();
            self.expect(RightParen);
            return Expr::Grouping(Box::new(e));
        }
        if self.match_next(vec![Nil]) {
            return Expr::Nil;
        }
        if self.match_next(vec![LeftBrace]) {
            return self.dictionary();
        }
        if self.match_next(vec![LeftSquare]) {
            return self.list();
        }

        if let Some(token) = self.lexer.peek() {
            let tok_type = token.get_type();
            self.syntax_error(
                self.lexer.peek().unwrap(),
                format!("{:?} can't be parsed as an expression!", tok_type),
            );
            self.had_error = true;
        }

        let previous_type = self.lexer.prev().unwrap().get_type();
        self.syntax_error(
            self.lexer.prev().unwrap(),
            format!("Premature EOF after {:?}!", previous_type),
        );
        self.had_error = true;

        Expr::Nil
    }

    fn dictionary(&mut self) -> Expr {
        let mut v = vec![];
        if !self.match_next(vec![RightBrace]) {
            while {
                let id = self.lexer.next().unwrap();
                if id.get_type() != Identifier {
                    panic!("Dictionary keys can only be identifiers!");
                }
                let id = id.as_Id();
                self.expect(Colon);

                let expr = self.expr();

                v.push((id, expr));
                self.match_next(vec![Comma])
            } {}
            self.expect(RightBrace);
        }
        Expr::DictDef(v)
    }

    fn list(&mut self) -> Expr {
        let mut v = vec![];
        if !self.match_next(vec![RightSquare]) {
            while {
                let expr = self.expr();

                v.push(expr);
                self.match_next(vec![Comma])
            } {}
            self.expect(RightSquare);
        }
        Expr::ListDef(v)
    }

    fn name(&mut self) -> String {
        let tok = self.lexer.next().unwrap().get_type();
        match tok {
            Identifier => self.lexer.prev().unwrap().as_Id(), // OK
            _ => {
                self.syntax_error(
                    self.lexer.prev().unwrap(),
                    String::from("Expected identifier!"),
                );
                self.had_error = true;
                String::from("none duh")
            }
        }
    }

    fn params(&mut self) -> Vec<Expr> {
        let mut v = vec![];
        if !self.match_next(vec![RightParen]) {
            while {
                let e = self.expr();
                v.push(e);
                self.match_next(vec![Comma])
            } {}
            self.expect(RightParen);
        }
        v
    }
    fn args(&mut self) -> Vec<String> {
        let mut v = vec![];
        if !self.match_next(vec![RightParen]) {
            while {
                let e = self.lexer.next().unwrap();
                if e.get_type() != Identifier {
                    panic!("Params can only be identifiers!");
                }
                v.push(e.as_Id());
                self.match_next(vec![Comma])
            } {}
            self.expect(RightParen);
        }
        v
    }
    fn body(&mut self) -> Vec<Stmt> {
        let mut v = vec![];
        while {
            let s = self.statement();
            v.push(s);
            !self.match_next(vec![End])
        } {}
        v
    }
}
