use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use dialoguer::{theme::ColorfulTheme, Select};
use regex::Regex;
use toml::Value;

fn main() {
    let config = fs::read_to_string("setting.toml").expect("Failed to read config file");
    let value = config.parse::<Value>().expect("Failed to parse TOML");

    let vars = value["var"].as_table().unwrap();
    let templates = value["template"].as_table().unwrap();

    let template_names: Vec<_> = templates.keys().collect();
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("テンプレートを選択してください")
        .default(0)
        .items(&template_names)
        .interact()
        .unwrap();
    let selected_template = template_names[selection];
    let template_string = templates[selected_template].as_str().unwrap();

    let re = Regex::new(r"\{([^}]+)\}").unwrap();
    let mut replacements = HashMap::new();
    for cap in re.captures_iter(template_string) {
        let key = cap[1].to_string();
        if let Some(value) = vars.get(key.as_str()) {
            let value = value.as_str().unwrap();
            if value.starts_with("{now") {
                let date_fmt = value.replace("{now:", "").replace("}", "");
                let now = chrono::Local::now();
                replacements.insert(key, now.format(&date_fmt).to_string());
            } else {
                replacements.insert(key, value.to_string());
            
            }
        } else {
            println!("{} の値を入力してください:", key);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read line");
            replacements.insert(key, input.trim().to_string());
        }
    }

    let mut new_filename = template_string.to_string();
    for (key, value) in replacements.iter() {
        new_filename = new_filename.replace(&format!("{{{}}}", key), value);
    }

    
    let args = std::env::args().collect::<Vec<_>>();
    let old_path = PathBuf::from(&args[1]);
    let ext = old_path.extension().unwrap().to_str().unwrap();
    let new_path = old_path.with_file_name(new_filename).with_extension(ext);
    fs::rename(&old_path, &new_path).expect("Failed to rename file");
    println!("ファイルがリネームされました: {:?}", new_path);
}