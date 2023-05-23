pub mod genesis;
pub use crate::genesis::Genesis;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::{io::Write, collections::HashMap};

    use crate::genesis::InfoParamType;

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);

        let mut f = Genesis::new();
        f.com("units,type=mm");
        std::io::stdout().write(f.status.as_bytes()).unwrap();
        std::io::stdout().write(f.comans.as_bytes()).unwrap();
        f.com("get_affect_layer");
        std::io::stdout().write(f.comans.as_bytes()).unwrap();

        let params: HashMap<InfoParamType, String> 
         = HashMap::<InfoParamType, String>::from([
            (InfoParamType::EntityType, String::from("matrix")),
            (InfoParamType::EntityPath, String::from("000/matrix")),
        ]);

        f.info(&params);
        for (k, v) in &f.doinfo_single_values {
            std::io::stdout().write(format!("{} => {}\n", k, v).as_bytes()).unwrap();
        }
        for (k, v) in &f.doinfo_array_values {
            std::io::stdout().write(format!("{} => {}\n", k, v.len()).as_bytes()).unwrap();
        }
    }
}
