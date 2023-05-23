use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{self, Write};
use std::result::Result;

static DIR_PREFIX: &'static str = "@%#%@";
static NULL_STRING: &'static str = "";

#[derive(PartialEq, Hash, Eq)]
pub enum InfoParamType {
    EntityPath,
    DataType,
    Parameters,
    SerialNumber,
    Options,
    Help,
    EntityType,
    Units,
}

pub struct Genesis {
    pub comans: String,
    pub readans: String,
    pub status: String,

    pub mouseans: String,
    pub pausans: String,

    pub doinfo_single_values: HashMap<String, String>,
    pub doinfo_array_values: HashMap<String, Vec<String>>,

    atty: bool,
}

impl Genesis {
    pub fn new() -> Self {
        let ret = Genesis {
            comans: String::from(""),
            readans: String::from(""),
            status: String::from(""),
            mouseans: String::from(""),
            pausans: String::from(""),
            doinfo_single_values: HashMap::<String, String>::new(),
            doinfo_array_values: HashMap::<String, Vec<String>>::new(),

            atty: atty::is(atty::Stream::Stdout),
        };

        ret
    }

    pub fn von(&mut self) {
        self.send_command("VON", NULL_STRING);
    }

    pub fn vof(&mut self) {
        self.send_command("VOF", NULL_STRING);
    }

    pub fn su_on(&mut self) {
        self.send_command("SU_ON", NULL_STRING);
    }

    pub fn su_off(&mut self) {
        self.send_command("SU_OFF", NULL_STRING);
    }

    pub fn pause(&mut self, command: &str) {
        self.send_command("PAUSE", command);
        self.status = self.get_reply();
        self.readans = self.get_reply();
        self.pausans = self.get_reply();
    }

    pub fn mouse(&mut self, command: &str) {
        self.send_command("MOUSE", command);
        self.status = self.get_reply();
        self.readans = self.get_reply();
        self.mouseans = self.get_reply();
    }

    pub fn com(&mut self, command: &str) {
        self.send_command("COM", command);
        self.status = self.get_reply();
        self.readans = self.get_reply();
        self.comans = self.readans.clone();
    }

    pub fn aux(&mut self, command: &str) {
        self.send_command("AUX", command);
        self.status = self.get_reply();
        self.readans = self.get_reply();
        self.comans = self.get_reply();
    }

    pub fn info(&mut self, params: &HashMap<InfoParamType, String>) {
        let mut entity_path = String::from(NULL_STRING);
        let mut data_type = String::from(NULL_STRING);
        let mut parameters = String::from(NULL_STRING);
        let mut serial_number = String::from(NULL_STRING);
        let mut options = String::from(NULL_STRING);
        let mut help = String::from(NULL_STRING);
        let mut entity_type = String::from(NULL_STRING);
        let mut units = String::from(NULL_STRING);
        for (key, value) in params {
            match key {
                InfoParamType::EntityType => {
                    entity_type = format!("-t {}", value);
                }
                InfoParamType::DataType => {
                    data_type = format!("-d {}", value);
                }
                InfoParamType::Parameters => {
                    parameters = format!("-p {}", value);
                }
                InfoParamType::SerialNumber => {
                    serial_number = format!("-s {}", value);
                }
                InfoParamType::Options => {
                    options = format!("-o {}", value);
                }
                InfoParamType::Help => {
                    help = format!("-help");
                }
                InfoParamType::EntityPath => {
                    entity_path = format!("-e {}", value);
                }
                InfoParamType::Units => {
                    units = format!("units={},", value);
                }
            }
        }

        if units.is_empty() {
            units = format!("units=mm,");
        }

        let csh_file = format!(
            "{}/share/tmp/info_csh.{}",
            std::env::var("GENESIS_DIR").unwrap(),
            std::process::id()
        );

        self.doinfo_single_values = HashMap::<String, String>::new();
        self.doinfo_array_values = HashMap::<String, Vec<String>>::new();

        let msg = format!(
            "info,out_file={},write_mode=replace,{}args={} {} {} {} {} {} {} -m script",
            csh_file,
            units,
            entity_type,
            entity_path,
            data_type,
            parameters,
            serial_number,
            options,
            help
        );
        self.com(&msg);
        self.parse_info_file(&csh_file);
        std::fs::remove_file(&csh_file).unwrap();
    }

