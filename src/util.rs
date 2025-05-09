use std::{
    io::{self, Read},
    process::Child,
};

use crate::args::RootArgs;

pub fn wait_for_child<F>(_args: &RootArgs, child: &mut Child, formatter: F) -> io::Result<()>
where
    F: Fn(&str) -> String,
{
    let mut output: String = String::new();
    child.stdout.take().unwrap().read_to_string(&mut output)?;

    let status = child.wait()?;

    if status.success() {
        let output = formatter(output.trim());
        print!("{output}");
    } else {
        std::process::exit(1);
    }
    Ok(())
}

pub fn get_height(args: &RootArgs) -> String {
    args.height.clone().unwrap_or("45%".to_string())
}
