use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut cargo_text = String::new();
    File::open("Cargo.toml").and_then(|mut f| f.read_to_string(&mut cargo_text)).unwrap();
    let decoded: toml::Value = toml::from_str(&cargo_text).unwrap();

    // Call the plugins function with the decoded Cargo.toml file data
    plugins(&decoded);
}

// This function extracts data from the decoded Cargo.toml file and uses it to generate Rust code
fn plugins(decoded: &toml::Value) {
    let mut inits = Vec::new();
    // Extract the data from the "package.metadata.plugins" field of the Cargo.toml file
    if let Some(plugins) = decoded
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("plugins"))
        .and_then(|plugins| plugins.as_table())
    {
        // Iterate over the plugins and extract the relevant data
        for (id, cfg) in plugins {
            let plugin_id = id.replace('-', "_");
            let name = cfg.get("name").and_then(|v| v.as_str()).unwrap_or(id);
            let descr = cfg.get("description").and_then(|v| v.as_str()).unwrap_or_default();
            let default_startup = cfg.get("default_startup").and_then(|v| v.as_bool()).unwrap_or(false);
            let immutable = cfg.get("immutable").and_then(|v| v.as_bool()).unwrap_or(false);
            println!(
                "plugin_id: {}, default_startup: {}, immutable: {}, name: {}, descr: {}",
                plugin_id, default_startup, immutable, name, descr
            );
            // Use the extracted data to generate Rust code and add it to the inits vector
            inits.push(format!(
                "    {}::register(rmqtt::Runtime::instance(), r#\"{}\"#, r#\"{}\"#, {} || default_startups.contains(&String::from(r#\"{}\"#)), {}).await?;",
                plugin_id, name, descr, default_startup, name, immutable
            ));
        }
    }

    // Write the generated code to the plugin.rs file in the OUT_DIR directory
    let out = std::env::var("OUT_DIR").unwrap();
    let mut plugin_rs = File::create(format!("{}/{}", out, "plugin.rs")).unwrap();

    plugin_rs.write_all(b"#[allow(clippy::all)]\n").unwrap();
    plugin_rs
        .write_all(b"pub(crate) async fn registers(default_startups: Vec<String>) -> rmqtt::Result<()>{\n")
        .unwrap();
    plugin_rs.write_all(inits.join("\n").as_bytes()).unwrap();
    plugin_rs.write_all(b"\n    Ok(())\n}").unwrap();
}
