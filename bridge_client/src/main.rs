use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::env;

use bridge_core::{BridgeCommand, BridgeResponse};

// Lokasi socket dilihat dari sisi Chroot
// Asumsi: folder /data/local/tmp/chroot di-mount sebagai / (root) atau akses relatif
// Jika di dalam Chroot ada folder /tmp yang mapping ke host, maka:
const SOCKET_PATH: &str = "/tmp/bridge.sock";

fn main() -> std::io::Result<()> {
    // Ambil argumen dari CLI (contoh: ./client "screencap -p")
    // Untuk contoh input: andro input tap 500 500
    // args[0] = andro
    // args[1] = input (program)
    // args[2..] = tap, 500, 500 (arguments)

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: andro <program> [args...]");
        eprintln!("Example: andro input tap 500 500");
        eprintln!("Example: andro pm list packages");
        return Ok(());
    }

    // Mapping command
    // Jika "tap" = DirectTap
    // Jika "input tap" = Shell biasa (Exec)
    let command = if args[1] == "tap" {
        if args.len() < 4 {
            eprintln!("Error: Gunakan format 'andro tap <x> <y>'");
            return Ok(());
        }

        let x = args[2].parse::<i32>().expect("X harus angka integer");
        let y = args[3].parse::<i32>().expect("Y harus angka integer");

        BridgeCommand::DirectTap { x, y }
    } else {
        // Jika "input tap" Mode Exec (Shell)
        BridgeCommand::Exec {
            program: args[1].clone(),
            args: args[2..].to_vec()
        }
    };

    // Kirim
    let mut stream = UnixStream::connect(SOCKET_PATH).map_err(|e| {
        eprintln!("Gagal connect ke {}. Pastikan Server nyala!", SOCKET_PATH);
        e
    })?;

    // Serialize Command ke Bytes (Bincode)
    let bin_payload = bincode::serialize(&command).expect("Gagal serialize");
    stream.write_all(&bin_payload)?;

    // Baca Response
    // Bincode butuh byte array, bukan String
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;
    if buffer.is_empty() {
        eprintln!("Server tidak memberikan respon.");
        return Ok(());
    }
    let response: BridgeResponse = bincode::deserialize(&buffer).expect("Gagal deserialize response");

    match response {
        BridgeResponse::Success(out) => {
            if !out.is_empty() {
            print!("{}", out); // Print stdout
            }
        },
        BridgeResponse::Error(err) => {
            eprintln!("Remote Error: {}", err);
        }
    }

    Ok(())
}
