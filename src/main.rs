use std::fs::File;
use std::path::Path;
use libloading::Library;
use anyhow::{ Result, anyhow };
use rustyline::DefaultEditor;
use colored::Colorize;

const BANNER: &'static str = r#"
    ___    __      ___                           ______                                             __  
   /   |  / /     /   | ____ __________ _       / ____/________ _____ ___  ___ _      ______  _____/ /__
  / /| | / /_____/ /| |/ __ `/ ___/ __ `/      / /_  / ___/ __ `/ __ `__ \/ _ \ | /| / / __ \/ ___/ //_/
 / ___ |/ /_____/ ___ / /_/ (__  ) /_/ /      / __/ / /  / /_/ / / / / / /  __/ |/ |/ / /_/ / /  / ,<   
/_/  |_/_/     /_/  |_\__, /____/\__,_/      /_/   /_/   \__,_/_/ /_/ /_/\___/|__/|__/\____/_/  /_/|_|  
                        /_/                                                                             
"#;


pub trait Module {
    fn help(&self) -> String;
    fn run(&self) -> Result<()>;
    fn options(&self) -> Vec<(String, String, bool)>;
    fn set(&mut self, k: String, v: String);
    fn get(&self, k: String) -> String;
}

enum Command {
    LoadModule(String),
    Set(String, String),
    ShowOptions,
    RunModule,
    HelpModule,
    Quit
}

fn main() -> Result<()> {
    print!("{}", BANNER.green().bold());
    println!("  🔻 {}\n\n", "CyberSecurity toolkit".blue().underline());
    
    let mut rl = DefaultEditor::new()?;

    let history_file_name = format!("{}/.aqsa_history", std::env::var("HOME")?);
    let history_file_name = history_file_name.as_str();

    if rl.load_history(&history_file_name).is_err() {
        File::create(Path::new(&history_file_name))?;
    }

    let command_prompt = format!("{}{}{}{}{} ",
        std::env::var("USER")?.blue().bold(),
        "(".purple().bold(),
        "Al-Aqsa".red().bold().underline(),
        ")".purple().bold(),
        ">".blue().bold()
    );
    let command_prompt = command_prompt.as_str();

    let mut input: String;

    let mut module_library: Library;
    let mut module: Option<Box<dyn Module>> = None;

    let mut should_run = true;
    while should_run {
        match rl.readline(command_prompt) {
            Ok(line) => { input = line; },
            Err(_) => { rl.save_history(&history_file_name)?; break }
        }

        if input.is_empty() { continue; }
        else { rl.add_history_entry(input.clone())?; }

        match parse_command(input.clone()) {
            Ok(Command::Quit) => { should_run = false; },
            Ok(Command::LoadModule(module_path)) => unsafe {
                if module_path.is_empty() { continue; }
                let module_path = module_path.replace("~", &std::env::var("HOME")?);

                match Library::new(module_path.clone()) {
                    Ok(lib) => {
                        module_library = lib;
                        module = Some(module_library.get::<fn () -> Box<dyn Module>>(b"get_plugin")?());
                        println!("{} {} {}", "*".red().bold(), "loaded module".bold(), module_path.clone().green().bold());
                    },
                    Err(e) => { println!("Error: {e}"); },
                };
            },
            Ok(Command::Set(key, val)) => module.as_deref_mut().map(|m| m.set(key.clone(), val.clone())).expect("MODULE_SET_FAILURE"),
            Ok(Command::ShowOptions) => module.as_deref_mut().map(|m| {
                println!("Options:");
                m.options().into_iter().for_each(|(key, desc, opt)| {
                    println!("   {} {:<12}=>  {:<12}  |  {}",
                        if opt { " ".bold() } else { "*".red().bold() },
                        key.bold().green(), m.get(key.clone()).bold(), desc.bold().blue()
                    );
                });
            }).expect("MODULE_OPTIONS_FAILURE"),
            Ok(Command::HelpModule) => module.as_deref_mut().map(|m| { println!("Module info:\n{}", m.help().italic()); }).expect("MODULE_HELP_FAILURE"),
            Ok(Command::RunModule) => module.as_deref_mut().map(|m| if let Err(e) = m.run() { println!("Error: {e}"); }).expect("MODULE_RUN_FAILURE"),
            Err(e) => { println!("Error: {e}"); }
        };
    }

    rl.save_history(&history_file_name)?;

    Ok(())
}

fn parse_command(command: String) -> Result<Command> {
    let command: Vec<&str> = command.split(" ").collect();

    match command[0] {
        "quit" | "q"  => Ok(Command::Quit),
        "load" | "l"  => {
            if command.len() == 2 { Ok(Command::LoadModule(command[1].to_owned())) }
            else { Err(anyhow!("usage: load <module path>")) }
        },
        "set" => {
            if command.len() >= 3 { Ok(Command::Set(command[1].to_owned(), command[2..].join(" "))) }
            else { Err(anyhow!("usage: set <key> <value>")) }
        },
        "run"  | "r"  => Ok(Command::RunModule),
        "?" | "options" | "o"  => Ok(Command::ShowOptions),
        "help" | "h" => Ok(Command::HelpModule),
        _ => Err(anyhow!("Unknown command {}", command[0]))
    }
}
