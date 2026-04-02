use serde_json::Value;

/// Expression evaluator for calculated_text elements.
/// This is a safe recursive descent parser — NOT an arbitrary code executor.
/// It only supports arithmetic, string operations, comparisons, and data path lookups.
///
/// Supported syntax:
/// - Path lookup: `firma.unvan`, `toplamlar.kdv`
/// - Arithmetic: `+`, `-`, `*`, `/`
/// - String concatenation: `+` when operand is string
/// - String literals: `"..."` or `'...'`
/// - Number literals: `42`, `3.14`
/// - Comparison: `>`, `<`, `>=`, `<=`, `==`, `!=`
/// - Ternary: `expr ? "a" : "b"`
/// - Parentheses: `(a + b) * c`

pub fn evaluate_expression(expr: &str, data: &Value) -> String {
    let tokens = tokenize(expr);
    if tokens.is_empty() {
        return String::new();
    }
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        data,
    };
    match parser.parse_ternary() {
        ExprValue::Num(n) => format_number(n),
        ExprValue::Str(s) => s,
        ExprValue::Bool(b) => b.to_string(),
        ExprValue::Null => String::new(),
    }
}

fn format_number(n: f64) -> String {
    if n == n.floor() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Format result with given format type
pub fn apply_format(value: &str, format: Option<&str>) -> String {
    match format {
        Some("currency") => format_currency(value),
        Some("percentage") => format_percentage(value),
        Some("number") => format_number_str(value),
        _ => value.to_string(),
    }
}

fn format_currency(value: &str) -> String {
    if let Ok(n) = value.parse::<f64>() {
        let abs = n.abs();
        let integer = abs.floor() as i64;
        let frac = ((abs - abs.floor()) * 100.0).round() as i64;

        let int_str = format_with_thousands(integer);
        let sign = if n < 0.0 { "-" } else { "" };
        format!("{}{},{:02} ₺", sign, int_str, frac)
    } else {
        value.to_string()
    }
}

fn format_percentage(value: &str) -> String {
    if let Ok(n) = value.parse::<f64>() {
        format!("%{:.2}", n)
    } else {
        value.to_string()
    }
}

fn format_number_str(value: &str) -> String {
    if let Ok(n) = value.parse::<f64>() {
        if n == n.floor() && n.abs() < 1e15 {
            format_with_thousands(n.abs() as i64)
        } else {
            format!("{:.2}", n)
        }
    } else {
        value.to_string()
    }
}

fn format_with_thousands(n: i64) -> String {
    let s = n.to_string();
    let len = s.len();
    if len <= 3 {
        return s;
    }
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('.');
        }
        result.push(ch);
    }
    result
}

// --- Tokenizer ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Str(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    Question,
    Colon,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '+' => { tokens.push(Token::Plus); i += 1; }
            '-' => {
                // Negative number: after operator or at start
                let is_unary = tokens.is_empty()
                    || matches!(tokens.last(), Some(
                        Token::Plus | Token::Minus | Token::Star | Token::Slash
                        | Token::LParen | Token::Question | Token::Colon
                        | Token::Gt | Token::Lt | Token::Gte | Token::Lte
                        | Token::Eq | Token::Neq
                    ));
                if is_unary && i + 1 < len && (chars[i + 1].is_ascii_digit() || chars[i + 1] == '.') {
                    let start = i;
                    i += 1;
                    while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    if let Ok(n) = num_str.parse::<f64>() {
                        tokens.push(Token::Num(n));
                    }
                } else {
                    tokens.push(Token::Minus);
                    i += 1;
                }
            }
            '*' => { tokens.push(Token::Star); i += 1; }
            '/' => { tokens.push(Token::Slash); i += 1; }
            '(' => { tokens.push(Token::LParen); i += 1; }
            ')' => { tokens.push(Token::RParen); i += 1; }
            '?' => { tokens.push(Token::Question); i += 1; }
            ':' => { tokens.push(Token::Colon); i += 1; }
            '>' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::Gte); i += 2;
                } else {
                    tokens.push(Token::Gt); i += 1;
                }
            }
            '<' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::Lte); i += 2;
                } else {
                    tokens.push(Token::Lt); i += 1;
                }
            }
            '=' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::Eq); i += 2;
                } else {
                    i += 1;
                }
            }
            '!' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::Neq); i += 2;
                } else {
                    i += 1;
                }
            }
            '"' | '\'' => {
                let quote = chars[i];
                i += 1;
                let start = i;
                while i < len && chars[i] != quote {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Str(s));
                if i < len { i += 1; }
            }
            c if c.is_ascii_digit() || (c == '.' && i + 1 < len && chars[i + 1].is_ascii_digit()) => {
                let start = i;
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                if let Ok(n) = num_str.parse::<f64>() {
                    tokens.push(Token::Num(n));
                }
            }
            c if c.is_alphanumeric() || c == '_' => {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                    i += 1;
                }
                // Trim trailing dots
                while i > start && chars[i - 1] == '.' {
                    i -= 1;
                }
                let ident: String = chars[start..i].iter().collect();
                match ident.as_str() {
                    "true" => tokens.push(Token::Num(1.0)),
                    "false" => tokens.push(Token::Num(0.0)),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            _ => i += 1,
        }
    }

    tokens
}

