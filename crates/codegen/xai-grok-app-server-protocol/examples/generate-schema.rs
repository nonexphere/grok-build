use std::{env, fs, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas/generated-protocol.schema.json");
    let mut generated =
        serde_json::to_string_pretty(&xai_grok_app_server_protocol::protocol_schema())
            .expect("protocol schema serializes");
    generated.push('\n');

    if env::args().any(|arg| arg == "--check") {
        match fs::read_to_string(&path) {
            Ok(checked_in) if checked_in == generated => ExitCode::SUCCESS,
            Ok(_) => {
                eprintln!("generated protocol schema is stale: {}", path.display());
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("cannot read {}: {error}", path.display());
                ExitCode::FAILURE
            }
        }
    } else {
        fs::write(&path, generated).expect("write generated protocol schema");
        println!("wrote {}", path.display());
        ExitCode::SUCCESS
    }
}
