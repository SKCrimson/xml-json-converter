use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    TagOpen(&'a str),                        // <name
    TagClose(&'a str),                       // </name>
    Attr(Option<&'a str>, &'a str, &'a str), // (namespace, key, value)
    TagEnd,                                  // >
    TagSelfClose,                            // />
    Text(&'a str),                           // content
}

#[derive(Debug)]
pub enum XmlNode {
    Element {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<XmlNode>,
    },
    Text(String),
}

impl XmlNode {
    pub fn to_json(&self) -> String {
        match self {
            XmlNode::Text(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            XmlNode::Element {
                name: _,
                attributes,
                children,
            } => {
                let mut parts = Vec::new();

                // 1. Обрабатываем атрибуты
                for (key, value) in attributes {
                    parts.push(format!("\"@{}\": \"{}\"", key, value.replace('"', "\\\"")));
                }

                // 2. Группируем детей по именам, чтобы найти дубликаты (массивы)
                let mut grouped_children: HashMap<String, Vec<&XmlNode>> = HashMap::new();
                let mut text_content = Vec::new();

                for child in children {
                    match child {
                        XmlNode::Element { name, .. } => {
                            grouped_children
                                .entry(name.clone())
                                .or_default()
                                .push(child);
                        }
                        XmlNode::Text(t) => text_content.push(t),
                    }
                }

                // 3. Обрабатываем сгруппированных детей
                for (name, nodes) in grouped_children {
                    if nodes.len() > 1 {
                        // Массив элементов
                        let items: Vec<String> = nodes.iter().map(|n| n.to_json()).collect();
                        parts.push(format!("\"{}\": [{}]", name, items.join(", ")));
                    } else {
                        // Одиночный элемент
                        parts.push(format!("\"{}\": {}", name, nodes[0].to_json()));
                    }
                }

                // 4. Если есть только текст и нет атрибутов/детей, возвращаем просто строку
                if parts.is_empty() && !text_content.is_empty() {
                    let joined_text = text_content
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    return format!("\"{}\"", joined_text.replace('"', "\\\""));
                }

                // Если есть текст вместе с другими элементами, добавим его как специальное поле
                if !text_content.is_empty() && !parts.is_empty() {
                    let joined_text = text_content
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    parts.push(format!(
                        "\"#text\": \"{}\"",
                        joined_text.replace('"', "\\\"")
                    ));
                }

                format!("{{ {} }}", parts.join(", "))
            }
        }
    }
}