// --- Parser (recursive descent) ---

#[derive(Debug, Clone)]
enum ExprValue {
    Num(f64),
    Str(String),
    Bool(bool),
    Null,
}

impl ExprValue {
    fn to_num(&self) -> f64 {
        match self {
            ExprValue::Num(n) => *n,
            ExprValue::Str(s) => s.parse().unwrap_or(0.0),
            ExprValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            ExprValue::Null => 0.0,
        }
    }

    fn to_str(&self) -> String {
        match self {
            ExprValue::Num(n) => format_number(*n),
            ExprValue::Str(s) => s.clone(),
            ExprValue::Bool(b) => b.to_string(),
            ExprValue::Null => String::new(),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            ExprValue::Num(n) => *n != 0.0,
            ExprValue::Str(s) => !s.is_empty(),
            ExprValue::Bool(b) => *b,
            ExprValue::Null => false,
        }
    }

    fn is_string(&self) -> bool {
        matches!(self, ExprValue::Str(_))
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    data: &'a Value,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn parse_ternary(&mut self) -> ExprValue {
        let cond = self.parse_comparison();
        if self.peek() == Some(&Token::Question) {
            self.advance();
            let then_val = self.parse_ternary();
            if self.peek() == Some(&Token::Colon) {
                self.advance();
            }
            let else_val = self.parse_ternary();
            if cond.is_truthy() { then_val } else { else_val }
        } else {
            cond
        }
    }

    fn parse_comparison(&mut self) -> ExprValue {
        let left = self.parse_additive();
        match self.peek() {
            Some(Token::Gt) => { self.advance(); let r = self.parse_additive(); ExprValue::Bool(left.to_num() > r.to_num()) }
            Some(Token::Lt) => { self.advance(); let r = self.parse_additive(); ExprValue::Bool(left.to_num() < r.to_num()) }
            Some(Token::Gte) => { self.advance(); let r = self.parse_additive(); ExprValue::Bool(left.to_num() >= r.to_num()) }
            Some(Token::Lte) => { self.advance(); let r = self.parse_additive(); ExprValue::Bool(left.to_num() <= r.to_num()) }
            Some(Token::Eq) => { self.advance(); let r = self.parse_additive(); ExprValue::Bool(left.to_str() == r.to_str()) }
            Some(Token::Neq) => { self.advance(); let r = self.parse_additive(); ExprValue::Bool(left.to_str() != r.to_str()) }
            _ => left,
        }
    }

    fn parse_additive(&mut self) -> ExprValue {
        let mut left = self.parse_multiplicative();
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    if left.is_string() || right.is_string() {
                        left = ExprValue::Str(format!("{}{}", left.to_str(), right.to_str()));
                    } else {
                        left = ExprValue::Num(left.to_num() + right.to_num());
                    }
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_multiplicative();
                    left = ExprValue::Num(left.to_num() - right.to_num());
                }
                _ => break,
            }
        }
        left
    }

    fn parse_multiplicative(&mut self) -> ExprValue {
        let mut left = self.parse_primary();
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.advance();
                    let right = self.parse_primary();
                    left = ExprValue::Num(left.to_num() * right.to_num());
                }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_primary();
                    let r = right.to_num();
                    left = ExprValue::Num(if r != 0.0 { left.to_num() / r } else { 0.0 });
                }
                _ => break,
            }
        }
        left
    }

    fn parse_primary(&mut self) -> ExprValue {
        match self.advance().cloned() {
            Some(Token::Num(n)) => ExprValue::Num(n),
            Some(Token::Str(s)) => ExprValue::Str(s),
            Some(Token::Ident(path)) => {
                let val = resolve_path(self.data, &path);
                json_to_expr(val)
            }
            Some(Token::LParen) => {
                let val = self.parse_ternary();
                if self.peek() == Some(&Token::RParen) {
                    self.advance();
                }
                val
            }
            Some(Token::Minus) => {
                let val = self.parse_primary();
                ExprValue::Num(-val.to_num())
            }
            _ => ExprValue::Null,
        }
    }
}

