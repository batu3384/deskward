mod capture;
mod input;

use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("deskward_host_windows=info".parse().unwrap()),
        )
        .init();

    let _cap = capture::WinScreenCapture;
    let _inp = input::WinInputInjector;
    info!("deskward-host-windows stub — Faz 2 DXGI/SendInput");
}
