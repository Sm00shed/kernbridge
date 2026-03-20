use std::fs;
use std::path::PathBuf;

struct Args { // [1]
    input:  PathBuf,
    output: PathBuf,
    scan:   bool,
}

fn parse_args() -> Args { // [2]
    let args: Vec<String> = std::env::args().collect();
    let mut input  = PathBuf::new();
    let mut output = PathBuf::from("/tmp/kb-out");
    let mut scan   = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--scan"   => { scan = true; i += 1; if i < args.len() { input = PathBuf::from(&args[i]); } }
            "--input"  => { i += 1; if i < args.len() { input = PathBuf::from(&args[i]); } }
            "--output" => { i += 1; if i < args.len() { output = PathBuf::from(&args[i]); } }
            _          => { if input.as_os_str().is_empty() { input = PathBuf::from(&args[i]); } }
        }
        i += 1;
    }

    Args { input, output, scan }
}

#[derive(Debug, Clone)]
enum FieldType { // [3]
    Flag,
    Value,
    ListValue,
    TextValue,
    Unknown(String),
}

#[derive(Debug, Clone)]
struct Field { // [4]
    tab:         String,
    field_type:  FieldType,
    name:        String,
    label:       String,
    description: String,
    datatype:    String,
    default:     String,
    options:     Vec<(String, String)>,
    depends:     Vec<(String, String)>,
}

#[derive(Debug)]
struct Tab { // [5]
    id:    String,
    label: String,
}

#[derive(Debug)]
struct ParseResult { // [6]
    package_name: String,
    tabs:         Vec<Tab>,
    fields:       Vec<Field>,
    warnings:     Vec<String>,
}

fn parse_js(content: &str, filename: &str) -> ParseResult { // [7]
    let package_name = std::path::Path::new(filename)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut result = ParseResult {
        package_name,
        tabs:     Vec::new(),
        fields:   Vec::new(),
        warnings: Vec::new(),
    };

    let mut current_field: Option<Field> = None;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("s.tab(") { // [8]
            if let Some(tab) = parse_tab(line) {
                result.tabs.push(tab);
            }
            continue;
        }

        if line.contains("s.taboption(") || line.contains("s.option(") { // [9]
            if let Some(existing) = current_field.take() {
                result.fields.push(existing);
            }
            match parse_field(line) {
                Ok(field) => {
                    println!("  ✅ {:?} \"{}\"", field.field_type, field.name);
                    current_field = Some(field);
                }
                Err(e) => {
                    result.warnings.push(format!("⚠️  {}: {}", line, e));
                    println!("  ⚠️  {}", line);
                }
            }
            continue;
        }

        if let Some(ref mut field) = current_field {
            if line.starts_with("o.value(") { // [10]
                if let Some((val, label)) = parse_option(line) {
                    field.options.push((val, label));
                }
            } else if line.starts_with("o.depends(") { // [11]
                if let Some((f, v)) = parse_depends(line) {
                    field.depends.push((f, v));
                }
            } else if line.starts_with("o.datatype") { // [12]
                field.datatype = extract_string_value(line);
            } else if line.starts_with("o.default") { // [13]
                field.default = extract_string_value(line);
            } else if line.starts_with("o.description") { // [14]
                field.description = extract_translated(line);
            }
        }
    }

    if let Some(field) = current_field.take() { // [15]
        result.fields.push(field);
    }

    result
}

fn parse_tab(line: &str) -> Option<Tab> { // [16]
    let args = extract_args(line)?;
    if args.len() < 2 { return None; }
    Some(Tab {
        id:    args[0].trim_matches('"').to_string(),
        label: clean_translated(&args[1]),
    })
}

fn parse_field(line: &str) -> Result<Field, String> { // [17]
    let args = extract_args(line).ok_or("no args")?;
    if args.len() < 3 { return Err("too few args".into()); }

    let tab       = args[0].trim_matches('"').to_string();
    let type_str  = args[1].trim().to_string();
    let name      = args[2].trim_matches('"').to_string();
    let label     = if args.len() > 3 { clean_translated(&args[3]) } else { String::new() };

    let field_type = match type_str.as_str() {
        "form.Flag"      => FieldType::Flag,
        "form.Value"     => FieldType::Value,
        "form.ListValue" => FieldType::ListValue,
        "form.TextValue" => FieldType::TextValue,
        other            => FieldType::Unknown(other.to_string()),
    };

    Ok(Field {
        tab, field_type, name, label,
        description: String::new(),
        datatype:    String::new(),
        default:     String::new(),
        options:     Vec::new(),
        depends:     Vec::new(),
    })
}

