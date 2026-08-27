use std::env;
use std::error::Error;
use std::fs;

use wit_component::ComponentEncoder;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let input = args
        .next()
        .ok_or("usage: lift <core.wasm> <component.wasm>")?;
    let output = args
        .next()
        .ok_or("usage: lift <core.wasm> <component.wasm>")?;
    if args.next().is_some() {
        return Err("usage: lift <core.wasm> <component.wasm>".into());
    }

    let module = fs::read(input)?;
    let component = ComponentEncoder::default()
        .module(&module)?
        .validate(true)
        .encode()?;
    fs::write(output, component)?;
    Ok(())
}