    fn parse_info_file(&mut self, file: &String) {
        let f = File::open(file).unwrap();
        let reader = BufReader::new(f);
        for _l in reader.lines() {
            let line = _l.unwrap();
            if self.parse_array_value(&line).is_ok() {
            } else {
                self.parse_single_value(&line);
            }
        }
    }

    fn parse_single_value(&mut self, line: &String) {
        /*
        set gNUM_ROWS = '30'
        set gNUM_COLS = '5'
        set gNUM_LAYERS = '4'
        set gNUM_STEPS = '1'
         */
        let reg_single = Regex::new(r"^set\s+([a-zA-Z_]+)\s+=\s+'(.+)'").unwrap();
        let m = reg_single.captures(line.as_str());
        if m.is_some() {
            let caps = m.unwrap();
            if caps.len() == 3 {
                return;
            }
            self.doinfo_single_values.insert(
                caps.get(1).unwrap().as_str().to_string(),
                caps.get(2).unwrap().as_str().to_string(),
            );
        }
    }

    fn parse_array_value(&mut self, line: &String) -> Result<(), ()> {
        /*
            set gCOLcol       = ('1'    '2'     '3'     '4'     '5'    )
            set gCOLtype      = ('step' 'empty' 'empty' 'empty' 'empty')
            set gCOLstep_name = ('orig' ''      ''      ''      ''     )
            set gATTRname = ()
            set gATTRval  = ()
         */
        let reg_arr = Regex::new(r"^set\s+([a-zA-Z_]+)\s+=\s+\('(.*)'\s*\)").unwrap();
        let reg_null = Regex::new(r"^set\s+([a-zA-Z_]+)\s+=\s+\(.*\)").unwrap();

        let m = reg_arr.captures(line);
        if m.is_some() {
            let caps = m.unwrap();
            assert!(caps.len() == 3);
            let reg_vec = Regex::new(r"'\s+'").unwrap();
            let res_split:Vec<&str> = reg_vec.split(caps.get(2).unwrap().as_str()).collect();
            let mut a  = Vec::<String>::new();
            for v_ in res_split {
                a.push(String::from(v_));
            }
            self.doinfo_array_values.insert(String::from(caps.get(1).unwrap().as_str()), a);

            return Ok(());
        }

        let m = reg_null.captures(line);
        if m.is_some() {
            let caps = m.unwrap();
            self.doinfo_array_values.insert(String::from(caps.get(1).unwrap().as_str()), Vec::<String>::new());
            return Ok(());
        }

        Err(())
    }

    #[allow(unused_must_use)]
    fn get_reply(&self) -> String {
        let mut line = String::new();
        io::stdin().read_line(&mut line);
        line
    }

    fn blank_status_results(&mut self) {
        self.status.clear();
        self.readans.clear();
        self.comans.clear();
        self.pausans.clear();
        self.mouseans.clear();
    }

    fn send_command(&mut self, cmd_type: &str, command: &str) {
        self.blank_status_results();
        let msg = format!("{}{} {}\n", DIR_PREFIX, cmd_type, command);
        if self.atty {
            self.send_command_to_socket(&msg);
        } else {
            self.send_command_to_pipe(&msg);
        }
    }

    #[allow(unused_must_use)]
    fn send_command_to_pipe(&mut self, command: &str) {
        io::stdout().write(command.as_bytes());
        io::stdout().flush();
    }

    fn send_command_to_socket(&mut self, _command: &str) {}
}
