use std::time::{Duration, Instant};

/// Block until the local HTTP server accepts TCP connections (or timeout).
pub fn wait_for_server(addr: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if std::net::TcpStream::connect_timeout(
            &addr.parse().expect("valid listen address"),
            Duration::from_millis(400),
        )
        .is_ok()
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(120));
    }
    false
}

/// Open `url` in the user's default browser, retrying a few times.
pub fn open(url: &str) {
    const ATTEMPTS: u32 = 4;
    for i in 0..ATTEMPTS {
        if try_open(url) {
            return;
        }
        if i + 1 < ATTEMPTS {
            std::thread::sleep(Duration::from_millis(350));
        }
    }
    eprintln!("未能自动打开浏览器，请手动访问: {}", url);
}

#[cfg(windows)]
fn try_open(url: &str) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    // 1) Shell protocol handler — most reliable default-browser launcher on Windows.
    if std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .is_ok()
    {
        return true;
    }

    // 2) PowerShell Start-Process (handles many edge cases cmd start misses).
    let ps = format!("Start-Process -WindowStyle Normal '{0}'", url.replace('\'', "''"));
    if std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .is_ok()
    {
        return true;
    }

    // 3) Classic cmd start fallback.
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .is_ok()
}

#[cfg(target_os = "macos")]
fn try_open(url: &str) -> bool {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .is_ok()
}

#[cfg(not(any(windows, target_os = "macos")))]
fn try_open(url: &str) -> bool {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .is_ok()
}
