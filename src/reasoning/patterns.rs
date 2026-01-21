//! Pattern matching e análise de código.
//!
//! Este módulo fornece utilitários para:
//! - Normalizar código (remover whitespace, comentários)
//! - Computar assinaturas SHA256
//! - Extrair keywords indicativas de patterns

use sha2::{Digest, Sha256};

/// Utilitários para pattern matching.
pub struct PatternMatcher;

impl PatternMatcher {
    /// Computa a assinatura SHA256 de um código normalizado.
    pub fn compute_signature(code: &str) -> String {
        let normalized = Self::normalize_code(code);
        let mut hasher = Sha256::new();
        hasher.update(normalized.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Normaliza código removendo whitespace extra e comentários.
    pub fn normalize_code(code: &str) -> String {
        code.lines()
            .map(|line| line.trim())
            .filter(|line| {
                !line.is_empty()
                    && !line.starts_with("//")
                    && !line.starts_with('#')
                    && !line.starts_with("/*")
                    && !line.starts_with('*')
                    && !line.starts_with("*/")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extrai keywords que indicam patterns conhecidos.
    pub fn extract_keywords(code: &str) -> Vec<String> {
        let mut keywords = Vec::new();
        let code_lower = code.to_lowercase();

        // Keywords de segurança
        if code_lower.contains("sql") || code_lower.contains("query") {
            keywords.push("sql".to_string());
        }
        if code_lower.contains("password") || code_lower.contains("secret") || code_lower.contains("credential") {
            keywords.push("credentials".to_string());
        }
        if code_lower.contains("eval") || code_lower.contains("exec") {
            keywords.push("code_execution".to_string());
        }
        if code_lower.contains("http") || code_lower.contains("request") || code_lower.contains("fetch") {
            keywords.push("network".to_string());
        }
        if code_lower.contains("file") || code_lower.contains("read") || code_lower.contains("write") {
            keywords.push("file_io".to_string());
        }

        // Keywords de lógica
        if code_lower.contains("for ") || code_lower.contains("while ") || code_lower.contains("loop") {
            keywords.push("loop".to_string());
        }
        if code_lower.contains("unwrap") || code_lower.contains(".get(") || code_lower.contains("expect(") {
            keywords.push("null_access".to_string());
        }
        if code_lower.contains("panic") || code_lower.contains("crash") {
            keywords.push("panic".to_string());
        }
        if code_lower.contains("unsafe") {
            keywords.push("unsafe".to_string());
        }
        if code_lower.contains("async") || code_lower.contains("await") {
            keywords.push("async".to_string());
        }
        if code_lower.contains("mutex") || code_lower.contains("lock") || code_lower.contains("atomic") {
            keywords.push("concurrency".to_string());
        }

        // Keywords de performance
        if code_lower.contains("clone()") || code_lower.contains(".clone()") {
            keywords.push("clone".to_string());
        }
        if code_lower.contains("vec!") || code_lower.contains("push(") {
            keywords.push("allocation".to_string());
        }
        if code_lower.contains("collect()") || code_lower.contains(".collect()") {
            keywords.push("collect".to_string());
        }

        // Keywords de estilo
        if code_lower.contains("todo") || code_lower.contains("fixme") {
            keywords.push("todo".to_string());
        }

        keywords
    }

    /// Calcula a similaridade entre dois códigos (0.0 - 1.0).
    pub fn similarity(code1: &str, code2: &str) -> f64 {
        let sig1 = Self::compute_signature(code1);
        let sig2 = Self::compute_signature(code2);

        // Se assinaturas são iguais, similaridade é 1.0
        if sig1 == sig2 {
            return 1.0;
        }

        // Calcula similaridade baseada em keywords comuns
        let kw1: std::collections::HashSet<_> = Self::extract_keywords(code1).into_iter().collect();
        let kw2: std::collections::HashSet<_> = Self::extract_keywords(code2).into_iter().collect();

        if kw1.is_empty() && kw2.is_empty() {
            return 0.0;
        }

        let intersection = kw1.intersection(&kw2).count();
        let union = kw1.union(&kw2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Detecta a linguagem de programação do código.
    pub fn detect_language(code: &str) -> String {
        let code_lower = code.to_lowercase();

        // Rust
        if code_lower.contains("fn ")
            || code_lower.contains("let ")
            || code_lower.contains("impl ")
            || code_lower.contains("struct ")
            || code_lower.contains("enum ")
        {
            return "rust".to_string();
        }

        // Python
        if code_lower.contains("def ")
            || code_lower.contains("import ")
            || code_lower.contains("class ")
            || code_lower.contains("elif ")
        {
            return "python".to_string();
        }

        // JavaScript/TypeScript
        if code_lower.contains("const ")
            || code_lower.contains("function ")
            || code_lower.contains("=>")
            || code_lower.contains("export ")
        {
            return "javascript".to_string();
        }

        // Go
        if code_lower.contains("func ")
            || code_lower.contains("package ")
            || code_lower.contains("go ")
        {
            return "go".to_string();
        }

        // Java
        if code_lower.contains("public class")
            || code_lower.contains("private ")
            || code_lower.contains("static void main")
        {
            return "java".to_string();
        }

        "unknown".to_string()
    }

    /// Categoriza o tipo de código.
    pub fn categorize_code(code: &str) -> Vec<String> {
        let mut categories = Vec::new();
        let keywords = Self::extract_keywords(code);

        if keywords.iter().any(|k| k == "sql" || k == "credentials" || k == "code_execution") {
            categories.push("security".to_string());
        }

        if keywords.iter().any(|k| k == "network" || k == "file_io") {
            categories.push("io".to_string());
        }

        if keywords.iter().any(|k| k == "loop" || k == "null_access" || k == "panic") {
            categories.push("logic".to_string());
        }

        if keywords.iter().any(|k| k == "async" || k == "concurrency") {
            categories.push("concurrency".to_string());
        }

        if keywords.iter().any(|k| k == "clone" || k == "allocation" || k == "collect") {
            categories.push("performance".to_string());
        }

        if categories.is_empty() {
            categories.push("general".to_string());
        }

        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_signature_same_code() {
        let code1 = "fn main() { println!(\"Hello\"); }";
        let code2 = "fn main() { println!(\"Hello\"); }";

        assert_eq!(
            PatternMatcher::compute_signature(code1),
            PatternMatcher::compute_signature(code2)
        );
    }

    #[test]
    fn test_compute_signature_different_code() {
        let code1 = "fn main() { println!(\"Hello\"); }";
        let code2 = "fn main() { println!(\"World\"); }";

        assert_ne!(
            PatternMatcher::compute_signature(code1),
            PatternMatcher::compute_signature(code2)
        );
    }

    #[test]
    fn test_normalize_code() {
        let code = r#"
            // This is a comment
            fn main() {
                // Another comment
                println!("Hello");
            }
        "#;

        let normalized = PatternMatcher::normalize_code(code);

        assert!(!normalized.contains("comment"));
        assert!(normalized.contains("fn main()"));
        assert!(normalized.contains("println!"));
    }

    #[test]
    fn test_extract_keywords_security() {
        let code = "let query = format!(\"SELECT * FROM users WHERE password = {}\", input);";
        let keywords = PatternMatcher::extract_keywords(code);

        assert!(keywords.contains(&"sql".to_string()));
        assert!(keywords.contains(&"credentials".to_string()));
    }

    #[test]
    fn test_extract_keywords_logic() {
        let code = "for i in 0..10 { data.get(i).unwrap(); }";
        let keywords = PatternMatcher::extract_keywords(code);

        assert!(keywords.contains(&"loop".to_string()));
        assert!(keywords.contains(&"null_access".to_string()));
    }

    #[test]
    fn test_similarity_same_code() {
        let code = "fn main() { println!(\"Hello\"); }";
        assert_eq!(PatternMatcher::similarity(code, code), 1.0);
    }

    #[test]
    fn test_similarity_similar_keywords() {
        let code1 = "for i in 0..10 { vec.push(i); }";
        let code2 = "for x in 0..5 { data.push(x); }";

        let similarity = PatternMatcher::similarity(code1, code2);
        assert!(similarity > 0.5); // Ambos têm loop e allocation
    }

    #[test]
    fn test_detect_language_rust() {
        let code = "fn main() { let x = 5; }";
        assert_eq!(PatternMatcher::detect_language(code), "rust");
    }

    #[test]
    fn test_detect_language_python() {
        let code = "def main():\n    import os\n    print('hello')";
        assert_eq!(PatternMatcher::detect_language(code), "python");
    }

    #[test]
    fn test_detect_language_javascript() {
        let code = "const x = () => { console.log('hello'); }";
        assert_eq!(PatternMatcher::detect_language(code), "javascript");
    }

    #[test]
    fn test_categorize_code_security() {
        let code = "execute_query(format!(\"SELECT * WHERE password = {}\", input));";
        let categories = PatternMatcher::categorize_code(code);

        assert!(categories.contains(&"security".to_string()));
    }

    #[test]
    fn test_categorize_code_concurrency() {
        let code = "async fn fetch() { let lock = mutex.lock().await; }";
        let categories = PatternMatcher::categorize_code(code);

        assert!(categories.contains(&"concurrency".to_string()));
    }
}