fn parse_option(line: &str) -> Option<(String, String)> { // [18]
    let args  = extract_args(line)?;
    let val   = args.get(0)?.trim_matches('"').to_string();
    let label = args.get(1).map(|s| clean_translated(s)).unwrap_or(val.clone());
    Some((val, label))
}

fn parse_depends(line: &str) -> Option<(String, String)> { // [19]
    let args = extract_args(line)?;
    if args.len() < 2 { return None; }
    Some((
        args[0].trim_matches('"').to_string(),
        args[1].trim_matches('"').to_string(),
    ))
}

fn extract_args(line: &str) -> Option<Vec<String>> { // [20]
    let start = line.find('(')?;
    let end   = line.rfind(')')?;
    Some(split_args(&line[start+1..end]))
}

fn split_args(s: &str) -> Vec<String> { // [21]
    let mut args    = Vec::new();
    let mut depth   = 0i32;
    let mut current = String::new();

    for ch in s.chars() {
        match ch {
            '(' | '[' => { depth += 1; current.push(ch); }
            ')' | ']' => { depth -= 1; current.push(ch); }
            ',' if depth == 0 => {
                args.push(current.trim().to_string());
                current = String::new();
            }
            _ => { current.push(ch); }
        }
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

fn clean_translated(s: &str) -> String { // [22]
    let s = s.trim();
    if s.starts_with("_(\"") && s.ends_with("\")") {
        return s[3..s.len()-2].to_string();
    }
    s.trim_matches('"').trim_matches('\'').to_string()
}

fn extract_translated(line: &str) -> String { // [23]
    if let Some(start) = line.find("_(\"") {
        if let Some(end) = line[start+3..].find('"') {
            return line[start+3..start+3+end].to_string();
        }
    }
    String::new()
}

fn extract_string_value(line: &str) -> String { // [24]
    if let Some(pos) = line.find('=') {
        return line[pos+1..].trim()
            .trim_matches('"').trim_matches('\'')
            .trim_end_matches(';').to_string();
    }
    String::new()
}

fn generate_html(result: &ParseResult) -> String { // [25]
    let mut html = String::new();
    let pkg = &result.package_name;

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str(&format!("  <title>{} - LuCI</title>\n", pkg));
    html.push_str("  <link rel=\"stylesheet\" href=\"/luci-static/bootstrap/cascade.css\">\n");
    html.push_str("</head>\n<body class=\"logged-in\">\n\n");
    html.push_str("<div id=\"maincontent\" class=\"container-fluid\">\n");
    html.push_str("  <div id=\"content\">\n");
    html.push_str(&format!("    <form class=\"cbi-map\" method=\"POST\" action=\"/cgi-bin/luci/admin/network/{}\">\n\n", pkg));

    if !result.tabs.is_empty() { // [26]
        html.push_str("      <ul class=\"nav nav-tabs\">\n");
        for (i, tab) in result.tabs.iter().enumerate() {
            let active = if i == 0 { " class=\"active\"" } else { "" };
            html.push_str(&format!(
                "        <li{}><a href=\"#{}\" data-toggle=\"tab\">{}</a></li>\n",
                active, tab.id, tab.label
            ));
        }
        html.push_str("      </ul>\n\n      <div class=\"tab-content\">\n\n");
    }

    let tab_ids: Vec<String> = if result.tabs.is_empty() {
        vec!["__default__".to_string()]
    } else {
        result.tabs.iter().map(|t| t.id.clone()).collect()
    };

    for (i, tab_id) in tab_ids.iter().enumerate() { // [27]
        let active = if i == 0 { " active" } else { "" };

        if result.tabs.is_empty() {
            html.push_str("      <div class=\"cbi-section\">\n        <div class=\"cbi-section-node\">\n\n");
        } else {
            html.push_str(&format!(
                "        <div class=\"tab-pane{}\" id=\"{}\">\n          <div class=\"cbi-section\">\n            <div class=\"cbi-section-node\">\n\n",
                active, tab_id
            ));
        }

        for field in result.fields.iter().filter(|f| f.tab == *tab_id || *tab_id == "__default__") {
            html.push_str(&generate_field_html(field));
        }

        if result.tabs.is_empty() {
            html.push_str("        </div>\n      </div>\n\n");
        } else {
            html.push_str("            </div>\n          </div>\n        </div>\n\n");
        }
    }

    if !result.tabs.is_empty() {
        html.push_str("      </div>\n\n");
    }

    html.push_str("      <div class=\"cbi-page-actions\">\n");
    html.push_str("        <input class=\"btn btn-primary\" type=\"submit\" value=\"Save &amp; Apply\">\n");
    html.push_str(&format!(
        "        <a class=\"btn btn-default\" href=\"/cgi-bin/luci/admin/network/{}\">Cancel</a>\n",
        pkg
    ));
    html.push_str("      </div>\n\n    </form>\n  </div>\n</div>\n\n");

    let has_depends = result.fields.iter().any(|f| !f.depends.is_empty()); // [28]
    if has_depends {
        html.push_str("<script>\nfunction kb_toggle() {\n");
        for field in result.fields.iter().filter(|f| !f.depends.is_empty()) {
            for (dep_field, dep_val) in &field.depends {
                html.push_str(&format!(
                    "  var el = document.getElementById('row_{}');\n  var dep = document.querySelector('[name=\"{}\"]');\n  if (dep && el) {{\n    var val = dep.type === 'checkbox' ? (dep.checked ? '1' : '0') : dep.value;\n    el.style.display = val === '{}' ? '' : 'none';\n  }}\n",
                    field.name, dep_field, dep_val
                ));
            }
        }
        html.push_str("}\ndocument.querySelectorAll('input,select').forEach(function(el) { el.addEventListener('change', kb_toggle); });\nkb_toggle();\n</script>\n");
    }

    html.push_str("<script src=\"/luci-static/bootstrap/luci.js\"></script>\n</body>\n</html>\n");
    html
}

fn generate_field_html(field: &Field) -> String { // [29]
    let mut html = String::new();

    html.push_str(&format!("              <div class=\"cbi-value\" id=\"row_{}\">\n", field.name));
    html.push_str(&format!("                <label class=\"cbi-value-title\">{}</label>\n", field.label));
    html.push_str("                <div class=\"cbi-value-field\">\n");

    match &field.field_type {
        FieldType::Flag => {
            html.push_str(&format!(
                "                  <input type=\"checkbox\" name=\"{}\" value=\"1\" {{% if config.{} == \"1\" %}}checked{{% endif %}}>\n",
                field.name, field.name
            ));
        }
        FieldType::Value => {
            let input_type = if field.datatype.contains("uinteger") || field.datatype.contains("integer") {
                "number"
            } else {
                "text"
            };
            html.push_str(&format!(
                "                  <input class=\"cbi-input-text\" type=\"{}\" name=\"{}\" value=\"{{{{ config.{} }}}}\"{}>\n",
                input_type, field.name, field.name,
                if input_type == "number" { " min=\"0\"" } else { "" }
            ));
        }
        FieldType::ListValue => {
            html.push_str(&format!("                  <select class=\"cbi-input-select\" name=\"{}\">\n", field.name));
            for (val, label) in &field.options {
                html.push_str(&format!(
                    "                    <option value=\"{}\" {{% if config.{} == \"{}\" %}}selected{{% endif %}}>{}{}",
                    val, field.name, val, label, "</option>\n"
                ));
            }
            html.push_str("                  </select>\n");
        }
        FieldType::TextValue => {
            html.push_str(&format!(
                "                  <textarea class=\"cbi-input-textarea\" name=\"{}\">{{{{ config.{} }}}}</textarea>\n",
                field.name, field.name
            ));
        }
        FieldType::Unknown(t) => {
            html.push_str(&format!("                  <!-- TODO: unknown type {} for {} -->\n", t, field.name));
        }
    }

    if !field.description.is_empty() {
        html.push_str(&format!("                  <div class=\"cbi-value-description\">{}</div>\n", field.description));
    }

    html.push_str("                </div>\n              </div>\n\n");
    html
}

fn generate_rust(result: &ParseResult) -> String { // [30]
    let mut rs = String::new();
    let pkg = &result.package_name;
    let cap = capitalize(pkg);

    rs.push_str("use anyhow::Result;\n");
    rs.push_str("use askama::Template;\n\n");

    rs.push_str(&format!("#[derive(Template)]\n#[template(path = \"{}/index.html\")]\n", pkg));
    rs.push_str(&format!("pub struct {}Template {{\n    pub config: {}Config,\n}}\n\n", cap, cap));

    rs.push_str("#[derive(serde::Serialize, serde::Deserialize, Debug)]\n");
    rs.push_str(&format!("pub struct {}Config {{\n", cap));
    for field in &result.fields {
        if !matches!(field.field_type, FieldType::Unknown(_)) {
            rs.push_str(&format!("    pub {}: String,\n", field.name));
        }
    }
    rs.push_str("}\n\n");

    rs.push_str("#[derive(serde::Deserialize, Debug)]\n");
    rs.push_str(&format!("pub struct {}Form {{\n", cap));
    for field in &result.fields {
        if !matches!(field.field_type, FieldType::Unknown(_)) {
            if matches!(field.field_type, FieldType::Flag) {
                rs.push_str(&format!("    pub {}: Option<String>,\n", field.name));
            } else {
                rs.push_str(&format!("    pub {}: String,\n", field.name));
            }
        }
    }
    rs.push_str("}\n\n");

    rs.push_str(&format!("pub fn read_{}_config() -> {}Config {{\n    {}Config {{\n", pkg, cap, cap));
    for field in &result.fields {
        if !matches!(field.field_type, FieldType::Unknown(_)) {
            rs.push_str(&format!(
                "        {}: uci_get(\"{}.@{}[0].{}\", \"{}\"),\n",
                field.name, pkg, pkg, field.name, field.default
            ));
        }
    }
    rs.push_str("    }\n}\n\n");

    rs.push_str(&format!("pub fn write_{}_config(form: &{}Form) -> Result<()> {{\n", pkg, cap));
    for field in &result.fields {
        if !matches!(field.field_type, FieldType::Unknown(_)) {
            if matches!(field.field_type, FieldType::Flag) {
                rs.push_str(&format!(
                    "    uci_set(\"{}.@{}[0].{}\", if form.{}.is_some() {{ \"1\" }} else {{ \"0\" }})?;\n",
                    pkg, pkg, field.name, field.name
                ));
            } else {
                rs.push_str(&format!(
                    "    uci_set(\"{}.@{}[0].{}\", &form.{})?;\n",
                    pkg, pkg, field.name, field.name
                ));
            }
        }
    }
    rs.push_str(&format!("\n    // TODO: service restart for {}\n", pkg));
    rs.push_str(&format!("    std::process::Command::new(\"uci\").args([\"commit\", \"{}\"]).status()?;\n", pkg));
    rs.push_str("    Ok(())\n}\n\n");

    rs.push_str("fn uci_get(key: &str, default: &str) -> String {\n");
    rs.push_str("    std::process::Command::new(\"uci\").args([\"get\", key]).output()\n");
    rs.push_str("        .map(|o| { let s = String::from_utf8_lossy(&o.stdout).trim().to_string(); if s.is_empty() { default.to_string() } else { s } })\n");
    rs.push_str("        .unwrap_or_else(|_| default.to_string())\n}\n\n");

    rs.push_str("fn uci_set(key: &str, value: &str) -> Result<()> {\n");
    rs.push_str("    std::process::Command::new(\"uci\").args([\"set\", &format!(\"{}={}\", key, value)]).status()?;\n");
    rs.push_str("    Ok(())\n}\n");

    rs
}

fn capitalize(s: &str) -> String { // [31]
    let mut c = s.chars();
    match c.next() {
        None    => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn main() { // [32]
    let args = parse_args();

    if args.input.as_os_str().is_empty() {
        eprintln!("Usage: kb-port [--scan] [--input] <file.js> [--output <dir>]");
        std::process::exit(1);
    }

    let content = match fs::read_to_string(&args.input) {
        Ok(c)  => c,
        Err(e) => { eprintln!("Cannot read {:?}: {}", args.input, e); std::process::exit(1); }
    };

    let filename = args.input.file_name().unwrap_or_default().to_string_lossy().to_string();

    println!("\nkb-port: {}\n", filename);

    let result = parse_js(&content, &filename);

    println!("\nTabs: {}", result.tabs.len());
    for tab in &result.tabs {
        println!("  {} → {}", tab.id, tab.label);
    }
    println!("Fields: {}", result.fields.len());

    if !result.warnings.is_empty() { // [33]
        println!("\nWarnings:");
        for w in &result.warnings { println!("  {}", w); }
    }

    if args.scan { // [34]
        println!("\nScan only — no files written.");
        return;
    }

    let pkg      = &result.package_name;
    let tmpl_dir = args.output.join("templates").join(pkg);
    let src_dir  = args.output.join("src");

    fs::create_dir_all(&tmpl_dir).expect("Cannot create template dir");
    fs::create_dir_all(&src_dir).expect("Cannot create src dir");

    let html_path = tmpl_dir.join("index.html");
    let rs_path   = src_dir.join(format!("{}.rs", pkg));

    fs::write(&html_path, generate_html(&result)).expect("Cannot write HTML");
    fs::write(&rs_path,   generate_rust(&result)).expect("Cannot write Rust");

    println!("\nWritten:"); // [35]
    println!("  {:?}", html_path);
    println!("  {:?}", rs_path);
    println!("\nNext: add to main.rs: mod {};", pkg);
}
