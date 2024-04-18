use std::collections::HashMap;

use genesis2000::{Genesis, InfoParamType};

fn main() {
    let mut f = Genesis::new();
    f.COM("units,type=mm");
    println!("status: {}\ncomans: {}", f.STATUS, f.COMANS);
    f.COM("get_affect_layer");
    println!("status: {}\ncomans: {}", f.STATUS, f.COMANS);
    f.MOUSE("p Select a point on the screen");
    println!("status: {}\nmouse: {}", f.STATUS, f.MOUSEANS);
    f.MOUSE("r Select an rectangle area on the screen");
    println!("status: {}\nmouse: {}", f.STATUS, f.MOUSEANS);

    let params: HashMap<InfoParamType, String> = HashMap::<InfoParamType, String>::from([
        (InfoParamType::EntityType, String::from("matrix")),
        (InfoParamType::EntityPath, String::from("000/matrix")),
    ]);

    f.INFO(&params);
    f.print_doinfo_single_values();
    f.print_doinfo_array_values();
}
