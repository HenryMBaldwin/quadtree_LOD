use macroquad::prelude::*;

#[macroquad::main("quadmap_LOD")]
async fn main() {
    loop {
        clear_background(BLACK);
        draw_text("Hello, world!", 20.0, 20.0, 30.0, WHITE);
        next_frame().await
    }
}