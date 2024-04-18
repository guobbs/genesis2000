use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
use std::sync::OnceLock;

static DIR_PREFIX: &'static str = "@%#%@";
static NULL_STRING: &'static str = "";
const PORT: u16 = 56753;

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

#[allow(non_snake_case)]
pub struct Genesis {
    pub COMANS: String,
    pub READANS: String,
    pub STATUS: String,

    pub MOUSEANS: String,
    pub PAUSANS: String,

    doinfo_single_values: HashMap<String, String>, // 单值
    doinfo_array_values: HashMap<String, Vec<String>>, // 数组值

    atty: bool,

    socket: Option<TcpStream>,
}

impl Drop for Genesis {
    fn drop(&mut self) {
        if self.atty {
            self.send_command("CLOSEDOWN", "");
        }
    }
}

impl Genesis {
    pub fn new() -> Self {
        let mut ret = Genesis {
            COMANS: String::from(""),
            READANS: String::from(""),
            STATUS: String::from(""),
            MOUSEANS: String::from(""),
            PAUSANS: String::from(""),
            doinfo_single_values: HashMap::<String, String>::new(),
            doinfo_array_values: HashMap::<String, Vec<String>>::new(),

            // atty: atty::is(atty::Stream::Stdout),
            atty: false,
            socket: None,
        };

        if cfg!(debug_assertions) || atty::is(atty::Stream::Stdout) {
            ret.atty = true;
            println!("Genesis script is running in debug mode.");
            ret.socket = ret.open_socket();

            ret.inherit_environment();
        }

        ret
    }

    fn open_socket(&self) -> Option<TcpStream> {
        let addr = format!("127.0.0.1:{PORT}");
        let ss = TcpStream::connect(addr).expect("Genesis server is not running.");
        Some(ss)
    }

    fn inherit_environment(&mut self) {
        self.send_command("GETENVIRONMENT", "");
        loop {
            let mut buf_reader = self.get_buf_reader();
            let line = Self::get_reply(self.atty, buf_reader.as_mut());
            if line.starts_with("END") {
                break;
            }
            let mut iter = line.split("=");
            let key = iter.next().unwrap();
            let value = iter.next().unwrap();
            std::env::set_var(key, value);
        }

        std::env::remove_var("LC_MESSAGES");
        std::env::remove_var("LC__FASTMSG");
    }

    #[allow(non_snake_case)]
    pub fn VON(&mut self) {
        self.send_command("VON", NULL_STRING);
    }

    #[allow(non_snake_case)]
    pub fn VOF(&mut self) {
        self.send_command("VOF", NULL_STRING);
    }

    #[allow(non_snake_case)]
    pub fn SU_ON(&mut self) {
        self.send_command("SU_ON", NULL_STRING);
    }

    #[allow(non_snake_case)]
    pub fn SU_OFF(&mut self) {
        self.send_command("SU_OFF", NULL_STRING);
    }

    #[allow(non_snake_case)]
    pub fn PAUSE(&mut self, command: &str) {
        self.send_command("PAUSE", command);
        let res = || -> (String, String, String) {
            let mut buf_reader = self.get_buf_reader();
            let STATUS = Self::get_reply(self.atty, buf_reader.as_mut());
            let READANS = Self::get_reply(self.atty, buf_reader.as_mut());
            let PAUSANS = Self::get_reply(self.atty, buf_reader.as_mut());
            (STATUS, READANS, PAUSANS)
        }();

        self.STATUS = res.0;
        self.READANS = res.1;
        self.PAUSANS = res.2;
    }

    #[allow(non_snake_case)]
    pub fn MOUSE(&mut self, command: &str) {
        self.send_command("MOUSE", command);
        let res = || -> (String, String, String) {
            let mut buf_reader = self.get_buf_reader();
            let STATUS = Self::get_reply(self.atty, buf_reader.as_mut());
            let READANS = Self::get_reply(self.atty, buf_reader.as_mut());
            let MOUSEANS = Self::get_reply(self.atty, buf_reader.as_mut());
            (STATUS, READANS, MOUSEANS)
        }();

        self.STATUS = res.0;
        self.READANS = res.1;
        self.MOUSEANS = res.2;
    }

