//! Binary entry point. Delegates to [`holon_render_3d::run`], which opens a window
//! under the render features and runs headless otherwise.

fn main() {
    holon_render_3d::run();
}
