use wasm_paths::get_paths;

use std::time::Instant;

#[test]
fn textbox() {
    const WIDTH: i32 = 600;
    const HEIGHT: i32 = 600;
    const HEIGHT_PAD: i32 = 100;
    let time_start = Instant::now();
    let paths = get_paths(0, 0, WIDTH, HEIGHT, 16, 3);
    let time_end = Instant::now();
    let duration = (time_end - time_start).as_millis();

    let svg_paths = paths
        .iter()
        .map(|p| format!("<path d=\"{}\"></path>", p))
        .collect::<Vec<String>>();

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
        WIDTH, HEIGHT + HEIGHT_PAD, WIDTH, HEIGHT + HEIGHT_PAD
    );
    svg += &format!(
        "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"cyan\" stroke-width=\"5\"></rect>",
        WIDTH, HEIGHT
    );
    svg += &format!("<g fill=\"black\" stroke=\"transparent\">");
    for path in svg_paths.iter() {
        svg += path;
    }
    svg += &format!(
        "<text x=\"{}\" y=\"{}\">Layout took {}ms</text>",
        20,
        HEIGHT + HEIGHT_PAD - 20,
        duration,
    );
    svg += "</g>/</svg>";

    std::fs::write("textbox.svg", svg).unwrap();
}
