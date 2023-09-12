use std::{collections::HashMap, path::{Path, PathBuf}};
use anyhow::{Result, anyhow, Ok};
use serde_yaml;
use clap::{Parser, Subcommand};

#[derive(Debug)]
enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Dictionary(HashMap<String, Value>),
}

impl Value {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Value::Array(_) => true,
            _ => false,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn is_dictionary(&self) -> bool {
        match self {
            Value::Dictionary(_) => true,
            _ => false,
        }
    }

    pub fn as_dictionary(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_dictionary_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
        match self {
            Value::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
enum Token {
    Equal,                // "="
    OpenBrace,            // "{"
    CloseBrace,           // "}"
    Comma,                // ","
    Identifier(String),   // "astver", "text" 等
    StringLiteral(String),// "2.0", "俺たちの新しい日常" 等
    IntegerLiteral(i64),  // 整数
    FloatLiteral(f64),    // 浮点数
}

fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '=' => tokens.push(Token::Equal),
            '{' => tokens.push(Token::OpenBrace),
            '}' => tokens.push(Token::CloseBrace),
            ',' => tokens.push(Token::Comma),
            '"' => {
                let mut s = String::new();
                while let Some(ch) = chars.peek() {
                    match ch {
                        '\\' => {
                            chars.next(); // Consume the backslash
                            if let Some(escaped) = chars.next() {
                                match escaped {
                                    'n' => s.push('\n'),
                                    't' => s.push('\t'),
                                    '"' => s.push('"'),
                                    '\\' => s.push('\\'),
                                    _ => return Err(anyhow!("Unknown escape sequence")),
                                }
                            } else {
                                return Err(anyhow!("Incomplete escape sequence"));
                            }
                        }
                        '"' => {
                            chars.next(); // skip the closing "
                            break;
                        }
                        _ => s.push(chars.next().unwrap()),
                    }
                }
                tokens.push(Token::StringLiteral(s));
            }
            _ if ch.is_whitespace() || ch == '\n' || ch == '\r' => {}
            _ if ch.is_numeric() || (ch == '-' && chars.peek().map_or(false, |next| next.is_numeric())) => {
                let mut number = ch.to_string();
                let mut is_float = false;
                while let Some(ch) = chars.peek() {
                    if *ch == '.' {
                        is_float = true;
                        number.push(chars.next().unwrap());
                    } else if ch.is_numeric() {
                        number.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if is_float {
                    tokens.push(Token::FloatLiteral(number.parse().unwrap()));
                } else {
                    tokens.push(Token::IntegerLiteral(number.parse().unwrap()));
                }
            }
            _ if ch.is_alphanumeric() || ch == '_' => {
                let mut name = ch.to_string();
                while let Some(ch) = chars.peek() {
                    if ch.is_alphanumeric() || *ch == '_' {
                        name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Identifier(name));
            }
            _ => return Err(anyhow!("Unexpected character")),
        }
    }
    Ok(tokens)
}


fn parse_tokens(tokens: &[Token]) -> Result<HashMap<String, Value>> {
    let mut index = 0;
    let mut result = HashMap::new();
    
    while index < tokens.len() {
        match &tokens[index] {
            Token::Identifier(s) => {
                index += 1;
                if let Token::Equal = tokens[index] {
                    index += 1;  // Skip '='
                    let value = parse_value(tokens, &mut index)?;
                    result.insert(s.clone(), value);
                } else {
                    anyhow::bail!("Expected '=' after Identifier");
                }
            }
            _ => anyhow::bail!("Unexpected token at top level"),
        }
    }
    Ok(result)
}

fn parse_value(tokens: &[Token], index: &mut usize) -> Result<Value> {
    match &tokens[*index] {
        Token::OpenBrace => parse_array(tokens, index),
        Token::StringLiteral(s) => {
            *index += 1;
            Ok(Value::String(s.clone()))
        }
        Token::IntegerLiteral(i) => {
            *index += 1;
            Ok(Value::Integer(*i))
        }
        Token::FloatLiteral(f) => {
            *index += 1;
            Ok(Value::Float(*f))
        }
        Token::Identifier(s) => {
            *index += 1;
            if let Token::Equal = tokens[*index] {
                *index += 1;  // Skip '='
                let value = parse_value(tokens, index)?;
                let mut map = HashMap::new();
                map.insert(s.clone(), value);
                Ok(Value::Dictionary(map))
            } else {
                Ok(Value::String(s.clone()))
            }
        }
        _ => anyhow::bail!(format!("Unexpected token: {:?}", tokens[*index])),
    }
}


fn parse_array(tokens: &[Token], index: &mut usize) -> Result<Value> {
    let mut values = Vec::new();
    *index += 1; // Skip '{'
    
    loop {
        match &tokens[*index] {
            Token::CloseBrace => {
                *index += 1;
                return Ok(Value::Array(values));
            }
            Token::Comma => {
                *index += 1;
                continue;
            }
            _ => {
                let value = parse_value(tokens, index)?;
                values.push(value);
            }
        }
    }
}


fn extract_secnario_toyaml(ast: &HashMap<String, Value>, output: impl AsRef<Path>) -> Result<()> {
    // extract all the text under the key "text"
    let ast_array = ast.get("ast")
        .ok_or(anyhow::anyhow!("ast key not found"))?
        .as_array()
        .ok_or(anyhow::anyhow!("ast is not a dictionary"))?;

    let mut all_texts = Vec::new();
    
    for block_value in ast_array.iter() {
        let blocks = block_value.as_dictionary().ok_or(anyhow::anyhow!("block is not a dict"))?;
        for (block_key, block_dict) in blocks.iter() {
            if !block_key.starts_with("block_") {
                continue;
            }
            if let Some(block_items) = block_dict.as_array() {
                for block_item in block_items {
                    if let Some(block_item) = block_item.as_dictionary() {
                        if let Some(text_value) = block_item.get("text") {
                            if let Some(text_array) = text_value.as_array() {
                                for text_block in text_array.iter() {
                                    let ja_texts = text_block.as_dictionary();
                                    if let Some(ja_texts) = ja_texts {
                                        if let Some(ja_texts) = ja_texts.get("ja") {
                                            if let Some(ja_texts) = ja_texts.as_array() {
                                                for subja in ja_texts {
                                                    if let Some(subja) = subja.as_array() {
                                                        for subj in subja.iter() {
                                                            if let Some(subj) = subj.as_string() {
                                                                all_texts.push(subj.to_string());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let s = serde_yaml::to_string(&all_texts)?;
    // write to file
    std::fs::write(output, s)?;
    Ok(())
}



fn replace_secnario(ast: &mut HashMap<String, Value>, secnario: Vec<String>) -> Result<()> {
    let mut scenario_iter = secnario.into_iter();

    fn replace_text_in_ja(subja: &mut Value, scenario_iter: &mut impl Iterator<Item=String>) -> Result<()> {
        if let Some(subj) = subja.as_string_mut() {
            if let Some(new_str) = scenario_iter.next() {
                *subj = new_str;
            } else {
                return Err(anyhow::anyhow!("Ran out of strings in secnario."));
            }
        }
        Ok(())
    }

    fn replace_texts_in_block(block: &mut Value, scenario_iter: &mut impl Iterator<Item=String>) -> Result<()> {
        if let Some(block_dict) = block.as_dictionary_mut() {
            if let Some(text_array) = block_dict.get_mut("text").and_then(Value::as_array_mut) {
                for text_block in text_array {
                    if let Some(ja_texts) = text_block.as_dictionary_mut().and_then(|dict| dict.get_mut("ja")).and_then(Value::as_array_mut) {
                        for subja in ja_texts {
                            replace_text_in_ja(subja, scenario_iter)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    if let Some(ast_array) = ast.get_mut("ast").and_then(Value::as_array_mut) {
        for block_value in ast_array {
            if let Some(blocks) = block_value.as_dictionary_mut() {
                for (_, block_dict) in blocks {
                    if block_dict.is_dictionary() {
                        replace_texts_in_block(block_dict, &mut scenario_iter)?;
                    }
                }
            }
        }
    }

    if scenario_iter.next().is_some() {
        return Err(anyhow::anyhow!("Not all strings in secnario were used."));
    }

    Ok(())
}



fn parse_ast(filename: impl AsRef<Path>) -> Result<HashMap<String, Value>> {
    let input = std::fs::read_to_string(filename)?;
    let tokens = tokenize(&input)?;
    parse_tokens(&tokens)
}


fn read_yaml_as_strings(yaml_file: impl AsRef<Path>) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(yaml_file)?;
    let parsed: Vec<String> = serde_yaml::from_str(&content)?;
    Ok(parsed)
}


fn value_to_script(value: &Value, indent_level: usize) -> Result<String> {
    let indent = "\t".repeat(indent_level);
    let next_indent = "\t".repeat(indent_level + 1);

    match value {
        Value::String(s) => Ok(format!("\"{}\"", s)),
        Value::Float(f) => {
            if f.fract() == 0.0 {
                Ok(format!("{:.1}", f)) 
            } else {
                Ok(f.to_string()) 
            }
        },
        Value::Integer(i) => Ok(i.to_string()),
        Value::Array(a) => {
            let contents: Result<Vec<String>> = a.iter().map(|v| value_to_script(v, indent_level + 1)).collect();
            contents.map(|c| format!("{{\n{}{}\n{}}}", 
                                     next_indent,
                                     c.join(&format!(",\n{}", next_indent)),
                                     indent))
        },
        Value::Dictionary(d) => {
            let mut contents = Vec::new();
            for (key, value) in d {
                let line = value_to_script(value, indent_level + 1)?;
                contents.push(format!("{}={}", key, line));
            }
            Ok(format!("\n{}{}\n{}", next_indent, contents.join(&format!(",\n{}", next_indent)), indent))
        }
    }
}



fn reconstruct_script(ast: &HashMap<String, Value>) -> Result<String> {
    let mut script = String::new();
    
    for (key, value) in ast.iter() {
        script.push_str(key);
        script.push_str(" = ");
        script.push_str(&value_to_script(value, 0)?);
        script.push('\n');
    }
    
    Ok(script)
}


fn prune_ast(ast: &mut HashMap<String, Value>) {
    if let Some(Value::Array(ast_array)) = ast.get_mut("ast") {
        for block_value in ast_array.iter_mut() {
            if let Value::Dictionary(blocks) = block_value {
                for (_, block_dict) in blocks.iter_mut() {
                    if let Value::Array(block_items) = block_dict {
                        let mut i = 0;
                        while i != block_items.len() {
                            match &mut block_items[i] {
                                Value::Dictionary(item_dict) => {
                                    item_dict.retain(|key, _| key == "linknext" || key == "line");
                                    i += 1;
                                },
                                _ => {
                                    block_items.remove(i);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}




#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract all secnario text to yaml
    Extract { input: PathBuf, output: PathBuf },
    /// Prune the ast file, remove all secnario text (for steam release)
    Prune { input: PathBuf, output: PathBuf },
    /// Merge corresponding secnario text back to ast file
    Merge { ast_input: PathBuf, yaml_input: PathBuf, output: PathBuf },
}


fn main() {
    let cli = Args::parse();
    match &cli.command {
        Commands::Extract { input, output } => {
            let ast = parse_ast(input).unwrap();
            extract_secnario_toyaml(&ast, output).unwrap();
        },
        Commands::Prune { input, output } => {
            let mut ast = parse_ast(input).unwrap();
            prune_ast(&mut ast);
            let s = reconstruct_script(&ast).unwrap();
            std::fs::write(output, s).unwrap();
        },
        Commands::Merge { ast_input, yaml_input, output } => {
            let mut ast = parse_ast(ast_input).unwrap();
            let secnario = read_yaml_as_strings(yaml_input).unwrap();
            replace_secnario(&mut ast, secnario).unwrap();
            let s = reconstruct_script(&ast).unwrap();
            std::fs::write(output, s).unwrap();
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ast() {
        let input = r#"astver = 2.0
        ast = {
            block_00000 = {
                {"savetitle", text="俺たちの新しい日常"},
                {"bg", time=2000, file="bg001a", path=":bg/"},
                {"se", file="seアラーム", loop=1, id=1},
                {"fg", ch="妃愛", size="no", mode=1, path=":fg/hiy[表情]/", file="hiy_nob0700", ex05="hiy_nob0000", face="b0032", head="hiy_nob", lv=2.2, id=20},
                {"text"},
                text = {
                    vo = {
                        {"vo", file="fem_hiy_00052", ch="hiy"},
                    },
                    ja = {
                        {
                            name = {"妃愛"},
                            "「お兄、あさー……むふー……」",
                            {"rt2"},
                        },
                    },
                },
                linknext = "block_00001",
                line = 18,
            },
        }
        "#;
    
        let tokens = tokenize(input).unwrap();
        let _value = parse_tokens(&tokens).unwrap();
    }


    fn read_yaml_as_strings2(yaml_file: &str) -> Result<Vec<String>> {
        let parsed: Vec<String> = serde_yaml::from_str(yaml_file)?;
        Ok(parsed)
    }

    #[test]
    fn test_prune_ast() {
        let input = r#"astver = 2.0
        ast = {
            block_00000 = {
                {"savetitle", text="俺たちの新しい日常"},
                {"bg", time=2000, file="bg001a", path=":bg/"},
                {"se", file="seアラーム", loop=1, id=1},
                {"fg", ch="妃愛", size="no", mode=1, path=":fg/hiy[表情]/", file="hiy_nob0700", ex05="hiy_nob0000", face="b0032", head="hiy_nob", lv=2.2, id=20},
                {"text"},
                text = {
                    vo = {
                        {"vo", file="fem_hiy_00052", ch="hiy"},
                    },
                    ja = {
                        {
                            name = {"妃愛"},
                            "「お兄、あさー……むふー……」",
                            {"rt2"},
                        },
                    },
                },
                linknext = "block_00001",
                line = 18,
            },
        }
        "#;
    
        let tokens = tokenize(input).unwrap();
        let mut value = parse_tokens(&tokens).unwrap();
        prune_ast(&mut value);
        let s = reconstruct_script(&value).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_reconstruct() {
        let input = r#"astver = 2.0
        ast = {
            block_00000 = {
                {"savetitle", text="俺たちの新しい日常"},
                {"bg", time=2000, file="bg001a", path=":bg/"},
                {"se", file="seアラーム", loop=1, id=1},
                {"fg", ch="妃愛", size="no", mode=1, path=":fg/hiy[表情]/", file="hiy_nob0700", ex05="hiy_nob0000", face="b0032", head="hiy_nob", lv=2.2, id=20},
                {"text"},
                text = {
                    vo = {
                        {"vo", file="fem_hiy_00052", ch="hiy"},
                    },
                    ja = {
                        {
                            name = {"妃愛"},
                            "「お兄、あさー……むふー……」",
                            {"rt2"},
                        },
                    },
                },
                linknext = "block_00001",
                line = 18,
            },
        }
        "#;
    
        let tokens = tokenize(input).unwrap();
        let value = parse_tokens(&tokens).unwrap();
        let s = reconstruct_script(&value).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_merge() {
        let input = r#"astver = 2.0
        ast = {
            block_00000 = {
                {"savetitle", text="俺たちの新しい日常"},
                {"bg", time=2000, file="bg001a", path=":bg/"},
                {"se", file="seアラーム", loop=1, id=1},
                {"fg", ch="妃愛", size="no", mode=1, path=":fg/hiy[表情]/", file="hiy_nob0700", ex05="hiy_nob0000", face="b0032", head="hiy_nob", lv=2.2, id=20},
                {"text"},
                text = {
                    vo = {
                        {"vo", file="fem_hiy_00052", ch="hiy"},
                    },
                    ja = {
                        {
                            name = {"妃愛"},
                            "「お兄、あさー……むふー……」",
                            {"rt2"},
                        },
                    },
                },
                linknext = "block_00001",
                line = 18,
            },
        }
        "#;
    
        let tokens = tokenize(input).unwrap();
        let _value = parse_tokens(&tokens).unwrap();
    }
}