    #[allow(non_snake_case)]
    pub fn COM(&mut self, command: &str) {
        self.send_command("COM", command);

        let res = || -> (String, String) {
            let mut buf_reader = self.get_buf_reader();
            let STATUS = Self::get_reply(self.atty, buf_reader.as_mut());
            let READANS = Self::get_reply(self.atty, buf_reader.as_mut());
            (STATUS, READANS)
        }();

        self.STATUS = res.0;
        self.READANS = res.1;
        self.COMANS = self.READANS.clone();
    }

    #[allow(non_snake_case)]
    pub fn AUX(&mut self, command: &str) {
        self.send_command("AUX", command);
        let res = || -> (String, String) {
            let mut buf_reader = self.get_buf_reader();
            let STATUS = Self::get_reply(self.atty, buf_reader.as_mut());
            let READANS = Self::get_reply(self.atty, buf_reader.as_mut());
            (STATUS, READANS)
        }();

        self.STATUS = res.0;
        self.READANS = res.1;
        self.COMANS = self.READANS.clone();
    }

    #[allow(non_snake_case)]
    pub fn INFO(&mut self, params: &HashMap<InfoParamType, String>) {
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
        self.COM(&msg);
        self.parse_info_file(&csh_file);
        std::fs::remove_file(&csh_file).unwrap();
    }

    pub fn get_info_single_value(&self, key: &str) -> Option<&String> {
        self.doinfo_single_values.get(key)
    }

    pub fn get_info_array_value(&self, key: &str) -> Option<&Vec<String>> {
        self.doinfo_array_values.get(key)
    }

    /// @note 仅供测试使用
    pub fn print_doinfo_single_values(&self) {
        for (k, v) in &self.doinfo_single_values {
            println!("{} => {}", k, v);
        }
    }

    /// @note 仅供测试使用
    pub fn print_doinfo_array_values(&self) {
        for (k, v) in &self.doinfo_array_values {
            println!("{} => {:?}", k, v);
        }
    }

    fn parse_info_file(&mut self, file: &String) {
        let f = File::open(file).unwrap();
        let reader = BufReader::new(f);
        for _l in reader.lines() {
            let line = _l.unwrap();

            if let Some((k, v)) = Self::parse_array_value(&line) {
                self.doinfo_array_values.insert(k, v);
                continue;
            }

            if let Some((k, v)) = Self::parse_single_value(&line) {
                self.doinfo_single_values.insert(k, v);
            }
        }
    }

    #[inline]
    fn parse_single_value(line: &String) -> Option<(String, String)> {
        /*
        set gNUM_ROWS = '30'
        set gNUM_COLS = '5'
        set gNUM_LAYERS = '4'
        set gNUM_STEPS = '1'
         */
        static REGS: OnceLock<[(Regex, usize); 3]> = OnceLock::new();
        let regs = REGS.get_or_init(|| {
            [
                (
                    Regex::new(r"^set\s+([a-zA-Z_]+)\s*=\s*'(.+)'\s*").unwrap(),
                    3,
                ),
                (
                    Regex::new(r"^set\s+([a-zA-Z_]+)\s*=\s*([^'\s]+)\s*").unwrap(),
                    3,
                ),
                (Regex::new(r"^set\s+([a-zA-Z_]+)\s*=\s*").unwrap(), 2),
            ]
        });
        for (reg, len) in regs.iter() {
            match reg.captures(line.as_str()) {
                Some(caps) => {
                    debug_assert!(caps.len() == *len);
                    if caps.len() < 3 {
                        return Some((caps.get(1).unwrap().as_str().to_string(), "".to_string()));
                    } else {
                        return Some((
                            caps.get(1).unwrap().as_str().to_string(),
                            caps.get(2).unwrap().as_str().to_string(),
                        ));
                    }
                }
                None => {}
            }
        }
        None
    }