fn resolve_path<'a>(data: &'a Value, path: &str) -> &'a Value {
    let mut current = data;
    for key in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(key).unwrap_or(&Value::Null),
            _ => &Value::Null,
        };
    }
    current
}

fn json_to_expr(v: &Value) -> ExprValue {
    match v {
        Value::Number(n) => ExprValue::Num(n.as_f64().unwrap_or(0.0)),
        Value::String(s) => ExprValue::Str(s.clone()),
        Value::Bool(b) => ExprValue::Bool(*b),
        Value::Null => ExprValue::Null,
        _ => ExprValue::Str(v.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_path() {
        let data = json!({"firma": {"unvan": "Acme A.Ş."}});
        assert_eq!(evaluate_expression("firma.unvan", &data), "Acme A.Ş.");
    }

    #[test]
    fn test_arithmetic() {
        let data = json!({"toplamlar": {"araToplam": 16000, "kdv": 2880}});
        assert_eq!(evaluate_expression("toplamlar.araToplam + toplamlar.kdv", &data), "18880");
    }

    #[test]
    fn test_multiplication() {
        let data = json!({"toplamlar": {"araToplam": 16000}});
        assert_eq!(evaluate_expression("toplamlar.araToplam * 0.20", &data), "3200");
    }

    #[test]
    fn test_string_concat() {
        let data = json!({"fatura": {"no": "FTR-001"}});
        assert_eq!(evaluate_expression("\"Fatura No: \" + fatura.no", &data), "Fatura No: FTR-001");
    }

    #[test]
    fn test_ternary() {
        let data = json!({"fatura": {"tutar": 5000}});
        assert_eq!(evaluate_expression("fatura.tutar > 0 ? \"Borclu\" : \"Alacakli\"", &data), "Borclu");
    }

    #[test]
    fn test_ternary_false() {
        let data = json!({"fatura": {"tutar": 0}});
        assert_eq!(evaluate_expression("fatura.tutar > 0 ? \"Borclu\" : \"Alacakli\"", &data), "Alacakli");
    }

    #[test]
    fn test_parentheses() {
        let data = json!({"a": 2, "b": 3, "c": 4});
        assert_eq!(evaluate_expression("(a + b) * c", &data), "20");
    }

    #[test]
    fn test_number_literal() {
        let data = json!({});
        assert_eq!(evaluate_expression("42", &data), "42");
        assert_eq!(evaluate_expression("3.14", &data), "3.14");
    }

    #[test]
    fn test_division_by_zero() {
        let data = json!({});
        assert_eq!(evaluate_expression("10 / 0", &data), "0");
    }

    #[test]
    fn test_missing_path() {
        let data = json!({});
        assert_eq!(evaluate_expression("missing.path", &data), "");
    }

    #[test]
    fn test_comparison_eq() {
        let data = json!({"status": "paid"});
        assert_eq!(evaluate_expression("status == \"paid\" ? \"Odendi\" : \"Odenmedi\"", &data), "Odendi");
    }

    #[test]
    fn test_format_currency() {
        assert_eq!(apply_format("18880", Some("currency")), "18.880,00 ₺");
        assert_eq!(apply_format("1000.5", Some("currency")), "1.000,50 ₺");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(apply_format("20", Some("percentage")), "%20.00");
    }

    #[test]
    fn test_negative_result() {
        let data = json!({"a": 10, "b": 20});
        assert_eq!(evaluate_expression("a - b", &data), "-10");
    }

    #[test]
    fn test_empty_expression() {
        let data = json!({});
        assert_eq!(evaluate_expression("", &data), "");
    }
}
