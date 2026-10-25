//! Печатает окна и мониторы, сохраняет миниатюру первого монитора.
//!
//!     cargo run -p s4s-core --example sources

fn main() -> anyhow::Result<()> {
    s4s_core::init()?;

    println!("Мониторы:");
    let monitors = s4s_core::sources::monitors();
    for m in &monitors {
        println!("  {:<20} {} — {}", m.id, m.title, m.subtitle);
    }

    println!("Окна:");
    for w in s4s_core::sources::windows() {
        let state = if w.minimized { " (свёрнуто)" } else { "" };
        println!("  {:<20} {} — {} [{}×{}]{state}", w.id, w.title, w.subtitle, w.width, w.height);
    }

    if let Some(first) = monitors.first() {
        let jpeg = s4s_core::thumbnail::capture_jpeg(first.id.parse()?, 480)?;
        std::fs::write("target/monitor-thumbnail.jpg", &jpeg)?;
        println!("Миниатюра {} сохранена в target/monitor-thumbnail.jpg ({} байт)", first.title, jpeg.len());
    }
    Ok(())
}