    #[inline]
    fn parse_array_value(line: &String) -> Option<(String, Vec<String>)> {
        /*
           set gCOLcol       = ('1'    '2'     '3'     '4'     '5'    )
           set gCOLtype      = ('step' 'empty' 'empty' 'empty' 'empty')
           set gCOLstep_name = ('orig' ''      ''      ''      ''     )
           set gATTRname = ()
           set gATTRval  = ()
        */
        static REG_ARR: OnceLock<Regex> = OnceLock::new();
        static REG_NULL: OnceLock<Regex> = OnceLock::new();
        let reg =
            REG_ARR.get_or_init(|| Regex::new(r"^set\s+([a-zA-Z_]+)\s+=\s+\('(.*)'\s*\)").unwrap());

        match reg.captures(line) {
            Some(caps) => {
                assert!(caps.len() == 3);
                let reg_vec = Regex::new(r"'\s+'").unwrap();
                let res_split: Vec<&str> = reg_vec.split(caps.get(2).unwrap().as_str()).collect();
                let mut a = Vec::<String>::new();
                a.reserve(res_split.len());
                for v_ in res_split {
                    a.push(String::from(v_));
                }

                return Some((String::from(caps.get(1).unwrap().as_str()), a));
            }
            _ => {}
        }

        let reg = REG_NULL.get_or_init(|| Regex::new(r"^set\s+([a-zA-Z_]+)\s+=\s+\(.*\)").unwrap());
        match reg.captures(line) {
            Some(caps) => Some((
                String::from(caps.get(1).unwrap().as_str()),
                Vec::<String>::new(),
            )),
            _ => None,
        }
    }

    fn get_buf_reader(&self) -> Option<BufReader<&TcpStream>> {
        let mut buf_reader = None;
        if self.atty {
            assert!(self.socket.is_some());
            buf_reader = Some(BufReader::new(self.socket.as_ref().unwrap()));
        }
        buf_reader
    }

    #[allow(unused_must_use)]
    fn get_reply(atty: bool, buf_reader: Option<&mut BufReader<&TcpStream>>) -> String {
        if atty {
            assert!(buf_reader.is_some());
            let mut line = String::new();
            buf_reader.unwrap().read_line(&mut line).unwrap();
            line.pop(); // remove '\n'
            line
        } else {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line);
            line.pop(); // remove '\n'
            line
        }
    }

    fn blank_status_results(&mut self) {
        self.STATUS.clear();
        self.READANS.clear();
        self.COMANS.clear();
        self.PAUSANS.clear();
        self.MOUSEANS.clear();

        self.doinfo_single_values.clear();
        self.doinfo_array_values.clear();
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
        std::io::stdout().write(command.as_bytes());
        std::io::stdout().flush();
    }

    fn send_command_to_socket(&mut self, _command: &str) {
        assert!(self.socket.is_some());
        if let Some(ss) = &mut self.socket {
            ss.write(_command.as_bytes()).unwrap();
            ss.flush().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_value() {
        let arr = [
            ("set gNUM_ROWS = '30'", ("gNUM_ROWS", "30")),
            ("set gNUM_COLS = '5'  ", ("gNUM_COLS", "5")),
            ("set gNUM_LAYERS =   '4'", ("gNUM_LAYERS", "4")),
            ("set gNUM_STEPS='1'", ("gNUM_STEPS", "1")),
            // ===
            ("set gNUM_COLS =      ", ("gNUM_COLS", "")),
            ("set gNUM_ROWS = 30     ", ("gNUM_ROWS", "30")),
        ];

        for (line, (k, v)) in &arr {
            let (k_, v_) = Genesis::parse_single_value(&String::from(*line)).unwrap();
            assert_eq!(k_, *k);
            assert_eq!(v_, *v);
        }
    }

    #[test]
    fn test_parse_array_value() {
        let arr = [
            (
                "set gCOLcol       = ('1'    '2'     '3'     '4'     '5'    )",
                ("gCOLcol", vec!["1", "2", "3", "4", "5"]),
            ),
            (
                "set gCOLtype      = ('step' 'empty' 'empty' 'empty' 'empty')",
                ("gCOLtype", vec!["step", "empty", "empty", "empty", "empty"]),
            ),
            (
                "set gCOLstep_name = ('orig' ''      ''      ''      ''     )",
                ("gCOLstep_name", vec!["orig", "", "", "", ""]),
            ),
            ("set gATTRname = ()", ("gATTRname", Vec::<&str>::new())),
            ("set gATTRval  = ()", ("gATTRval", Vec::<&str>::new())),
        ];

        for (line, (k, v)) in &arr {
            let (k_, v_) = Genesis::parse_array_value(&String::from(*line)).unwrap();
            assert_eq!(k_, *k);
            assert_eq!(v_, *v);
        }
    }
}
