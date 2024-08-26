use macroquad::prelude::*;

#[macroquad::main("Equilateral Triangle")]
async fn main() {
    let radius = 200.0;

    loop {
        clear_background(WHITE);

        let center = vec2(screen_width() / 2.0, screen_height() / 2.0);

        draw_equilateral_triangle(center, radius);

        next_frame().await;
    }
}

fn draw_equilateral_triangle(center: Vec2, radius: f32) {
    let angle_offset = std::f32::consts::PI / 6.0; // Offset to align the triangle's bottom edge

    let vertices = [
        vec2(
            center.x + radius * (angle_offset).cos(),
            center.y + radius * (angle_offset).sin(),
        ),
        vec2(
            center.x + radius * (angle_offset + 2.0 * std::f32::consts::PI / 3.0).cos(),
            center.y + radius * (angle_offset + 2.0 * std::f32::consts::PI / 3.0).sin(),
        ),
        vec2(
            center.x + radius * (angle_offset + 4.0 * std::f32::consts::PI / 3.0).cos(),
            center.y + radius * (angle_offset + 4.0 * std::f32::consts::PI / 3.0).sin(),
        ),
    ];

    draw_line(
        vertices[0].x,
        vertices[0].y,
        vertices[1].x,
        vertices[1].y,
        2.0,
        BLACK,
    );
    draw_line(
        vertices[1].x,
        vertices[1].y,
        vertices[2].x,
        vertices[2].y,
        2.0,
        BLACK,
    );
    draw_line(
        vertices[2].x,
        vertices[2].y,
        vertices[0].x,
        vertices[0].y,
        2.0,
        BLACK,
    );
}
