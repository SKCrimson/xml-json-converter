#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    StringVal(&'a str), // Храним ссылку, а не копию
    Number(f64),
    BoolVal(bool),
    Null,
}

#[derive(Debug)]
pub enum JsonNode<'a> {
    Object(Vec<(&'a str, JsonNode<'a>)>),
    Array(Vec<JsonNode<'a>>),
    StringVal(&'a str),
    Number(f64),
    BoolVal(bool),
    Null,
}

impl<'a> JsonNode<'a> {
    /// Публичный метод для преобразования в XML строку.
    /// Использует "root" как имя корневого элемента.
    #[allow(dead_code)]
    pub fn to_xml(&self) -> String {
        self.to_xml_internal("root")
    }

    /// Публичный метод для красивого вывода
    pub fn to_pretty_xml(&self, indent_size: usize) -> String {
        self.to_xml_recursive("root", 0, indent_size)
    }

    /// Внутренний рекурсивный метод
    #[allow(dead_code)]
    fn to_xml_internal(&self, label: &str) -> String {
        // Очищаем имя тега (XML не любит пробелы и спецсимволы в именах)
        let safe_label = self.sanitize_tag_name(label);

        match self {
            JsonNode::Object(pairs) => {
                let mut inner = String::new();
                for (key, val) in pairs {
                    // Ключи объекта становятся именами вложенных тегов
                    inner.push_str(&val.to_xml_internal(key));
                }
                format!("<{}>{}</{}>", safe_label, inner, safe_label)
            }
            JsonNode::Array(elements) => {
                let mut inner = String::new();
                for el in elements {
                    // Элементы массива получают имя родителя или "item"
                    // В XML принято использовать либо имя в единственном числе, либо <item>
                    inner.push_str(&el.to_xml_internal("item"));
                }
                format!("<{}>{}</{}>", safe_label, inner, safe_label)
            }
            JsonNode::StringVal(s) => {
                format!(
                    "<{}>{}</{}>",
                    safe_label,
                    self.escape_xml_text(s),
                    safe_label
                )
            }
            JsonNode::Number(n) => {
                format!("<{}>{}</{}>", safe_label, n, safe_label)
            }
            JsonNode::BoolVal(b) => {
                format!("<{}>{}</{}>", safe_label, b, safe_label)
            }
            JsonNode::Null => {
                format!("<{} />", safe_label) // Самозакрывающийся тег для null
            }
        }
    }

    fn to_xml_recursive(&self, label: &str, depth: usize, indent_size: usize) -> String {
        let safe_label = self.sanitize_tag_name(label);
        let indent = " ".repeat(depth * indent_size);

        match self {
            JsonNode::Object(pairs) => {
                let mut inner = String::new();
                for (key, val) in pairs {
                    inner.push_str("\n");
                    inner.push_str(&val.to_xml_recursive(key, depth + 1, indent_size));
                }
                format!(
                    "{}<{}>{} \n{}</{}>",
                    indent, safe_label, inner, indent, safe_label
                )
            }
            JsonNode::Array(elements) => {
                let mut inner = String::new();
                for el in elements {
                    inner.push_str("\n");
                    inner.push_str(&el.to_xml_recursive("item", depth + 1, indent_size));
                }
                format!(
                    "{}<{}>{} \n{}</{}>",
                    indent, safe_label, inner, indent, safe_label
                )
            }
            JsonNode::StringVal(s) => {
                format!(
                    "{}<{}>{}</{}>",
                    indent,
                    safe_label,
                    self.escape_xml_text(s),
                    safe_label
                )
            }
            JsonNode::Number(n) => {
                format!("{}<{}>{}</{}>", indent, safe_label, n, safe_label)
            }
            JsonNode::BoolVal(b) => {
                format!("{}<{}>{}</{}>", indent, safe_label, b, safe_label)
            }
            JsonNode::Null => {
                format!("{}<{} />", indent, safe_label)
            }
        }
    }

    /// Вспомогательная функция для экранирования спецсимволов в тексте XML
    fn escape_xml_text(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Вспомогательная функция для валидации имен тегов
    fn sanitize_tag_name(&self, name: &str) -> String {
        let mut sanitized =
            name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");

        // XML тег не может начинаться с цифры
        if let Some(first) = sanitized.chars().next() {
            if first.is_numeric() {
                sanitized.insert(0, '_');
            }
        }
        sanitized
    }
}
