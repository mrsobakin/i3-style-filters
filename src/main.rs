#[macro_use] extern crate lazy_static;
extern crate linked_hash_map;
use std::path::Path;
use std::env;
use std::process;
use std::fs;
use std::fs::create_dir_all;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;
extern crate includedir;
extern crate phf;

extern crate yaml_rust;
use yaml_rust::{YamlLoader, YamlEmitter};

extern crate clap;
use clap::{Arg, App};

mod theme;
mod writer;

include!(concat!(env!("OUT_DIR"), "/data.rs"));

fn exit_error(msg: &str) {
    println!("{}", msg);
    process::exit(1);

}

fn get_run_tmp_dir() -> String {
    let start = SystemTime::now();
    let elapsed = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let sec = (elapsed.as_secs() as u64) + (elapsed.subsec_nanos() as u64) / 1000_000;
    let mut tmp_dir = env::temp_dir();
    tmp_dir.push("i3-style");
    tmp_dir.push(sec.to_string());
    create_dir_all(tmp_dir.as_path()).expect("Could not create temporary directory");
    String::from(tmp_dir.to_str().unwrap())
}

fn get_system_config_path() -> Option<String> {
    let home = String::from(env::var("HOME").unwrap());

    let config_path = vec![
        format!("{}/{}", home, ".i3/config"),
        format!("{}/{}", home, ".config/i3/config"),
        String::from("/etc/i3/config"),
        String::from("/etc/xdg/i3/config")
    ];

    for p in config_path {
        if Path::new(&p).exists() {
            return Some(p);
        }
    }

    None
}

fn validate_config(path: &String) -> bool {
    let status = Command::new("command")
        .arg("-v")
        .arg("i3")
        .status()
        .unwrap();

    if !status.success() {
        println!("Could not find i3 binary in the system PATH to validate the config before parsing");
        return false
    }

    let status = Command::new("i3")
        .arg("-C")
        .arg("-c")
        .arg(path)
        .status()
        .unwrap();

    status.success()
}

fn get_embedded_theme(name: &str) -> Option<theme::Theme> {
    let file = format!("./themes/{}", name);

    if !FILES.is_available(&file) {
        return None;
    }

    let contents = String::from_utf8(FILES.get(&format!("{}/{}", "./themes", name)).unwrap().to_vec()).expect("Theme yaml is not utf-8");

    let docs = YamlLoader::load_from_str(contents.as_str()).expect("Could not parse yaml for theme");
    let doc = &docs[0];

    Some(theme::from_yaml(&doc))
}

fn list_themes() {
    println!("\nAvailable themes:\n");
    let prefix_count = "./themes/".chars().count();

    for file in FILES.file_names() {
        let contents = String::from_utf8(FILES.get(file).unwrap().to_vec()).expect("Theme yaml is not utf-8");

        let docs = YamlLoader::load_from_str(contents.as_str()).expect("Could not parse yaml for theme");
        let doc = &docs[0];

        let description = doc["meta"]["description"].as_str().expect("Theme doesn't have a description");

        let entry: String = file.chars().skip(prefix_count).collect();
        let entry = format!("  {:width$} - {}", entry, description, width=18);
        println!("{}", entry);
    }
}

fn main() {
    let mut cli = App::new("i3-style")
        .version("1.0")
        .about("Make your i3 config a bit more stylish")
        .arg(Arg::with_name("theme")
             .help("The theme to use")
             .index(1)
             )
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("file")
            .help("The config file the theme should be applied to. Defaults to the default i3 location.")
            .takes_value(true)
            )
        .arg(Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("file")
             .help("Apply the theme, attempt to validate the result, and write it to <file>")
             .takes_value(true)
            )
        .arg(Arg::with_name("save")
             .short("s")
             .long("save")
             .help("Set the output file to the path of the input file")
            )
        .arg(Arg::with_name("reload")
             .short("r")
             .long("reload")
             .help("Apply the theme by reloading the config")
            )
        .arg(Arg::with_name("list-all")
             .short("l")
             .long("list-all")
             .help("Print a list of all available themes")
            )
        .arg(Arg::with_name("to-theme")
             .short("t")
             .long("to-theme")
             .value_name("file")
             .help("Prints an i3-style theme based on the given config suitable for sharing with others")
             .takes_value(true)
             .default_value("")
            );

    let app = cli.clone().get_matches();

    if app.is_present("list-all") {
        list_themes();
        process::exit(0);
    }

    if app.occurrences_of("to-theme") > 0 {
        let mut config: String = String::from(app.value_of("to-theme").unwrap());

        if config.is_empty() {
            config = match app.value_of("config") {
                Some(c) => String::from(c),
                None => match get_system_config_path() {
                    Some(c) =>  c,
                    None => {
                        exit_error("Could not find i3 config");
                        // not reached
                        String::from("")
                    }
                }
            }
        }

        if (!validate_config(&config)) {
            println!("Could not validate config at {}. Use `i3 -C -c {}` to see validation errors.", config, config);
            process::exit(1);
        }

        let theme = theme::from_config_file(&config);

        let yaml = theme.to_yaml_with_colors();

        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(&yaml).unwrap(); // dump the YAML object to a String
        }
        println!("{}", out_str);

        process::exit(0);
    }

    if !app.is_present("theme") {
        if app.args.is_empty() {
            cli.print_help().unwrap();
            process::exit(0);
        } else {
            println!("{}\n", app.usage());
            println!("Select a theme as the first argument. Use `--list-all` to see the themes.");
            process::exit(1);
        }
    }

    let config = match app.value_of("config") {
        Some(c) => Some(String::from(c)),
        None => get_system_config_path()
    };

    if config.is_none() {
        exit_error("Could not find i3 config");
    }
    let config = config.unwrap();

    if (!validate_config(&config)) {
        println!("Could not validate config at {}. Use `i3 -C -c {}` to see validation errors.", config, config);
        process::exit(1);
    }

    let theme_name = app.value_of("theme").unwrap();
    let theme = get_embedded_theme(theme_name);

    if theme.is_none() {
        // TODO get theme from the filesystem
        exit_error(format!("Could not find theme: {}", theme_name).as_str());
    }
    let theme = theme.unwrap();

    let output =  if app.value_of("output").is_some() {
        app.value_of("output")
    } else if app.is_present("save") {
        Some(config.as_str().clone())
    } else {
        None
    };

    if output.is_some() {
        let i3_style_tmp = get_run_tmp_dir();
        let tmp_output = format!("{}/{}", i3_style_tmp, "config-output");
        let tmp_input = format!("{}/{}", i3_style_tmp, "config-input");
        // 1. write the new config in the tmp folder
        writer::write_config(&config, Some(&tmp_output), theme);
        // 2. copy the config to the tmp folder
        println!("saving config at {} to {}", &config, &tmp_input);
        fs::copy(&config, &tmp_input).unwrap();
        // 3. copy the new config to the config location
        fs::copy(&tmp_output, &config).unwrap();
    } else {
        writer::write_config(&config, None, theme);
    }
}
