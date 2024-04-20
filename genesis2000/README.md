# Developer Information
- Author：GUO
- e-mail: guobbs@live.com
- QQ: 349384061
- date：2024-04-20

# What is Genesis2000？
- Genesis2000 is a CAM software in the PCB industry, developed by an Israeli company, Frontline.
- Genesis2000 provides a rich secondary development interface to help us automate CAM operations, one of which is script development support. For details, please refer to the official Genesis2000 script development documentation.

# Why provide a Rust crate?
- The script development interface provided by Genesis2000 officially only supports three shell languages: csh, perl, and tcl. Many developers who are not familiar with these shells may have to spend a lot of effort to learn them again.
- Scripts written in languages like csh and perl are in plain text, which cannot meet the confidentiality requirements. If using native compiled languages like Rust, you can protect your source code from being exposed."

# How use it?

## use
Use cargo to create a new script project, for example, the project name is demo
```
cargo new --bin demo
```

Enter the demo project directory from the command line terminal.
```
cd demo
```

Add the genesis2000 dependency crate to the demo project, run the following Cargo command in your project directory:
```
cargo add genesis2000
```

Open the src/main.rs file with an editor and write code in the following way:
```rust
use std::collections::HashMap;

use genesis2000::{Genesis, InfoParamType};

fn main() {
    // Create a structure instance object of Genesis.
    let mut f = Genesis::new();
    // Demonstrate how to use the COM interface.
    f.COM("units,type=mm");
    // print results of COM
    println!("status: {}\ncomans: {}", f.STATUS, f.COMANS);
    f.COM("get_affect_layer");
    println!("status: {}\ncomans: {}", f.STATUS, f.COMANS);
    // Demonstrate how to get the coordinates of the point clicked by the mouse in the main graphic area.
    f.MOUSE("p Select a point on the screen");
    // f.MOUSEANS is a coordinate point
    println!("status: {}\nmouse: {}", f.STATUS, f.MOUSEANS);
    // Demonstrate how to get the coordinates of two diagonal points of a rectangular area selected by the mouse in the main graphic area.
    f.MOUSE("r Select an rectangle area on the screen");
    // f.MOUSEANS is two coordinate points
    println!("status: {}\nmouse: {}", f.STATUS, f.MOUSEANS);

    let params: HashMap<InfoParamType, String> = HashMap::<InfoParamType, String>::from([
        (InfoParamType::EntityType, String::from("matrix")),
        (InfoParamType::EntityPath, String::from("000/matrix")),
    ]);
    // Info is essentially a COM command. It is a powerful and complex command interface, so a separate encapsulation has been made. Here is a demonstration of how to use info.
    f.INFO(&params);
    // The results of info are divided into two categories, here we view all single-value results.
    f.print_doinfo_single_values();
    // The results of info are divided into two categories, here we view all array results.
    f.print_doinfo_array_values();
}

```

# Interface Introduction
The public functions are:
```
VON, VOF, SU_ON, SU_OFF, PAUSE, MOUSE, COM, AUX, and INFO
```

For other script development assistance, please refer to the official Genesis2000 help document: 0204.pdf.

