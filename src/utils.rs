/// Returns a color based on the kid's name for the avatar circle.
pub fn kid_color(name: &str) -> &'static str {
    let hash: u32 = name
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    let colors = [
        "#6366f1", // indigo
        "#8b5cf6", // violet
        "#ec4899", // pink
        "#f43f5e", // rose
        "#f97316", // orange
        "#eab308", // yellow
        "#22c55e", // green
        "#14b8a6", // teal
        "#06b6d4", // cyan
        "#3b82f6", // blue
    ];
    colors[(hash as usize) % colors.len()]
}
