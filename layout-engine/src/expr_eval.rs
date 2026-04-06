use dexpr::ast::value::Value as DexprValue;
use dexpr::compiler::Compiler;
use dexpr::vm::VM;
use serde_json::Value;

/// Expression evaluator for calculated_text elements using dexpr engine.
/// Supports arithmetic, string ops, comparisons, conditionals, methods, and more.
///
/// Data JSON's top-level keys are set as global variables in dexpr.
/// Expressions like `firma.unvan` or `toplamlar.kdv + toplamlar.araToplam` work directly.
pub fn evaluate_expression(expr: &str, data: &Value) -> String {
    if expr.is_empty() {
        return String::new();
    }

    let mut compiler = Compiler::new();
    let bytecode = match compiler.compile_from_source(expr) {
        Ok((bc, _)) => bc,
        Err(_) => return String::new(),
    };

    let mut vm = VM::new(&bytecode);

    // Set each top-level key in data as a dexpr global
    if let Value::Object(map) = data {
        for (key, val) in map {
            if let Ok(dval) = DexprValue::from_json_value(val) {
                vm.set_global(key, dval);
            }
        }
    }

    match vm.execute() {
        Ok(result) => dexpr_value_to_string(&result),
        Err(_) => String::new(),
    }
}

/// Convert dexpr Value to display string
fn dexpr_value_to_string(val: &DexprValue) -> String {
    match val {
        DexprValue::Null => String::new(),
        DexprValue::Boolean(b) => b.to_string(),
        DexprValue::Number(n) => {
            // Format: no trailing zeros for integers
            if n.scale() == 0 {
                n.to_string()
            } else {
                n.normalize().to_string()
            }
        }
        DexprValue::String(s) => s.to_string(),
        DexprValue::NumberList(list) => {
            let items: Vec<String> = list.iter().map(|n| n.to_string()).collect();
            format!("[{}]", items.join(", "))
        }
        DexprValue::StringList(list) => {
            let items: Vec<String> = list.iter().map(|s| s.to_string()).collect();
            format!("[{}]", items.join(", "))
        }
        DexprValue::Object(map) => {
            let items: Vec<String> = map.iter().map(|(k, v)| format!("{}: {}", k, dexpr_value_to_string(v))).collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

/// Format result with given format type (varsayılan Türk formatı)
pub fn apply_format(value: &str, format: Option<&str>) -> String {
    apply_format_with_config(value, format, &dreport_core::models::FormatConfig::default())
}

/// Format result with given format type and config
pub fn apply_format_with_config(value: &str, format: Option<&str>, config: &dreport_core::models::FormatConfig) -> String {
    match format {
        Some("currency") => format_currency(value, config),
        Some("percentage") => format_percentage(value),
        Some("number") => format_number_str(value, config),
        _ => value.to_string(),
    }
}

fn format_currency(value: &str, config: &dreport_core::models::FormatConfig) -> String {
    if let Ok(n) = value.parse::<f64>() {
        let abs = n.abs();
        let integer = abs.floor() as i64;
        let frac = ((abs - abs.floor()) * 100.0).round() as i64;

        let int_str = format_with_thousands(integer, &config.thousands_separator);
        let sign = if n < 0.0 { "-" } else { "" };
        if config.currency_position == "prefix" {
            format!("{}{}{}{}{:02}", config.currency_symbol, sign, int_str, config.decimal_separator, frac)
        } else {
            format!("{}{}{}{:02} {}", sign, int_str, config.decimal_separator, frac, config.currency_symbol)
        }
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

fn format_number_str(value: &str, config: &dreport_core::models::FormatConfig) -> String {
    if let Ok(n) = value.parse::<f64>() {
        if n == n.floor() && n.abs() < 1e15 {
            format_with_thousands(n.abs() as i64, &config.thousands_separator)
        } else {
            // Ondalık ayırıcıyı config'den al
            let formatted = format!("{:.2}", n);
            formatted.replace('.', &config.decimal_separator)
        }
    } else {
        value.to_string()
    }
}

fn format_with_thousands(n: i64, separator: &str) -> String {
    let s = n.to_string();
    let len = s.len();
    if len <= 3 {
        return s;
    }
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push_str(separator);
        }
        result.push(ch);
    }
    result
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
        assert_eq!(
            evaluate_expression("if fatura.tutar > 0 then \"Borclu\" else \"Alacakli\" end", &data),
            "Borclu"
        );
    }

    #[test]
    fn test_ternary_false() {
        let data = json!({"fatura": {"tutar": 0}});
        assert_eq!(
            evaluate_expression("if fatura.tutar > 0 then \"Borclu\" else \"Alacakli\" end", &data),
            "Alacakli"
        );
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
    fn test_missing_path() {
        let data = json!({});
        // dexpr returns Null for undefined globals
        assert_eq!(evaluate_expression("missing.path", &data), "");
    }

    #[test]
    fn test_numeric_comparison() {
        let data = json!({"fatura": {"tutar": 5000}});
        assert_eq!(
            evaluate_expression("if fatura.tutar > 1000 then \"Yuksek\" else \"Dusuk\" end", &data),
            "Yuksek"
        );
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

    // dexpr-specific features
    #[test]
    fn test_string_methods() {
        let data = json!({"name": "Acme Teknoloji"});
        assert_eq!(evaluate_expression("name.upper()", &data), "ACME TEKNOLOJI");
        assert_eq!(evaluate_expression("name.length()", &data), "14");
    }

    #[test]
    fn test_modulo_and_power() {
        let data = json!({});
        assert_eq!(evaluate_expression("10 % 3", &data), "1");
        assert_eq!(evaluate_expression("2 ** 10", &data), "1024");
    }

    #[test]
    fn test_logical_operators() {
        let data = json!({"a": true, "b": false});
        assert_eq!(evaluate_expression("a && b", &data), "false");
        assert_eq!(evaluate_expression("a || b", &data), "true");
    }

    #[test]
    fn test_compound_expression() {
        let data = json!({"toplamlar": {"araToplam": 16000, "kdvOran": 18}});
        assert_eq!(
            evaluate_expression("toplamlar.araToplam * toplamlar.kdvOran / 100", &data),
            "2880"
        );
    }
}
