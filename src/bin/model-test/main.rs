use std::error::Error;
use std::process::ExitCode;

use gloog::loader;


pub fn main() -> ExitCode {
    let Some(model_path) = std::env::args().skip(1).next() else {
        eprintln!("Missing model filepath");
        return ExitCode::FAILURE;
    };

    match run(model_path) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        },
    }
}


fn run(model_path: String) -> Result<(), Box<dyn Error>> {
    loader::obj::load(model_path)?;

    Ok(())
}
