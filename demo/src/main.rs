use std::{collections::HashMap, io::*};

use genesis2000::{Genesis, InfoParamType};

fn main() {
    let mut f = Genesis::new();
    f.COM("units,type=mm");
    std::io::stdout().write(f.STATUS.as_bytes()).unwrap();
    std::io::stdout().write(f.COMANS.as_bytes()).unwrap();
    f.COM("get_affect_layer");
    std::io::stdout().write(f.COMANS.as_bytes()).unwrap();

    let params: HashMap<InfoParamType, String> = HashMap::<InfoParamType, String>::from([
        (InfoParamType::EntityType, String::from("matrix")),
        (InfoParamType::EntityPath, String::from("000/matrix")),
    ]);

    f.INFO(&params);
    f.print_doinfo_single_values();
    f.print_doinfo_array_values();
}
